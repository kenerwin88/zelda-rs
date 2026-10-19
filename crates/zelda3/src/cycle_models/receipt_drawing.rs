//! Item receipt animation and real coordinate preparation before OAM drawing.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReceiptDrawingTail {
    Sparkle,
    Graphics { source: u8 },
    Draw,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReceiptDrawing {
    tail: Option<ReceiptDrawingTail>,
}
impl ReceiptDrawing {
    pub(crate) fn tail(&self) -> Option<ReceiptDrawingTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.db, cpu.d, cpu.p & 0x18), (8, 8, 0, 0x10));
        assert!(cpu.x < 10);
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
            0xc5f7 | 0xc613 | 0xf6d6 | 0xc68e | 0xf6c2 | 0xc684 => {
                w.width(0x20, matches!(pc, 0xf6c2 | 0xc684));
                pc + 2
            }
            0xc5f9 | 0xc615 | 0xc622 | 0xc629 | 0xc63a | 0xc64e | 0xc665 | 0xf6ae | 0xf6b3
            | 0xf6b8 | 0xf6bd => {
                let base = match pc {
                    0xc5f9 | 0xc615 | 0xc64e => 0xc5e,
                    0xc622 => 0xc54,
                    0xc629 => 0xc68,
                    0xc63a | 0xc665 => 0x3a4,
                    0xf6ae => 0xbfa,
                    0xf6b3 => 0xc0e,
                    0xf6b8 => 0xc04,
                    _ => 0xc18,
                };
                w.drain(16);
                let a = base + w.cpu.x;
                if base & 0xff00 != a & 0xff00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a), 1);
                w.load(v, 1);
                pc + 3
            }
            0xc61c | 0xc648 | 0xc672 | 0xc679 | 0xf6a7 => {
                let base = match pc {
                    0xc61c | 0xc648 => 0xc37b,
                    0xc672 => 0xc378,
                    0xc679 => 0xc375,
                    _ => 0xf66d,
                };
                let a = base + if pc == 0xc61c { 0 } else { w.cpu.y };
                w.drain(16);
                if base & 0xff00 != a & 0xff00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a), 1);
                w.load(v, 1);
                pc + 3
            }
            0xc5fc | 0xc618 | 0xc625 | 0xc62c | 0xc63e | 0xc651 | 0xc655 | 0xc659 | 0xc668 => {
                w.compare_immediate(
                    match pc {
                        0xc5fc => 0x20,
                        0xc618 => 1,
                        0xc625 => 2,
                        0xc62c => 16,
                        0xc63e | 0xc668 => 3,
                        0xc651 => 0x34,
                        0xc655 => 0x35,
                        _ => 0x36,
                    },
                    1,
                );
                pc + 2
            }
            0xc5fe | 0xc61a | 0xc627 | 0xc62e | 0xc633 | 0xc640 | 0xc653 | 0xc657 | 0xc65b
            | 0xc660 | 0xc66a => {
                let z = w.cpu.p & ZERO != 0;
                let (take, target) = match pc {
                    0xc5fe => (!z, 0xc613),
                    0xc61a | 0xc65b => (!z, if pc == 0xc61a { 0xc64e } else { 0xc681 }),
                    0xc627 => (z, 0xc64e),
                    0xc62e => (w.cpu.p & CARRY == 0, 0xc642),
                    0xc633 => (w.cpu.p & NEGATIVE == 0, 0xc64e),
                    0xc640 => (!z, 0xc644),
                    0xc653 | 0xc657 => (z, 0xc65d),
                    0xc660 => (w.cpu.p & NEGATIVE == 0, 0xc681),
                    _ => (!z, 0xc671),
                };
                w.branch(take);
                if take {
                    target
                } else {
                    pc + 2
                }
            }
            0xc600 | 0xc61f | 0xc637 | 0xc644 | 0xc64b | 0xc66e | 0xc675 => {
                let base = match pc {
                    0xc600 => 0x29e,
                    0xc61f | 0xc64b => 0xbf0,
                    0xc637 | 0xc675 => 0x39f,
                    _ => 0x3a4,
                };
                w.operand_then_internal(2);
                w.write(
                    w.absolute(base + w.cpu.x),
                    if pc == 0xc600 { 0 } else { w.cpu.a & 255 },
                    1,
                );
                pc + 3
            }
            0xc630 | 0xc65d | 0xc662 => {
                w.operand_then_internal(2);
                let a = w.absolute(if pc == 0xc662 { 0x3a4 } else { 0x39f } + w.cpu.x);
                let old = w.read(a, 1);
                let v = if pc == 0xc662 {
                    old.wrapping_add(1)
                } else {
                    old.wrapping_sub(1)
                } & 255;
                w.drain(6);
                w.flags(v, 1);
                w.write(a, v, 1);
                pc + 3
            }
            0xc635 | 0xc642 | 0xc66c => {
                w.immediate(if pc == 0xc635 { 2 } else { 0 }, 1);
                pc + 2
            }
            0xc63d => {
                w.increment_a(1);
                pc + 1
            }
            0xc647 | 0xc671 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0xc678 => {
                w.drain(6);
                w.push(w.cpu.x, 1);
                pc + 1
            }
            0xc603 | 0xc67c | 0xc681 => {
                let far = pc == 0xc67c;
                let width = if far { 3 } else { 2 };
                w.operand_then_internal(width);
                if far {
                    w.push(8, 1);
                }
                w.push(pc + u16::from(width), 2);
                if far {
                    w.cpu.bank = 0;
                    0xd4db
                } else if pc == 0xc603 {
                    0xcc93
                } else {
                    0xf6a4
                }
            }
            0xf6a4 => {
                w.drain(16);
                w.cpu.y = w.read(w.absolute(0xc7c + w.cpu.x), 1);
                w.flags(w.cpu.y, 1);
                pc + 3
            }
            0xf6aa | 0xf6ac | 0xf6b1 | 0xf6b6 | 0xf6bb | 0xf6c0 | 0xf6ca | 0xf6d2 | 0xf6d4
            | 0xc68c => {
                let a = match pc {
                    0xf6aa => 0x65,
                    0xf6ac => 0x64,
                    0xf6b1 | 0xf6ca => 0,
                    0xf6b6 => 1,
                    0xf6bb | 0xf6d2 => 2,
                    0xf6c0 => 3,
                    0xf6d4 => 4,
                    _ => 8,
                };
                w.dp_store(
                    a,
                    if pc == 0xf6ac {
                        0
                    } else {
                        w.cpu.a & if m == 1 { 255 } else { 65535 }
                    },
                    m,
                );
                pc + 2
            }
            0xf6c4 | 0xf6cc | 0xc686 => {
                let v = w.dp_read(if pc == 0xf6cc { 2 } else { 0 }, 2);
                w.load(v, 2);
                pc + 2
            }
            0xf6c6 | 0xf6ce | 0xc688 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(pc != 0xc688);
                pc + 1
            }
            0xf6c7 | 0xf6cf => {
                w.drain(16);
                let v = w.read(w.absolute(if pc == 0xf6c7 { 0x122 } else { 0x11e }), 2);
                w.sbc(v, 2);
                pc + 3
            }
            0xc689 => {
                w.drain(16);
                w.adc(8);
                pc + 3
            }
            0xf6d8 => {
                w.rts();
                w.cpu.pc
            }
            _ => unreachable!("receipt drawing pc={pc:04x}"),
        };
        w.cpu.pc = next;
        self.tail = match (w.cpu.bank, next) {
            (8, 0xcc93) => Some(ReceiptDrawingTail::Sparkle),
            (0, 0xd4db) => Some(ReceiptDrawingTail::Graphics {
                source: w.cpu.a as u8,
            }),
            (8, 0xc690) => Some(ReceiptDrawingTail::Draw),
            _ => None,
        };
        Some(w.cycles)
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    let k = case % 10;
    let v = case / 10;
    r[0xfa0] = k as u8;
    r[0xc5e + k] = (v % 76) as u8;
    r[0xc7c + k] = (v / 76 % 4) as u8;
    r[0xc54 + k] = (v / 304 % 4) as u8;
    r[0xc68 + k] = [0, 15, 16, 255][v / 1216 % 4];
    r[0x3a4 + k] = (v / 4864 % 3) as u8;
    r[0x39f + k] = [0, 1, 128, 255][v / 304 % 4];
    let y = [0u16, 0xff, 0xffff, 0x1000][v / 76 % 4];
    let x = [0u16, 0xff, 0xfff, 0xffff][v / 304 % 4];
    for (base, pos) in [(0xbfa, y), (0xc04, x)] {
        r[base + k] = pos as u8;
        r[base + 20 + k] = (pos >> 8) as u8;
    }
    for (a, pos) in [
        (0x11e, (case as u16).wrapping_mul(113)),
        (0x122, (case as u16).wrapping_mul(229)),
    ] {
        r[a..a + 2].copy_from_slice(&pos.to_le_bytes());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receipt_drawing_matches_source_animation_tables_coordinates_and_real_child_stacks() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 3];
        let mut graphics = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..145920usize {
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
                pc: 0xc5f7,
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
                snes::CpuRasterPosition::new(59, 1334),
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
            let mut work = ReceiptDrawing::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("receipt drawing case={case} pc={pc:04x}"),
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
            match work.tail().unwrap() {
                ReceiptDrawingTail::Sparkle => tails[0] += 1,
                ReceiptDrawingTail::Draw => tails[2] += 1,
                ReceiptDrawingTail::Graphics { source } => {
                    tails[1] += 1;
                    graphics.insert(source);
                }
            }
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [
            0x08c5f7, 0x08c600, 0x08c61f, 0x08c630, 0x08c642, 0x08c648, 0x08c662, 0x08c66e,
            0x08c67c, 0x08c681, 0x08f6a4, 0x08f6c7, 0x08f6d8, 0x08c68c,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        assert!(tails.iter().all(|n| *n > 0));
        assert_eq!(
            graphics,
            std::collections::BTreeSet::from([0x24, 0x25, 0x26])
        );
        println!("native receipt drawing: 145920 cases, {count} source instructions, {} PCs, tails={tails:?}, all three graphics sources",visited.len());
    }
    #[test]
    fn receipt_animation_table_has_only_nine_data_bytes() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x08c375..=0x08c37d {
            assert_eq!(
                crate::ZeldaState::native_receipt_animation_table_byte(a),
                Some(rom[8 * 0x8000 + (a as usize & 0x7fff)])
            );
        }
        for a in [0x08c374, 0x08c37e, 0x08c38a] {
            assert_eq!(
                crate::ZeldaState::native_receipt_animation_table_byte(a),
                None
            );
        }
    }
}
