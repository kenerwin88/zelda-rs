//! Erase-menu selection and confirmation. Live slot flags, stripe writes and
//! SRAM clearing share the retained caller stack, including drawing children.
use super::{
    file_select_draw::FileSelectDraw,
    intro_clear::Access,
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct KillFile {
    pc: u16,
    child: Option<FileSelectDraw>,
    complete: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erase_menu_native_instructions_match_source_bus_and_registers() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for (entry, cursors) in [(0xd49f, 4), (0xd4b1, 2)] {
            for cursor in 0..cursors {
                for input in [0, 4, 8, 0x20, 0x80] {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    machine.ram[0xc8] = cursor;
                    machine.ram[0xb0] = 1;
                    machine.ram[0xf4] = input;
                    for slot in 0..3 {
                        machine.ram[0xbf + slot * 2] = 1;
                        machine.cart.ram[slot * 0x500 + 0x36c] = (3 + slot * 8) as u8 * 8;
                    }
                    machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
                    let mut cpu = NativeNmiContext {
                        a: 0xab03,
                        x: 224,
                        y: 3,
                        d: 0,
                        db: 0,
                        bank: 12,
                        pc: entry,
                        sp: 0x1fc,
                        p: 0x31,
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let mut sram = machine.cart.ram.clone();
                    let timeline = snes::CpuMasterTimeline::at_raster(
                        30_002,
                        snes::CpuRasterPosition::new(100, 500),
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
                    let mut work = KillFile::new(entry);
                    for _ in 0..100_000 {
                        if work.complete() {
                            break;
                        }
                        let label = format!(
                            "entry={entry:04x} cursor={cursor} input={input:02x} pc={:04x}",
                            cpu.pc
                        );
                        if std::env::var_os("ZELDA3_DEBUG_KILL_FILE_INSTRUCTIONS").is_some() {
                            eprintln!("{label}");
                        }
                        super::super::native_work::compare_instruction_with_devices(
                            &label,
                            &mut cpu,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            None,
                            |_, access| match access {
                                Access::Read(a, width) if (0x700000..0x702000).contains(&a) => {
                                    let at = (a - 0x700000) as usize;
                                    Some(if width == 1 {
                                        u16::from(sram[at])
                                    } else {
                                        u16::from_le_bytes([sram[at], sram[at + 1]])
                                    })
                                }
                                Access::Write(a, v, width) if (0x700000..0x702000).contains(&a) => {
                                    let at = (a - 0x700000) as usize;
                                    sram[at..at + usize::from(width)]
                                        .copy_from_slice(&v.to_le_bytes()[..usize::from(width)]);
                                    Some(0)
                                }
                                _ => None,
                            },
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        assert_eq!(sram, source.snes().cart.ram, "{label}");
                    }
                    assert!(work.complete());
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (0, 0x805a, 0x1ff));
                }
            }
        }
    }
}
impl KillFile {
    pub(crate) fn new(pc: u16) -> Self {
        assert!(matches!(pc, 0xd49f | 0xd4b1));
        Self {
            pc,
            child: None,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        self.child
            .as_ref()
            .map_or(0x0c0000 | u32::from(self.pc), |w| w.routine())
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
        if let Some(child) = self.child.as_mut() {
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                self.child = None;
                assert_eq!((cpu.bank, cpu.pc), (12, self.pc));
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (12, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let xm = if x == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match self.pc {
            0xd49f | 0xd4a0 | 0xd4b1 | 0xd4b2 => {
                w.drain(6);
                w.push(
                    u16::from(if matches!(self.pc, 0xd4a0 | 0xd4b2) {
                        w.cpu.bank
                    } else {
                        w.cpu.db
                    }),
                    1,
                );
                self.pc + 1
            }
            0xd4a1 | 0xd4b3 | 0xd0a0 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xd4a2 | 0xd4fa | 0xd500 | 0xd54a | 0xd559 | 0xd589 | 0xd599 | 0xd5b0 | 0xd5c8
            | 0xd5d7 | 0xd5e5 => {
                let address = match self.pc {
                    0xd4fa | 0xd500 | 0xd5b0 => 0xf4,
                    0xd54a | 0xd5c8 => 0xf6,
                    0xd599 | 0xd5e5 => 0xb0,
                    _ => 0xc8,
                };
                let v = w.dp_read(address, m);
                w.load(v, m);
                self.pc + 2
            }
            0xd4a4 | 0xd55b => {
                w.compare_immediate(3, m);
                self.pc + 1 + u16::from(m)
            }
            0xd4a6 | 0xd4c6 | 0xd4d4 | 0xd4e5 | 0xd4fe | 0xd504 | 0xd508 | 0xd514 | 0xd51c
            | 0xd51f | 0xd523 | 0xd52f | 0xd537 | 0xd53c | 0xd53e | 0xd542 | 0xd546 | 0xd552
            | 0xd55d | 0xd568 | 0xd570 | 0xd58f | 0xd5b4 | 0xd5b8 | 0xd5bb | 0xd5d0 | 0xd5d9
            | 0xd626 => {
                let (take, target) = match self.pc {
                    0xd4a6 => (w.cpu.p & CARRY != 0, 0xd4ab),
                    0xd4c6 => (w.cpu.p & ZERO == 0, 0xd4bf),
                    0xd4d4 => (w.cpu.p & ZERO != 0, 0xd4de),
                    0xd4e5 => (w.cpu.p & CARRY == 0, 0xd4cd),
                    0xd4fe => (w.cpu.p & ZERO == 0, 0xd525),
                    0xd504 => (w.cpu.p & ZERO != 0, 0xd548),
                    0xd508 => (w.cpu.p & ZERO == 0, 0xd525),
                    0xd514 => (w.cpu.p & NEGATIVE != 0, 0xd521),
                    0xd51c | 0xd537 | 0xd542 => (w.cpu.p & ZERO == 0, 0xd548),
                    0xd51f => (w.cpu.p & NEGATIVE == 0, 0xd516),
                    0xd523 | 0xd53e => (true, 0xd548),
                    0xd52f => (w.cpu.p & CARRY != 0, 0xd540),
                    0xd53c => (w.cpu.p & ZERO == 0, 0xd531),
                    0xd546 => (true, 0xd531),
                    0xd552 => (w.cpu.p & ZERO != 0, 0xd596),
                    0xd55d => (w.cpu.p & ZERO != 0, 0xd591),
                    0xd568 => (w.cpu.p & NEGATIVE == 0, 0xd561),
                    0xd570 => (w.cpu.p & ZERO != 0, 0xd589),
                    0xd58f => (true, 0xd596),
                    0xd5b4 => (w.cpu.p & ZERO != 0, 0xd5c8),
                    0xd5b8 => (w.cpu.p & ZERO == 0, 0xd5bd),
                    0xd5bb => (true, 0xd5be),
                    0xd5d0 => (w.cpu.p & ZERO != 0, 0xd62f),
                    0xd5d9 => (w.cpu.p & ZERO == 0, 0xd62a),
                    _ => (w.cpu.p & ZERO == 0, 0xd5f7),
                };
                if w.branch(take) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xd4a8 | 0xd50c | 0xd527 | 0xd556 | 0xd5c5 | 0xd5d4 | 0xd5dd | 0xd5e0 => {
                let address = match self.pc {
                    0xd4a8 => 0xb9d,
                    0xd556 | 0xd5d4 | 0xd5e0 => 0x12e,
                    _ => 0x12f,
                };
                w.drain(16);
                w.write(
                    w.absolute(address),
                    if self.pc == 0xd5e0 { 0 } else { w.cpu.a & 255 },
                    1,
                );
                self.pc + 3
            }
            0xd4ab | 0xd4b4 | 0xd4db | 0xd4f5 | 0xd593 | 0xd5ab | 0xd62a => {
                let target = match self.pc {
                    0xd4ab => 0xd4ba,
                    0xd4b4 => 0xd599,
                    0xd4db => 0xd63c,
                    0xd4f5 | 0xd5ab => 0xd7a5,
                    _ => 0xd22d,
                };
                w.drain(22);
                w.push(self.pc + 2, 2);
                if matches!(target, 0xd63c | 0xd7a5) {
                    self.child = Some(FileSelectDraw::new(target));
                    w.cpu.pc = target;
                    self.pc + 3
                } else {
                    target
                }
            }
            0xd4ae | 0xd4b7 => {
                w.drain(16);
                0xd09c
            }
            0xd4ba | 0xd4c8 | 0xd4e7 | 0xd591 | 0xd59e | 0xd5e3 | 0xd628 => {
                w.width(
                    if self.pc == 0xd4ba {
                        0x10
                    } else if self.pc == 0xd4c8 {
                        0x20
                    } else {
                        0x30
                    },
                    matches!(self.pc, 0xd4ba | 0xd4c8 | 0xd5e3),
                );
                self.pc + 2
            }
            0xd4bc | 0xd4ca | 0xd4f8 | 0xd50f | 0xd521 | 0xd544 | 0xd55f | 0xd5ae | 0xd5f3 => {
                let v = match self.pc {
                    0xd4bc => 253,
                    0xd4f8 | 0xd5ae => 2,
                    0xd50f => 254,
                    0xd521 => 3,
                    0xd55f => 100,
                    _ => 0,
                };
                w.drain(8 * u32::from(x));
                if matches!(self.pc, 0xd4f8 | 0xd50f | 0xd5ae | 0xd5f3) {
                    w.cpu.y = v
                } else {
                    w.cpu.x = v
                }
                w.flags(v, x);
                self.pc + 1 + u16::from(x)
            }
            0xd4bf | 0xd4eb | 0xd4f0 | 0xd561 | 0xd572 | 0xd5a6 => {
                let base = match self.pc {
                    0xd4bf => 0xe53e,
                    0xd4eb => 0xd416,
                    0xd4f0 => 0xd41a,
                    0xd561 => 0xd41e,
                    0xd572 => 0xd483,
                    _ => 0xd597,
                };
                w.drain(16);
                if x == 2 || (base & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(base) + u32::from(w.cpu.x), m);
                w.load(v, m);
                self.pc + 3
            }
            0xd4c2 | 0xd564 | 0xd578 | 0xd57b | 0xd580 | 0xd586 => {
                let base = match self.pc {
                    0xd4c2 => 0x1001,
                    0xd57b => 0x1008,
                    0xd580 => 0x1003,
                    0xd586 => 0x1009,
                    _ => 0x1002,
                };
                w.drain(22);
                w.write(w.absolute(base) + u32::from(w.cpu.x), w.cpu.a & 255, 1);
                self.pc + 3
            }
            0xd4c5 | 0xd513 | 0xd51e | 0xd567 | 0xd5ba => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & xm;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xd4e0 | 0xd4e1 | 0xd52c | 0xd539 | 0xd5bd | 0xd61f | 0xd620 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & xm;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xd621 | 0xd622 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & xm;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xd4cd | 0xd548 => {
                w.dp_store(if self.pc == 0xd4cd { 0 } else { 0xc8 }, w.cpu.x, x);
                self.pc + 2
            }
            0xd4cf => {
                w.drain(14);
                let v = w.read(u32::from(0xbfu16.wrapping_add(w.cpu.x)), m);
                w.load(v, m);
                self.pc + 2
            }
            0xd4d1 | 0xd4fc | 0xd502 | 0xd506 | 0xd54c | 0xd550 | 0xd5b2 | 0xd5b6 | 0xd5bf
            | 0xd5ca | 0xd5ce | 0xd5e7 => {
                let value = match self.pc {
                    0xd4d1 | 0xd5bf => 1,
                    0xd4fc => 0x20,
                    0xd502 => 12,
                    0xd506 => 4,
                    0xd54c | 0xd5ca => 0xc0,
                    0xd550 | 0xd5ce => 0xd0,
                    0xd5b2 => 0x2c,
                    0xd5b6 => 0x24,
                    _ => 255,
                };
                w.mask(value, m);
                self.pc + 1 + u16::from(m)
            }
            0xd4d6 | 0xd5ee => {
                w.drain(24);
                let v = w.read(0x848c + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                self.pc + 4
            }
            0xd4da | 0xd575 | 0xd5eb | 0xd5f2 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & xm;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xd4de | 0xd4e9 | 0xd511 | 0xd52a | 0xd56c | 0xd5a0 => {
                let value = w.dp_read(if self.pc == 0xd4de { 0 } else { 0xc8 }, x);
                w.cpu.x = value;
                w.flags(value, x);
                self.pc + 2
            }
            0xd4e2 | 0xd52d | 0xd53a | 0xd540 | 0xd56e | 0xd623 => {
                let v = match self.pc {
                    0xd4e2 => 6,
                    0xd540 => 4,
                    0xd56e => 2,
                    0xd623 => 0x100,
                    _ => 3,
                };
                w.drain(8 * u32::from(x));
                let value = if self.pc == 0xd623 { w.cpu.y } else { w.cpu.x };
                let result = value.wrapping_sub(v) & xm;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(value >= v);
                w.flags(result, x);
                self.pc + 1 + u16::from(x)
            }
            0xd4ee | 0xd4f3 | 0xd58b | 0xd59c | 0xd5a4 | 0xd5a9 | 0xd5c1 | 0xd09e | 0xd22f
            | 0xd233 => {
                let a = match self.pc {
                    0xd4ee | 0xd59c | 0xd5a4 => 0,
                    0xd4f3 | 0xd5a9 => 1,
                    0xd58b => 0xb0,
                    0xd5c1 => 0xc8,
                    0xd09e => 0x14,
                    0xd22f => 0x10,
                    _ => 0x11,
                };
                w.dp_store(a, w.cpu.a & if m == 1 { 255 } else { 65535 }, m);
                self.pc + 2
            }
            0xd50a | 0xd525 | 0xd554 | 0xd576 | 0xd57e | 0xd5a2 | 0xd5c3 | 0xd5d2 | 0xd5db
            | 0xd09c | 0xd22d | 0xd231 => {
                let v = match self.pc {
                    0xd50a | 0xd525 | 0xd5c3 => 0x20,
                    0xd554 | 0xd5d2 => 0x2c,
                    0xd576 => 0x62,
                    0xd57e => 0x67,
                    0xd5a2 => 0x1c,
                    0xd5db => 0x22,
                    _ => 1,
                };
                w.immediate(v, m);
                self.pc + 1 + u16::from(m)
            }
            0xd516 | 0xd531 | 0xd5be | 0xd5f6 => {
                w.drain(6);
                w.load(if self.pc == 0xd5f6 { w.cpu.y } else { w.cpu.x }, m);
                self.pc + 1
            }
            0xd517 | 0xd532 | 0xd59b | 0xd5ea => {
                w.shift_left(m);
                self.pc + 1
            }
            0xd518 | 0xd533 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & xm;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xd519 | 0xd534 => {
                w.drain(16);
                if x == 2 || 0xbf + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(0xbf) + u32::from(w.cpu.y), m);
                w.load(v, m);
                self.pc + 3
            }
            0xd54e | 0xd5cc => {
                let v = w.dp_read(0xf4, m);
                w.load((w.cpu.a | v) & if m == 1 { 255 } else { 65535 }, m);
                self.pc + 2
            }
            0xd56a => {
                w.dp_increment(0x11, 1, true);
                self.pc + 2
            }
            0xd583 => {
                w.drain(6);
                w.cpu.p &= !CARRY;
                self.pc + 1
            }
            0xd584 => {
                w.drain(8);
                w.adc_byte(0x20);
                self.pc + 2
            }
            0xd58d | 0xd62d | 0xd235 | 0xd237 => {
                w.dp_store(
                    if matches!(self.pc, 0xd62d | 0xd235) {
                        0xb0
                    } else {
                        0xc8
                    },
                    0,
                    1,
                );
                self.pc + 2
            }
            0xd5ec => {
                w.drain(14);
                w.write(u32::from(0xbfu16.wrapping_add(w.cpu.x)), 0, 2);
                self.pc + 2
            }
            0xd5f7 | 0xd5fb | 0xd5ff | 0xd603 | 0xd607 | 0xd60b | 0xd60f | 0xd613 | 0xd617
            | 0xd61b => {
                let word = (self.pc - 0xd5f7) / 4;
                let page = if word < 5 { word } else { word + 10 };
                w.drain(24);
                w.write(
                    0x700000 + u32::from(page) * 0x100 + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0xd596 | 0xd62f | 0xd239 => {
                w.rts();
                w.cpu.pc
            }
            0xd0a1 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("erase-menu continuation {:04x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}
