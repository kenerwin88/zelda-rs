//! The two map16 stripe builders, including WRAM reads and map8 asset writes,
//! on the caller's real CPU, stack and synchronous timeline.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
    overworld_map_scroll::MapScrollStripe,
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldStripes {
    pc: u16,
    complete: bool,
    entry_sp: u16,
}
impl OverworldStripes {
    pub(crate) fn new(direction: MapScrollStripe, cpu: &NativeNmiContext) -> Self {
        let pc = match direction {
            MapScrollStripe::Horizontal => 0xf3b9,
            MapScrollStripe::Vertical => 0xf482,
        };
        assert_eq!((cpu.bank, cpu.pc), (2, pc));
        Self {
            pc,
            complete: false,
            entry_sp: cpu.sp,
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38), (2, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xf3b9 | 0xf482 => {
                w.drain(16);
                let v = w.read(w.absolute(0x416), 2);
                w.load(v, 2);
                pc + 3
            }
            0xf3bc | 0xf485 | 0xf3e3 | 0xf3f5 | 0xf4ad | 0xf4bb | 0xf40e | 0xf4d0 => {
                w.mask(
                    match pc {
                        0xf3bc => 2,
                        0xf485 => 4,
                        0xf40e | 0xf4d0 => 15,
                        _ => 63,
                    },
                    2,
                );
                pc + 3
            }
            0xf3bf | 0xf3cb | 0xf3e6 | 0xf3f8 | 0xf455 | 0xf489 | 0xf495 | 0xf4b0 | 0xf4be
            | 0xf51d => {
                w.tax_word();
                pc + 1
            }
            0xf3c0 | 0xf3c8 | 0xf405 | 0xf418 | 0xf42d | 0xf439 | 0xf48a | 0xf492 | 0xf4c7
            | 0xf4da | 0xf4ef | 0xf4ff => {
                let a = match pc {
                    0xf3c0 | 0xf48a => 0x84,
                    0xf3c8 | 0xf4c7 => 0x88,
                    0xf405 | 0xf492 => 0x86,
                    0xf418 | 0xf4da => 2,
                    0xf42d | 0xf4ff => 12,
                    _ => 0,
                };
                let v = w.dp_read(a, 2);
                w.load(v, 2);
                pc + 2
            }
            0xf3c2 | 0xf48c => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xf3c3 | 0xf48d => {
                w.drain(24);
                let v = w.read(
                    (if pc == 0xf3c3 { 0x02f883 } else { 0x02f885 }) + u32::from(w.cpu.x),
                    2,
                );
                w.sbc(v, 2);
                pc + 4
            }
            0xf3c7 | 0xf3ec | 0xf3fe | 0xf491 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                pc + 1
            }
            0xf3ca
            | 0xf41a
            | 0xf452
            | 0xf453
            | 0xf454
            | 0xf494
            | 0xf4dc..=0xf4e1
            | 0xf51a
            | 0xf51b
            | 0xf51c => {
                w.shift_left(2);
                pc + 1
            }
            0xf488 => {
                w.drain(6);
                let old = w.cpu.a;
                w.cpu.p = (w.cpu.p & !CARRY) | (old as u8 & 1);
                w.load(old >> 1, 2);
                pc + 1
            }
            0xf3cc | 0xf3d1 | 0xf3d6 | 0xf413 | 0xf444 | 0xf496 | 0xf49b | 0xf4a0 | 0xf4d5
            | 0xf50c => {
                w.immediate(
                    match pc {
                        0xf3cc | 0xf496 => 0x2000,
                        0xf3d1 | 0xf49b => 0x7e,
                        0xf413 => 0x400,
                        0xf4d5 => 0x800,
                        _ => 16,
                    },
                    2,
                );
                pc + 3
            }
            0xf3cf | 0xf3d4 | 0xf3d9 | 0xf407 | 0xf411 | 0xf416 | 0xf41e | 0xf424 | 0xf42f
            | 0xf435 | 0xf447 | 0xf47f | 0xf499 | 0xf49e | 0xf4a3 | 0xf4c9 | 0xf4d3 | 0xf4d8
            | 0xf4e5 | 0xf4eb | 0xf50a | 0xf50f | 0xf547 => {
                let a = match pc {
                    0xf3d4 | 0xf407 | 0xf411 | 0xf435 | 0xf49e | 0xf4c9 | 0xf4d3 | 0xf50a => 2,
                    0xf3d9 | 0xf4a3 => 3,
                    0xf424 | 0xf4eb => 12,
                    0xf447 | 0xf50f => 6,
                    0xf47f | 0xf547 => 14,
                    _ => 0,
                };
                w.dp_store(a, w.cpu.a, 2);
                pc + 2
            }
            0xf3db | 0xf3ed | 0xf4a5 | 0xf4b3 => {
                let a = w.indirect_y_address(0);
                let v = w.read(a, 2);
                w.load(v, 2);
                pc + 2
            }
            0xf3dd | 0xf3ef | 0xf4a7 | 0xf4b5 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0x500 + w.cpu.x), w.cpu.a, 2);
                pc + 3
            }
            0xf3e0 | 0xf3e1 | 0xf3f2 | 0xf3f3 | 0xf44e | 0xf44f | 0xf4aa | 0xf4ab | 0xf4b8
            | 0xf4b9 | 0xf516 | 0xf517 => {
                w.increment_x();
                pc + 1
            }
            0xf3e2 | 0xf3f4 | 0xf4ac | 0xf4ba => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                pc + 1
            }
            0xf3e7 | 0xf3f9 | 0xf47a | 0xf542 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                pc + 1
            }
            0xf3e8 | 0xf3fa | 0xf41b | 0xf420 | 0xf47b | 0xf4e2 | 0xf4e7 | 0xf543 => {
                w.clear_carry();
                pc + 1
            }
            0xf3e9 | 0xf3fb | 0xf421 | 0xf47c | 0xf4e8 | 0xf544 => {
                w.drain(16);
                w.adc(match pc {
                    0xf3e9 | 0xf3fb => 128,
                    0xf421 => 0x800,
                    0xf4e8 => 0x400,
                    0xf47c => 66,
                    _ => 64,
                });
                pc + 3
            }
            0xf3ff | 0xf476 | 0xf4c1 | 0xf53e => {
                w.dp_increment(if matches!(pc, 0xf3ff | 0xf4c1) { 3 } else { 6 }, 2, false);
                pc + 2
            }
            0xf401 | 0xf478 | 0xf4c3 | 0xf540 | 0xf40c | 0xf4ce => {
                let (take, target) = match pc {
                    0xf401 => (w.cpu.p & ZERO == 0, 0xf3db),
                    0xf478 => (w.cpu.p & ZERO == 0, 0xf449),
                    0xf4c3 => (w.cpu.p & ZERO == 0, 0xf4a5),
                    0xf540 => (w.cpu.p & ZERO == 0, 0xf511),
                    0xf40c => (w.cpu.p & CARRY == 0, 0xf418),
                    _ => (w.cpu.p & CARRY == 0, 0xf4da),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0xf403 | 0xf4c5 => {
                w.dp_store(0, 0, 2);
                pc + 2
            }
            0xf409 | 0xf4cb => {
                w.compare_immediate(16, 2);
                pc + 3
            }
            0xf41c | 0xf4e3 => {
                let v = w.dp_read(0, 2);
                w.adc(v);
                pc + 2
            }
            0xf426 | 0xf431 | 0xf4f6 | 0xf506 => {
                w.drain(24);
                let v = w.read(
                    if matches!(pc, 0xf426 | 0xf4f6) {
                        0x02f889
                    } else {
                        0x02f88b
                    },
                    2,
                );
                w.load(v, 2);
                pc + 4
            }
            0xf42a | 0xf4fa => {
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                if pc == 0xf42a {
                    0xf435
                } else {
                    0xf50a
                }
            }
            0xf437 | 0xf4ed | 0xf4fd => {
                w.cpu.y = w.dp_read(14, 2);
                w.flags(w.cpu.y, 2);
                pc + 2
            }
            0xf43b | 0xf43f | 0xf45a | 0xf461 | 0xf46a | 0xf471 | 0xf4f1 | 0xf501 | 0xf522
            | 0xf529 | 0xf532 | 0xf539 => {
                let base = match pc {
                    0xf43f | 0xf461 | 0xf471 => 0x1142,
                    0xf529 | 0xf539 => 0x1140,
                    _ => 0x1100,
                };
                w.operand_then_internal(2);
                w.write(w.absolute(base + w.cpu.y), w.cpu.a, 2);
                pc + 3
            }
            0xf43e => {
                w.increment_a(2);
                pc + 1
            }
            0xf442 | 0xf443 | 0xf464 | 0xf465 | 0xf474 | 0xf475 | 0xf4b1 | 0xf4b2 | 0xf4bf
            | 0xf4c0 | 0xf4f4 | 0xf4f5 | 0xf504 | 0xf505 | 0xf52c | 0xf52d | 0xf53c | 0xf53d => {
                w.increment_y();
                pc + 1
            }
            0xf449 | 0xf511 => {
                w.cpu.x = w.dp_read(2, 2);
                w.flags(w.cpu.x, 2);
                pc + 2
            }
            0xf44b | 0xf513 => {
                w.drain(16);
                w.drain(6);
                let v = w.read(w.absolute(0x500 + w.cpu.x), 2);
                w.load(v, 2);
                pc + 3
            }
            0xf450 | 0xf518 => {
                w.dp_store(2, w.cpu.x, 2);
                pc + 2
            }
            0xf456 | 0xf45d | 0xf466 | 0xf46d | 0xf51e | 0xf525 | 0xf52e | 0xf535 => {
                let offset = match pc {
                    0xf45d | 0xf52e => 2,
                    0xf466 | 0xf525 => 4,
                    0xf46d | 0xf535 => 6,
                    _ => 0,
                };
                w.drain(24);
                let v = w.read(0x0f8000 + offset + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                pc + 4
            }
            0xf481 | 0xf549 => {
                w.rts();
                self.complete = w.cpu.sp == self.entry_sp + 2;
                w.cpu.pc
            }
            _ => unreachable!("overworld stripe PC {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
/// Direction backtracks and half-strip indices, followed by asset 70's bytes.
pub(crate) fn table_byte(address: u32, map8: &[u8]) -> Option<u8> {
    const STRIP_DATA: [u8; 10] = [0xd0, 3, 0x10, 4, 0x10, 0xf4, 0, 0, 0x20, 0];
    if (0x02f883..0x02f88d).contains(&address) {
        Some(STRIP_DATA[(address - 0x02f883) as usize])
    } else {
        address
            .checked_sub(0x0f8000)
            .and_then(|offset| map8.get(offset as usize).copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stripes_match_source_wram_rotation_map8_stores_and_nested_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let map8 = &rom[0x78000..0x80000];
        for i in 0..10 {
            assert_eq!(
                table_byte(0x02f883 + i, map8),
                Some(rom[0x17883 + i as usize])
            );
        }
        let mut instructions = 0;
        let mut visited = std::collections::BTreeSet::new();
        for dir in [1u8, 2, 4, 5, 6, 8, 9, 10] {
            let direction = if dir < 4 {
                MapScrollStripe::Horizontal
            } else {
                MapScrollStripe::Vertical
            };
            for pos in [0u16, 0x3d0, 0x13cf, 0x140f, 0xe3cf, 0xe40f, 0xd40f, 0xffff] {
                for case in 0..32usize {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    for i in 0x2000..0x12010 {
                        machine.ram[i] = ((i ^ (i >> 7) ^ case) & 1) as u8;
                    }
                    for (a, v) in [
                        (0x84, pos),
                        (0x86, case as u16),
                        (0x88, (31 - case) as u16),
                        (0x0e, if case & 16 == 0 { 2 } else { 262 }),
                    ] {
                        machine.ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
                    }
                    machine.ram[0x416] = dir;
                    let returned = if direction == MapScrollStripe::Horizontal {
                        0xf372u16
                    } else {
                        0xf300
                    };
                    machine.ram[0x1f3..0x1f5].copy_from_slice(&(returned - 1).to_le_bytes());
                    let mut cpu = NativeNmiContext {
                        a: 0x1111,
                        x: 12,
                        y: 266,
                        d: 0,
                        db: [0, 2, 0x80, 0x82][case % 4],
                        bank: 2,
                        pc: if direction == MapScrollStripe::Horizontal {
                            0xf3b9
                        } else {
                            0xf482
                        },
                        sp: 0x1f2,
                        p: 0x43,
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.db = cpu.db;
                    machine.cpu.k = 2;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let clock = CpuMasterTimeline::at_raster(
                        4937,
                        snes::CpuRasterPosition::new(224, 1300),
                        snes::CpuBusWorkload::default(),
                        snes::CpuFieldTiming::NON_INTERLACE_EVEN,
                    );
                    let mut source = snes::RomCpuTimingProbe::new(
                        machine,
                        clock,
                        snes::SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    let mut clock = source.timeline().clone();
                    let mut work = OverworldStripes::new(direction, &cpu);
                    for _ in 0..5000 {
                        if work.complete() {
                            break;
                        }
                        visited.insert(cpu.pc);
                        super::super::native_work::compare_instruction_with_devices(
                            &format!(
                                "stripes dir={dir} pos={pos:04x} case={case} pc={:04x}",
                                cpu.pc
                            ),
                            &mut cpu,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            None,
                            |_, access| match access {
                                super::super::intro_clear::Access::Read(address, width)
                                    if address & 0x8000 != 0 && address < 0x7e0000 =>
                                {
                                    let read = |a| {
                                        table_byte(a, map8)
                                            .expect("only immutable strip and map8 tables")
                                    };
                                    Some(if width == 1 {
                                        u16::from(read(address))
                                    } else {
                                        u16::from_le_bytes([read(address), read(address + 1)])
                                    })
                                }
                                _ => None,
                            },
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        work = work.clone();
                        instructions += 1;
                    }
                    assert!(work.complete());
                    assert_eq!(ram, source.snes().ram);
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, returned, 0x1f4));
                    assert_eq!(
                        u16::from_le_bytes([ram[14], ram[15]]),
                        if case & 16 == 0 { 2 } else { 262 }
                            + if direction == MapScrollStripe::Horizontal {
                                264
                            } else {
                                260
                            }
                    );
                }
            }
        }
        for pc in [
            0xf3c3, 0xf3db, 0xf40e, 0xf42a, 0xf481, 0xf48d, 0xf4a5, 0xf4d0, 0xf4fa, 0xf549,
        ] {
            assert!(visited.contains(&pc));
        }
        eprintln!("2048 native stripe builders, {instructions} Source instruction boundaries, {} distinct PCs; no behavioral ROM reads",visited.len());
    }
}
