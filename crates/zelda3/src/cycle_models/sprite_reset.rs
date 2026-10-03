//! Shared dungeon reset: outgoing cache, ordered disables, history and records.
//! CPU/scratch state retains each live operand, including the temporary Y/X
//! bytes at0FB5/0FB6 between loads, coordinate stores and subtype publication.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Call {
    entry: u16,
    returned: u16,
    far: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpriteReset {
    pc: u16,
    entry: u16,
    calls: Vec<Call>,
    complete: bool,
}
impl SpriteReset {
    pub(crate) fn from_entry(entry: u32) -> Self {
        assert!(matches!(
            entry,
            0x09c114 | 0x09c176 | 0x09c22f | 0x09c290 | 0x09c327 | 0x09c3e8 | 0x09c44e
        ));
        Self {
            pc: entry as u16,
            entry: entry as u16,
            calls: Vec::new(),
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x090000 | u32::from(self.calls.last().map_or(self.entry, |c| c.entry))
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (9, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xc114 => {
                w.drain(6);
                w.push(u16::from(w.cpu.db), 1);
                self.pc + 1
            }
            0xc115 => {
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                self.pc + 1
            }
            0xc116 | 0xc173 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                self.pc + 1
            }
            0xc117 | 0xc170 | 0xc2c7 | 0xc33e | 0xc357 | 0xc427 => {
                let target = (match self.pc {
                    0xc117 => 0xc176,
                    0xc170 => 0xc290,
                    0xc2c7 => 0xc327,
                    0xc33e => 0xc345,
                    0xc357 => 0xc3e8,
                    0xc427 => 0xc871,
                    _ => unreachable!(),
                }) as u16;
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.calls.push(Call {
                    entry: target,
                    returned: self.pc + 3,
                    far: false,
                });
                target
            }
            0xc11a | 0xc11d | 0xc452 | 0xc455 | 0xc458 | 0xc45b | 0xc45e | 0xc461 | 0xc464
            | 0xc24f | 0xc252 | 0xc255 | 0xc258 | 0xc25b | 0xc25e | 0xc261 | 0xc264 | 0xc267
            | 0xc26a | 0xc26d | 0xc270 | 0xc273 | 0xc276 | 0xc279 => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0xc11a => 0x309,
                    0xc11d => 0x308,
                    0xc452 => 0xfdd,
                    0xc455 => 0xfdc,
                    0xc458 | 0xc273 => 0xffd,
                    0xc45b => 0x2f0,
                    0xc45e | 0xc276 => 0xfc6,
                    0xc461 | 0xc252 => 0xb6a,
                    0xc464 => 0xfb3,
                    0xc24f => 0x2ec,
                    0xc255 => 0xb9b,
                    0xc258 => 0xb88,
                    0xc25b => 0xb99,
                    0xc25e => 0xfb4,
                    0xc261 => 0xb9e,
                    0xc264 => 0xcf4,
                    0xc267 => 0xff9,
                    0xc26a => 0xff8,
                    0xc26d => 0xffb,
                    0xc270 => 0xffc,
                    0xc279 => 0x3fc,
                    _ => unreachable!(),
                });
                w.write(a, 0, 1);
                self.pc + 3
            }
            0xc120 | 0xc44e => {
                let target = (0xc22f) as u16;
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(self.pc + 3, 2);
                self.calls.push(Call {
                    entry: target,
                    returned: self.pc + 4,
                    far: true,
                });
                target
            }
            0xc124 | 0xc474 | 0xc290 | 0xc365 => {
                w.width(
                    match self.pc {
                        0xc124 => 0x20,
                        0xc474 => 0x10,
                        0xc290 | 0xc365 => 0x30,
                        _ => unreachable!(),
                    },
                    true,
                );
                self.pc + 2
            }
            0xc126 | 0xc167 => {
                w.drain(16);
                let v = match self.pc {
                    0xc126 => 0xffff,
                    0xc167 => 0x0,
                    _ => unreachable!(),
                };
                w.load(v, 2);
                self.pc + 3
            }
            0xc129 | 0xc12c | 0xc147 | 0xc14d | 0xc153 | 0xc159 => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0xc129 => 0xfba,
                    0xc12c => 0xfb8,
                    0xc147 => 0xb86,
                    0xc14d => 0xb84,
                    0xc153 => 0xb82,
                    0xc159 => 0xb80,
                    _ => unreachable!(),
                });
                w.write(a, w.cpu.a & 65535, 2);
                self.pc + 3
            }
            0xc12f | 0xc17d | 0xc22f | 0xc247 | 0xc27c | 0xc284 | 0xc3e8 => {
                w.drain(8);
                let v = match self.pc {
                    0xc12f => 0x0,
                    0xc17d | 0xc22f => 0xf,
                    0xc247 => 0x9,
                    0xc27c | 0xc3e8 => 0x7,
                    0xc284 => 0x1d,
                    _ => unreachable!(),
                };
                w.cpu.x = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0xc131 | 0xc13f | 0xc144 | 0xc14a | 0xc150 | 0xc156 | 0xc292 | 0xc29c | 0xc369 => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0xc131 | 0xc156 | 0xc292 | 0xc29c | 0xc369 => 0x48e,
                    0xc13f => 0xb86,
                    0xc144 => 0xb84,
                    0xc14a => 0xb82,
                    0xc150 => 0xb80,
                    _ => unreachable!(),
                });
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xc134 => {
                w.drain(16);
                let a: u16 = 0xb80;
                if false || (a & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(w.cpu.x), 2);
                let a = w.cpu.a & 65535;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & 65535, 2);
                self.pc + 3
            }
            0xc137 | 0xc163 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    (0xc16e) as u16
                } else {
                    self.pc + 2
                }
            }
            0xc139 | 0xc13a => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0xc13b | 0xc32c => {
                w.compare_x_immediate(
                    match self.pc {
                        0xc13b => 0x7,
                        0xc32c => 0xe4,
                        _ => unreachable!(),
                    },
                    1,
                );
                self.pc + 2
            }
            0xc13d => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    (0xc134) as u16
                } else {
                    self.pc + 2
                }
            }
            0xc142 | 0xc29a => {
                w.dp_store(0x0, w.cpu.a & 65535, 2);
                self.pc + 2
            }
            0xc15c => {
                w.width(0x10, true);
                self.pc + 2
            }
            0xc15e | 0xc36e => {
                let v = w.dp_read(
                    match self.pc {
                        0xc15e => 0x0,
                        0xc36e => 0x2,
                        _ => unreachable!(),
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0xc160 | 0xc37b => {
                w.drain(16);
                let v = match self.pc {
                    0xc160 => 0xffff,
                    0xc37b => 0x0,
                    _ => unreachable!(),
                };
                let a = w.cpu.a & 65535;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & 65535, 2);
                self.pc + 3
            }
            0xc165 | 0xc295 | 0xc36c | 0xc370 => {
                w.shift_left(2);
                self.pc + 1
            }
            0xc166 | 0xc36d => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 65535;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xc16a => {
                w.drain(24);
                let a = 0x7fdf80 + u32::from(w.cpu.x);
                w.write(a, w.cpu.a & 65535, 2);
                self.pc + 4
            }
            0xc16e | 0xc2a2 | 0xc37e => {
                w.width(0x30, false);
                self.pc + 2
            }
            0xc174 | 0xc498 | 0xc28f => {
                w.rtl();
                if let Some(call) = self.calls.pop() {
                    assert_eq!(call.far, true);
                    assert_eq!((w.cpu.bank, w.cpu.pc), (9, call.returned));
                } else {
                    self.complete = true;
                }
                w.cpu.pc
            }
            0xc467 => {
                w.drain(24);
                let a = 0x7ef3cc;
                let v = w.read(a, 1);
                w.load(v, 1);
                self.pc + 4
            }
            0xc46b | 0xc1ae | 0xc1b2 | 0xc23d | 0xc2c3 | 0xc336 | 0xc33a | 0xc353 | 0xc42d
            | 0xc431 | 0xc435 => {
                w.drain(8);
                let v = match self.pc {
                    0xc46b => 0xd,
                    0xc1ae => 0x4,
                    0xc1b2 | 0xc42d => 0xa,
                    0xc23d => 0x6c,
                    0xc2c3 => 0xff,
                    0xc336 => 0xfe,
                    0xc33a => 0xfd,
                    0xc353 => 0xe0,
                    0xc431 => 0xb,
                    0xc435 => 0x3,
                    _ => unreachable!(),
                };
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a >= v);
                w.flags(a.wrapping_sub(v) & 255, 1);
                self.pc + 2
            }
            0xc46d | 0xc178 | 0xc1b0 | 0xc1b4 | 0xc234 | 0xc23f | 0xc2c5 | 0xc338 | 0xc380
            | 0xc3ed | 0xc42f | 0xc433 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    (match self.pc {
                        0xc46d => 0xc474,
                        0xc178 => 0xc175,
                        0xc1b0 | 0xc1b4 => 0xc228,
                        0xc234 | 0xc23f => 0xc244,
                        0xc2c5 => 0xc2d4,
                        0xc338 => 0xc345,
                        0xc380 => 0xc383,
                        0xc3ed => 0xc3f3,
                        0xc42f | 0xc433 => 0xc448,
                        _ => unreachable!(),
                    }) as u16
                } else {
                    self.pc + 2
                }
            }
            0xc46f | 0xc479 | 0xc490 | 0xc286 | 0xc2b7 | 0xc349 | 0xc387 | 0xc439 | 0xc448 => {
                w.drain(8);
                let v = match self.pc {
                    0xc46f => 0xfe,
                    0xc479 | 0xc286 => 0x0,
                    0xc490 | 0xc439 => 0xff,
                    0xc2b7 | 0xc349 => 0x1,
                    0xc387 => 0x8,
                    0xc448 => 0xa0,
                    _ => unreachable!(),
                };
                w.load(v, 1);
                self.pc + 2
            }
            0xc471 | 0xc17a | 0xc2a6 | 0xc2af | 0xc2b4 | 0xc38e | 0xc3ac | 0xc3ce => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0xc471 => 0x4b4,
                    0xc17a => 0xffa,
                    0xc2a6 => 0xfb1,
                    0xc2af => 0xfb0,
                    0xc2b4 => 0xfb3,
                    0xc38e | 0xc3ce => 0xfb5,
                    0xc3ac => 0xfb6,
                    _ => unreachable!(),
                });
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 3
            }
            0xc476 | 0xc482 => {
                w.drain(16);
                let v = match self.pc {
                    0xc476 => 0xfff,
                    0xc482 => 0x1ff,
                    _ => unreachable!(),
                };
                w.cpu.x = v;
                w.flags(v, 2);
                self.pc + 3
            }
            0xc47b | 0xc485 | 0xc1f2 | 0xc1f9 | 0xc200 | 0xc207 | 0xc20e | 0xc215 | 0xc21d
            | 0xc224 | 0xc288 => {
                w.drain(24);
                let a = match self.pc {
                    0xc47b => 0x7fdf80,
                    0xc485 => 0x7fef80,
                    0xc1f2 => 0x7ffa5c,
                    0xc1f9 => 0x7ffa6c,
                    0xc200 => 0x7ffa7c,
                    0xc207 => 0x7ffa8c,
                    0xc20e => 0x7ffa9c,
                    0xc215 => 0x7ffaac,
                    0xc21d => 0x7ffacc,
                    0xc224 => 0x7ffadc,
                    0xc288 => 0x7ff800,
                    _ => unreachable!(),
                } + u32::from(w.cpu.x);
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 4
            }
            0xc47f | 0xc489 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 65535;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xc480 | 0xc48a | 0xc496 | 0xc245 | 0xc24d | 0xc282 | 0xc28d | 0xc3f0 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    (match self.pc {
                        0xc480 => 0xc47b,
                        0xc48a => 0xc485,
                        0xc496 => 0xc492,
                        0xc245 => 0xc231,
                        0xc24d => 0xc249,
                        0xc282 => 0xc27e,
                        0xc28d => 0xc286,
                        0xc3f0 => 0xc3ea,
                        _ => unreachable!(),
                    }) as u16
                } else {
                    self.pc + 2
                }
            }
            0xc48c => {
                w.width(0x10, false);
                self.pc + 2
            }
            0xc48e => {
                w.drain(8);
                let v = 0x7;
                w.cpu.y = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0xc492 => {
                w.operand_then_internal(2);
                let a = w.absolute(0xb80) + u32::from(w.cpu.y);
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 3
            }
            0xc495 | 0xc330 | 0xc331 | 0xc34f | 0xc385 | 0xc386 | 0xc3f9 | 0xc3fa => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                self.pc + 1
            }
            0xc175 | 0xc22e | 0xc2d4 | 0xc344 | 0xc34e | 0xc35c | 0xc382 | 0xc3e7 | 0xc3f2
            | 0xc447 | 0xc44d | 0xc880 => {
                w.rts();
                if let Some(call) = self.calls.pop() {
                    assert_eq!(call.far, false);
                    assert_eq!((w.cpu.bank, w.cpu.pc), (9, call.returned));
                } else {
                    self.complete = true;
                }
                w.cpu.pc
            }
            0xc176 | 0xc236 | 0xc3df => {
                let v = w.dp_read(
                    match self.pc {
                        0xc176 | 0xc236 => 0x1b,
                        0xc3df => 0x2,
                        _ => unreachable!(),
                    },
                    1,
                );
                w.load(v, 1);
                self.pc + 2
            }
            0xc17f | 0xc241 | 0xc249 | 0xc27e | 0xc3c4 | 0xc3e4 | 0xc877 | 0xc87a | 0xc87d => {
                w.operand_then_internal(2);
                let a = w.absolute(match self.pc {
                    0xc17f => 0x1d00,
                    0xc241 => 0xdd0,
                    0xc249 => 0xc4a,
                    0xc27e => 0xb00,
                    0xc3c4 => 0xe30,
                    0xc3e4 => 0xcba,
                    0xc877 => 0xb30,
                    0xc87a => 0xb28,
                    0xc87d => 0xb38,
                    _ => unreachable!(),
                }) + u32::from(w.cpu.x);
                w.write(a, 0, 1);
                self.pc + 3
            }
            0xc182 | 0xc188 | 0xc18e | 0xc194 | 0xc19a | 0xc1a0 | 0xc1a6 | 0xc1ab | 0xc1b9
            | 0xc1bf | 0xc1c5 | 0xc1cb | 0xc1d1 | 0xc1d7 | 0xc1dd | 0xc1e3 | 0xc1e9 | 0xc1ef
            | 0xc1f6 | 0xc1fd | 0xc204 | 0xc20b | 0xc212 | 0xc221 | 0xc231 | 0xc23a | 0xc3ea
            | 0xc42a | 0xc43e => {
                w.drain(16);
                let a: u16 = match self.pc {
                    0xc182 | 0xc23a => 0xe20,
                    0xc188 => 0xd10,
                    0xc18e => 0xdc0,
                    0xc194 => 0xd30,
                    0xc19a => 0xd00,
                    0xc1a0 => 0xd20,
                    0xc1a6 => 0xf00,
                    0xc1ab | 0xc231 => 0xdd0,
                    0xc1b9 => 0xd90,
                    0xc1bf => 0xeb0,
                    0xc1c5 => 0xf50,
                    0xc1cb => 0xb89,
                    0xc1d1 => 0xde0,
                    0xc1d7 => 0xe40,
                    0xc1dd => 0xf20,
                    0xc1e3 => 0xd80,
                    0xc1e9 => 0xe60,
                    0xc1ef => 0xda0,
                    0xc1f6 => 0xdb0,
                    0xc1fd => 0xe90,
                    0xc204 => 0xe80,
                    0xc20b => 0xf70,
                    0xc212 => 0xdf0,
                    0xc221 => 0xba0,
                    0xc3ea | 0xc42a => 0xb00,
                    0xc43e => 0xb08,
                    _ => unreachable!(),
                };
                if false || (a & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                self.pc + 3
            }
            0xc185 | 0xc18b | 0xc191 | 0xc197 | 0xc19d | 0xc1a3 | 0xc1b6 | 0xc1bc | 0xc1c2
            | 0xc1c8 | 0xc1ce | 0xc1d4 | 0xc1da | 0xc1e0 | 0xc1e6 | 0xc1ec | 0xc34b | 0xc389
            | 0xc395 | 0xc39e | 0xc3a6 | 0xc3b3 | 0xc3bb | 0xc3c1 | 0xc3dc | 0xc3e1 | 0xc3f6
            | 0xc401 | 0xc40a | 0xc412 | 0xc41c | 0xc424 | 0xc43b | 0xc444 | 0xc44a | 0xc874 => {
                w.operand_then_internal(2);
                let a = w.absolute(match self.pc {
                    0xc185 => 0x1d10,
                    0xc18b => 0x1d20,
                    0xc191 => 0x1d60,
                    0xc197 => 0x1d30,
                    0xc19d => 0x1d40,
                    0xc1a3 => 0x1d50,
                    0xc1b6 => 0x1d00,
                    0xc1bc => 0x1d70,
                    0xc1c2 => 0x1d80,
                    0xc1c8 => 0x1d90,
                    0xc1ce => 0x1da0,
                    0xc1d4 => 0x1db0,
                    0xc1da => 0x1dc0,
                    0xc1e0 => 0x1dd0,
                    0xc1e6 => 0x1de0,
                    0xc1ec => 0x1df0,
                    0xc34b => 0xcba,
                    0xc389 => 0xdd0,
                    0xc395 => 0xf20,
                    0xc39e => 0xd00,
                    0xc3a6 => 0xd20,
                    0xc3b3 => 0xd10,
                    0xc3bb => 0xd30,
                    0xc3c1 => 0xe20,
                    0xc3dc => 0xe30,
                    0xc3e1 => 0xbc0,
                    0xc3f6 => 0xb00,
                    0xc401 => 0xb40,
                    0xc40a => 0xb18,
                    0xc412 => 0xb20,
                    0xc41c | 0xc444 => 0xb08,
                    0xc424 => 0xb10,
                    0xc43b | 0xc44a => 0xb30,
                    0xc874 => 0xcca,
                    _ => unreachable!(),
                }) + u32::from(w.cpu.x);
                w.write(a, w.cpu.a & 255, 1);
                self.pc + 3
            }
            0xc1a9 | 0xc238 | 0xc32e | 0xc33c | 0xc363 | 0xc437 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    (match self.pc {
                        0xc1a9 => 0xc228,
                        0xc238 => 0xc241,
                        0xc32e => 0xc34f,
                        0xc33c => 0xc35d,
                        0xc363 => 0xc383,
                        0xc437 => 0xc447,
                        _ => unreachable!(),
                    }) as u16
                } else {
                    self.pc + 2
                }
            }
            0xc219 | 0xc35d => {
                w.drain(24);
                let a = match self.pc {
                    0xc219 => 0x7ff9c2,
                    0xc35d => 0xdb725,
                    _ => unreachable!(),
                } + u32::from(w.cpu.x);
                let v = w.read(a, 1);
                w.load(v, 1);
                self.pc + 4
            }
            0xc228 | 0xc244 | 0xc24c | 0xc281 | 0xc28c | 0xc3ef => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0xc229 => {
                if w.branch(w.cpu.p & NEGATIVE != 0) {
                    (0xc22e) as u16
                } else {
                    self.pc + 2
                }
            }
            0xc22b => {
                w.drain(16);
                (0xc17f) as u16
            }
            0xc296 | 0xc371 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 65535;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xc297 => {
                w.drain(16);
                let a: u16 = 0xd62e;
                if true || (a & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(w.cpu.y), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xc29f | 0xc2a0 | 0xc2a1 => {
                w.drain(6);
                let a = w.cpu.a & 65535;
                w.cpu.p = (w.cpu.p & !CARRY) | (a & 1) as u8;
                w.load(a >> 1, 2);
                self.pc + 1
            }
            0xc2a4 | 0xc2ac | 0xc361 | 0xc391 | 0xc3ca | 0xc3fd => {
                w.drain(8);
                let v = match self.pc {
                    0xc2a4 => 0xfe,
                    0xc2ac => 0xf,
                    0xc361 => 0x1,
                    0xc391 | 0xc3fd => 0x80,
                    0xc3ca => 0x60,
                    _ => unreachable!(),
                };
                w.load((w.cpu.a & v) & 255, 1);
                self.pc + 2
            }
            0xc2a9 | 0xc3a1 | 0xc3b6 | 0xc3c7 | 0xc3d1 | 0xc40d | 0xc41f | 0xc871 => {
                w.drain(16);
                let a = w.absolute(match self.pc {
                    0xc2a9 => 0x48e,
                    0xc3a1 | 0xc40d => 0xfb1,
                    0xc3b6 | 0xc41f => 0xfb0,
                    0xc3c7 => 0xfb5,
                    0xc3d1 => 0xfb6,
                    0xc871 => 0x40a,
                    _ => unreachable!(),
                });
                let v = w.read(a, 1);
                w.load(v, 1);
                self.pc + 3
            }
            0xc2ae | 0xc393 | 0xc39a | 0xc39b | 0xc39c | 0xc39d | 0xc3af | 0xc3b0 | 0xc3b1
            | 0xc3b2 | 0xc3ff | 0xc406 | 0xc407 | 0xc408 | 0xc409 | 0xc418 | 0xc419 | 0xc41a
            | 0xc41b => {
                w.shift_left(1);
                self.pc + 1
            }
            0xc2b2 => {
                let pointer = w.dp_read(0x0, 2);
                let address = w.absolute(pointer);
                let v = w.read(address, 1);
                w.load(v, 1);
                self.pc + 2
            }
            0xc2b9 => {
                w.dp_store(0x4, w.cpu.a & 255, 1);
                self.pc + 2
            }
            0xc2bb | 0xc2bd => {
                w.dp_store(
                    match self.pc {
                        0xc2bb => 0x2,
                        0xc2bd => 0x3,
                        _ => unreachable!(),
                    },
                    0,
                    1,
                );
                self.pc + 2
            }
            0xc2bf => {
                let v = w.dp_read(0x4, 1);
                w.cpu.y = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0xc2c1 | 0xc329 | 0xc332 | 0xc350 | 0xc38c | 0xc398 | 0xc3aa | 0xc3bf | 0xc3f3
            | 0xc3fb | 0xc404 | 0xc416 => {
                let pointer = w.dp_read(0x0, 2);
                if false || (pointer & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let address = w.absolute(pointer) + u32::from(w.cpu.y);
                let v = w.read(address, 1);
                w.load(v, 1);
                self.pc + 2
            }
            0xc2ca | 0xc2cc | 0xc2ce | 0xc2d0 => {
                w.dp_increment(
                    match self.pc {
                        0xc2ca => 0x2,
                        0xc2cc | 0xc2ce | 0xc2d0 => 0x4,
                        _ => unreachable!(),
                    },
                    1,
                    true,
                );
                self.pc + 2
            }
            0xc2d2 => {
                if w.branch(true) {
                    (0xc2bf) as u16
                } else {
                    self.pc + 2
                }
            }
            0xc327 | 0xc328 | 0xc334 | 0xc335 | 0xc352 | 0xc3a9 | 0xc3be | 0xc415 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & 255;
                w.flags(w.cpu.y, 1);
                self.pc + 1
            }
            0xc32b => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0xc341 => {
                w.operand_then_internal(2);
                let a = w.absolute(0xcba) + u32::from(w.cpu.x);
                let v = w.read(a, 1).wrapping_add(1) & 255;
                w.drain(6);
                w.flags(v, 1);
                w.write(a, v, 1);
                self.pc + 3
            }
            0xc345 | 0xc35a => {
                w.dp_increment(0x2, 1, false);
                self.pc + 2
            }
            0xc347 | 0xc383 => {
                let v = w.dp_read(0x2, 1);
                w.cpu.x = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0xc355 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    (0xc35d) as u16
                } else {
                    self.pc + 2
                }
            }
            0xc367 => {
                w.drain(6);
                w.push(w.cpu.y, 2);
                self.pc + 1
            }
            0xc368 => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                self.pc + 1
            }
            0xc372 => {
                w.drain(24);
                let a = 0x7fdf80 + u32::from(w.cpu.x);
                let v = w.read(a, 2);
                w.load(v, 2);
                self.pc + 4
            }
            0xc376 => {
                w.drain(16);
                let a: u16 = 0xc2d5;
                if true || (a & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(w.cpu.y), 2);
                w.load((w.cpu.a & v) & 65535, 2);
                self.pc + 3
            }
            0xc379 => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xc37a => {
                w.drain(12);
                w.cpu.y = w.pull(2);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xc394 | 0xc400 => {
                w.drain(6);
                let a = w.cpu.a & 255;
                let carry = u16::from(w.cpu.p & CARRY != 0);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(a & 128 != 0);
                w.load(((a << 1) | carry) & 255, 1);
                self.pc + 1
            }
            0xc3a4 | 0xc3b9 | 0xc410 | 0xc422 => {
                w.drain(8);
                w.adc_byte(0x0);
                self.pc + 2
            }
            0xc3cc | 0xc3cd | 0xc3d4 | 0xc3d5 | 0xc3d6 | 0xc3d7 | 0xc3d8 => {
                w.drain(6);
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (a & 1) as u8;
                w.load(a >> 1, 1);
                self.pc + 1
            }
            0xc3d9 => {
                w.drain(16);
                let a = w.absolute(0xfb5);
                let v = w.read(a, 1);
                w.load((w.cpu.a | v) & 255, 1);
                self.pc + 3
            }
            0xc3f5 => {
                w.drain(6);
                self.pc + 1
            }
            0xc441 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xc442 => {
                w.drain(8);
                w.sbc(0x8, 1);
                self.pc + 2
            }
            _ => unreachable!("sprite reset continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
#[cfg(test)]
#[path = "sprite_reset_tests.rs"]
mod tests;
#[cfg(test)]
pub(super) const OWNED_POSITIONS: &[u16] = &[
    0xc114, 0xc115, 0xc116, 0xc117, 0xc11a, 0xc11d, 0xc120, 0xc124, 0xc126, 0xc129, 0xc12c, 0xc12f,
    0xc131, 0xc134, 0xc137, 0xc139, 0xc13a, 0xc13b, 0xc13d, 0xc13f, 0xc142, 0xc144, 0xc147, 0xc14a,
    0xc14d, 0xc150, 0xc153, 0xc156, 0xc159, 0xc15c, 0xc15e, 0xc160, 0xc163, 0xc165, 0xc166, 0xc167,
    0xc16a, 0xc16e, 0xc170, 0xc173, 0xc174, 0xc175, 0xc176, 0xc178, 0xc17a, 0xc17d, 0xc17f, 0xc182,
    0xc185, 0xc188, 0xc18b, 0xc18e, 0xc191, 0xc194, 0xc197, 0xc19a, 0xc19d, 0xc1a0, 0xc1a3, 0xc1a6,
    0xc1a9, 0xc1ab, 0xc1ae, 0xc1b0, 0xc1b2, 0xc1b4, 0xc1b6, 0xc1b9, 0xc1bc, 0xc1bf, 0xc1c2, 0xc1c5,
    0xc1c8, 0xc1cb, 0xc1ce, 0xc1d1, 0xc1d4, 0xc1d7, 0xc1da, 0xc1dd, 0xc1e0, 0xc1e3, 0xc1e6, 0xc1e9,
    0xc1ec, 0xc1ef, 0xc1f2, 0xc1f6, 0xc1f9, 0xc1fd, 0xc200, 0xc204, 0xc207, 0xc20b, 0xc20e, 0xc212,
    0xc215, 0xc219, 0xc21d, 0xc221, 0xc224, 0xc228, 0xc229, 0xc22b, 0xc22e, 0xc22f, 0xc231, 0xc234,
    0xc236, 0xc238, 0xc23a, 0xc23d, 0xc23f, 0xc241, 0xc244, 0xc245, 0xc247, 0xc249, 0xc24c, 0xc24d,
    0xc24f, 0xc252, 0xc255, 0xc258, 0xc25b, 0xc25e, 0xc261, 0xc264, 0xc267, 0xc26a, 0xc26d, 0xc270,
    0xc273, 0xc276, 0xc279, 0xc27c, 0xc27e, 0xc281, 0xc282, 0xc284, 0xc286, 0xc288, 0xc28c, 0xc28d,
    0xc28f, 0xc290, 0xc292, 0xc295, 0xc296, 0xc297, 0xc29a, 0xc29c, 0xc29f, 0xc2a0, 0xc2a1, 0xc2a2,
    0xc2a4, 0xc2a6, 0xc2a9, 0xc2ac, 0xc2ae, 0xc2af, 0xc2b2, 0xc2b4, 0xc2b7, 0xc2b9, 0xc2bb, 0xc2bd,
    0xc2bf, 0xc2c1, 0xc2c3, 0xc2c5, 0xc2c7, 0xc2ca, 0xc2cc, 0xc2ce, 0xc2d0, 0xc2d2, 0xc2d4, 0xc327,
    0xc328, 0xc329, 0xc32b, 0xc32c, 0xc32e, 0xc330, 0xc331, 0xc332, 0xc334, 0xc335, 0xc336, 0xc338,
    0xc33a, 0xc33c, 0xc33e, 0xc341, 0xc344, 0xc345, 0xc347, 0xc349, 0xc34b, 0xc34e, 0xc34f, 0xc350,
    0xc352, 0xc353, 0xc355, 0xc357, 0xc35a, 0xc35c, 0xc35d, 0xc361, 0xc363, 0xc365, 0xc367, 0xc368,
    0xc369, 0xc36c, 0xc36d, 0xc36e, 0xc370, 0xc371, 0xc372, 0xc376, 0xc379, 0xc37a, 0xc37b, 0xc37e,
    0xc380, 0xc382, 0xc383, 0xc385, 0xc386, 0xc387, 0xc389, 0xc38c, 0xc38e, 0xc391, 0xc393, 0xc394,
    0xc395, 0xc398, 0xc39a, 0xc39b, 0xc39c, 0xc39d, 0xc39e, 0xc3a1, 0xc3a4, 0xc3a6, 0xc3a9, 0xc3aa,
    0xc3ac, 0xc3af, 0xc3b0, 0xc3b1, 0xc3b2, 0xc3b3, 0xc3b6, 0xc3b9, 0xc3bb, 0xc3be, 0xc3bf, 0xc3c1,
    0xc3c4, 0xc3c7, 0xc3ca, 0xc3cc, 0xc3cd, 0xc3ce, 0xc3d1, 0xc3d4, 0xc3d5, 0xc3d6, 0xc3d7, 0xc3d8,
    0xc3d9, 0xc3dc, 0xc3df, 0xc3e1, 0xc3e4, 0xc3e7, 0xc3e8, 0xc3ea, 0xc3ed, 0xc3ef, 0xc3f0, 0xc3f2,
    0xc3f3, 0xc3f5, 0xc3f6, 0xc3f9, 0xc3fa, 0xc3fb, 0xc3fd, 0xc3ff, 0xc400, 0xc401, 0xc404, 0xc406,
    0xc407, 0xc408, 0xc409, 0xc40a, 0xc40d, 0xc410, 0xc412, 0xc415, 0xc416, 0xc418, 0xc419, 0xc41a,
    0xc41b, 0xc41c, 0xc41f, 0xc422, 0xc424, 0xc427, 0xc42a, 0xc42d, 0xc42f, 0xc431, 0xc433, 0xc435,
    0xc437, 0xc439, 0xc43b, 0xc43e, 0xc441, 0xc442, 0xc444, 0xc447, 0xc448, 0xc44a, 0xc44d, 0xc44e,
    0xc452, 0xc455, 0xc458, 0xc45b, 0xc45e, 0xc461, 0xc464, 0xc467, 0xc46b, 0xc46d, 0xc46f, 0xc471,
    0xc474, 0xc476, 0xc479, 0xc47b, 0xc47f, 0xc480, 0xc482, 0xc485, 0xc489, 0xc48a, 0xc48c, 0xc48e,
    0xc490, 0xc492, 0xc495, 0xc496, 0xc498, 0xc871, 0xc874, 0xc877, 0xc87a, 0xc87d, 0xc880,
];
