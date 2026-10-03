//! Object attributes retain the live list cursors and attribute identities.
//! The chest helper returns through its real stack frame before the next list.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObjectAttributes {
    pc: u16,
    chest_return: bool,
    complete: bool,
}
impl Default for ObjectAttributes {
    fn default() -> Self {
        Self {
            pc: 0xb967,
            chest_return: false,
            complete: false,
        }
    }
}
impl ObjectAttributes {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.chest_return {
            0x01bddb
        } else {
            0x01b967
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
        if self.pc != 0xb967 {
            assert_eq!(cpu.p & 0x30, 0);
        }
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xb967 => {
                w.width(0x30, true);
                self.pc + 2
            }
            0xb969 | 0xb98e | 0xbb47 | 0xbb51 | 0xbb56 | 0xbbe4 | 0xbc4c | 0xbcbd | 0xbd33
            | 0xbd3d | 0xbd42 | 0xbd9f => {
                w.drain(16);
                let a = match self.pc {
                    0xb969 => 0x432,
                    0xb98e => 0x438,
                    0xbb47 => 0x43c,
                    0xbb51 => 0x43e,
                    0xbb56 => 0x440,
                    0xbbe4 => 0x442,
                    0xbc4c => 0x42c,
                    0xbcbd => 0x496,
                    0xbd33 => 0x49a,
                    0xbd3d => 0x49c,
                    0xbd42 => 0x49e,
                    0xbd9f => 0x4ae,
                    _ => unreachable!(),
                };
                let v = w.read(a, 2);
                w.cpu.x = v;
                w.flags(v, 2);
                self.pc + 3
            }
            0xb96c | 0xb991 | 0xb9ba | 0xb9e4 | 0xba0e | 0xba3b | 0xba72 | 0xba94 | 0xbabe
            | 0xbae8 | 0xbb15 | 0xbb59 | 0xbbaf | 0xbbe7 | 0xbc1f | 0xbc4f | 0xbc5f | 0xbc85
            | 0xbcc0 | 0xbcca | 0xbccf | 0xbcd4 | 0xbce8 | 0xbced | 0xbcf2 | 0xbd04 | 0xbd45
            | 0xbda2 | 0xbdde => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    (match self.pc {
                        0xb96c => 0x1b986,
                        0xb991 => 0x1b9b7,
                        0xb9ba => 0x1b9e1,
                        0xb9e4 => 0x1ba0b,
                        0xba0e => 0x1ba38,
                        0xba3b => 0x1ba65,
                        0xba72 => 0x1ba91,
                        0xba94 => 0x1babb,
                        0xbabe => 0x1bae5,
                        0xbae8 => 0x1bb12,
                        0xbb15 => 0x1bb3f,
                        0xbb59 => 0x1bbac,
                        0xbbaf => 0x1bbe1,
                        0xbbe7 => 0x1bc1c,
                        0xbc1f => 0x1bc49,
                        0xbc4f => 0x1bc82,
                        0xbc5f => 0x1bc73,
                        0xbc85 => 0x1bcb5,
                        0xbcc0 => 0x1bd01,
                        0xbcca | 0xbccf | 0xbcd4 | 0xbce8 | 0xbced | 0xbcf2 | 0xbd04 => 0x1bd2b,
                        0xbd45 => 0x1bd9c,
                        0xbda2 => 0x1bdd7,
                        0xbdde => 0x1be07,
                        _ => unreachable!(),
                    }) as u16
                } else {
                    self.pc + 2
                }
            }
            0xb96e | 0xb98b | 0xbb3f | 0xbbe1 | 0xbc49 | 0xbcba | 0xbd2b | 0xbd9c => {
                w.drain(16);
                let v = 0x0;
                w.cpu.y = v;
                w.flags(v, 2);
                self.pc + 3
            }
            0xb971 | 0xb986 | 0xb996 | 0xb99d | 0xb9bf | 0xb9e9 | 0xba13 | 0xba1e | 0xba40
            | 0xba4b | 0xba77 | 0xba99 | 0xbac3 | 0xbaed | 0xbaf8 | 0xbb1a | 0xbb25 | 0xbb44
            | 0xbb4e | 0xbb5d | 0xbb6c | 0xbb80 | 0xbbb1 | 0xbbba | 0xbbce | 0xbbe9 | 0xbbf2
            | 0xbbfe | 0xbc0a | 0xbc21 | 0xbc2a | 0xbc36 | 0xbc51 | 0xbc89 | 0xbcb5 | 0xbd30
            | 0xbd3a | 0xbd49 | 0xbd58 | 0xbd63 | 0xbd6e | 0xbd79 | 0xbda4 | 0xbdad | 0xbdb9
            | 0xbdc5 => {
                w.drain(16);
                let v = match self.pc {
                    0xb971 => 0x3b3b,
                    0xb986 => 0x3030,
                    0xb996 | 0xba13 | 0xba40 | 0xbaed | 0xbb1a => 0x0,
                    0xb99d | 0xba77 => 0x2626,
                    0xb9bf | 0xba99 => 0x5e5e,
                    0xb9e9 | 0xbac3 => 0x5f5f,
                    0xba1e | 0xbaf8 => 0x3838,
                    0xba4b | 0xbb25 => 0x3939,
                    0xbb44 => 0x1f1f,
                    0xbb4e => 0x1e1e,
                    0xbb5d => 0x1d1d,
                    0xbb6c | 0xbbb1 | 0xbbe9 | 0xbc21 | 0xbd58 | 0xbda4 => 0x2,
                    0xbb80 | 0xbd63 => 0x1,
                    0xbbba | 0xbbfe | 0xbc36 | 0xbdad => 0xa03,
                    0xbbce => 0x803,
                    0xbbf2 | 0xbc2a | 0xbdb9 => 0x3,
                    0xbc0a | 0xbdc5 => 0x808,
                    0xbc51 => 0x7070,
                    0xbc89 => 0xc0c0,
                    0xbcb5 => 0x5858,
                    0xbd30 => 0x3f3f,
                    0xbd3a => 0x3e3e,
                    0xbd49 => 0x3d3d,
                    0xbd6e => 0x200,
                    0xbd79 => 0x100,
                    _ => unreachable!(),
                };
                w.load(v, 2);
                self.pc + 3
            }
            0xb974 | 0xb993 | 0xb9bc | 0xb9e6 | 0xba10 | 0xba3d | 0xba74 | 0xba96 | 0xbac0
            | 0xbaea | 0xbb17 | 0xbb69 | 0xbbb7 | 0xbbef | 0xbc27 | 0xbd55 | 0xbdaa => {
                w.operand_then_internal(2);
                let a = match self.pc {
                    0xb974 => 0x6a0,
                    0xb993 | 0xb9bc | 0xb9e6 | 0xba10 | 0xba3d | 0xba74 | 0xba96 | 0xbac0
                    | 0xbaea | 0xbb17 => 0x6b0,
                    0xbb69 | 0xbbb7 | 0xbbef | 0xbc27 => 0x6b8,
                    0xbd55 | 0xbdaa => 0x6ec,
                    _ => unreachable!(),
                };
                let v = w.read(a + u32::from(w.cpu.y), 2);
                w.cpu.x = v;
                w.flags(v, 2);
                self.pc + 3
            }
            0xb977 | 0xb97b | 0xb999 | 0xb9a0 | 0xb9a6 | 0xb9c2 | 0xb9c6 | 0xb9ca | 0xb9d0
            | 0xb9ec | 0xb9f0 | 0xb9f4 | 0xb9fa | 0xba16 | 0xba1a | 0xba21 | 0xba27 | 0xba43
            | 0xba47 | 0xba4e | 0xba54 | 0xba7a | 0xba80 | 0xba9c | 0xbaa0 | 0xbaa4 | 0xbaaa
            | 0xbac6 | 0xbaca | 0xbace | 0xbad4 | 0xbaf0 | 0xbaf4 | 0xbafb | 0xbb01 | 0xbb1d
            | 0xbb21 | 0xbb28 | 0xbb2e | 0xbb6f | 0xbb73 | 0xbb78 | 0xbb7c | 0xbb83 | 0xbb87
            | 0xbb8c | 0xbb90 | 0xbb96 | 0xbb9a | 0xbb9e | 0xbba2 | 0xbbbd | 0xbbc1 | 0xbbc6
            | 0xbbca | 0xbbd1 | 0xbbd6 | 0xbbf5 | 0xbbfa | 0xbc01 | 0xbc06 | 0xbc0d | 0xbc11
            | 0xbc2d | 0xbc32 | 0xbc39 | 0xbc3e | 0xbc6b | 0xbc6f | 0xbc98 | 0xbc9c | 0xbd16
            | 0xbd1a | 0xbd5b | 0xbd5f | 0xbd66 | 0xbd6a | 0xbd71 | 0xbd75 | 0xbd7c | 0xbd80
            | 0xbd86 | 0xbd8a | 0xbd8e | 0xbd92 | 0xbdb0 | 0xbdb5 | 0xbdbc | 0xbdc1 | 0xbdc8
            | 0xbdcc | 0xbde7 | 0xbdeb | 0xbdfb | 0xbdff | 0xbe03 => {
                w.drain(24);
                let a = match self.pc {
                    0xb977 | 0xbb6f | 0xbbbd | 0xbbf5 | 0xbc2d | 0xbc6b | 0xbc98 | 0xbd16
                    | 0xbde7 => 0x7f2000,
                    0xb97b | 0xbb83 | 0xbbd1 | 0xbc0d | 0xbc6f | 0xbc9c | 0xbd1a | 0xbdeb => {
                        0x7f2040
                    }
                    0xb999 | 0xb9c6 | 0xb9f0 | 0xba16 | 0xba54 | 0xba80 | 0xbaa0 | 0xbaca
                    | 0xbaf0 | 0xbb2e | 0xbb9e | 0xbd8e => 0x7f2081,
                    0xb9a0 | 0xb9c2 | 0xb9ec | 0xba21 | 0xba43 | 0xba9c | 0xbac6 | 0xbafb
                    | 0xbb1d => 0x7f2001,
                    0xb9a6 | 0xb9d0 | 0xb9fa | 0xba27 | 0xba47 | 0xbaaa | 0xbad4 | 0xbb01
                    | 0xbb21 | 0xbb96 | 0xbd86 => 0x7f2041,
                    0xb9ca | 0xb9f4 | 0xba1a | 0xba4e | 0xba7a | 0xbaa4 | 0xbace | 0xbaf4
                    | 0xbb28 => 0x7f20c1,
                    0xbb73 | 0xbdb0 => 0x7f30c0,
                    0xbb78 | 0xbbc6 | 0xbbfa | 0xbc32 => 0x7f2002,
                    0xbb7c | 0xbdb5 => 0x7f30c2,
                    0xbb87 => 0x7f3080,
                    0xbb8c | 0xbbd6 | 0xbc11 | 0xbdfb => 0x7f2042,
                    0xbb90 => 0x7f3082,
                    0xbb9a | 0xbd8a => 0x7f3041,
                    0xbba2 | 0xbd92 => 0x7f3081,
                    0xbbc1 | 0xbc01 | 0xbc39 | 0xbd5b => 0x7f3000,
                    0xbbca | 0xbc06 | 0xbc3e | 0xbd71 => 0x7f3002,
                    0xbd5f | 0xbdbc => 0x7f20c0,
                    0xbd66 => 0x7f3040,
                    0xbd6a | 0xbdc8 | 0xbdff => 0x7f2080,
                    0xbd75 | 0xbdc1 => 0x7f20c2,
                    0xbd7c => 0x7f3042,
                    0xbd80 | 0xbdcc | 0xbe03 => 0x7f2082,
                    _ => unreachable!(),
                };
                w.write(a + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0xb97f | 0xb980 | 0xb9b0 | 0xb9b1 | 0xb9da | 0xb9db | 0xba04 | 0xba05 | 0xba31
            | 0xba32 | 0xba5e | 0xba5f | 0xba8a | 0xba8b | 0xbab4 | 0xbab5 | 0xbade | 0xbadf
            | 0xbb0b | 0xbb0c | 0xbb38 | 0xbb39 | 0xbba6 | 0xbba7 | 0xbbda | 0xbbdb | 0xbc15
            | 0xbc16 | 0xbc42 | 0xbc43 | 0xbc7b | 0xbc7c | 0xbca9 | 0xbcaa | 0xbd24 | 0xbd25
            | 0xbd96 | 0xbd97 | 0xbdd0 | 0xbdd1 | 0xbe0f | 0xbe10 => {
                w.increment_y();
                self.pc + 1
            }
            0xb981 | 0xb9b2 | 0xb9b7 | 0xb9dc | 0xb9e1 | 0xba06 | 0xba0b | 0xba33 | 0xba38
            | 0xba60 | 0xba6f | 0xba8c | 0xba91 | 0xbab6 | 0xbabb | 0xbae0 | 0xbae5 | 0xbb0d
            | 0xbb12 | 0xbb3a | 0xbbac | 0xbbdc | 0xbc17 | 0xbc1c | 0xbc44 | 0xbc7d | 0xbc82
            | 0xbcab | 0xbd01 | 0xbd26 | 0xbdd2 | 0xbe11 => {
                w.drain(16);
                let v = w.read(
                    match self.pc {
                        0xb981 => 0x432,
                        0xb9b2 => 0x438,
                        0xb9b7 | 0xb9dc => 0x47e,
                        0xb9e1 | 0xba06 => 0x482,
                        0xba0b | 0xba33 => 0x4a2,
                        0xba38 | 0xba60 => 0x4a4,
                        0xba6f | 0xba8c => 0x43a,
                        0xba91 | 0xbab6 => 0x480,
                        0xbabb | 0xbae0 => 0x484,
                        0xbae5 | 0xbb0d => 0x4a6,
                        0xbb12 | 0xbb3a => 0x4a8,
                        0xbbac | 0xbbdc => 0x448,
                        0xbc17 => 0x442,
                        0xbc1c | 0xbc44 => 0x444,
                        0xbc7d => 0x42c,
                        0xbc82 | 0xbcab => 0x42e,
                        0xbd01 | 0xbd26 => 0x498,
                        0xbdd2 => 0x4ae,
                        0xbe11 => 0x496,
                        _ => unreachable!(),
                    },
                    2,
                );
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(w.cpu.y.wrapping_sub(v), 2);
                self.pc + 3
            }
            0xb984 | 0xb9b5 | 0xb9df | 0xba09 | 0xba36 | 0xba63 | 0xba8f | 0xbab9 | 0xbae3
            | 0xbb10 | 0xbb3d | 0xbb4a | 0xbb54 | 0xbbaa | 0xbbdf | 0xbc1a | 0xbc47 | 0xbc80
            | 0xbcae | 0xbd29 | 0xbd36 | 0xbd40 | 0xbd9a | 0xbdd5 | 0xbe14 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    (match self.pc {
                        0xb984 => 0x1b974,
                        0xb9b5 => 0x1b993,
                        0xb9df => 0x1b9bc,
                        0xba09 => 0x1b9e6,
                        0xba36 => 0x1ba10,
                        0xba63 => 0x1ba3d,
                        0xba8f => 0x1ba74,
                        0xbab9 => 0x1ba96,
                        0xbae3 => 0x1bac0,
                        0xbb10 => 0x1baea,
                        0xbb3d => 0x1bb17,
                        0xbb4a | 0xbb54 => 0x1bb60,
                        0xbbaa => 0x1bb69,
                        0xbbdf => 0x1bbb7,
                        0xbc1a => 0x1bbef,
                        0xbc47 => 0x1bc27,
                        0xbc80 => 0x1bc56,
                        0xbcae => 0x1bc8e,
                        0xbd29 => 0x1bd06,
                        0xbd36 | 0xbd40 => 0x1bd4c,
                        0xbd9a => 0x1bd55,
                        0xbdd5 => 0x1bdaa,
                        0xbe14 => 0x1bddb,
                        _ => unreachable!(),
                    }) as u16
                } else {
                    self.pc + 2
                }
            }
            0xb989 | 0xb9ae | 0xb9d8 | 0xba02 | 0xba2f | 0xba5c | 0xba6d | 0xba88 | 0xbab2
            | 0xbadc | 0xbb09 | 0xbb36 | 0xbb60 | 0xbc54 | 0xbc79 | 0xbc8c | 0xbca7 | 0xbcb8
            | 0xbd22 | 0xbd4c | 0xbe0d => {
                w.dp_store(0x0, w.cpu.a, 2);
                self.pc + 2
            }
            0xb9a4 | 0xb9ce | 0xb9f8 | 0xba25 | 0xba52 | 0xba65 | 0xba7e | 0xbaa8 | 0xbad2
            | 0xbaff | 0xbb2c | 0xbb62 | 0xbb94 | 0xbc69 | 0xbc73 | 0xbc96 | 0xbcb0 | 0xbcc2
            | 0xbce0 | 0xbd14 | 0xbd4e | 0xbd84 | 0xbde5 | 0xbdf9 | 0xbe07 => {
                let v = w.dp_read(
                    match self.pc {
                        0xb9a4 | 0xb9ce | 0xb9f8 | 0xba25 | 0xba52 | 0xba65 | 0xba7e | 0xbaa8
                        | 0xbad2 | 0xbaff | 0xbb2c | 0xbb94 | 0xbc69 | 0xbc73 | 0xbc96 | 0xbd14
                        | 0xbd84 | 0xbde5 | 0xbdf9 | 0xbe07 => 0x0,
                        0xbb62 | 0xbd4e => 0x2,
                        0xbcb0 => 0x4,
                        0xbcc2 => 0xae,
                        0xbce0 => 0xaf,
                        _ => unreachable!(),
                    },
                    2,
                );
                w.load(v, 2);
                self.pc + 2
            }
            0xb9aa | 0xb9d4 | 0xb9fe | 0xba2b | 0xba58 | 0xba84 | 0xbaae | 0xbad8 | 0xbb05
            | 0xbb32 | 0xbc75 | 0xbca3 | 0xbd1e | 0xbe09 => {
                w.clear_carry();
                self.pc + 1
            }
            0xb9ab | 0xb9d5 | 0xb9ff | 0xba2c | 0xba59 | 0xba85 | 0xbaaf | 0xbad9 | 0xbb06
            | 0xbb33 | 0xbc76 | 0xbca4 | 0xbd1f | 0xbe0a => {
                w.drain(16);
                w.adc(0x101);
                self.pc + 3
            }
            0xba67 | 0xbc59 | 0xbc64 | 0xbc91 | 0xbca0 | 0xbcc4 | 0xbce2 | 0xbd0f | 0xbde0 => {
                w.mask(
                    match self.pc {
                        0xba67 => 0x707,
                        0xbc59 => 0xf0,
                        0xbc64 | 0xbc91 => 0x3fff,
                        0xbca0 => 0xefef,
                        0xbcc4 | 0xbce2 => 0xff,
                        0xbd0f | 0xbde0 => 0x7fff,
                        _ => unreachable!(),
                    },
                    2,
                );
                self.pc + 3
            }
            0xba6a | 0xbd09 => {
                w.drain(16);
                let v = match self.pc {
                    0xba6a => 0x3434,
                    0xbd09 => 0x8000,
                    _ => unreachable!(),
                };
                w.load(w.cpu.a | v, 2);
                self.pc + 3
            }
            0xbb42 | 0xbd2e => {
                w.dp_store(0x2, w.cpu.y, 2);
                self.pc + 2
            }
            0xbb4c | 0xbb5b | 0xbd38 | 0xbd47 => {
                w.dp_increment(0x2, 2, true);
                self.pc + 2
            }
            0xbb64 | 0xbbb4 | 0xbbec | 0xbc24 | 0xbcb2 | 0xbd50 | 0xbda7 => {
                w.drain(16);
                let a = match self.pc {
                    0xbb64 | 0xbbb4 | 0xbbec | 0xbc24 | 0xbd50 | 0xbda7 => 0x44a,
                    0xbcb2 => 0x42e,
                    _ => unreachable!(),
                };
                w.write(a, w.cpu.a, 2);
                self.pc + 3
            }
            0xbb67 | 0xbd53 => {
                w.dp_store(0x2, w.cpu.x, 2);
                self.pc + 2
            }
            0xbb77 | 0xbb8b | 0xbbc5 | 0xbbd5 | 0xbbf9 | 0xbc05 | 0xbc31 | 0xbc3d | 0xbdb4
            | 0xbdc0 => {
                w.swap_bytes();
                self.pc + 1
            }
            0xbba8 | 0xbd98 => {
                let v = w.dp_read(0x2, 2);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= v);
                w.flags(w.cpu.y.wrapping_sub(v), 2);
                self.pc + 2
            }
            0xbc56 | 0xbc61 | 0xbc8e | 0xbd06 | 0xbddb | 0xbdef => {
                w.operand_then_internal(2);
                let a = match self.pc {
                    0xbc56 => 0x500,
                    0xbc61 | 0xbc8e => 0x540,
                    0xbd06 | 0xbddb | 0xbdef => 0x6e0,
                    _ => unreachable!(),
                };
                let v = w.read(a + u32::from(w.cpu.y), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xbc5c | 0xbcc7 | 0xbccc | 0xbcd1 | 0xbcd6 | 0xbcdb | 0xbce5 | 0xbcea | 0xbcef
            | 0xbcf4 | 0xbcf9 => {
                w.compare_immediate(
                    match self.pc {
                        0xbc5c => 0x30,
                        0xbcc7 | 0xbce5 => 0x27,
                        0xbccc | 0xbcea => 0x3c,
                        0xbcd1 | 0xbcef => 0x3e,
                        0xbcd6 | 0xbcf4 => 0x29,
                        0xbcdb | 0xbcf9 => 0x33,
                        _ => unreachable!(),
                    },
                    2,
                );
                self.pc + 3
            }
            0xbc67 | 0xbc94 | 0xbd12 | 0xbde3 | 0xbdf5 => {
                w.drain(6);
                let v = w.cpu.a;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, 2);
                self.pc + 1
            }
            0xbc68 | 0xbc95 | 0xbd13 | 0xbde4 => {
                w.drain(6);
                w.cpu.x = w.cpu.a;
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0xbc87 => {
                w.dp_store(0x4, 0, 2);
                self.pc + 2
            }
            0xbcd9 | 0xbcde | 0xbcf7 | 0xbcfc | 0xbdf3 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    (match self.pc {
                        0xbcd9 => 0x1bce0,
                        0xbcde | 0xbcfc => 0x1bd2b,
                        0xbcf7 => 0x1bcfe,
                        0xbdf3 => 0x1be07,
                        _ => unreachable!(),
                    }) as u16
                } else {
                    self.pc + 2
                }
            }
            0xbcfe => {
                w.operand_then_internal(2);
                w.push(0xbd00, 2);
                assert!(!self.chest_return);
                self.chest_return = true;
                0xbddb
            }
            0xbd0c | 0xbdf6 => {
                w.operand_then_internal(2);
                let a = 0x6e0;
                w.write(a + u32::from(w.cpu.y), w.cpu.a, 2);
                self.pc + 3
            }
            0xbdd7 => {
                w.drain(16);
                let a = 0x200;
                let v = w.read(a, 2).wrapping_add(1);
                w.drain(6);
                w.flags(v, 2);
                w.write(a, v, 2);
                self.pc + 3
            }
            0xbdda | 0xbe16 => {
                w.rts();
                if self.pc == 0xbe16 {
                    assert!(self.chest_return);
                    assert_eq!(w.cpu.pc, 0xbd01);
                    self.chest_return = false;
                } else {
                    assert!(!self.chest_return);
                    self.complete = true;
                }
                w.cpu.pc
            }
            0xbdf2 => {
                w.shift_left(2);
                self.pc + 1
            }
            _ => unreachable!("object attribute continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
