//! Door attributes own the live door records, special tile lists and switch
//! conversion, including the nested calls' physical returns and width changes.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DoorAttributes {
    pc: u16,
    child_return: Option<u16>,
    complete: bool,
}
impl Default for DoorAttributes {
    fn default() -> Self {
        Self {
            pc: 0xbe17,
            child_return: None,
            complete: false,
        }
    }
}
impl DoorAttributes {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.child_return {
            Some(0xbe24) => 0x01be35,
            Some(0xbe2e) => 0x01d51f,
            Some(0xbe31) => 0x01c1ba,
            _ => 0x01be17,
        }
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
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 8),
            (1, self.pc, 0, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let n = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match self.pc {
            0xbe17 | 0xbea6 | 0xc0a6 | 0xc1ba => {
                w.width(
                    match self.pc {
                        0xbe17 | 0xc1ba => 0x30,
                        0xbea6 | 0xc0a6 => 0x20,
                        _ => unreachable!(),
                    },
                    true,
                );
                self.pc + 2
            }
            0xbe19 | 0xd524 | 0xd569 | 0xc1c9 => {
                w.drain(8 * u32::from(n));
                let v = match self.pc {
                    0xbe19 | 0xd524 | 0xd569 => 0x0,
                    0xc1c9 => 0x4,
                    _ => unreachable!(),
                };
                w.cpu.y = v;
                w.flags(v, n);
                self.pc + 1 + u16::from(n)
            }
            0xbe1c | 0xbe35 | 0xbea8 | 0xbec9 | 0xbed1 | 0xbf1d | 0xbf61 | 0xbf8b | 0xc0a8
            | 0xc0bf | 0xc0c7 | 0xc10e | 0xc12d | 0xc15e | 0xc18d => {
                w.operand_then_internal(2);
                let a = match self.pc {
                    0xbe1c | 0xbea8 | 0xbed1 | 0xbf1d | 0xbf61 | 0xbf8b | 0xc0a8 | 0xc0c7
                    | 0xc10e | 0xc12d | 0xc15e | 0xc18d => 0x19a0,
                    0xbe35 => 0x1980,
                    0xbec9 | 0xc0bf => 0x19c0,
                    _ => unreachable!(),
                };
                let v = w.read(a + u32::from(w.cpu.y), m);
                w.load(v, m);
                self.pc + 3
            }
            0xbe1f | 0xbe3d | 0xbe42 | 0xbe47 | 0xbe4c | 0xbe51 | 0xbe56 | 0xbe5b | 0xbe60
            | 0xbe65 | 0xbe82 | 0xbe87 | 0xbed7 | 0xbedc | 0xbee1 | 0xbf23 | 0xbf28 | 0xbf2d
            | 0xbf32 | 0xbf37 | 0xc088 | 0xc08d | 0xc102 | 0xc107 | 0xc10c | 0xc117 | 0xc11c
            | 0xc121 | 0xd522 | 0xd567 | 0xc1bf | 0xc1d9 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    match self.pc {
                        0xbe1f => 0xbe24,
                        0xbe3d | 0xbe42 | 0xbe47 | 0xbe4c | 0xbe56 => 0xbeb8,
                        0xbe51 | 0xbe5b | 0xbe60 | 0xbe65 => 0xbe6c,
                        0xbe82 | 0xbe87 => 0xbe8f,
                        0xbed7 | 0xbedc | 0xbee1 => 0xbee8,
                        0xbf23 | 0xbf28 | 0xbf2d | 0xbf32 | 0xbf37 => 0xbf3e,
                        0xc088 | 0xc08d => 0xc0b8,
                        0xc102 | 0xc107 | 0xc10c | 0xc117 | 0xc11c | 0xc121 => 0xc128,
                        0xd522 => 0xd564,
                        0xd567 => 0xd5a9,
                        0xc1bf => 0xc219,
                        0xc1d9 => 0xc201,
                        _ => unreachable!(),
                    }
                } else {
                    self.pc + 2
                }
            }
            0xbe21 | 0xbe2b | 0xbe2e => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                assert!(self.child_return.is_none());
                self.child_return = Some(self.pc + 3);
                match self.pc {
                    0xbe21 => 0xbe35,
                    0xbe2b => 0xd51f,
                    0xbe2e => 0xc1ba,
                    _ => unreachable!(),
                }
            }
            0xbe24 | 0xbe25 | 0xd545 | 0xd546 | 0xd55d | 0xd55e | 0xd58a | 0xd58b | 0xd5a2
            | 0xd5a3 | 0xc1d2 => {
                w.increment_y();
                self.pc + 1
            }
            0xbe26 => {
                w.drain(8 * u32::from(n));
                let v = 0x20;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(w.cpu.y.wrapping_sub(v), n);
                self.pc + 1 + u16::from(n)
            }
            0xbe29 | 0xbe6a | 0xbe72 | 0xbe9a | 0xbecf | 0xbee6 | 0xbf1b | 0xbf3c | 0xbf5f
            | 0xc09a | 0xc0c5 | 0xc0fd | 0xc126 | 0xc15c | 0xd534 | 0xd54a | 0xd562 | 0xd579
            | 0xd58f | 0xd5a7 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0xbe29 => 0xbe1c,
                        0xbe6a => 0xbe6f,
                        0xbe72 => 0xbe77,
                        0xbe9a => 0xbeb8,
                        0xbecf => 0xbf18,
                        0xbee6 => 0xbeed,
                        0xbf1b => 0xbf5c,
                        0xbf3c => 0xbf43,
                        0xbf5f => 0xbf8b,
                        0xc09a => 0xc0b8,
                        0xc0c5 => 0xc0fa,
                        0xc0fd => 0xc159,
                        0xc126 => 0xc12d,
                        0xc15c => 0xc18d,
                        0xd534 => 0xd54e,
                        0xd54a | 0xd562 => 0xd527,
                        0xd579 => 0xd593,
                        0xd58f | 0xd5a7 => 0xd56c,
                        _ => unreachable!(),
                    }
                } else {
                    self.pc + 2
                }
            }
            0xbe31 => {
                w.drain(16);
                let a = 0x200;
                let v = w.read(a, m).wrapping_add(1) & mask;
                w.drain(6);
                w.flags(v, m);
                w.write(a, v, m);
                self.pc + 3
            }
            0xbe34 | 0xbeb7 | 0xbf17 | 0xbf5b | 0xbf8a | 0xbfb1 | 0xbfb2 | 0xc0b7 | 0xc0f9
            | 0xc158 | 0xc18c | 0xc1b9 | 0xd5a9 | 0xc200 | 0xc21b => {
                w.rts();
                if let Some(ret) = self.child_return.take() {
                    assert_eq!(w.cpu.pc, ret);
                } else {
                    assert_eq!(self.pc, 0xbe34);
                    self.complete = true;
                }
                w.cpu.pc
            }
            0xbe38 | 0xbe8a | 0xbe90 | 0xbecc | 0xbeee | 0xbf65 | 0xbf7f | 0xbfa6 | 0xc090
            | 0xc0c2 | 0xc0cb | 0xc111 | 0xc162 | 0xd52e | 0xd573 | 0xc1c4 | 0xc1d6 => {
                w.drain(8 * u32::from(m));
                let v = match self.pc {
                    0xbe38 => 0xfe,
                    0xbe8a => 0xf,
                    0xbe90 | 0xbf7f | 0xc090 => 0xff,
                    0xbecc | 0xc0c2 => 0x3,
                    0xbeee | 0xc0cb => 0x783f,
                    0xbf65 | 0xc162 => 0xffe0,
                    0xbfa6 => 0xff00,
                    0xc111 => 0x1fff,
                    0xd52e | 0xd573 => 0xf0,
                    0xc1c4 => 0x3fff,
                    0xc1d6 => 0x1000,
                    _ => unreachable!(),
                };
                w.load(w.cpu.a & v & mask, m);
                self.pc + 1 + u16::from(m)
            }
            0xbe3b | 0xbea2 | 0xbea4 | 0xbec7 | 0xc0a2 | 0xc0a4 | 0xc0bd => {
                w.dp_store(
                    match self.pc {
                        0xbe3b => 0x2,
                        0xbea2 | 0xbec7 | 0xc0a2 | 0xc0bd => 0x0,
                        0xbea4 | 0xc0a4 => 0x1,
                        _ => unreachable!(),
                    },
                    w.cpu.a & mask,
                    m,
                );
                self.pc + 2
            }
            0xbe3f | 0xbe44 | 0xbe49 | 0xbe4e | 0xbe53 | 0xbe58 | 0xbe5d | 0xbe62 | 0xbe67
            | 0xbe6f | 0xbe77 | 0xbe7f | 0xbe84 | 0xbf18 | 0xbf5c | 0xc085 | 0xc08a | 0xc0fa
            | 0xc159 | 0xd531 | 0xd576 => {
                w.drain(8 * u32::from(m));
                let v = match self.pc {
                    0xbe3f => 0x6,
                    0xbe44 => 0x12,
                    0xbe49 => 0xa,
                    0xbe4e => 0xc,
                    0xbe53 => 0xe,
                    0xbe58 => 0x10,
                    0xbe5d => 0x4,
                    0xbe62 | 0xbf5c | 0xc159 => 0x2,
                    0xbe67 => 0x8,
                    0xbe6f => 0x30,
                    0xbe77 | 0xc085 => 0x40,
                    0xbe7f => 0x18,
                    0xbe84 => 0x44,
                    0xbf18 | 0xc0fa => 0x1,
                    0xc08a => 0x46,
                    0xd531 | 0xd576 => 0x80,
                    _ => unreachable!(),
                };
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= v);
                w.flags(left.wrapping_sub(v) & mask, m);
                self.pc + 1 + u16::from(m)
            }
            0xbe6c | 0xbe74 | 0xbe7c => {
                w.drain(16);
                match self.pc {
                    0xbe6c => 0xc0b8,
                    0xbe74 => 0xbfb2,
                    0xbe7c => 0xc085,
                    _ => unreachable!(),
                }
            }
            0xbe7a | 0xbebd | 0xbec2 | 0xc1d0 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    match self.pc {
                        0xbe7a => 0xbe7f,
                        0xbebd => 0xbec4,
                        0xbec2 => 0xbeb7,
                        0xc1d0 => 0xc1d3,
                        _ => unreachable!(),
                    }
                } else {
                    self.pc + 2
                }
            }
            0xbe89 | 0xbe8f | 0xbe9e | 0xc08f | 0xc09e => {
                w.drain(6);
                w.load(w.cpu.y & mask, m);
                self.pc + 1
            }
            0xbe8d | 0xd54c | 0xd591 => {
                if w.branch(true) {
                    match self.pc {
                        0xbe8d => 0xbe93,
                        0xd54c => 0xd564,
                        0xd591 => 0xd5a9,
                        _ => unreachable!(),
                    }
                } else {
                    self.pc + 2
                }
            }
            0xbe93 | 0xbeac | 0xbef1 | 0xbf44 | 0xbf68 | 0xbf8f | 0xc093 | 0xc0ac | 0xc0ce
            | 0xc135 | 0xc165 | 0xc192 | 0xc1c8 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & if n == 1 { 255 } else { 65535 };
                w.flags(w.cpu.x, n);
                self.pc + 1
            }
            0xbe94 | 0xc094 | 0xc1bc | 0xc1c1 | 0xc1cc | 0xc1d3 => {
                w.drain(16);
                let a = match self.pc {
                    0xbe94 | 0xc094 => 0x68c,
                    0xc1bc | 0xc1c1 | 0xc1cc => 0x4b0,
                    0xc1d3 => 0x402,
                    _ => unreachable!(),
                };
                let v = w.read(a, m);
                w.load(v, m);
                self.pc + 3
            }
            0xbe97 | 0xc097 => {
                w.operand_then_internal(2);
                let a = 0x98c0;
                let v = w.read(a + u32::from(w.cpu.x), m);
                w.load(w.cpu.a & v & mask, m);
                self.pc + 3
            }
            0xbe9c | 0xc09c | 0xc1fe | 0xc219 => {
                w.width(
                    match self.pc {
                        0xbe9c | 0xc09c => 0x20,
                        0xc1fe | 0xc219 => 0x30,
                        _ => unreachable!(),
                    },
                    false,
                );
                self.pc + 2
            }
            0xbe9f | 0xbeab | 0xbeed | 0xbf43 | 0xbf64 | 0xbf8e | 0xc09f | 0xc0ab | 0xc0ca
            | 0xc130 | 0xc161 | 0xc190 | 0xc1c7 => {
                w.drain(6);
                let v = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, m);
                self.pc + 1
            }
            0xbea0 | 0xc0a0 | 0xd53a | 0xd552 | 0xd57f | 0xd597 => {
                w.drain(8 * u32::from(m));
                let v = match self.pc {
                    0xbea0 | 0xc0a0 => 0xf0,
                    0xd53a | 0xd552 => 0x1010,
                    0xd57f | 0xd597 => 0x2020,
                    _ => unreachable!(),
                };
                w.load((w.cpu.a | v) & mask, m);
                self.pc + 1 + u16::from(m)
            }
            0xbead | 0xbef2 | 0xbf45 | 0xbf69 | 0xbf90 | 0xc0ad | 0xc0cf | 0xc136 | 0xc166
            | 0xc193 => {
                let v = w.dp_read(0x0, m);
                w.load(v, m);
                self.pc + 2
            }
            0xbeaf | 0xbeb3 | 0xbef4 | 0xbef8 | 0xbefc | 0xbf00 | 0xbf04 | 0xbf08 | 0xbf0c
            | 0xbf13 | 0xbf47 | 0xbf4b | 0xbf4f | 0xbf53 | 0xbf57 | 0xbf6f | 0xbf73 | 0xbf77
            | 0xbf7b | 0xbf82 | 0xbf86 | 0xbf96 | 0xbf9a | 0xbf9e | 0xbfa2 | 0xbfa9 | 0xbfad
            | 0xc0af | 0xc0b3 | 0xc0d1 | 0xc0d5 | 0xc0d9 | 0xc0dd | 0xc0e1 | 0xc0e5 | 0xc0e9
            | 0xc0ed | 0xc0f1 | 0xc0f5 | 0xc138 | 0xc13c | 0xc140 | 0xc144 | 0xc148 | 0xc14c
            | 0xc150 | 0xc154 | 0xc16c | 0xc170 | 0xc174 | 0xc178 | 0xc17c | 0xc180 | 0xc184
            | 0xc188 | 0xc199 | 0xc19d | 0xc1a1 | 0xc1a5 | 0xc1a9 | 0xc1ad | 0xc1b1 | 0xc1b5
            | 0xd53d | 0xd541 | 0xd555 | 0xd559 | 0xd582 | 0xd586 | 0xd59a | 0xd59e | 0xc1de
            | 0xc1e2 | 0xc1e9 | 0xc1ed | 0xc1f1 | 0xc1f5 | 0xc204 | 0xc208 | 0xc20c | 0xc210 => {
                w.drain(24);
                let a = match self.pc {
                    0xbeaf | 0xbef8 | 0xbf47 | 0xc0af | 0xc0d5 | 0xc13c => 0x7f2041,
                    0xbeb3 | 0xbefc | 0xbf4b | 0xc0b3 | 0xc0d9 | 0xc140 => 0x7f2081,
                    0xbef4 | 0xc0d1 | 0xc138 => 0x7f2001,
                    0xbf00 | 0xbf4f | 0xc0dd | 0xc144 => 0x7f20c1,
                    0xbf04 | 0xbf53 | 0xc0e1 | 0xc148 => 0x7f2101,
                    0xbf08 | 0xbf57 | 0xc0e5 | 0xc14c => 0x7f2141,
                    0xbf0c | 0xc0e9 | 0xc150 => 0x7f2181,
                    0xbf13 | 0xc0ed | 0xc154 => 0x7f21c1,
                    0xbf6f | 0xbfa9 | 0xc16c | 0xc199 | 0xd541 | 0xd586 => 0x7f2040,
                    0xbf73 | 0xbf96 | 0xc170 | 0xc19d => 0x7f2042,
                    0xbf77 | 0xbfad | 0xc17c | 0xc1a9 | 0xc1e9 | 0xc204 => 0x7f2080,
                    0xbf7b | 0xbf9e | 0xc180 | 0xc1ad => 0x7f2082,
                    0xbf82 | 0xbf9a | 0xc174 | 0xc1a1 => 0x7f2044,
                    0xbf86 | 0xbfa2 | 0xc184 | 0xc1b1 => 0x7f2084,
                    0xc0f1 => 0x7f2201,
                    0xc0f5 => 0x7f2241,
                    0xc178 | 0xc1a5 => 0x7f2046,
                    0xc188 | 0xc1b5 => 0x7f2086,
                    0xd53d | 0xd582 | 0xc1de => 0x7f2000,
                    0xd555 | 0xd59a => 0x7f3000,
                    0xd559 | 0xd59e => 0x7f3040,
                    0xc1e2 => 0x7f2280,
                    0xc1ed | 0xc208 => 0x7f2100,
                    0xc1f1 | 0xc20c => 0x7f2180,
                    0xc1f5 | 0xc210 => 0x7f2200,
                    _ => unreachable!(),
                };
                w.write(a + u32::from(w.cpu.x), w.cpu.a & mask, m);
                self.pc + 4
            }
            0xbeb8 | 0xc0b8 => {
                let v = w.dp_read(0x2, n);
                w.cpu.x = v;
                w.flags(v, n);
                self.pc + 2
            }
            0xbeba | 0xbebf | 0xbf20 | 0xbf25 | 0xc0ff | 0xc104 | 0xc109 => {
                w.compare_x_immediate(
                    match self.pc {
                        0xbeba => 0x20,
                        0xbebf => 0x28,
                        0xbf20 => 0xa,
                        0xbf25 => 0xe,
                        0xc0ff => 0xc,
                        0xc104 => 0x10,
                        0xc109 => 0x4,
                        _ => unreachable!(),
                    },
                    n,
                );
                self.pc + 1 + u16::from(n)
            }
            0xbec4 | 0xc0ba => {
                w.operand_then_internal(2);
                let a = 0x9a52;
                let v = w.read(a + u32::from(w.cpu.x), m);
                w.load(v, m);
                self.pc + 3
            }
            0xbed4 | 0xbed9 | 0xbede | 0xbee3 | 0xbf2a | 0xbf2f | 0xbf34 | 0xbf39 | 0xc114
            | 0xc119 | 0xc11e | 0xc123 => {
                w.drain(16);
                let a = match self.pc {
                    0xbed4 | 0xbf2a | 0xc114 => 0x19e2,
                    0xbed9 | 0xbf2f | 0xc119 => 0x19e4,
                    0xbede | 0xbf34 | 0xc11e => 0x19e6,
                    0xbee3 | 0xbf39 | 0xc123 => 0x19e8,
                    _ => unreachable!(),
                };
                let v = w.read(a, m);
                let left = w.cpu.a & mask;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= v);
                w.flags(left.wrapping_sub(v) & mask, m);
                self.pc + 3
            }
            0xbee8 | 0xbf3e | 0xc128 => {
                w.drain(8 * u32::from(n));
                let v = 0x8e8e;
                w.cpu.x = v;
                w.flags(v, n);
                self.pc + 1 + u16::from(n)
            }
            0xbeeb | 0xbf41 | 0xc12b => {
                w.dp_store(0x0, w.cpu.x, n);
                self.pc + 2
            }
            0xbf10 | 0xc1db | 0xc1e6 | 0xc201 => {
                w.drain(8 * u32::from(m));
                let v = match self.pc {
                    0xbf10 | 0xc1e6 => 0x0,
                    0xc1db => 0x101,
                    0xc201 => 0x2323,
                    _ => unreachable!(),
                };
                w.load(v, m);
                self.pc + 1 + u16::from(m)
            }
            0xbf6b | 0xbf92 | 0xc131 | 0xc168 | 0xc195 => {
                w.clear_carry();
                self.pc + 1
            }
            0xbf6c | 0xbf93 | 0xc132 | 0xc169 | 0xc196 => {
                w.drain(8 * u32::from(m));
                if m == 1 {
                    w.adc_byte(match self.pc {
                        0xbf6c | 0xbf93 | 0xc169 | 0xc196 => 0x101,
                        0xc132 => 0x40,
                        _ => unreachable!(),
                    });
                } else {
                    w.adc(match self.pc {
                        0xbf6c | 0xbf93 | 0xc169 | 0xc196 => 0x101,
                        0xc132 => 0x40,
                        _ => unreachable!(),
                    });
                }
                self.pc + 1 + u16::from(m)
            }
            0xc191 => {
                w.increment_a(m);
                self.pc + 1
            }
            0xd51f | 0xd564 => {
                w.drain(16);
                let a = match self.pc {
                    0xd51f => 0x44e,
                    0xd564 => 0x450,
                    _ => unreachable!(),
                };
                let v = w.read(a, n);
                w.cpu.x = v;
                w.flags(v, n);
                self.pc + 3
            }
            0xd527 | 0xd56c => {
                w.operand_then_internal(2);
                let a = match self.pc {
                    0xd527 => 0x6c0,
                    0xd56c => 0x6d0,
                    _ => unreachable!(),
                };
                let v = w.read(a + u32::from(w.cpu.y), n);
                w.cpu.x = v;
                w.flags(v, n);
                self.pc + 3
            }
            0xd52a | 0xd536 | 0xd54e | 0xd56f | 0xd57b | 0xd593 => {
                w.drain(24);
                let a = match self.pc {
                    0xd52a | 0xd536 | 0xd56f | 0xd57b => 0x7f2000,
                    0xd54e | 0xd593 => 0x7f3000,
                    _ => unreachable!(),
                };
                let v = w.read(a + u32::from(w.cpu.x), m);
                w.load(v, m);
                self.pc + 4
            }
            0xd547 | 0xd55f | 0xd58c | 0xd5a4 => {
                w.drain(16);
                let a = match self.pc {
                    0xd547 | 0xd55f => 0x44e,
                    0xd58c | 0xd5a4 => 0x450,
                    _ => unreachable!(),
                };
                let v = w.read(a, n);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(w.cpu.y.wrapping_sub(v), n);
                self.pc + 3
            }
            0xc1cf => {
                w.shift_left(m);
                self.pc + 1
            }
            0xc1f9 | 0xc1fa | 0xc214 | 0xc215 => {
                w.increment_x();
                self.pc + 1
            }
            0xc1fb | 0xc216 => {
                w.decrement_y();
                self.pc + 1
            }
            0xc1fc | 0xc217 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    match self.pc {
                        0xc1fc => 0xc1db,
                        0xc217 => 0xc201,
                        _ => unreachable!(),
                    }
                } else {
                    self.pc + 2
                }
            }
            _ => unreachable!("door attribute continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
