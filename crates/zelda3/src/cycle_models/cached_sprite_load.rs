//! Actual byte backups and cached-field loads before Sprite_ExecuteSingle_.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CachedSpriteLoad {
    backup: [u8; 24],
    complete: bool,
}
impl CachedSpriteLoad {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn backup(&self) -> [u8; 24] {
        assert!(self.complete);
        self.backup
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
        assert_eq!((cpu.bank, cpu.db, cpu.d, cpu.p & 0x30), (0x1d, 0, 0, 0x30));
        assert!(cpu.x < 16);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = w.cpu.pc;
        w.fetch();
        let next = match pc {
            0xea00 => {
                w.drain(16);
                let value = w.read(w.absolute(0xdd0 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[0] = value as u8;
                pc + 3
            }
            0xea04 => {
                w.drain(16);
                let value = w.read(w.absolute(0xe20 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[1] = value as u8;
                pc + 3
            }
            0xea08 => {
                w.drain(16);
                let value = w.read(w.absolute(0xd10 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[2] = value as u8;
                pc + 3
            }
            0xea0c => {
                w.drain(16);
                let value = w.read(w.absolute(0xd30 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[3] = value as u8;
                pc + 3
            }
            0xea10 => {
                w.drain(16);
                let value = w.read(w.absolute(0xd00 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[4] = value as u8;
                pc + 3
            }
            0xea14 => {
                w.drain(16);
                let value = w.read(w.absolute(0xd20 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[5] = value as u8;
                pc + 3
            }
            0xea18 => {
                w.drain(16);
                let value = w.read(w.absolute(0xdc0 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[6] = value as u8;
                pc + 3
            }
            0xea1c => {
                w.drain(16);
                let value = w.read(w.absolute(0xd90 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[7] = value as u8;
                pc + 3
            }
            0xea20 => {
                w.drain(16);
                let value = w.read(w.absolute(0xeb0 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[8] = value as u8;
                pc + 3
            }
            0xea24 => {
                w.drain(16);
                let value = w.read(w.absolute(0xf50 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[9] = value as u8;
                pc + 3
            }
            0xea28 => {
                w.drain(16);
                let value = w.read(w.absolute(0xb89 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[10] = value as u8;
                pc + 3
            }
            0xea2c => {
                w.drain(16);
                let value = w.read(w.absolute(0xde0 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[11] = value as u8;
                pc + 3
            }
            0xea30 => {
                w.drain(16);
                let value = w.read(w.absolute(0xe40 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[12] = value as u8;
                pc + 3
            }
            0xea34 => {
                w.drain(16);
                let value = w.read(w.absolute(0xf20 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[13] = value as u8;
                pc + 3
            }
            0xea38 => {
                w.drain(16);
                let value = w.read(w.absolute(0xd80 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[14] = value as u8;
                pc + 3
            }
            0xea3c => {
                w.drain(16);
                let value = w.read(w.absolute(0xe60 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[15] = value as u8;
                pc + 3
            }
            0xea40 => {
                w.drain(16);
                let value = w.read(w.absolute(0xda0 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[16] = value as u8;
                pc + 3
            }
            0xea44 => {
                w.drain(16);
                let value = w.read(w.absolute(0xdb0 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[17] = value as u8;
                pc + 3
            }
            0xea48 => {
                w.drain(16);
                let value = w.read(w.absolute(0xe90 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[18] = value as u8;
                pc + 3
            }
            0xea4c => {
                w.drain(16);
                let value = w.read(w.absolute(0xe80 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[19] = value as u8;
                pc + 3
            }
            0xea50 => {
                w.drain(16);
                let value = w.read(w.absolute(0xf70 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[20] = value as u8;
                pc + 3
            }
            0xea54 => {
                w.drain(16);
                let value = w.read(w.absolute(0xdf0 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[21] = value as u8;
                pc + 3
            }
            0xea58 => {
                w.drain(24);
                let value = w.read(0x7ff9c2 + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                self.backup[22] = value as u8;
                pc + 4
            }
            0xea5d => {
                w.drain(16);
                let value = w.read(w.absolute(0xba0 + w.cpu.x), 1);
                w.load(value, 1);
                self.backup[23] = value as u8;
                pc + 3
            }
            0xea61 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d00 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea67 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d10 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea6d => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d20 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea73 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d30 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea79 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d40 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea7f => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d50 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea85 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d60 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea8b => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d70 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea91 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d80 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea97 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1d90 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xea9d => {
                w.drain(16);
                let value = w.read(w.absolute(0x1da0 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xeaa3 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1db0 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xeaa9 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1dc0 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xeaaf => {
                w.drain(16);
                let value = w.read(w.absolute(0x1dd0 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xeab5 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1de0 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xeabb => {
                w.drain(16);
                let value = w.read(w.absolute(0x1df0 + w.cpu.x), 1);
                w.load(value, 1);
                pc + 3
            }
            0xeac1 => {
                w.drain(24);
                let value = w.read(0x7ffa5c + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                pc + 4
            }
            0xeac8 => {
                w.drain(24);
                let value = w.read(0x7ffa6c + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                pc + 4
            }
            0xeacf => {
                w.drain(24);
                let value = w.read(0x7ffa7c + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                pc + 4
            }
            0xead6 => {
                w.drain(24);
                let value = w.read(0x7ffa8c + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                pc + 4
            }
            0xeadd => {
                w.drain(24);
                let value = w.read(0x7ffa9c + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                pc + 4
            }
            0xeae4 => {
                w.drain(24);
                let value = w.read(0x7ffaac + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                pc + 4
            }
            0xeaeb => {
                w.drain(24);
                let value = w.read(0x7ffacc + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                pc + 4
            }
            0xeaf3 => {
                w.drain(24);
                let value = w.read(0x7ffadc + u32::from(w.cpu.x), 1);
                w.load(value, 1);
                pc + 4
            }
            0xea03 | 0xea07 | 0xea0b | 0xea0f | 0xea13 | 0xea17 | 0xea1b | 0xea1f | 0xea23
            | 0xea27 | 0xea2b | 0xea2f | 0xea33 | 0xea37 | 0xea3b | 0xea3f | 0xea43 | 0xea47
            | 0xea4b | 0xea4f | 0xea53 | 0xea57 | 0xea5c | 0xea60 => {
                w.drain(6);
                w.push(w.cpu.a & 255, 1);
                pc + 1
            }
            0xea64 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xdd0 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xea6a => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xe20 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xea70 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xd10 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xea76 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xd30 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xea7c => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xd00 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xea82 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xd20 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xea88 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xdc0 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xea8e => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xd90 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xea94 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xeb0 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xea9a => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xf50 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeaa0 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xb89 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeaa6 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xde0 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeaac => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xe40 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeab2 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xf20 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeab8 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xd80 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeabe => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xe60 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeac5 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xda0 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeacc => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xdb0 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xead3 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xe90 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeada => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xe80 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeae1 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xf70 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeae8 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xdf0 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeaef => {
                w.drain(24);
                w.write(0x7ff9c2 + u32::from(w.cpu.x), w.cpu.a & 255, 1);
                pc + 4
            }
            0xeaf7 => {
                w.operand_then_internal(2);
                w.write(w.absolute(0xba0 + w.cpu.x), w.cpu.a & 255, 1);
                pc + 3
            }
            0xeafa => {
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(0xeafd, 2);
                w.cpu.bank = 6;
                self.complete = true;
                0x84da
            }
            _ => unreachable!("cached field load {pc:04x}"),
        };
        w.cpu.pc = next;
        Some(w.cycles)
    }
}
