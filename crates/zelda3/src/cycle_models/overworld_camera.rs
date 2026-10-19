//! Overworld camera and boundary helper on the caller's real bus and clock.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldCamera {
    pc: u16,
    complete: bool,
}
impl Default for OverworldCamera {
    fn default() -> Self {
        Self {
            pc: 0xbb90,
            complete: false,
        }
    }
}
// Immutable direction bits consumed by the boundary helper, not instructions.
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    const DIRECTIONS: [u8; 8] = [8, 0, 4, 0, 2, 0, 1, 0];
    (0x02bb88..0x02bb90)
        .contains(&address)
        .then(|| DIRECTIONS[(address - 0x02bb88) as usize])
}
impl OverworldCamera {
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
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x18),
            (2, self.pc, 0, 0x10)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xbb90 | 0xbb91 => {
                w.drain(6);
                w.push(
                    u16::from(if pc == 0xbb90 { w.cpu.db } else { w.cpu.bank }),
                    1,
                );
                pc + 1
            }
            0xbb92 | 0xbd60 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xbb93 | 0xbd5e => {
                w.width(0x20, pc == 0xbb93);
                pc + 2
            }
            0xbb95 | 0xbb9c | 0xbba8 | 0xbbb8 | 0xbbd5 | 0xbbea | 0xbbfc | 0xbc0b | 0xbc3e
            | 0xbc44 | 0xbc60 | 0xbc6d | 0xbc8c | 0xbca1 | 0xbcb3 | 0xbcc2 | 0xbcf5 | 0xbd19
            | 0xbd2a | 0xbd3d | 0xbd48 | 0xbd4c | 0xbd53 | 0xbd5a | 0xbd7d => {
                let a = match pc {
                    0xbb95 => 0x78,
                    0xbb9c => 0x24,
                    0xbba8 => 0x20,
                    0xbbb8 | 0xbbd5 => 0x30,
                    0xbbea | 0xbca1 => 0x0e,
                    0xbbfc | 0xbc0b | 0xbcb3 | 0xbcc2 | 0xbd7d => 4,
                    0xbc3e | 0xbd19 | 0xbd3d => 0xe6,
                    0xbc44 => 0x8a,
                    0xbc60 => 0x22,
                    0xbc6d | 0xbc8c => 0x31,
                    0xbcf5 | 0xbd2a => 0xe0,
                    0xbd48 | 0xbd5a => 0xe2,
                    0xbd4c => 0xa0,
                    0xbd53 => 0xe8,
                    _ => unreachable!(),
                };
                let value = w.dp_read(a, 2);
                w.load(value, 2);
                pc + 2
            }
            0xbba6 | 0xbbb1 | 0xbbb6 | 0xbbd1 | 0xbc33 | 0xbc42 | 0xbc55 | 0xbc5e | 0xbc66
            | 0xbc6b | 0xbc86 | 0xbcea | 0xbcf9 | 0xbd1e | 0xbd2f | 0xbd46 | 0xbd4a | 0xbd58
            | 0xbd5c | 0xbd82 => {
                let a = match pc {
                    0xbba6 | 0xbbb1 | 0xbc66 => 0x0e,
                    0xbbb6 | 0xbc6b => 0,
                    0xbbd1 | 0xbc86 => 2,
                    0xbc33 | 0xbcea => 6,
                    0xbc42 | 0xbc55 | 0xbc5e | 0xbd1e | 0xbd46 | 0xbd58 => 0xe6,
                    0xbcf9 | 0xbd2f | 0xbd4a | 0xbd5c => 0xe0,
                    0xbd82 => 4,
                    _ => unreachable!(),
                };
                w.dp_store(a, w.cpu.a, 2);
                pc + 2
            }
            0xbb97 | 0xbbba | 0xbbd7 | 0xbc46 | 0xbc6f | 0xbc8e => {
                w.mask(if pc == 0xbc46 { 0x3f } else { 255 }, 2);
                pc + 3
            }
            0xbb9e | 0xbbc4 | 0xbbda | 0xbc21 | 0xbc2b | 0xbc49 | 0xbc79 | 0xbc91 | 0xbcd8
            | 0xbce2 | 0xbd4e | 0xbda0 => {
                w.compare_immediate(
                    match pc {
                        0xbb9e => 65535,
                        0xbbc4 | 0xbbda | 0xbc79 | 0xbc91 => 0x80,
                        0xbc21 | 0xbcd8 => 0x3000,
                        0xbc2b | 0xbce2 => 0x7000,
                        0xbc49 => 0x1b,
                        0xbd4e => 0x181,
                        0xbda0 => 16,
                        _ => unreachable!(),
                    },
                    2,
                );
                pc + 3
            }
            0xbb9a | 0xbba1 | 0xbbbd | 0xbbc7 | 0xbbdd | 0xbbe4 | 0xbbef | 0xbbfa | 0xbc05
            | 0xbc09 | 0xbc0d | 0xbc18 | 0xbc1c | 0xbc24 | 0xbc2e | 0xbc4c | 0xbc53 | 0xbc5c
            | 0xbc72 | 0xbc7c | 0xbc94 | 0xbc9b | 0xbca6 | 0xbcb1 | 0xbcbc | 0xbcc0 | 0xbcc4
            | 0xbccf | 0xbcd3 | 0xbcdb | 0xbce5 | 0xbcff | 0xbd05 | 0xbd09 | 0xbd0d | 0xbd51
            | 0xbd67 | 0xbda3 => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let n = w.cpu.p & NEGATIVE != 0;
                let (take, target) = match pc {
                    0xbb9a => (z, 0xbba6),
                    0xbba1 => (!z, 0xbba6),
                    0xbbbd => (!z, 0xbbc2),
                    0xbbc7 => (!c, 0xbbd1),
                    0xbbdd => (!c, 0xbbea),
                    0xbbe4 | 0xbbef => (!c, 0xbbf8),
                    0xbbfa => (!z, 0xbbd5),
                    0xbc05 | 0xbc09 | 0xbc0d => (z, 0xbc60),
                    0xbc18 => (z, 0xbc1e),
                    0xbc1c => (!z, 0xbc2b),
                    0xbc24 | 0xbc2e => (!c, 0xbc33),
                    0xbc4c => (!z, 0xbc60),
                    0xbc53 => (!c, 0xbc57),
                    0xbc5c => (c, 0xbc60),
                    0xbc72 => (!z, 0xbc77),
                    0xbc7c => (!c, 0xbc86),
                    0xbc94 => (!c, 0xbca1),
                    0xbc9b | 0xbca6 => (!c, 0xbcaf),
                    0xbcb1 => (!z, 0xbc8c),
                    0xbcbc | 0xbcc0 | 0xbcc4 => (z, 0xbcfb),
                    0xbccf => (z, 0xbcd5),
                    0xbcd3 => (!z, 0xbce2),
                    0xbcdb | 0xbce5 => (!c, 0xbcea),
                    0xbcff => (z, 0xbd4c),
                    0xbd05 => (z, 0xbd33),
                    0xbd09 => (z, 0xbd0f),
                    0xbd0d => (!z, 0xbd4c),
                    0xbd51 => (!z, 0xbd5e),
                    0xbd67 => (!z, 0xbd78),
                    0xbda3 => (n, 0xbdb5),
                    _ => unreachable!(),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0xbba3 | 0xbbb3 | 0xbc4e | 0xbc57 | 0xbc68 | 0xbd6e | 0xbdb5 => {
                w.immediate(
                    match pc {
                        0xbbb3 | 0xbc68 => 1,
                        0xbc4e => 0x600,
                        0xbc57 => 0x6c0,
                        _ => 0,
                    },
                    2,
                );
                pc + 3
            }
            0xbbbf | 0xbc74 => {
                w.drain(16);
                if pc == 0xbbbf {
                    0xbc60
                } else {
                    0xbcfb
                }
            }
            0xbbaa | 0xbd36 | 0xbda5 | 0xbdb8 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xbbad | 0xbc38 | 0xbc62 | 0xbcef | 0xbd12 | 0xbd23 | 0xbd42 | 0xbd78 | 0xbd7f
            | 0xbd89 => {
                w.clear_carry();
                pc + 1
            }
            0xbbab => {
                let v = w.dp_read(0x0e, 2);
                w.sbc(v, 2);
                pc + 2
            }
            0xbbae | 0xbc63 | 0xbd13 | 0xbd1b | 0xbd24 | 0xbd2c => {
                w.drain(16);
                w.adc(match pc {
                    0xbbae => 12,
                    0xbc63 => 8,
                    0xbd13 | 0xbd24 => 0x2000,
                    _ => 0,
                });
                pc + 3
            }
            0xbbc2 | 0xbbd3 | 0xbc0f | 0xbc77 | 0xbcc6 => {
                w.dp_store(
                    if matches!(pc, 0xbbc2 | 0xbc77) {
                        4
                    } else if pc == 0xbbd3 {
                        8
                    } else {
                        0
                    },
                    0,
                    2,
                );
                pc + 2
            }
            0xbbc9 | 0xbc7e | 0xbd6a | 0xbd95 => {
                w.drain(16);
                w.load(w.cpu.a ^ if pc < 0xbd00 { 255 } else { 2 }, 2);
                pc + 3
            }
            0xbbcc | 0xbc81 | 0xbd8f | 0xbd90 | 0xbd9c => {
                w.increment_a(2);
                pc + 1
            }
            0xbbcd | 0xbbcf | 0xbc82 | 0xbc84 | 0xbbf8 | 0xbcaf => {
                w.dp_increment(if matches!(pc, 0xbbf8 | 0xbcaf) { 2 } else { 0 }, 2, false);
                pc + 2
            }
            0xbbe6 | 0xbbf1 | 0xbc9d | 0xbca8 => {
                w.drain(8);
                w.cpu.y = match pc {
                    0xbbe6 => 0,
                    0xbbf1 => 2,
                    0xbc9d => 4,
                    _ => 6,
                };
                w.flags(w.cpu.y, 1);
                pc + 2
            }
            0xbbf3 | 0xbc88 | 0xbcaa => {
                w.drain(8);
                w.cpu.x = match pc {
                    0xbbf3 => 6,
                    0xbc88 => 4,
                    _ => 0,
                };
                w.flags(w.cpu.x, 1);
                pc + 2
            }
            0xbbf5 | 0xbcac => {
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                0xbd62
            }
            0xbbdf | 0xbc35 | 0xbc96 | 0xbcec | 0xbd0f | 0xbd20 | 0xbd33 => {
                w.drain(16);
                let a = match pc {
                    0xbbdf => 0x618,
                    0xbc96 => 0x61c,
                    0xbc35 | 0xbd0f | 0xbd33 => 0x622,
                    _ => 0x620,
                };
                let v = w.read(w.absolute(a), 2);
                w.load(v, 2);
                pc + 3
            }
            0xbbe2 | 0xbbec | 0xbc51 | 0xbc5a | 0xbc99 | 0xbca3 | 0xbd64 => {
                let v = if matches!(pc, 0xbbe2 | 0xbc51 | 0xbc5a | 0xbc99) {
                    w.dp_read(
                        if matches!(pc, 0xbc51 | 0xbc5a) {
                            0xe6
                        } else {
                            0x0e
                        },
                        2,
                    )
                } else {
                    w.drain(16);
                    let base = match pc {
                        0xbbec => 0x61a,
                        0xbca3 => 0x61e,
                        _ => 0x600,
                    };
                    let index = if pc == 0xbd64 { w.cpu.y } else { 0 };
                    if (base & 255) + index > 255 {
                        w.drain(6);
                    }
                    w.read(w.absolute(base.wrapping_add(index)), 2)
                };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a >= v);
                w.flags(w.cpu.a.wrapping_sub(v), 2);
                pc + if matches!(pc, 0xbbe2 | 0xbc51 | 0xbc5a | 0xbc99) {
                    2
                } else {
                    3
                }
            }
            0xbbe8 | 0xbc29 | 0xbc9f | 0xbce0 | 0xbd31 => {
                w.branch(true);
                match pc {
                    0xbbe8 => 0xbbf3,
                    0xbc29 => 0xbc33,
                    0xbc9f => 0xbcaa,
                    0xbce0 => 0xbcea,
                    _ => 0xbd4c,
                }
            }
            0xbbfe | 0xbc3b | 0xbcb5 | 0xbcf2 | 0xbd16 | 0xbd27 | 0xbd3a | 0xbdb2 => {
                w.drain(16);
                let a = match pc {
                    0xbbfe => 0x69e,
                    0xbcb5 => 0x69f,
                    0xbc3b | 0xbd16 | 0xbd3a => 0x622,
                    0xbdb2 => 0x416,
                    _ => 0x620,
                };
                w.write(w.absolute(a), w.cpu.a, 2);
                pc + 3
            }
            0xbc01 | 0xbc14 | 0xbcb8 | 0xbccb | 0xbcfb | 0xbd01 | 0xbd84 => {
                w.cpu.x = w.dp_read(
                    match pc {
                        0xbcfb => 0x8a,
                        0xbd84 => 8,
                        _ => 0x8c,
                    },
                    1,
                );
                w.flags(w.cpu.x, 1);
                pc + 2
            }
            0xbc03 | 0xbc07 | 0xbc16 | 0xbc1a | 0xbcba | 0xbcbe | 0xbccd | 0xbcd1 | 0xbcfd
            | 0xbd03 | 0xbd07 | 0xbd0b => {
                w.compare_x_immediate(
                    match pc {
                        0xbc03 | 0xbcba | 0xbd07 => 0x97,
                        0xbc07 | 0xbcbe | 0xbd0b => 0x9d,
                        0xbc16 => 0xb5,
                        0xbc1a => 0xbe,
                        0xbccd => 0x95,
                        0xbcd1 => 0x9e,
                        0xbcfd => 0x47,
                        _ => 0x9c,
                    },
                    1,
                );
                pc + 2
            }
            0xbc11 | 0xbc1e | 0xbcc8 | 0xbcd5 => {
                w.drain(6);
                let old = w.cpu.a;
                w.cpu.p = (w.cpu.p & !CARRY) | (old as u8 & 1);
                w.load(old >> 1, 2);
                pc + 1
            }
            0xbc12 | 0xbc1f | 0xbcc9 | 0xbcd6 => {
                let old = w.dp_read(0, 2);
                let v = (old >> 1) | if w.cpu.p & CARRY != 0 { 0x8000 } else { 0 };
                w.cpu.p = (w.cpu.p & !CARRY) | (old as u8 & 1);
                w.drain(6);
                w.flags(v, 2);
                w.write_high_low(0, v, 2);
                pc + 2
            }
            0xbc26 | 0xbc30 | 0xbcdd | 0xbce7 | 0xbd55 => {
                w.drain(16);
                w.load(w.cpu.a | if pc == 0xbd55 { 0x100 } else { 0xf000 }, 2);
                pc + 3
            }
            0xbc39 | 0xbc40 | 0xbcf0 | 0xbcf7 | 0xbd79 | 0xbd80 | 0xbd8a => {
                let v = w.dp_read(if matches!(pc, 0xbc40 | 0xbcf7) { 6 } else { 0 }, 2);
                w.adc(v);
                pc + 2
            }
            0xbc8a => {
                w.dp_store(8, w.cpu.x, 1);
                pc + 2
            }
            0xbd37 | 0xbd3f | 0xbda6 => {
                w.drain(16);
                w.sbc(
                    match pc {
                        0xbd37 => 0x2000,
                        0xbda6 => 16,
                        _ => 0,
                    },
                    2,
                );
                pc + 3
            }
            0xbd43 | 0xbdaf => {
                w.drain(16);
                let v = w.read(w.absolute(if pc == 0xbd43 { 0x69e } else { 0x416 }), 2);
                if pc == 0xbd43 {
                    w.adc(v);
                } else {
                    w.load(w.cpu.a | v, 2);
                }
                pc + 3
            }
            0xbd62 => {
                w.operand_then_internal(1);
                let v = w.read(u32::from(0xe2 + w.cpu.x), 2);
                w.load(v, 2);
                pc + 2
            }
            0xbd69 | 0xbd94 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                pc + 1
            }
            0xbd6d | 0xbd98 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0xbd71 | 0xbd74 | 0xbd8c | 0xbd91 | 0xbd9d | 0xbda9 | 0xbdbc => {
                let base = if pc == 0xbd8c {
                    0x61a
                } else if pc == 0xbd91 {
                    0x618
                } else {
                    0x624
                };
                let index = if matches!(pc, 0xbd71 | 0xbd9d | 0xbda9) {
                    w.cpu.y
                } else {
                    w.cpu.x
                };
                w.operand_then_internal(2);
                w.write(w.absolute(base + index), w.cpu.a, 2);
                pc + 3
            }
            0xbd7b => {
                w.operand_then_internal(1);
                w.write(u32::from(0xe2 + w.cpu.x), w.cpu.a, 2);
                pc + 2
            }
            0xbd86 | 0xbd99 | 0xbdac | 0xbdb9 => {
                let base = match pc {
                    0xbd86 => 0x61a,
                    0xbdac => 0xbb88,
                    _ => 0x624,
                };
                let index = if pc == 0xbd86 { w.cpu.x } else { w.cpu.y };
                w.drain(16);
                if (base & 255) + index > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(base + index), 2);
                if pc == 0xbdb9 {
                    w.sbc(v, 2);
                } else {
                    w.load(v, 2);
                }
                pc + 3
            }
            0xbd61 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            0xbd77 | 0xbdbf => {
                w.rts();
                w.cpu.pc
            }
            _ => unreachable!("overworld camera PC {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overworld_camera_matches_source_motion_boundaries_overlays_and_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for offset in 0..8 {
            assert_eq!(
                table_byte(0x02bb88 + offset),
                Some(rom[0x13b88 + offset as usize])
            );
        }
        let mut visited = std::collections::BTreeSet::new();
        let mut instructions = 0;
        for vy in [0u8, 1, 3, 127, 128, 253, 255] {
            for vx in [0u8, 1, 3, 127, 128, 253, 255] {
                for case in 0..64usize {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    super::super::dungeon_camera::seed_case(&mut machine.ram, vy, vx, case);
                    machine.ram[0x8c] = [0, 0x95, 0x9e, 0xb5, 0xbe, 0x97, 0x9d, 0x9c][case % 8];
                    for (address, value) in [
                        (0x8a, [0u16, 0x1b, 0x47, 0x5b][case / 8 % 4]),
                        (0xa0, if case & 32 == 0 { 0x20 } else { 0x181 }),
                        (0xe6, [0x5ffu16, 0x600, 0x6bf, 0x6c0][case / 16 % 4]),
                        (0x624, [0u16, 15, 16, 65535][case / 8 % 4]),
                        (0x626, [0u16, 65535, 16, 15][case / 8 % 4]),
                        (0x628, [15u16, 0, 65535, 16][case / 8 % 4]),
                        (0x62a, [65535u16, 16, 0, 15][case / 8 % 4]),
                        (0x69e, 0xa55a),
                    ] {
                        machine.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
                    }
                    machine.ram[0x1f9..0x1fb].copy_from_slice(&[0xb8, 0xac]);
                    machine.ram[0x1fb..0x1fd].copy_from_slice(&[0x7d, 0xa4]);
                    machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
                    let mut cpu = NativeNmiContext {
                        a: 0x1100,
                        x: 0,
                        y: 6,
                        d: 0,
                        db: [0, 2, 0x80, 0x82][case % 4],
                        bank: 2,
                        pc: 0xbb90,
                        sp: 0x1f8,
                        p: 0x30 | [0, 1, 0x41, 0xc3][case % 4],
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.db = cpu.db;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let clock = CpuMasterTimeline::at_raster(
                        4937,
                        snes::CpuRasterPosition::new(224, 1300),
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
                    let mut work = OverworldCamera::default();
                    for _ in 0..20000 {
                        if work.complete() {
                            break;
                        }
                        visited.insert(cpu.pc);
                        super::super::native_work::compare_instruction_with_devices(
                            &format!(
                                "overworld camera vy={vy} vx={vx} case={case} pc={:04x}",
                                cpu.pc
                            ),
                            &mut cpu,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            None,
                            |_, access| match access {
                                super::super::intro_clear::Access::Read(address, width)
                                    if address & 0x8000 != 0 && address < 0x7e0000 =>
                                {
                                    let read = |a| {
                                        table_byte(a).expect("camera reads only its direction bits")
                                    };
                                    Some(if width == 1 {
                                        u16::from(read(address))
                                    } else {
                                        u16::from_le_bytes([read(address), read(address + 1)])
                                    })
                                }
                                _ => None,
                            },
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        work = work.clone();
                        instructions += 1;
                    }
                    assert!(work.complete());
                    assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.db),
                        (2, 0xacb9, 0x1fa, [0, 2, 0x80, 0x82][case % 4])
                    );
                    assert_eq!(ram, source.snes().ram);
                }
            }
        }
        for pc in [
            0xbb9c, 0xbba3, 0xbbc9, 0xbbf5, 0xbc1e, 0xbc26, 0xbc30, 0xbc55, 0xbc5e, 0xbc7e, 0xbcac,
            0xbcd5, 0xbcdd, 0xbce7, 0xbd0f, 0xbd33, 0xbd53, 0xbd71, 0xbd7b, 0xbda9, 0xbdac, 0xbdbf,
        ] {
            assert!(visited.contains(&pc), "unexercised camera path {pc:04x}");
        }
        eprintln!("3136 native overworld cameras, {instructions} Source instruction boundaries, {} distinct PCs; no behavioral ROM reads",visited.len());
    }
}
