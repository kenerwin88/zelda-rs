//! Large room structures and floodgate flooring, retaining live calls and pointers.
use super::{
    dungeon_objects::DungeonObjectTiles,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonRoomDecor {
    pc: u16,
    depth: u8,
    tiles: Option<DungeonObjectTiles>,
    tail: bool,
    complete: bool,
}
impl DungeonRoomDecor {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(
            entry,
            0x9a12 | 0x9aa9 | 0x9b56 | 0x9bf8 | 0x9de5 | 0x9e30 | 0x9ea3 | 0xa095 | 0xa1d1
        ));
        Self {
            pc: entry,
            depth: 0,
            tiles: None,
            tail: false,
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
        if let Some(tiles) = self.tiles.as_mut() {
            let cycles = tiles.step(cpu, clock, bus);
            self.pc = cpu.pc;
            if tiles.complete() {
                self.tiles = None;
                if self.tail {
                    if self.depth == 0 {
                        self.complete = true;
                    } else {
                        self.depth -= 1;
                    }
                }
            }
            return cycles;
        }
        let mode = match self.pc {
            0x9c2f | 0xcca8 => 0x30,
            0xccad..=0xcccd => 0x20,
            _ => 0,
        };
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
            (1, self.pc, 0, 0, mode)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x9a12 | 0x9b56 => {
                w.drain(6);
                w.cpu.y = w.cpu.x;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x9a13 | 0x9a1a | 0x9abb | 0x9bcc | 0x9c0c | 0x9c1b | 0x9c1e | 0x9c21 | 0x9e23
            | 0xccd4 | 0xcd29 => {
                let address = match self.pc {
                    0x9a13 | 0x9abb => 0xbf,
                    0x9c1b => 0xb7,
                    0x9c1e => 0xb8,
                    0x9c21 => 0xba,
                    0x9e23 => 0,
                    0xccd4 => 0xb2,
                    _ => 8,
                };
                let value = w.dp_read(address, 2);
                w.load(value, 2);
                self.pc + 2
            }
            0x9a15 | 0x9abd | 0xcca1 => {
                w.compare_immediate(if self.pc == 0xcca1 { 0xffff } else { 0x4000 }, 2);
                self.pc + 3
            }
            0x9a18 | 0x9a63 | 0x9aaf | 0x9ac0 | 0x9aeb | 0x9bc4 | 0x9bfe | 0x9e21 | 0x9e2d
            | 0x9ea0 | 0x9ef5 | 0x9f49 | 0x9fdd | 0xa008 | 0xa036 | 0xa06f | 0xa092 | 0xa0c6
            | 0xa11d | 0xa191 | 0xa232 | 0xa252 | 0xcca4 | 0xcd1d | 0xcd27 | 0xcd34 => {
                let target = match self.pc {
                    0x9a18 => 0x9a21,
                    0x9a63 => 0x9a28,
                    0x9aaf => 0x9aed,
                    0x9ac0 | 0x9aeb => 0x9ac7,
                    0x9bc4 => 0x9b5e,
                    0x9bfe => 0x9c12,
                    0x9e21 => 0x9e14,
                    0x9e2d => 0x9e0d,
                    0x9ea0 => 0x9e3a,
                    0x9ef5 => 0x9ead,
                    0x9f49 => 0x9f01,
                    0x9fdd => 0x9fe2,
                    0xa008 => 0x9fec,
                    0xa036 => 0xa014,
                    0xa06f => 0xa042,
                    0xa092 => 0xa078,
                    0xa0c6 => 0xa09f,
                    0xa11d => 0xa0cf,
                    0xa191 => 0xa14f,
                    0xa232 => 0xa1db,
                    0xa252 => 0xa23e,
                    0xcca4 => 0xcca9,
                    0xcd1d => 0xccdd,
                    0xcd27 => 0xccd8,
                    _ => 0xccd4,
                };
                let take = if self.pc == 0x9fdd {
                    w.cpu.p & ZERO != 0
                } else {
                    w.cpu.p & ZERO == 0
                };
                if w.branch(take) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x9a1c | 0x9ac3 | 0x9b79 | 0x9ba7 | 0x9e41 | 0x9f20 | 0x9f64 | 0x9f82 | 0x9fa0
            | 0x9fbe | 0xa07c | 0xa087 | 0xa0b9 | 0xa0e2 | 0xa104 | 0xa12c | 0xa13e => {
                w.drain(16);
                w.load(
                    w.cpu.a
                        | if matches!(self.pc, 0x9a1c | 0x9ac3 | 0xa07c | 0xa087) {
                            0x2000
                        } else {
                            0x4000
                        },
                    2,
                );
                self.pc + 3
            }
            0x9e53 | 0x9e65 | 0x9e73 | 0x9e81 | 0x9e8f | 0xa156 | 0xa164 | 0xa172 | 0xa180 => {
                w.drain(16);
                w.load(w.cpu.a ^ 0x4000, 2);
                self.pc + 3
            }
            0x9a1f | 0x9a26 | 0x9ab4 | 0x9b5c | 0x9c27 | 0x9c32 | 0x9c35 | 0x9c38 | 0x9e06
            | 0x9e0b | 0x9e10 | 0x9e29 | 0x9e33 | 0x9ea6 | 0x9efa | 0x9f4e | 0x9fe5 | 0xa00d
            | 0xa03b | 0xa074 | 0xa098 | 0xa0cb | 0xa148 | 0xa1d9 | 0xa23c | 0xcc95 | 0xcca9
            | 0xccd6 | 0xccdb | 0xcd2f => {
                let address = match self.pc {
                    0x9a1f | 0xcd2f => 8,
                    0x9c27 => 0xb9,
                    0x9c32 => 0xba,
                    0x9c35 => 0xb8,
                    0x9c38 | 0xcc95 => 0xb7,
                    0x9e06 | 0x9e29 | 0xcca9 => 0,
                    0x9e0b => 2,
                    0x9e10 => 0x0c,
                    0xccd6 => 0x0a,
                    0xccdb => 4,
                    _ => 0x0e,
                };
                w.dp_store(address, w.cpu.a, 2);
                self.pc + 2
            }
            0x9a21 | 0x9ab9 | 0x9b57 | 0x9e12 | 0x9e38 | 0x9eab | 0x9eff | 0x9f53 | 0x9fea
            | 0xa012 | 0xa040 | 0xa076 | 0xa09d | 0xa0cd | 0xa11f | 0xa14d | 0xa1d4 | 0xa237
            | 0xcccf => {
                w.cpu.x = w.dp_read(if self.pc == 0x9e12 { 0 } else { 8 }, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0x9a23 | 0x9ab1 | 0x9b59 | 0x9bd3 | 0x9c00 | 0x9c06 | 0x9c15 | 0x9c24 | 0x9c29
            | 0x9de8 | 0x9df1 | 0x9dfa | 0x9e03 | 0x9e08 | 0x9e0d | 0x9e30 | 0x9ea3 | 0x9ef7
            | 0x9f4b | 0x9fe2 | 0xa00a | 0xa038 | 0xa071 | 0xa095 | 0xa0c8 | 0xa145 | 0xa1d6
            | 0xa239 | 0xccd8 => {
                w.immediate(
                    match self.pc {
                        0x9a23 => 8,
                        0x9ab1 | 0xa0c8 | 0xa239 => 3,
                        0x9b59 | 0x9ea3 | 0x9f4b | 0x9fe2 | 0xa00a | 0xa095 => 6,
                        0x9bd3 | 0x9c24 | 0xa071 | 0xa145 => 4,
                        0x9c00 | 0x9c15 => 10,
                        0x9c06 => 15,
                        0x9c29 => 0xf1cd,
                        0x9de8 => 0x0514,
                        0x9df1 => 0x0554,
                        0x9dfa => 0x1514,
                        0x9e03 => 0x1554,
                        0x9e08 | 0x9e0d => 12,
                        0x9e30 => 14,
                        0x9ef7 | 0xa038 => 5,
                        0xa1d6 => 22,
                        _ => 2,
                    },
                    2,
                );
                self.pc + 3
            }
            0x9ab6 | 0x9de5 | 0x9dee | 0x9df7 | 0x9e00 | 0x9e35 | 0x9ea8 | 0x9efc | 0x9f50
            | 0x9fe7 | 0xa00f | 0xa03d | 0xa09a | 0xa14a | 0xa1d1 | 0xa234 | 0xccd1 => {
                w.drain(16);
                w.cpu.y = match self.pc {
                    0x9ab6 => 0x1dd6,
                    0x9de5 => 0x16dc,
                    0x9dee => 0x17f6,
                    0x9df7 => 0x1914,
                    0x9e00 => 0x1a2a,
                    0x9e35 => 0x1b4a,
                    0x9ea8 => 0x1bf2,
                    0x9efc => 0x1c22,
                    0x9f50 => 0x1c2c,
                    0x9fe7 => 0x1c5c,
                    0xa00f => 0x1c74,
                    0xa03d => 0x1c8c,
                    0xa09a | 0xa14a => 0x202e,
                    0xa1d1 => 0x20f6,
                    0xa234 => 0x22da,
                    _ => 0x0110,
                };
                w.flags(w.cpu.y, 2);
                self.pc + 3
            }
            0x9a28 | 0x9a2f | 0x9a36 | 0x9a3d | 0x9a44 | 0x9a4b | 0x9a52 | 0x9ac7 | 0x9ad6
            | 0x9b5e | 0x9b94 | 0x9e14 | 0x9e3a | 0x9e48 | 0x9e5e | 0x9e6c | 0x9e7a | 0x9e88
            | 0x9ead | 0x9ebc | 0x9ecb | 0x9eda | 0x9f01 | 0x9f55 | 0x9f73 | 0x9f91 | 0x9faf
            | 0x9fec | 0x9ff7 | 0xa014 | 0xa01f | 0xa042 | 0xa049 | 0xa050 | 0xa057 | 0xa05e
            | 0xa09f | 0xa0b2 | 0xa0cf | 0xa0f5 | 0xa121 | 0xa137 | 0xa14f | 0xa15d | 0xa16b
            | 0xa179 | 0xa1db | 0xa1e2 | 0xa1e9 | 0xa1f0 | 0xa1f7 | 0xa1fe | 0xa205 | 0xa20c
            | 0xa213 | 0xa21a | 0xa221 | 0xa23e | 0xa245 | 0xccdd | 0xcce4 | 0xcceb | 0xccf2
            | 0xccf9 | 0xcd00 | 0xcd07 | 0xcd0e => {
                let offset = match self.pc {
                    0x9a2f | 0x9ad6 | 0x9ebc | 0x9f73 | 0xa01f | 0xa049 | 0xa0b2 | 0xa1e2
                    | 0xcce4 => 2,
                    0x9a36 | 0x9ecb | 0x9f91 | 0xa050 | 0xa0cf | 0xa1e9 | 0xcceb => 4,
                    0x9a3d | 0x9eda | 0x9faf | 0xa057 | 0xa1f0 | 0xa245 | 0xccf2 => 6,
                    0x9a44 | 0xa05e | 0xa1f7 | 0xccf9 => 8,
                    0x9a4b | 0xa0f5 | 0xa121 | 0xa1fe | 0xcd00 => 10,
                    0x9a52 | 0x9b94 | 0x9ff7 | 0xa137 | 0xa205 | 0xcd07 => 12,
                    0xa20c | 0xcd0e => 14,
                    0xa213 => 16,
                    0xa21a => 18,
                    0xa14f | 0xa221 => 20,
                    0x9e48 | 0xa15d => 28,
                    0xa16b => 36,
                    0xa179 => 44,
                    0x9e5e => 56,
                    0x9e6c => 84,
                    0x9e7a => 112,
                    0x9e88 => 140,
                    _ => 0,
                };
                w.operand_then_internal(2);
                let value = w.read(0x009b52 + u32::from(w.cpu.y) + offset, 2);
                w.load(value, 2);
                self.pc + 3
            }
            0x9a59 | 0x9bc6 | 0xa065 | 0xa228 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x9ac2 | 0x9bbc | 0x9e96 | 0x9f41 | 0x9fd5 | 0xa02e | 0xa115 | 0xa189 | 0xcd15
            | 0xcd1f => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x9a5a | 0x9bbd | 0x9bc7 | 0x9bce | 0x9e25 | 0x9e97 | 0x9f42 | 0x9fd6 | 0xa02f
            | 0xa066 | 0xa116 | 0xa18a | 0xa229 | 0xcd16 | 0xcd2b => {
                w.clear_carry();
                self.pc + 1
            }
            0x9a5b | 0x9bbe | 0x9bc8 | 0x9bcf | 0x9e26 | 0x9e98 | 0x9f43 | 0x9fd7 | 0xa030
            | 0xa067 | 0xa117 | 0xa18b | 0xa22a | 0xcd17 | 0xcd2c => {
                w.drain(16);
                w.adc(match self.pc {
                    0x9a5b => 14,
                    0x9bc8 => 12,
                    0x9bcf => 20,
                    0xa067 => 10,
                    0xa22a => 22,
                    0xcd17 => 0x100,
                    0xcd2c => 0x200,
                    _ => 0x80,
                });
                self.pc + 3
            }
            0x9a5e | 0x9bd2 | 0xa06a | 0xa22d => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0x9ac6 | 0x9bc1 | 0x9bcb | 0x9e9b | 0x9f46 | 0x9fda | 0xa033 | 0xa11a | 0xa18e
            | 0xcd1a | 0xcd24 | 0xcd31 => {
                w.tax_word();
                self.pc + 1
            }
            0x9a5f
            | 0x9a60
            | 0x9ae5..=0x9ae8
            | 0x9e1d
            | 0x9e1e
            | 0x9ef1
            | 0x9ef2
            | 0xa004
            | 0xa005
            | 0xa06b
            | 0xa06c
            | 0xa08e
            | 0xa08f
            | 0xa0c0..=0xa0c3
            | 0xa22e
            | 0xa22f
            | 0xa24e
            | 0xa24f => {
                w.increment_x();
                self.pc + 1
            }
            0x9bba
            | 0x9bbb
            | 0x9e1b
            | 0x9e1c
            | 0x9e9c
            | 0x9e9d
            | 0x9ee9..=0x9ef0
            | 0x9f3f
            | 0x9f40
            | 0x9fcd..=0x9fd4
            | 0xa002
            | 0xa003
            | 0xa02a..=0xa02d
            | 0xa113
            | 0xa114
            | 0xa187
            | 0xa188
            | 0xa24c
            | 0xa24d
            | 0xccbf..=0xccc1 => {
                w.increment_y();
                self.pc + 1
            }
            0x9a61 | 0x9ae9 | 0x9bc2 | 0x9e1f | 0x9e2b | 0x9e9e | 0x9ef3 | 0x9f47 | 0x9fdb
            | 0xa006 | 0xa034 | 0xa06d | 0xa090 | 0xa0c4 | 0xa11b | 0xa18f | 0xa230 | 0xa250
            | 0xcd1b | 0xcd25 | 0xcd32 => {
                w.dp_increment(
                    match self.pc {
                        0x9e1f => 0x0c,
                        0x9e2b => 2,
                        0xcd1b => 4,
                        0xcd25 => 0x0a,
                        0xcd32 => 0xb4,
                        _ => 0x0e,
                    },
                    2,
                    false,
                );
                self.pc + 2
            }
            0x9aa9 | 0x9bf8 => {
                w.drain(16);
                let value = w.read(0x0402, 2);
                w.load(value, 2);
                self.pc + 3
            }
            0x9aac | 0x9bfb => {
                w.mask(if self.pc == 0x9aac { 0x1000 } else { 0x0800 }, 2);
                self.pc + 3
            }
            0xa078 | 0xa083 => {
                w.drain(24);
                let value = w.read(
                    if self.pc == 0xa078 {
                        0x7e2e1c
                    } else {
                        0x7e2e9c
                    } + u32::from(w.cpu.x),
                    2,
                );
                w.load(value, 2);
                self.pc + 4
            }
            0x9bd6 => {
                w.drain(16);
                self.tiles = Some(DungeonObjectTiles::new(0x018d80).unwrap());
                self.tail = true;
                0x8d80
            }
            0x9c03 | 0x9c18 => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.tiles = Some(DungeonObjectTiles::new(0x0197f0).unwrap());
                self.tail = false;
                0x97f0
            }
            0x9c2c | 0x9deb | 0x9df4 | 0x9dfd => {
                w.operand_then_internal(2);
                w.push(self.pc + 2, 2);
                self.depth += 1;
                if self.pc == 0x9c2c {
                    0xcc95
                } else {
                    0x9e06
                }
            }
            0x9c09 | 0x9c0e => {
                w.drain(16);
                w.write(if self.pc == 0x9c09 { 0x0470 } else { 0x0472 }, w.cpu.a, 2);
                self.pc + 3
            }
            0x9c12 => {
                w.drain(16);
                w.cpu.x = 0x13e8;
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x9c1d | 0x9c20 | 0x9c23 => {
                w.drain(6);
                w.push(w.cpu.a, 2);
                self.pc + 1
            }
            0x9c31 | 0x9c34 | 0x9c37 => {
                w.drain(12);
                let value = w.pull(2);
                w.load(value, 2);
                self.pc + 1
            }
            0x9c2f | 0xcca6 | 0xccab | 0xcccd => {
                w.width(
                    if matches!(self.pc, 0x9c2f | 0xcca6) {
                        0x30
                    } else {
                        0x20
                    },
                    matches!(self.pc, 0x9c2f | 0xcccd),
                );
                self.pc + 2
            }
            0xcc97 | 0xcc99 | 0xcc9b => {
                w.dp_store(
                    match self.pc {
                        0xcc97 => 0xba,
                        0xcc99 => 0xb2,
                        _ => 0xb4,
                    },
                    0,
                    2,
                );
                self.pc + 2
            }
            0xcc9d => {
                w.cpu.y = w.dp_read(0xba, 2);
                w.flags(w.cpu.y, 2);
                self.pc + 2
            }
            0xcc9f => {
                let address = w.indirect_y_address(0xb7);
                let value = w.read(address, 2);
                w.load(value, 2);
                self.pc + 2
            }
            0xccad | 0xccb3 | 0xccba => {
                w.mask(if self.pc == 0xccad { 0xfc } else { 3 }, 1);
                self.pc + 2
            }
            0xccaf | 0xccb6 | 0xccbd | 0xcccb => {
                w.dp_store(
                    match self.pc {
                        0xccaf => 8,
                        0xccb6 => 0xb2,
                        0xccbd => 0xb4,
                        _ => 9,
                    },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0xccb1 | 0xccb8 | 0xccc4 => {
                let value = w.dp_read(if self.pc == 0xccb1 { 0 } else { 1 }, 1);
                w.load(value, 1);
                self.pc + 2
            }
            0xccb5 | 0xccbc => {
                w.increment_a(1);
                self.pc + 1
            }
            0xccc2 => {
                w.dp_store(0xba, w.cpu.y, 2);
                self.pc + 2
            }
            0xccc6..=0xccc8 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a & 1) as u8;
                w.load((w.cpu.a & 255) >> 1, 1);
                self.pc + 1
            }
            0xccc9 => {
                let old = w.dp_read(8, 1);
                let value = (old >> 1) | (u16::from(w.cpu.p & CARRY) << 7);
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.drain(6);
                w.write(8, value, 1);
                w.flags(value, 1);
                self.pc + 2
            }
            0xcd20 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xcd21 => {
                w.drain(16);
                w.sbc(0x01f8, 2);
                self.pc + 3
            }
            0x9fdf | 0xcd36 => {
                w.drain(16);
                if self.pc == 0x9fdf {
                    0x9f55
                } else {
                    0xcc99
                }
            }
            0x9a65 | 0x9aed | 0x9c11 | 0x9c3a | 0x9e2f | 0x9ea2 | 0xa094 | 0xa193 | 0xa254
            | 0xcca8 => {
                w.rts();
                if self.depth == 0 {
                    self.complete = true;
                } else {
                    self.depth -= 1;
                }
                w.cpu.pc
            }
            _ => {
                // Explicit tile destinations retain each individual long store's bus phase.
                let address = match self.pc {
                    0x9a2b | 0x9aca | 0x9b61 | 0x9e3d | 0xa124 => 0x7e2000,
                    0x9a32 | 0x9ad9 | 0xa128 => 0x7e2080,
                    0x9a39 | 0xa13a => 0x7e2100,
                    0x9a40 | 0x9ace | 0xa0d2 => 0x7e2180,
                    0x9a47 | 0x9add => 0x7e2200,
                    0x9a4e => 0x7e2280,
                    0x9a55 | 0x9ad2 => 0x7e2300,
                    0x9ae1 => 0x7e2380,
                    0x9b65 | 0x9e6f => 0x7e2008,
                    0x9b69 | 0x9e84 => 0x7e2010,
                    0x9b6d => 0x7e201c,
                    0x9b71 => 0x7e2024,
                    0x9b75 => 0x7e202c,
                    0x9b7c | 0x9e4b | 0xa0a2 => 0x7e2002,
                    0x9b80 | 0x9e7d => 0x7e200a,
                    0x9b84 | 0x9e76 => 0x7e2012,
                    0x9b88 => 0x7e201e,
                    0x9b8c => 0x7e2026,
                    0x9b90 => 0x7e202e,
                    0x9b97 | 0x9e4f | 0xa0a6 => 0x7e2004,
                    0x9b9b | 0x9e8b => 0x7e200c,
                    0x9b9f => 0x7e2020,
                    0x9ba3 => 0x7e2028,
                    0x9baa | 0x9e61 => 0x7e2006,
                    0x9bae | 0x9e92 => 0x7e200e,
                    0x9bb2 => 0x7e2022,
                    0x9bb6 => 0x7e202a,
                    0x9e17 | 0xa1de | 0xcce0 => 0x7e4000,
                    0x9e44 | 0xa12f => 0x7e201a,
                    0x9e56 => 0x7e2016,
                    0x9e5a => 0x7e2018,
                    0x9e68 => 0x7e2014,
                    0x9eb0 => 0x7e220e,
                    0x9eb4 => 0x7e221a,
                    0x9eb8 => 0x7e2226,
                    0x9ebf | 0x9f18 => 0x7e228e,
                    0x9ec3 => 0x7e229a,
                    0x9ec7 => 0x7e22a6,
                    0x9ece => 0x7e230e,
                    0x9ed2 => 0x7e231a,
                    0x9ed6 => 0x7e2326,
                    0x9edd => 0x7e238e,
                    0x9ee1 => 0x7e239a,
                    0x9ee5 => 0x7e23a6,
                    0x9f04 => 0x7e2504,
                    0x9f08 => 0x7e2486,
                    0x9f0c => 0x7e2408,
                    0x9f10 => 0x7e238a,
                    0x9f14 => 0x7e230c,
                    0x9f1c => 0x7e2210,
                    0x9f23 => 0x7e222e,
                    0x9f27 => 0x7e22b0,
                    0x9f2b => 0x7e2332,
                    0x9f2f => 0x7e23b4,
                    0x9f33 => 0x7e2436,
                    0x9f37 => 0x7e24b8,
                    0x9f3b => 0x7e253a,
                    0x9f58 => 0x7e2584,
                    0x9f5c => 0x7e2884,
                    0x9f60 => 0x7e2b84,
                    0x9f67 => 0x7e25ba,
                    0x9f6b => 0x7e28ba,
                    0x9f6f => 0x7e2bba,
                    0x9f76 => 0x7e2586,
                    0x9f7a => 0x7e2886,
                    0x9f7e => 0x7e2b86,
                    0x9f85 => 0x7e25b8,
                    0x9f89 => 0x7e28b8,
                    0x9f8d => 0x7e2bb8,
                    0x9f94 => 0x7e2588,
                    0x9f98 => 0x7e2888,
                    0x9f9c => 0x7e2b88,
                    0x9fa3 => 0x7e25b6,
                    0x9fa7 => 0x7e28b6,
                    0x9fab => 0x7e2bb6,
                    0x9fb2 => 0x7e258a,
                    0x9fb6 => 0x7e288a,
                    0x9fba => 0x7e2b8a,
                    0x9fc1 => 0x7e25b4,
                    0x9fc5 => 0x7e28b4,
                    0x9fc9 => 0x7e2bb4,
                    0x9fef => 0x7e2498,
                    0x9ff3 => 0x7e24a4,
                    0x9ffa => 0x7e2518,
                    0x9ffe => 0x7e2524,
                    0xa017 => 0x7e270e,
                    0xa01b => 0x7e2a0e,
                    0xa022 => 0x7e2710,
                    0xa026 => 0x7e2a10,
                    0xa045 => 0x7e248e,
                    0xa04c => 0x7e250e,
                    0xa053 => 0x7e258e,
                    0xa05a => 0x7e260e,
                    0xa061 => 0x7e268e,
                    0xa07f => 0x7e2e1c,
                    0xa08a => 0x7e2e9c,
                    0xa0aa => 0x7e2082,
                    0xa0ae => 0x7e2084,
                    0xa0b5 => 0x7e2102,
                    0xa0bc => 0x7e2104,
                    0xa0d6 => 0x7e2184,
                    0xa0da => 0x7e2194,
                    0xa0de => 0x7e2198,
                    0xa0e5 => 0x7e2182,
                    0xa0e9 => 0x7e2186,
                    0xa0ed => 0x7e2196,
                    0xa0f1 => 0x7e219a,
                    0xa0f8 => 0x7e2188,
                    0xa0fc => 0x7e218c,
                    0xa100 => 0x7e2190,
                    0xa107 => 0x7e218a,
                    0xa10b => 0x7e218e,
                    0xa10f => 0x7e2192,
                    0xa133 => 0x7e209a,
                    0xa141 => 0x7e211a,
                    0xa152 => 0x7e2506,
                    0xa159 => 0x7e2514,
                    0xa160 => 0x7e2508,
                    0xa167 => 0x7e2512,
                    0xa16e => 0x7e250a,
                    0xa175 => 0x7e2510,
                    0xa17c => 0x7e250c,
                    0xa183 => 0x7e250e,
                    0xa1e5 | 0xccfc => 0x7e4080,
                    0xa1ec => 0x7e4100,
                    0xa1f3 => 0x7e4180,
                    0xa1fa => 0x7e4200,
                    0xa201 => 0x7e4280,
                    0xa208 => 0x7e4300,
                    0xa20f => 0x7e4380,
                    0xa216 => 0x7e4400,
                    0xa21d => 0x7e4480,
                    0xa224 => 0x7e4500,
                    0xa241 => 0x7e4592,
                    0xa248 => 0x7e4612,
                    0xcce7 => 0x7e4002,
                    0xccee => 0x7e4004,
                    0xccf5 => 0x7e4006,
                    0xcd03 => 0x7e4082,
                    0xcd0a => 0x7e4084,
                    0xcd11 => 0x7e4086,
                    _ => panic!("unowned room decoration continuation {:04x}", self.pc),
                };
                w.drain(24);
                w.write(address + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
