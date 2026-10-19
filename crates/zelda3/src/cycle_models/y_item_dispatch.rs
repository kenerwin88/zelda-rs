//! Retained Y-item selector. Item bodies and cape unequip are explicit child
//! boundaries; dispatch consumes the real caller stack rather than a cycle sum.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

const ITEM_TARGETS: [u16; 20] = [
    0xa138, 0xa0bb, 0xa006, 0x9f7b, 0x9eef, 0x9eef, 0xaff8, 0xa313, 0xa24d, 0xa293, 0xa15b, 0xa471,
    0xaf3e, 0xab25, 0xa569, 0xa494, 0xa64b, 0xaec0, 0xadc1, 0xa91a,
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let offset = address.checked_sub(0x079ae6)? as usize;
    Some(ITEM_TARGETS.get(offset / 2)?.to_le_bytes()[offset & 1])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum YItemChild {
    ForceUnequipCape,
    Item(u8),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct YItemDispatch {
    pc: u16,
    child: Option<YItemChild>,
    complete: bool,
}
impl Default for YItemDispatch {
    fn default() -> Self {
        Self {
            pc: 0x9b0e,
            child: None,
            complete: false,
        }
    }
}
impl YItemDispatch {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn child(&self) -> Option<YItemChild> {
        self.child
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        assert_eq!(cpu.bank, 7);
        match self.child.take().expect("Y-item child return") {
            YItemChild::ForceUnequipCape => {
                assert_eq!(cpu.pc, 0x9b66);
                self.pc = cpu.pc;
            }
            YItemChild::Item(_) => {
                self.pc = cpu.pc;
                self.complete = true;
            }
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete || self.child.is_some() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (7, self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x9b0e | 0x9b52 | 0x9b5f => {
                let value = w.dp_read(
                    match self.pc {
                        0x9b0e => 0x3c,
                        0x9b52 => 0x3a,
                        _ => 0x55,
                    },
                    1,
                );
                w.load(value, 1);
                self.pc + 2
            }
            0x9b16 | 0x9b1b | 0x9b2c | 0x9b43 | 0x9b58 | 0x9b66 | 0x9b7e => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match self.pc {
                        0x9b16 | 0x9b2c => 0x2e0,
                        0x9b1b => 0x303,
                        0x9b66 => 0x301,
                        _ => 0x304,
                    }),
                    1,
                );
                w.load(value, 1);
                self.pc + 3
            }
            0x9b27 | 0x9b3b | 0x9b6e => {
                w.drain(16);
                w.cpu.y = w.read(
                    w.absolute(match self.pc {
                        0x9b27 => 0x3fc,
                        0x9b3b => 0x304,
                        _ => 0x303,
                    }),
                    1,
                );
                w.flags(w.cpu.y, 1);
                self.pc + 3
            }
            0x9b12 | 0x9b1e | 0x9b22 | 0x9b46 | 0x9b5b => {
                w.compare_immediate(
                    match self.pc {
                        0x9b12 => 9,
                        0x9b1e => 11,
                        0x9b22 => 20,
                        0x9b46 => 8,
                        _ => 19,
                    },
                    1,
                );
                self.pc + 2
            }
            0x9b31 | 0x9b76 | 0x9b7a => {
                w.drain(8);
                let value = match self.pc {
                    0x9b31 => 2,
                    0x9b76 => 5,
                    _ => 6,
                };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= value);
                w.flags(w.cpu.y.wrapping_sub(value) & 255, 1);
                self.pc + 2
            }
            0x9b3e => {
                w.drain(16);
                let value = w.read(w.absolute(0x303), 1);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= value);
                w.flags(w.cpu.y.wrapping_sub(value) & 255, 1);
                0x9b41
            }
            0x9b10 | 0x9b14 | 0x9b19 | 0x9b20 | 0x9b24 | 0x9b2a | 0x9b2f | 0x9b33 | 0x9b41
            | 0x9b48 | 0x9b50 | 0x9b5d | 0x9b61 | 0x9b6c | 0x9b74 | 0x9b78 | 0x9b7c | 0x9b89 => {
                let (taken, target) = match self.pc {
                    0x9b10 => (w.cpu.p & ZERO != 0, 0x9b16),
                    0x9b14 => (w.cpu.p & CARRY == 0, 0x9ae5),
                    0x9b19 | 0x9b20 | 0x9b24 => (w.cpu.p & ZERO != 0, 0x9b27),
                    0x9b2a => (w.cpu.p & ZERO != 0, 0x9b3b),
                    0x9b2f => (w.cpu.p & ZERO == 0, 0x9b3b),
                    0x9b33 => (w.cpu.p & ZERO != 0, 0x9b38),
                    0x9b41 | 0x9b61 => (w.cpu.p & ZERO != 0, 0x9b66),
                    0x9b48 => (w.cpu.p & ZERO == 0, 0x9b58),
                    0x9b50 => (w.cpu.p & ZERO != 0, 0x9b58),
                    0x9b5d => (w.cpu.p & ZERO == 0, 0x9b66),
                    0x9b6c => (w.cpu.p & ZERO == 0, 0x9b74),
                    0x9b74 => (w.cpu.p & ZERO != 0, 0x9b91),
                    0x9b78 => (w.cpu.p & ZERO != 0, 0x9b7e),
                    0x9b7c => (w.cpu.p & ZERO == 0, 0x9b88),
                    _ => (w.cpu.p & NEGATIVE != 0, 0x9b91),
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x9b35 | 0x9b38 => {
                w.operand_then_internal(2);
                self.child = Some(YItemChild::Item(if self.pc == 0x9b35 { 21 } else { 3 }));
                if self.pc == 0x9b35 {
                    0xa32c
                } else {
                    0xa006
                }
            }
            0x9b4a => {
                w.drain(24);
                let value = w.read(0x7ef34c, 1);
                w.load(value, 1);
                0x9b4e
            }
            0x9b4e | 0x9b54 => {
                w.mask(if self.pc == 0x9b4e { 2 } else { 0xbf }, 1);
                self.pc + 2
            }
            0x9b56 => {
                w.dp_store(0x3a, w.cpu.a & 255, 1);
                0x9b58
            }
            0x9b63 => {
                w.item_call();
                self.child = Some(YItemChild::ForceUnequipCape);
                0xae47
            }
            0x9b69 => {
                w.drain(16);
                let value = w.read(w.absolute(0x37a), 1);
                w.load((w.cpu.a & 255) | value, 1);
                0x9b6c
            }
            0x9b71 => {
                w.drain(16);
                w.write(w.absolute(0x304), w.cpu.y, 1);
                0x9b74
            }
            0x9b81 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0x9b82
            }
            0x9b82 => {
                w.drain(8);
                w.sbc(5, 1);
                0x9b84
            }
            0x9b84 => {
                w.increment_a(1);
                0x9b85
            }
            0x9b85 => {
                w.drain(16);
                w.write(w.absolute(0x307), w.cpu.a & 255, 1);
                0x9b88
            }
            0x9b88 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                0x9b89
            }
            0x9b8b => {
                w.drain(6);
                w.load(w.cpu.y, 1);
                0x9b8c
            }
            0x9b8c => {
                w.shift_left(1);
                0x9b8d
            }
            0x9b8d => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                0x9b8e
            }
            0x9b8e => {
                w.operand_then_internal(2);
                let target = w.read(0x079ae6 + u32::from(w.cpu.x), 2);
                let item = u8::try_from(w.cpu.y + 1).unwrap();
                assert_eq!(Some(&target), ITEM_TARGETS.get(usize::from(item - 1)));
                self.child = Some(YItemChild::Item(item));
                target
            }
            0x9ae5 | 0x9b26 | 0x9b91 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("native Y-item dispatch {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
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
    fn y_item_selector_matches_source_through_every_child_boundary() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x079ae6..0x079b0e {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        let mut instructions = 0;
        let mut children = [false; 22];
        let mut cape_child = false;
        let mut early_returns = 0;
        for item in 0..=20u8 {
            for previous in 0..=20u8 {
                for variant in 0..12u8 {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    let r = &mut machine.ram;
                    r[0x303] = item;
                    r[0x304] = previous;
                    r[0x3c] = match variant {
                        1 => 1,
                        2 => 8,
                        3 => 9,
                        _ => 0,
                    };
                    r[0x2e0] = u8::from(variant == 4);
                    r[0x3fc] = match variant {
                        5 => 1,
                        6 => 2,
                        _ => 0,
                    };
                    r[0x301] = u8::from(variant == 7);
                    r[0x37a] = if variant == 8 { 2 } else { 0 };
                    r[0xf34c] = if variant == 9 { 2 } else { 0 };
                    r[0x3a] = 0xc0;
                    r[0x55] = u8::from(variant == 10);
                    r[0x1f3..0x1f5].copy_from_slice(&[0xc8, 0x81]);
                    let mut cpu = NativeNmiContext {
                        a: (u16::from(item) << 8) | 0xa5,
                        x: 0x6e,
                        y: 0xf3,
                        d: 0,
                        db: if variant & 2 != 0 { 0 } else { 7 },
                        bank: 7,
                        pc: 0x9b0e,
                        sp: 0x1f2,
                        p: 0x30 | if variant == 11 { 0xc3 } else { variant & 3 },
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
                            if variant & 1 == 0 { 21 } else { 261 },
                            if variant & 1 == 0 { 238 } else { 1350 },
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
                    let mut work = YItemDispatch::default();
                    while !work.complete() && work.child().is_none() {
                        let label = format!(
                            "Y-item={item} previous={previous} variant={variant} pc={:04x}",
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
                                let super::super::intro_clear::Access::Read(address, 2) = access
                                else {
                                    return None;
                                };
                                table_byte(address).map(|low| {
                                    u16::from_le_bytes([low, table_byte(address + 1).unwrap()])
                                })
                            },
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        work = work.clone();
                        instructions += 1;
                    }
                    assert_eq!(ram, source.snes().ram);
                    match work.child() {
                        Some(YItemChild::ForceUnequipCape) => {
                            cape_child = true;
                            assert_eq!((cpu.pc, cpu.sp), (0xae47, 0x1f0));
                            assert_eq!(&ram[0x1f1..0x1f5], &[0x65, 0x9b, 0xc8, 0x81]);
                        }
                        Some(YItemChild::Item(selected)) => {
                            children[usize::from(selected)] = true;
                            let target = if selected == 21 {
                                0xa32c
                            } else {
                                ITEM_TARGETS[usize::from(selected - 1)]
                            };
                            assert_eq!((cpu.pc, cpu.sp), (target, 0x1f2));
                            assert_eq!(&ram[0x1f3..0x1f5], &[0xc8, 0x81]);
                        }
                        None => {
                            early_returns += 1;
                            assert!(work.complete());
                            assert_eq!((cpu.pc, cpu.sp), (0x81c9, 0x1f4));
                        }
                    }
                }
            }
        }
        assert!(children[1..].iter().all(|seen| *seen));
        assert!(cape_child && early_returns > 0);
        println!("Y-item selector: 5292 cases, {instructions} source instructions, all 20 ordinary item boundaries, minigame shovel, and cape unequip");
    }
}
