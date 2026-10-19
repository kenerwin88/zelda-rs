//! Tile attribute lookup, dispatch and common effect handlers on the live CPU bus.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
};
use snes::CpuMasterTimeline;
// Immutable tile-effect pointers, independently checked against the reference ROM in tests.
const EFFECT_POINTERS: [u8; 512] = [
    84, 220, 80, 220, 80, 220, 80, 220, 80, 220, 84, 220, 84, 220, 84, 220, 182, 220, 27, 221, 188,
    220, 80, 220, 152, 220, 202, 221, 158, 220, 164, 220, 97, 220, 97, 220, 97, 220, 97, 220, 84,
    220, 84, 220, 84, 220, 84, 220, 93, 220, 93, 220, 93, 220, 93, 220, 194, 220, 114, 220, 125,
    220, 125, 220, 139, 220, 84, 220, 134, 220, 84, 220, 84, 220, 84, 220, 80, 220, 225, 221, 187,
    222, 173, 222, 197, 222, 197, 222, 207, 222, 221, 222, 207, 222, 221, 222, 134, 220, 134, 220,
    134, 220, 134, 220, 134, 220, 134, 220, 134, 220, 134, 220, 84, 220, 84, 220, 84, 220, 84, 220,
    84, 220, 157, 221, 161, 221, 161, 221, 97, 222, 84, 220, 84, 220, 80, 220, 177, 221, 84, 220,
    17, 223, 84, 220, 103, 222, 84, 220, 103, 222, 255, 222, 84, 220, 84, 220, 84, 220, 84, 220,
    126, 222, 126, 222, 126, 222, 126, 222, 126, 222, 126, 222, 126, 222, 25, 223, 106, 221, 106,
    221, 106, 221, 106, 221, 106, 221, 106, 221, 84, 220, 84, 220, 247, 221, 84, 220, 84, 220, 69,
    222, 84, 220, 84, 220, 84, 220, 23, 222, 41, 222, 45, 222, 53, 222, 58, 222, 80, 220, 80, 220,
    80, 220, 80, 220, 65, 221, 65, 221, 65, 221, 65, 221, 65, 221, 65, 221, 65, 221, 65, 221, 65,
    221, 65, 221, 65, 221, 65, 221, 65, 221, 65, 221, 65, 221, 65, 221, 10, 221, 10, 221, 234, 220,
    234, 220, 10, 221, 10, 221, 10, 221, 10, 221, 10, 221, 10, 221, 10, 221, 10, 221, 10, 221, 10,
    221, 79, 222, 79, 222, 200, 220, 200, 220, 200, 220, 200, 220, 200, 220, 200, 220, 200, 220,
    200, 220, 212, 220, 212, 220, 212, 220, 212, 220, 212, 220, 212, 220, 212, 220, 212, 220, 0,
    221, 0, 221, 224, 220, 224, 220, 0, 221, 0, 221, 84, 220, 84, 220, 212, 220, 212, 220, 212,
    220, 212, 220, 212, 220, 212, 220, 212, 220, 212, 220, 139, 220, 139, 220, 139, 220, 139, 220,
    139, 220, 139, 220, 139, 220, 139, 220, 139, 220, 139, 220, 139, 220, 139, 220, 139, 220, 139,
    220, 84, 220, 84, 220, 174, 220, 174, 220, 174, 220, 174, 220, 174, 220, 174, 220, 174, 220,
    174, 220, 174, 220, 174, 220, 174, 220, 174, 220, 174, 220, 174, 220, 174, 220, 174, 220, 84,
    220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220,
    84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84,
    220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220, 84, 220,
    235, 221, 235, 221, 235, 221, 235, 221, 235, 221, 235, 221, 235, 221, 235, 221, 235, 221, 235,
    221, 235, 221, 235, 221, 235, 221, 235, 221, 235, 221, 235, 221,
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    if let Some(i) = address.checked_sub(0x07d7d8) {
        if let Some(v) = EFFECT_POINTERS.get(i as usize) {
            return Some(*v);
        }
    }
    [4, 0, 0, 0, 6, 0, 2, 0]
        .get(address.checked_sub(0x07dc55)? as usize)
        .copied()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TileClassifierTail {
    Complete,
    Effect { tile: u8, offset: u16 },
    OutdoorAttribute,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TileClassifier {
    pc: Option<u16>,
    tail: Option<TileClassifierTail>,
    tile: u8,
    offset: u16,
}
impl TileClassifier {
    pub(crate) fn tail(&self) -> Option<TileClassifierTail> {
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
        let pc = self.pc.unwrap_or(0xd9d8);
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (7, pc, 0));
        assert_eq!(cpu.p & 8, 0);
        let width = if cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0xd9d8 | 0xd9e1 | 0xda04 => {
                w.width(
                    if pc == 0xda04 {
                        0x10
                    } else if pc == 0xd9d8 {
                        0x30
                    } else {
                        0x20
                    },
                    pc != 0xd9d8,
                );
                pc + 2
            }
            0xd9da | 0xd9e3 | 0xd9ea | 0xd9f4 | 0xdc65 | 0xdc72 | 0xdc7d | 0xdd9d | 0xdda1
            | 0xdc50 | 0xdc5d | 0xdc61 | 0xdc76 | 0xdc81 | 0xdc86 | 0xdc93 | 0xdc98 | 0xdc9e
            | 0xdda5 | 0xdca4 | 0xdcae | 0xdcb6 | 0xdcbc | 0xdcc2 | 0xddbe | 0xddc3 | 0xddd7
            | 0xdde1 | 0xdde5 | 0xddeb | 0xde05 | 0xde0c | 0xde17 | 0xde1e | 0xde29 | 0xde2d
            | 0xde35 | 0xde3a | 0xde5b | 0xde61 | 0xde67 => {
                let v = w.dp_read(
                    match pc {
                        0xd9da => 0x1b,
                        0xd9e3 => 0x49,
                        0xd9ea => 0x0,
                        0xd9f4 => 0x2,
                        0xdc65 => 0x6,
                        0xdc72 => 0x6,
                        0xdc7d => 0x6,
                        0xdd9d => 0x6,
                        0xdda1 => 0x6,
                        0xdc50 => 0xa,
                        0xdda5 => 0xa,
                        0xdc5d => 0xa,
                        0xdc61 => 0xa,
                        0xdc76 => 0xa,
                        0xdc81 => 0xa,
                        0xdc86 => 0xa,
                        0xdc93 => 0xa,
                        0xdc98 => 0xa,
                        0xdc9e => 0xa,
                        0xdca4 => 0xa,
                        0xdcae => 0xa,
                        0xdcb6 => 0xa,
                        0xdcbc => 0xa,
                        0xdcc2 => 0xa,
                        0xddbe => 0xa,
                        0xddc3 => 0xa,
                        0xddd7 => 0xa,
                        0xdde1 => 0xa,
                        0xdde5 => 0xa,
                        0xddeb => 0xa,
                        0xde05 => 0xa,
                        0xde0c => 0xa,
                        0xde17 => 0xa,
                        0xde1e => 0xa,
                        0xde29 => 0xa,
                        0xde2d => 0xa,
                        0xde35 => 0xa,
                        0xde3a => 0xa,
                        0xde5b => 0xa,
                        0xde61 => 0xa,
                        0xde67 => 0xa,
                        _ => unreachable!(),
                    },
                    width,
                );
                w.load(v, width);
                pc + 2
            }
            0xd9e8 | 0xd9f2 | 0xda1e | 0xdc6f | 0xdc74 | 0xdc7f | 0xdda3 => {
                w.dp_store(
                    match pc {
                        0xd9e8 => 0x49,
                        0xd9f2 => 0x6,
                        0xda1e => 0x6,
                        0xdc6f => 0x6e,
                        0xdc74 => 0x76,
                        0xdc7f => 0x76,
                        0xdda3 => 0x76,
                        _ => unreachable!(),
                    },
                    w.cpu.a,
                    width,
                );
                if pc == 0xda1e {
                    self.tile = w.cpu.a as u8;
                }
                pc + 2
            }
            0xd9e5 | 0xd9ec | 0xd9f6 | 0xda0f | 0xda1b | 0xdc67 | 0xdc8e | 0xddb9 | 0xddd2
            | 0xddfd => {
                w.mask(
                    match pc {
                        0xd9e5 => 0xff,
                        0xd9ec => 0xfff8,
                        0xd9f6 => 0x3f,
                        0xda0f => 0xff,
                        0xda1b => 0xff,
                        0xdc67 => 0x3,
                        0xdc8e => 0xff,
                        0xddb9 => 0x80,
                        0xddd2 => 0x80,
                        0xddfd => 0xff,
                        _ => unreachable!(),
                    },
                    width,
                );
                pc + 1 + u16::from(width)
            }
            0xd9ef | 0xd9f0 | 0xd9f1 | 0xda25 | 0xdc6a | 0xdca6 | 0xdca7 | 0xdca8 | 0xdca9
            | 0xdda9 | 0xddaa | 0xddab | 0xddac | 0xddef | 0xddf0 | 0xddf1 | 0xddf2 | 0xddd9
            | 0xddda | 0xdddb | 0xdddc | 0xde0f | 0xde10 | 0xde11 | 0xde12 | 0xde21 | 0xde22
            | 0xde23 | 0xde24 | 0xde2f | 0xde30 | 0xde31 | 0xde32 | 0xde3d | 0xde3e | 0xde3f
            | 0xde40 => {
                w.shift_left(width);
                pc + 1
            }
            0xd9f9 | 0xda00 => {
                w.clear_carry();
                pc + 1
            }
            0xd9fa => {
                let v = w.dp_read(6, 2);
                w.adc(v);
                pc + 2
            }
            0xda01 => {
                w.drain(16);
                w.adc(0x1000);
                pc + 3
            }
            0xd9fc => {
                let v = w.dp_read(0xee, 1);
                w.cpu.x = v;
                w.flags(v, 1);
                pc + 2
            }
            0xddf7 => {
                let v = w.dp_read(0xbd, 2);
                w.cpu.x = v;
                w.flags(v, 2);
                pc + 2
            }
            0xda06 | 0xda26 => {
                w.tax_word();
                pc + 1
            }
            0xdc6b => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                pc + 1
            }
            0xda07 | 0xddf9 => {
                w.drain(24);
                let v = w.read(
                    (if pc == 0xda07 { 0x7f2000 } else { 0x7f2040 }) + u32::from(w.cpu.x),
                    2,
                );
                w.load(v, 2);
                pc + 4
            }
            0xda0b => {
                w.drain(6);
                w.push(w.cpu.a, 2);
                pc + 1
            }
            0xda14 | 0xda1a => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                pc + 1
            }
            0xda0c | 0xdc8b | 0xddb1 | 0xddb6 | 0xddca | 0xddcf => {
                w.drain(16);
                let v = w.read(
                    w.absolute(match pc {
                        0xda0c => 0x37f,
                        0xdc8b => 0x2f5,
                        0xddb1 => 0xffc,
                        0xddb6 => 0x403,
                        0xddca => 0xffc,
                        0xddcf => 0x403,
                        _ => unreachable!(),
                    }),
                    width,
                );
                w.load(v, width);
                pc + 3
            }
            0xda15 => {
                w.drain(16);
                w.load(0, 2);
                pc + 3
            }
            0xda20 => {
                w.drain(16);
                w.write(w.absolute(0x114), w.cpu.a, 2);
                pc + 3
            }
            0xda23 => {
                w.dp_store(0xbd, w.cpu.x, 2);
                self.offset = w.cpu.x;
                pc + 2
            }
            0xd9dc | 0xd9fe | 0xda12 | 0xdc91 | 0xddb4 | 0xddbc | 0xddcd | 0xddd5 | 0xde03 => {
                let (t, target) = match pc {
                    0xd9dc => (w.cpu.p & ZERO == 0, 0xd9e1),
                    0xd9fe => (w.cpu.p & ZERO != 0, 0xda04),
                    0xda12 => (w.cpu.p & ZERO != 0, 0xda1a),
                    0xdc91 => (w.cpu.p & ZERO == 0, 0xdc97),
                    0xddb4 => (w.cpu.p & ZERO == 0, 0xddbe),
                    0xddbc => (w.cpu.p & ZERO != 0, 0xddc3),
                    0xddcd => (w.cpu.p & ZERO == 0, 0xdde0),
                    0xddd5 => (w.cpu.p & ZERO == 0, 0xdde0),
                    0xde03 => (w.cpu.p & ZERO == 0, 0xde0c),
                    _ => unreachable!(),
                };
                if w.branch(t) {
                    target
                } else {
                    pc + 2
                }
            }
            0xda18 | 0xdc7b | 0xdd9f | 0xde2b | 0xde33 | 0xde38 => {
                w.branch(true);
                match pc {
                    0xda18 => 0xda1b,
                    0xdc7b => 0xdc86,
                    0xdd9f => 0xdda3,
                    0xde2b => 0xde41,
                    0xde33 => 0xde41,
                    0xde38 => 0xde41,
                    _ => unreachable!(),
                }
            }
            0xd9de => {
                w.operand_then_internal(2);
                0xdc2a
            }
            0xdc2a => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 0;
                self.tail = Some(TileClassifierTail::OutdoorAttribute);
                0x882e
            }
            0xda27 => {
                w.drain(16);
                w.drain(6);
                let target = w.read(0x070000 | u32::from(0xd7d8 + w.cpu.x), 2);
                if ![
                    0xdc50, 0xdc54, 0xdc5d, 0xdc61, 0xdc72, 0xdc7d, 0xdc86, 0xdc8b, 0xdc98, 0xdc9e,
                    0xdca4, 0xdcae, 0xdcb6, 0xdcbc, 0xdcc2, 0xdd9d, 0xdda1, 0xddb1, 0xddca, 0xdde1,
                    0xddeb, 0xddf7, 0xde17, 0xde29, 0xde2d, 0xde35, 0xde3a, 0xde5b, 0xde61, 0xde67,
                ]
                .contains(&target)
                {
                    self.tail = Some(TileClassifierTail::Effect {
                        tile: self.tile,
                        offset: self.offset,
                    });
                }
                target
            }
            0xdc6c => {
                w.drain(16);
                w.drain(6);
                let v = w.read(w.absolute(0xdc55 + w.cpu.y), 2);
                w.load(v, 2);
                pc + 3
            }
            0xdc52 | 0xdc5f | 0xdc63 | 0xdc88 | 0xdc95 | 0xdcb0 | 0xdda7 | 0xddc0 | 0xdde3
            | 0xdded | 0xde19 => {
                let address: u32 = match pc {
                    0xdc52 => 0xe,
                    0xdc5f => 0x38,
                    0xdc63 => 0xc,
                    0xdc88 => 0x58,
                    0xdc95 => 0x59,
                    0xdcb0 => 0xe,
                    0xdda7 => 0x58,
                    0xddc0 => 0xe,
                    0xdde3 => 0xe,
                    0xdded => 0xe,
                    0xde19 => 0xe,
                    _ => unreachable!(),
                };
                let old = w.dp_read(address as u16, 2);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(address, old | w.cpu.a, 2);
                pc + 2
            }
            0xdc78 | 0xdc83 | 0xdc9a | 0xdca0 | 0xdcaa | 0xdcb2 | 0xdcb8 | 0xdcbe | 0xdcc4
            | 0xddad | 0xddc6 | 0xdddd | 0xdde7 | 0xddf3 | 0xde08 | 0xde13 | 0xde1b | 0xde25
            | 0xde41 | 0xde5d | 0xde63 | 0xde69 | 0xde6c => {
                w.drain(16);
                let address = w.absolute(match pc {
                    0xdc78 => 0x2c0,
                    0xdc83 => 0x2c0,
                    0xdc9a => 0x320,
                    0xdca0 => 0x348,
                    0xdcaa => 0x348,
                    0xdcb2 => 0x2f6,
                    0xdcb8 => 0x341,
                    0xdcbe => 0x343,
                    0xdcc4 => 0x34c,
                    0xddad => 0x2c0,
                    0xddc6 => 0x2e7,
                    0xdddd => 0x2ee,
                    0xdde7 => 0x2f6,
                    0xddf3 => 0x2f6,
                    0xde08 => 0x2f6,
                    0xde13 => 0x2f6,
                    0xde1b => 0x2f6,
                    0xde25 => 0x2e7,
                    0xde41 => 0x3f1,
                    0xde5d => 0x343,
                    0xde63 => 0x357,
                    0xde69 => 0x35b,
                    0xde6c => 0x343,
                    _ => unreachable!(),
                });
                let old = w.read(address, 2);
                w.cpu.p = (w.cpu.p & !ZERO) | if old & w.cpu.a == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write_high_low(address, old | w.cpu.a, 2);
                pc + 3
            }
            0xde20 | 0xddc5 | 0xde07 | 0xde0e | 0xde37 | 0xde3c => {
                w.swap_bytes();
                pc + 1
            }
            0xde00 => {
                w.compare_immediate(0x60, 2);
                pc + 3
            }
            0xdc54 | 0xdc71 | 0xdc8a | 0xdc97 | 0xdc9d | 0xdca3 | 0xdcad | 0xdcb5 | 0xdcbb
            | 0xdcc1 | 0xdcc7 | 0xddb0 | 0xddc2 | 0xddc9 | 0xdde0 | 0xddea | 0xddf6 | 0xde0b
            | 0xde16 | 0xde28 | 0xde44 | 0xde60 | 0xde66 | 0xde6f => {
                w.rts();
                self.tail = Some(TileClassifierTail::Complete);
                assert!(matches!(w.cpu.pc, 0xceba | 0xcec6 | 0xce0f | 0xce1b | 0xce27 | 0xd0f1));
                w.cpu.pc
            }
            _ => panic!("tile classifier PC {pc:04x}"),
        };
        w.cpu.pc = next;
        self.pc = Some(next);
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
    fn tile_classifier_matches_source_attributes_layers_cheats_and_effects() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x07d7d8..0x07d9d8 {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        for address in 0x07dc55..0x07dc5d {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        let mut instructions = 0;

        for timer in 0..=255u8 {
            for variant in 0..32u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7) ^ timer as usize) as u8;
                }
                let return_pc: u16 = if variant & 2 == 0 { 0xceba } else { 0xcec6 };
                r[0x1ef..0x1f1].copy_from_slice(&(return_pc - 1).to_le_bytes());
                let x = variant as u16 * 5;
                let y = variant as u16 * 16;
                r[0..2].copy_from_slice(&y.to_le_bytes());
                r[2..4].copy_from_slice(&x.to_le_bytes());
                r[0x1b] = u8::from(variant != 31);
                r[0xee] = variant & 1;
                r[0x37f] = u8::from(variant == 30);
                let offset = ((y & !7) << 3) + (x & 63) + if variant & 1 != 0 { 0x1000 } else { 0 };
                r[0x12000 + offset as usize] = timer;
                r[0x12000 + offset as usize + 64] = if variant & 2 == 0 { timer } else { 0 };
                r[0x2f5] = u8::from(variant & 4 != 0);
                r[0xffc..0xffe].copy_from_slice(&u16::from(variant & 8 != 0).to_le_bytes());
                r[0x403..0x405]
                    .copy_from_slice(&if variant & 16 != 0 { 0x80u16 } else { 0 }.to_le_bytes());
                r[0xa..0xc]
                    .copy_from_slice(&[1u16, 2, 4, 0xffff][(variant & 3) as usize].to_le_bytes());
                let pc = 0xd9d8;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: u16::from(variant & 1) * 2,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: 0x1ee,
                    p: timer & 0xc3,
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
                let mut clock = CpuMasterTimeline::at_raster(
                    4586,
                    CpuRasterPosition::new(
                        if variant & 1 == 0 { 22 } else { 261 },
                        if variant & 1 == 0 { 26 } else { 1350 },
                    ),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(variant & 1 != 0),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = TileClassifier::default();
                while work.tail().is_none() {
                    let label = format!(
                        "classifier tile={timer} variant={variant} pc={:04x}",
                        cpu.pc
                    );
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| {
                            if let super::super::intro_clear::Access::Read(a, width) = access {
                                let low = table_byte(a)?;
                                Some(if width == 1 {
                                    u16::from(low)
                                } else {
                                    u16::from_le_bytes([low, table_byte(a + 1)?])
                                })
                            } else {
                                None
                            }
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                match work.tail().unwrap() {
                    TileClassifierTail::Complete => {
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (7, return_pc, 0x1f0))
                    }
                    TileClassifierTail::Effect { tile, offset: _ } => {
                        assert_eq!(tile, if variant == 30 { 0 } else { timer })
                    }
                    TileClassifierTail::OutdoorAttribute => {
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (0, 0x882e, 0x1eb))
                    }
                }
            }
        }

        println!("tile classifier: 8192 cases, {instructions} source instructions, all byte attributes, layer/cheat/hazard/effect branches");
    }
}
