//! Item-receipt OAM body with real XY helper calls and the slot-handler RTS.
use super::{
    ancilla_main::AncillaMain,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReceiptOam {
    xy: Option<AncillaMain>,
    xy_return: u16,
    return_pc: u16,
    complete: bool,
}
impl Default for ReceiptOam {
    fn default() -> Self {
        Self::new(0x8338)
    }
}
impl ReceiptOam {
    pub(crate) fn new(return_pc: u16) -> Self {
        Self {
            xy: None,
            xy_return: 0,
            return_pc,
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
        if let Some(xy) = &mut self.xy {
            let cycles = xy.step(cpu, clock, bus);
            if xy.complete() {
                assert_eq!((cpu.bank, cpu.pc), (8, self.xy_return));
                self.xy = None;
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.db, cpu.d, cpu.p & 0x18), (8, 8, 0, 0x10));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = w.cpu.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        w.fetch();
        let next = match pc {
            0xc690 | 0xc6b2 | 0xc6e3 => {
                w.drain(6);
                w.push(if pc == 0xc690 { w.cpu.x } else { w.cpu.y }, 1);
                pc + 1
            }
            0xc691 | 0xc696 => {
                w.drain(16);
                let v = w.read(
                    w.absolute(if pc == 0xc691 { 0xbf0 } else { 0xc5e } + w.cpu.x),
                    1,
                );
                w.load(v, 1);
                pc + 3
            }
            0xc694 => {
                w.dp_store(0x74, w.cpu.a & 255, 1);
                pc + 2
            }
            0xc699 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0xc69a => {
                w.drain(8);
                w.cpu.y = 0;
                w.flags(0, 1);
                pc + 2
            }
            0xc69c | 0xc6cd => {
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                self.xy = Some(AncillaMain::set_oam_xy());
                self.xy_return = pc + 3;
                0xf6e1
            }
            0xc69f | 0xc6d0 | 0xc6eb => {
                w.immediate(
                    match pc {
                        0xc69f => 0x24,
                        0xc6d0 => 0x34,
                        _ => 0,
                    },
                    1,
                );
                pc + 2
            }
            0xc6a1 | 0xc6af | 0xc6be | 0xc6d2 | 0xc6e0 | 0xc6ed => {
                w.drain(8);
                let pointer = w.read(
                    if matches!(pc, 0xc6be | 0xc6ed) {
                        0x92
                    } else {
                        0x90
                    },
                    2,
                );
                w.drain(6);
                w.write(w.absolute(pointer) + u32::from(w.cpu.y), w.cpu.a & 255, 1);
                pc + 2
            }
            0xc6a3 | 0xc6b1 | 0xc6d4 | 0xc6e2 => {
                w.drain(6);
                w.cpu.y = (w.cpu.y + 1) & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0xc6a4 | 0xc6ba | 0xc6d5 => {
                w.drain(24);
                let v = w.read(
                    if pc == 0xc6ba { 0x098450 } else { 0x09849c } + u32::from(w.cpu.x),
                    1,
                );
                w.load(v, 1);
                pc + 4
            }
            0xc6a8 | 0xc6d9 | 0xc6c3 => {
                let take = if pc == 0xc6c3 {
                    w.cpu.p & ZERO != 0
                } else {
                    w.cpu.p & NEGATIVE == 0
                };
                w.branch(take);
                if take {
                    match pc {
                        0xc6a8 => 0xc6ac,
                        0xc6d9 => 0xc6dd,
                        _ => 0xc6f0,
                    }
                } else {
                    pc + 2
                }
            }
            0xc6aa | 0xc6db | 0xc6c7 => {
                let v = w.dp_read(if pc == 0xc6c7 { 8 } else { 0x74 }, m);
                w.load(v, m);
                pc + 2
            }
            0xc6ac | 0xc6dd => {
                w.shift_left(1);
                pc + 1
            }
            0xc6ad | 0xc6de => {
                w.drain(8);
                w.load((w.cpu.a & 255) | 0x30, 1);
                pc + 2
            }
            0xc6b3 | 0xc6e4 => {
                w.drain(6);
                w.load(w.cpu.y, 1);
                pc + 1
            }
            0xc6b4 | 0xc6e5 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xc6b5 | 0xc6e6 => {
                w.drain(8);
                w.sbc(4, 1);
                pc + 2
            }
            0xc6b7 | 0xc6b8 | 0xc6e8 | 0xc6e9 => {
                w.drain(6);
                let v = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, 1);
                pc + 1
            }
            0xc6b9 | 0xc6ea => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0xc6c0 | 0xc6ef | 0xc6f0 => {
                w.drain(12);
                let v = w.pull(1);
                if pc == 0xc6f0 {
                    w.cpu.x = v;
                } else {
                    w.cpu.y = v;
                }
                w.flags(v, 1);
                pc + 1
            }
            0xc6c1 => {
                w.compare_immediate(2, 1);
                pc + 2
            }
            0xc6c5 | 0xc6cb => {
                w.width(0x20, pc == 0xc6c5);
                pc + 2
            }
            0xc6c9 => {
                w.dp_store(0, w.cpu.a, 2);
                pc + 2
            }
            0xc6f1 => {
                w.rts();
                assert_eq!(w.cpu.pc, self.return_pc);
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("receipt OAM pc={pc:04x}"),
        };
        w.cpu.pc = next;
        Some(w.cycles)
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    let k = case % 10;
    let v = case / 10;
    r[0xc5e + k] = (v % 76) as u8;
    r[0xbf0 + k] = [0, 1, 4, 5, 128, 255][v / 76 % 6];
    let x = [0u16, 239, 255, 256, 0xffff][v / 456 % 5];
    let y = [0u16, 231, 239, 240, 255, 256, 0xffff][v / 2280 % 7];
    for (a, pos) in [
        (0, y),
        (2, x),
        (8, y.wrapping_add(8)),
        (0x90, 0x800 + ((v / 15960 % 32) * 8) as u16),
        (0x92, 0xa20 + (v / 15960 % 32) as u16 * 2),
    ] {
        r[a..a + 2].copy_from_slice(&pos.to_le_bytes());
    }
    r[0x1e9..0x1eb].copy_from_slice(&0x8337u16.to_le_bytes());
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receipt_oam_matches_source_all_items_flags_clipping_sizes_and_real_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;

        let mut visited = std::collections::BTreeSet::new();
        for case in 0..159600usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: (case % 10) as u16,
                y: 7,
                d: 0,
                db: 8,
                bank: 8,
                pc: 0xc690,
                sp: 0x1e8,
                p: 0x30 | (case as u8 & 0xc3),
            };
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
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(60, 1252),
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
            let mut work = ReceiptOam::default();
            while !work.complete() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("receipt OAM case={case} pc={pc:04x}"),
                    &mut cpu,
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
            }

            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [
            0x08c690, 0x08c694, 0x08c69c, 0x08c6aa, 0x08c6be, 0x08c6c5, 0x08c6cd, 0x08c6db,
            0x08c6ed, 0x08c6f0, 0x08c6f1, 0x08f6e1, 0x08f6f6, 0x08f6f7, 0x08f6fd,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        println!("native receipt OAM: 159600 cases, {count} source instructions, {} PCs, all76 items and one/two tiles and clipping returns",visited.len());
    }
    #[test]
    fn receipt_oam_flag_table_contains_only_76_data_bytes() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x09849c..=0x0984e7 {
            assert_eq!(
                crate::ZeldaState::native_ancilla_oam_table_byte(a),
                Some(rom[9 * 0x8000 + (a as usize & 0x7fff)])
            );
        }
        for a in [0x09849b, 0x0984e8] {
            assert_eq!(crate::ZeldaState::native_ancilla_oam_table_byte(a), None);
        }
    }
}
