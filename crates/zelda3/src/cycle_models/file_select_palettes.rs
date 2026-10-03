//! Save-slot palette loads use live SRAM words and retain their copy loops,
//! stack, and partially retired palette stores across interrupts.
use super::{
    intro_clear::Access,
    native_work::Work,
    nmi_stack::{NativeNmiContext, ZERO},
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    address.checked_sub(0x1bebb4).and_then(|i| {
        [
            0, 0, 6, 12, 18, 24, 30, 36, 42, 48, 54, 60, 66, 0, 0, 8, 16, 24,
        ]
        .get(i as usize)
        .copied()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn file_palettes_match_each_instruction_for_all_equipment_levels() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for a in 0x1bebb4..0x1bebc6 {
            assert_eq!(table_byte(a), Some(rom[0xd8000 + (a as usize & 0x7fff)]));
        }
        for variant in 0..20 {
            for (v, h) in [(224, 1338), (261, 1350)] {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8)) as u8;
                }
                machine.cart.ram.fill(0xa7);
                for slot in 0..3 {
                    let i = variant + slot;
                    for (offset, value) in [
                        (0x354, i % 3),
                        (0x359, i % 5),
                        (0x35a, i % 4),
                        (0x35b, (i / 3) % 3),
                    ] {
                        machine.cart.ram[slot * 0x500 + offset] = value as u8;
                    }
                }
                machine.ram[0x1f8..0x1fb].copy_from_slice(&[9, 0xce, 12]);
                let mut native = NativeNmiContext {
                    a: 0xab83,
                    x: 0x71,
                    y: 0x83,
                    d: 0,
                    db: (variant & 1) as u8,
                    bank: 27,
                    pc: 0xef96,
                    sp: 0x1f7,
                    p: 0x31,
                };
                machine.cpu.a = native.a;
                machine.cpu.x = native.x;
                machine.cpu.y = native.y;
                machine.cpu.k = native.bank;
                machine.cpu.db = native.db;
                machine.cpu.pc = native.pc;
                machine.cpu.sp = native.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(native.p);
                let mut ram = machine.ram.clone();
                let sram = machine.cart.ram.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
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
                let mut work = FileSelectPalettes::default();
                let mut count = 0;
                while !work.complete() {
                    let label = format!("file palettes variant{variant} pc{:04x}", native.pc);
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut native,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, a| {
                            if let Access::Read(a, 2) = a {
                                if (0x700000..0x702000).contains(&a) {
                                    let i = (a - 0x700000) as usize;
                                    Some(u16::from_le_bytes([sram[i], sram[i + 1]]))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    work = work.clone();
                    count += 1;
                    assert!(count < 10000);
                }
                assert_eq!((native.bank, native.pc, native.sp), (12, 0xce0a, 0x1fa));
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileSelectPalettes {
    pc: u16,
    complete: bool,
}
impl Default for FileSelectPalettes {
    fn default() -> Self {
        Self {
            pc: 0xef96,
            complete: false,
        }
    }
}
impl FileSelectPalettes {
    pub(crate) fn complete(&self) -> bool {
        self.complete
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (27, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xef96 | 0xef99 => {
                w.drain(6);
                w.push(
                    if self.pc == 0xef96 {
                        u16::from(w.cpu.db)
                    } else {
                        w.cpu.a & 255
                    },
                    1,
                );
                self.pc + 1
            }
            0xef97 => {
                w.immediate(27, 1);
                0xef99
            }
            0xef9a | 0xf030 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xef9b => {
                w.width(0x30, true);
                0xef9d
            }
            0xef9d..=0xf006 => {
                let slot = (self.pc - 0xef9d) / 0x24;
                let i = (self.pc - 0xef9d) % 0x24;
                assert!(slot < 3);
                match i {
                    0 | 16 | 26 => {
                        w.drain(16);
                        w.cpu.x = slot * 0x40;
                        w.flags(w.cpu.x, 2);
                        self.pc + 3
                    }
                    3 | 9 | 19 | 29 => {
                        let offset = match i {
                            3 => 0x354,
                            9 => 0x35b,
                            19 => 0x359,
                            _ => 0x35a,
                        };
                        w.drain(24);
                        let v = w.read(0x700000 + u32::from(slot) * 0x500 + offset, 2);
                        w.load(v, 2);
                        self.pc + 4
                    }
                    7 => {
                        w.dp_store(12, w.cpu.a, 2);
                        self.pc + 2
                    }
                    13 | 23 | 33 => {
                        w.drain(22);
                        w.push(self.pc + 2, 2);
                        match i {
                            13 => 0xf032,
                            23 => 0xf072,
                            _ => 0xf09a,
                        }
                    }
                    _ => unreachable!("save palette caller {:04x}", self.pc),
                }
            }
            0xf009 | 0xf00c => {
                w.drain(16);
                if self.pc == 0xf009 {
                    w.cpu.y = 0
                } else {
                    w.cpu.x = 0
                };
                w.flags(0, 2);
                self.pc + 3
            }
            0xf00f | 0xf01a | 0xf038 | 0xf048 | 0xf066 | 0xf076 | 0xf086 | 0xf09e | 0xf0ae => {
                let base = match self.pc {
                    0xf00f => 0xd226,
                    0xf01a => 0xd244,
                    0xf038 => 0xec06,
                    0xf066 => 0xedf5,
                    0xf076 => 0xebb4,
                    0xf09e => 0xebc1,
                    _ => 0xd218,
                };
                w.drain(22);
                let v = w.read(w.absolute(base).wrapping_add(u32::from(w.cpu.y)), 2);
                w.load(v, 2);
                self.pc + 3
            }
            0xf012 | 0xf016 | 0xf01d | 0xf021 | 0xf04b | 0xf04f | 0xf069 | 0xf06d | 0xf089
            | 0xf08d | 0xf0b1 | 0xf0b5 => {
                let base = match self.pc {
                    0xf012 => 0x7ec4d0,
                    0xf016 => 0x7ec6d0,
                    0xf01d => 0x7ec4f0,
                    0xf021 => 0x7ec6f0,
                    0xf04b => 0x7ec402,
                    0xf04f => 0x7ec602,
                    0xf069 => 0x7ec41a,
                    0xf06d => 0x7ec61a,
                    0xf089 => 0x7ec432,
                    0xf08d => 0x7ec632,
                    0xf0b1 => 0x7ec438,
                    _ => 0x7ec638,
                };
                w.drain(24);
                w.write(base + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0xf025 | 0xf026 | 0xf053 | 0xf054 | 0xf091 | 0xf092 | 0xf0b9 | 0xf0ba => {
                w.increment_y();
                self.pc + 1
            }
            0xf027 | 0xf028 | 0xf055 | 0xf056 | 0xf093 | 0xf094 | 0xf0bb | 0xf0bc => {
                w.increment_x();
                self.pc + 1
            }
            0xf029 => {
                w.compare_x_immediate(14, 2);
                0xf02c
            }
            0xf02c | 0xf059 | 0xf097 | 0xf0bf => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0xf02c => 0xf00f,
                        0xf059 => 0xf048,
                        0xf097 => 0xf086,
                        _ => 0xf0ae,
                    }
                } else {
                    self.pc + 2
                }
            }
            0xf02e => {
                w.width(0x30, false);
                0xf030
            }
            0xf031 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0xf032 => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                0xf033
            }
            0xf033 | 0xf03b | 0xf05e | 0xf072 | 0xf079 | 0xf09a | 0xf0a1 => {
                w.mask(255, 2);
                self.pc + 3
            }
            0xf036 | 0xf064 => {
                w.shift_left(2);
                self.pc + 1
            }
            0xf037 | 0xf042 | 0xf065 | 0xf075 | 0xf080 | 0xf09d | 0xf0a8 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xf03e | 0xf07c | 0xf0a4 => {
                w.clear_carry();
                self.pc + 1
            }
            0xf03f | 0xf07d | 0xf0a5 => {
                w.drain(16);
                w.adc(match self.pc {
                    0xf03f => 0xf0,
                    0xf07d => 0x418,
                    _ => 0x430,
                });
                self.pc + 3
            }
            0xf043 | 0xf081 | 0xf0a9 => {
                w.immediate(
                    match self.pc {
                        0xf043 => 15,
                        0xf081 => 3,
                        _ => 4,
                    },
                    2,
                );
                self.pc + 3
            }
            0xf046 | 0xf084 | 0xf0ac => {
                w.dp_store(14, w.cpu.a, 2);
                self.pc + 2
            }
            0xf057 | 0xf095 | 0xf0bd => {
                w.dp_increment(14, 2, false);
                self.pc + 2
            }
            0xf05b => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                0xf05c
            }
            0xf05c => {
                let v = w.dp_read(12, 2);
                w.load(v, 2);
                0xf05e
            }
            0xf061 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0xf071
                } else {
                    0xf063
                }
            }
            0xf063 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1), 2);
                0xf064
            }
            0xf071 | 0xf099 | 0xf0c1 => {
                w.rts();
                w.cpu.pc
            }
            _ => unreachable!("save palette continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
