//! Retained two-slot push-block drawing, including the shared OAM allocation
//! and the real index saves and returns on the caller's stack.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
    oam_allocate::OamAllocate,
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    crate::ZeldaState::native_push_block_drawing_table_byte(address)
        .or_else(|| crate::ZeldaState::native_oam_table_byte(address))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PushBlockDrawing {
    pc: u16,
    allocator: Option<OamAllocate>,
    complete: bool,
}
impl Default for PushBlockDrawing {
    fn default() -> Self {
        Self {
            pc: 0xf0ac,
            allocator: None,
            complete: false,
        }
    }
}
impl PushBlockDrawing {
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
        if let Some(allocator) = self.allocator.as_mut() {
            let cycles = allocator.step(cpu, clock, bus);
            if allocator.complete() {
                assert_eq!((cpu.bank, cpu.pc), (7, 0xf0e0));
                self.allocator = None;
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x18),
            (7, self.pc, 0, 0x10)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match pc {
            0xf0ac | 0xf0ad => {
                w.drain(6);
                w.push(
                    u16::from(if pc == 0xf0ac { w.cpu.db } else { w.cpu.bank }),
                    1,
                );
                pc + 1
            }
            0xf0ae | 0xf0c9 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xf0ca => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0xf0af | 0xf0b2 | 0xf0b9 | 0xf0e1 | 0xf0e6 | 0xf0eb | 0xf0f0 | 0xf109 | 0xf10c
            | 0xf112 => {
                let (base, index) = match pc {
                    0xf0af => (0x05fc, 0),
                    0xf0b2 => (0x05fd, 0),
                    0xf0b9 => (0x05fc, w.cpu.x),
                    0xf0e1 => (0x05f0, w.cpu.y),
                    0xf0e6 => (0x05ec, w.cpu.y),
                    0xf0eb => (0x05e4, w.cpu.y),
                    0xf0f0 => (0x05e0, w.cpu.y),
                    0xf109 => (0x02c3, 0),
                    0xf10c => (0xf0d0, w.cpu.y),
                    _ => (0xf0cc, w.cpu.x),
                };
                w.drain(16);
                if (base & 255) + index > 255 {
                    w.drain(6);
                }
                let value = w.read(
                    w.absolute(base) + u32::from(index),
                    if pc == 0xf109 { 1 } else { m },
                );
                if pc == 0xf109 {
                    w.cpu.y = value;
                    w.flags(value, 1);
                } else {
                    w.load(
                        if pc == 0xf0b2 {
                            (w.cpu.a | value) & mask
                        } else {
                            value
                        },
                        m,
                    );
                }
                pc + 3
            }
            0xf0b5 | 0xf0bc | 0xf0c7 | 0xf117 | 0xf119 => {
                let (take, target) = match pc {
                    0xf0b5 => (w.cpu.p & ZERO != 0, 0xf0c9),
                    0xf0bc => (w.cpu.p & ZERO != 0, 0xf0c6),
                    0xf0c7 => (w.cpu.p & NEGATIVE == 0, 0xf0b9),
                    0xf117 => (w.cpu.p & ZERO == 0, 0xf11b),
                    _ => (true, 0xf13a),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0xf0b7 | 0xf110 => {
                w.drain(8);
                if pc == 0xf0b7 {
                    w.cpu.x = 1;
                } else {
                    w.cpu.y = 0;
                }
                w.flags(u16::from(pc == 0xf0b7), 1);
                pc + 2
            }
            0xf0be | 0xf12f => {
                w.drain(6);
                w.load(if pc == 0xf0be { w.cpu.x } else { w.cpu.y }, m);
                pc + 1
            }
            0xf0bf => {
                w.shift_left(m);
                pc + 1
            }
            0xf0c0 | 0xf10f | 0xf135 => {
                w.drain(6);
                let value = w.cpu.a & 255;
                if pc == 0xf10f {
                    w.cpu.x = value;
                } else {
                    w.cpu.y = value;
                }
                w.flags(value, 1);
                pc + 1
            }
            0xf0c1 | 0xf0d9 | 0xf108 => {
                w.drain(6);
                w.push(if pc == 0xf0c1 { w.cpu.x } else { w.cpu.y }, 1);
                pc + 1
            }
            0xf0c5 | 0xf0e0 | 0xf13a => {
                w.drain(12);
                let value = w.pull(1);
                if pc == 0xf0c5 {
                    w.cpu.x = value;
                } else {
                    w.cpu.y = value;
                }
                w.flags(value, 1);
                pc + 1
            }
            0xf0c2 => {
                w.operand_then_internal(2);
                w.push(0xf0c4, 2);
                0xf0d9
            }
            0xf13b => {
                w.rts();
                assert_eq!(w.cpu.pc, 0xf0c5);
                w.cpu.pc
            }
            0xf0c6 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0xf0da | 0xf12a | 0xf136 => {
                w.drain(8 * u32::from(m));
                w.load(
                    match pc {
                        0xf0da => 4,
                        0xf12a => 0x20,
                        _ => 2,
                    },
                    m,
                );
                pc + 2
            }
            0xf0dc => {
                w.operand_then_internal(3);
                w.push(7, 1);
                w.push(0xf0df, 2);
                w.cpu.bank = 13;
                w.cpu.pc = 0xba84;
                self.allocator = Some(OamAllocate::new(0xba84));
                0xf0e0
            }
            0xf0e4 | 0xf0e9 | 0xf0ee | 0xf0f3 | 0xf0fd | 0xf104 => {
                let at = match pc {
                    0xf0e4 | 0xf0fd => 0,
                    0xf0e9 => 1,
                    0xf0ee | 0xf104 => 2,
                    _ => 3,
                };
                w.dp_store(at, w.cpu.a & mask, m);
                pc + 2
            }
            0xf0f5 | 0xf106 => {
                w.width(0x20, pc == 0xf0f5);
                pc + 2
            }
            0xf0f7 | 0xf0ff | 0xf11c | 0xf121 => {
                let value = w.dp_read(if matches!(pc, 0xf0f7 | 0xf121) { 0 } else { 2 }, m);
                w.load(value, m);
                pc + 2
            }
            0xf0f9 | 0xf101 | 0xf130 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xf0fa | 0xf102 => {
                let value = w.dp_read(if pc == 0xf0fa { 0xe8 } else { 0xe2 }, m);
                w.sbc(value, m);
                pc + 2
            }
            0xf0fc => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1), m);
                pc + 1
            }
            0xf115 => {
                w.compare_immediate(255, m);
                pc + 2
            }
            0xf11b | 0xf126 => {
                w.swap_bytes();
                pc + 1
            }
            0xf11e | 0xf123 | 0xf127 | 0xf12c | 0xf138 => {
                // STA (dp),Y uses a two-byte pointer in DB, unlike [dp],Y.
                w.drain(8);
                let pointer = w.read(if pc == 0xf138 { 0x92 } else { 0x90 }, 2);
                w.drain(6);
                w.write(w.absolute(pointer) + u32::from(w.cpu.y), w.cpu.a & mask, m);
                pc + 2
            }
            0xf120 | 0xf125 | 0xf129 | 0xf12e => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0xf131 => {
                w.drain(8);
                w.sbc(4, m);
                pc + 2
            }
            0xf133 | 0xf134 => {
                w.drain(6);
                let value = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (value & 1) as u8;
                w.load(value >> 1, m);
                pc + 1
            }
            _ => unreachable!("push-block drawing {pc:04x}"),
        };
        if self.allocator.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{intro_clear::Access, native_work::compare_instruction};
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn push_block_drawing_matches_source_slots_animation_and_allocator_overflow() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut visited = BTreeSet::new();
        let mut instructions = 0;
        for case in 0..1152usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(37).wrapping_add(case as u8);
            }
            let r = &mut machine.ram;
            r[0x5fc] = (case % 4 & 1) as u8;
            r[0x5fd] = (case % 4 & 2) as u8;
            r[0x2c3] = (case / 4 % 9) as u8;
            // Region B's limit is $0201. Exercise both sides of allocation
            // overflow without inventing an OAM pointer into unmapped I/O.
            let base: u16 = [0, 0x1f9, 0x1fc, 0x1fd, 0x1fe, 0x200, 0x201, 0x202]
                [case / 36 % 8];
            r[0xfe2..0xfe4].copy_from_slice(&base.to_le_bytes());
            r[0xfee..0xff0].copy_from_slice(&(case as u16).to_le_bytes());
            for (i, at) in [
                0x5e0, 0x5e2, 0x5e4, 0x5e6, 0x5ec, 0x5ee, 0x5f0, 0x5f2, 0xe2, 0xe8,
            ]
            .into_iter()
            .enumerate()
            {
                let value = (case as u16)
                    .wrapping_mul(997)
                    .wrapping_add((i * 257) as u16);
                r[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            r[0x1fd..0x200].copy_from_slice(&[0x3d, 0x88, 2]);
            let mut cpu = NativeNmiContext {
                a: 0xabcd,
                x: 3,
                y: 7,
                d: 0,
                db: if case & 1 == 0 { 0 } else { 2 },
                bank: 7,
                pc: 0xf0ac,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let original_db = cpu.db;
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.db = cpu.db;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut ram = machine.ram.clone();
            let timeline = CpuMasterTimeline::at_raster(
                500 + case as u64,
                snes::CpuRasterPosition::new(
                    [0, 224, 261, 261][case / 288],
                    [0, 1300, 1200, 1350][case / 288],
                ),
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
            let mut work = PushBlockDrawing::default();
            for _ in 0..1000 {
                if work.complete() {
                    break;
                }
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(cpu.pc));
                compare_instruction(
                    &format!(
                        "block drawing case={case} pc={:02x}:{:04x}",
                        cpu.bank, cpu.pc
                    ),
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
                                            "drawing table {a:06x}"
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
            assert_eq!(
                (cpu.bank, cpu.pc, cpu.sp, cpu.db),
                (2, 0x883e, 0x1ff, original_db)
            );
            assert_eq!(ram, source.snes().ram);
        }
        for pc in [
            0x07f0ac, 0x07f0c2, 0x07f0ca, 0x07f119, 0x07f11b, 0x07f138, 0x07f13b, 0x0dbb1e,
            0x0dbb41,
        ] {
            assert!(visited.contains(&pc), "uncovered drawing path {pc:06x}");
        }
        eprintln!("1152 push-block drawing fixtures, {instructions} instruction boundaries, {} distinct PCs",visited.len());
    }
}
