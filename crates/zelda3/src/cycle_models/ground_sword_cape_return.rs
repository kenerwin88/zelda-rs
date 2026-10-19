//! Player return through sword cooldown and passive cape depletion. Unported
//! sword controls and cape unequip remain explicit calls on the live stack.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    [4, 8, 8]
        .get(address.checked_sub(0x07adbe)? as usize)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn ground_sword_cape_return_matches_source_timers_magic_and_child_stack() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in 0x07adbe..0x07adc1 {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        let mut instructions = 0;
        let mut tails = [false; 5];
        for timer in 0..=255u8 {
            for variant in 0..18u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                r[0x2e3] = timer;
                r[0x4c] = timer;
                r[0x30] = 0xa5;
                r[0x31] = 0x5a;
                r[0xf3c5] = u8::from(variant > 0 && variant < 7);
                r[0x301] = u8::from(variant == 2);
                r[0x37a] = u8::from(variant == 3);
                r[0x3c] = if variant == 4 { 9 } else { 0 };
                r[0x372] = u8::from(matches!(variant, 1 | 6));
                r[0x5d] = if variant == 6 { 3 } else { 0 };
                if variant >= 7 {
                    r[0x308] = if variant == 7 { 0 } else { 0x80 };
                    r[0x55] = u8::from(variant != 8);
                    r[0x304] = if variant == 9 { 18 } else { 19 };
                    r[0x303] = if variant == 10 { 18 } else { 19 };
                }
                r[0xf37b] = timer.wrapping_add(variant) % 3;
                r[0xf36e] = match variant {
                    11 => 0,
                    12 => 1,
                    13 => 2,
                    14 => 127,
                    15 => 128,
                    16 => 255,
                    _ => 64,
                };
                let pc = if variant >= 7 { 0x81df } else { 0x81c9 };
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: 8,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: 0x1f4,
                    p: 0x30 | (timer & 0xc3),
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
                let mut work = GroundSwordCapeReturn::from_entry(pc);
                while work.tail().is_none() {
                    let label = format!(
                        "sword/cape timer={timer} variant={variant} pc={:04x}",
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
                            let super::super::intro_clear::Access::Read(address, 1) = access else {
                                return None;
                            };
                            table_byte(address).map(u16::from)
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                let (index, end, sp) = match work.tail().unwrap() {
                    GroundSwordCapeTail::Movement => (0, 0x81e2, 0x1f4),
                    GroundSwordCapeTail::ClearMovement => (1, 0x82d2, 0x1f4),
                    GroundSwordCapeTail::CheckSwordSwing => (2, 0x9cd9, 0x1f0),
                    GroundSwordCapeTail::SwordControls => (3, 0x9d72, 0x1f0),
                    GroundSwordCapeTail::SwordSwingActive => unreachable!("guard child is disabled in this fixture"),
                    GroundSwordCapeTail::ForceUnequipCape => (4, 0xae47, 0x1f0),
                };
                tails[index] = true;
                assert_eq!((cpu.pc, cpu.sp), (end, sp));
            }
        }
        assert!(tails.iter().all(|seen| *seen));
        println!("sword/cape return: 4608 cases, {instructions} source instructions, all five tail boundaries");
    }

    #[test]
    fn sword_swing_guards_retain_idle_return_and_active_child_stack() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let mut count = 0;
        for blocked in [0u8, 0x10, 0xff] {
            for held in [0u8, 0x80] {
                for pressed in [0u8, 0x80] {
                    for phase in 0..4 {
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        machine.ram[0xf3c5] = 1;
                        machine.ram[0x3b] = blocked;
                        machine.ram[0x3a] = held;
                        machine.ram[0xf4] = pressed;
                        let mut cpu = NativeNmiContext {
                            a: 0xab03, x: 255, y: 20, d: 0, db: 7,
                            bank: 7, pc: 0x81c9, sp: 0x1f4, p: 0x30 | (phase as u8 & 3),
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
                        let mut clock = CpuMasterTimeline::at_raster(37148 + phase,
                            CpuRasterPosition::new(224, 1340), CpuBusWorkload::default(),
                            CpuFieldTiming::non_interlace(phase & 1 != 0));
                        let mut source = RomCpuTimingProbe::new(machine, clock.clone(),
                            SourcePpuReadState::snes9x_reset()).unwrap();
                        clock.begin_synchronous_timeline().unwrap();
                        let mut work = GroundSwordCapeReturn::from_entry(0x81c9)
                            .with_sword_swing_guards();
                        while work.tail().is_none() {
                            super::super::native_work::compare_instruction(
                                &format!("swing blocked={blocked:02x} held={held:02x} pressed={pressed:02x} phase={phase} pc={:04x}", cpu.pc),
                                &mut cpu, &mut clock, &mut source, &mut ram, &rom,
                                |cpu, clock, bus| { work.step(cpu, clock, bus).unwrap(); });
                            work = work.clone();
                            count += 1;
                        }
                        assert_eq!(ram, source.snes().ram);
                        if blocked & 0x10 != 0 || (held | pressed) & 0x80 == 0 {
                            assert_eq!(work.tail(), Some(GroundSwordCapeTail::Movement));
                            assert_eq!((cpu.pc, cpu.sp), (0x81e2, 0x1f4));
                        } else {
                            assert_eq!(work.tail(), Some(GroundSwordCapeTail::SwordSwingActive));
                            assert_eq!((cpu.pc, cpu.sp), (if held & 0x80 != 0 { 0x9d03 } else { 0x9ce7 }, 0x1f0));
                        }
                    }
                }
            }
        }
        println!("sword swing guards: 48 cases, {count} Source instruction boundaries, idle return and both active child stacks");
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroundSwordCapeTail {
    Movement,
    ClearMovement,
    CheckSwordSwing,
    SwordSwingActive,
    SwordControls,
    ForceUnequipCape,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GroundSwordCapeReturn {
    pc: u16,
    tail: Option<GroundSwordCapeTail>,
    sword_swing_guards: bool,
}
impl GroundSwordCapeReturn {
    pub(crate) fn from_entry(pc: u16) -> Self {
        assert!(matches!(pc, 0x81c9 | 0x81df));
        Self { pc, tail: None, sword_swing_guards: false }
    }
    pub(crate) fn with_sword_swing_guards(mut self) -> Self {
        self.sword_swing_guards = true;
        self
    }
    pub(crate) fn tail(&self) -> Option<GroundSwordCapeTail> {
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
            0x81c9 | 0xaea5 | 0xaeaf => {
                w.drain(24);
                let value = w.read(
                    match self.pc {
                        0x81c9 => 0x7ef3c5,
                        0xaea5 => 0x7ef37b,
                        _ => 0x7ef36e,
                    },
                    1,
                );
                w.load(value, 1);
                self.pc + 4
            }
            0x81cd | 0x81d6 | 0x9ac5 | 0x9ad0 | 0x9ad6 | 0x9adb | 0xae8d | 0xae91 | 0xae98
            | 0xae9d | 0xaea3 | 0xaeb3 | 0xaeba => {
                let (taken, target) = match self.pc {
                    0x81cd => (w.cpu.p & ZERO != 0, 0x81df),
                    0x81d6 => (w.cpu.p & ZERO == 0, 0x81df),
                    0x9ac5 => (w.cpu.p & NEGATIVE == 0, 0x9ae5),
                    0x9ad0 | 0x9adb => (w.cpu.p & ZERO == 0, 0x9ae5),
                    0x9ad6 => (w.cpu.p & CARRY != 0, 0x9ae2),
                    0xae8d | 0xae91 => (w.cpu.p & ZERO != 0, 0xae87),
                    0xae98 => (w.cpu.p & ZERO == 0, 0xae87),
                    0xae9d => (w.cpu.p & ZERO == 0, 0xaebc),
                    0xaea3 | 0xaeba => (w.cpu.p & ZERO == 0, 0xaebf),
                    _ => (w.cpu.p & ZERO != 0, 0xaebf),
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x81cf | 0x81df | 0x9add | 0x9ae2 | 0xaebc => {
                w.item_call();
                match self.pc {
                    0x81cf => 0x9ac2,
                    0x81df => 0xae88,
                    0x9add => {
                        if !self.sword_swing_guards {
                            self.tail = Some(GroundSwordCapeTail::CheckSwordSwing);
                        }
                        0x9cd9
                    }
                    0x9ae2 => {
                        self.tail = Some(GroundSwordCapeTail::SwordControls);
                        0x9d72
                    }
                    _ => {
                        self.tail = Some(GroundSwordCapeTail::ForceUnequipCape);
                        0xae47
                    }
                }
            }
            0x81d2 | 0x9ad2 | 0xae8f | 0xaea1 => {
                let value = w.dp_read(
                    match self.pc {
                        0x81d2 => 0x5d,
                        0x9ad2 => 0x3c,
                        0xae8f => 0x55,
                        _ => 0x4c,
                    },
                    1,
                );
                w.load(value, 1);
                self.pc + 2
            }
            0x81d4 | 0x9ad4 | 0xae96 => {
                w.compare_immediate(
                    match self.pc {
                        0x81d4 => 3,
                        0x9ad4 => 9,
                        _ => 19,
                    },
                    1,
                );
                self.pc + 2
            }
            0x81d8 | 0x81da => {
                w.dp_store(if self.pc == 0x81d8 { 0x30 } else { 0x31 }, 0, 1);
                self.pc + 2
            }
            0x81dc => {
                w.operand_then_internal(2);
                self.tail = Some(GroundSwordCapeTail::ClearMovement);
                0x82d2
            }
            0x9ac2 => {
                w.drain(16);
                let address = w.absolute(0x2e3);
                let value = w.read(address, 1).wrapping_sub(1) & 255;
                w.drain(6);
                w.flags(value, 1);
                w.write(address, value, 1);
                0x9ac5
            }
            0x9ac7 => {
                w.drain(16);
                w.write(w.absolute(0x2e3), 0, 1);
                0x9aca
            }
            0x9aca | 0x9ad8 | 0xae88 | 0xae93 => {
                w.drain(16);
                let value = w.read(
                    w.absolute(match self.pc {
                        0x9aca => 0x301,
                        0x9ad8 => 0x372,
                        0xae88 => 0x308,
                        _ => 0x304,
                    }),
                    1,
                );
                w.load(value, 1);
                self.pc + 3
            }
            0x9acd => {
                w.drain(16);
                let value = w.read(w.absolute(0x37a), 1);
                w.load((w.cpu.a & 255) | value, 1);
                0x9ad0
            }
            0x9ae0 => {
                w.branch(true);
                0x9ae5
            }
            0x9cd9 => {
                let value = w.dp_read(0x3b, 1);
                w.load(value, 1);
                0x9cdb
            }
            0x9cdb => {
                w.mask(0x10, 1);
                0x9cdd
            }
            0x9cdf | 0x9ce3 => {
                let value = w.dp_read(if self.pc == 0x9cdf { 0x3a } else { 0xf4 }, 1) as u8;
                w.cpu.p = (w.cpu.p & !0xc2) | (value & 0xc0)
                    | if (w.cpu.a as u8) & value == 0 { ZERO } else { 0 };
                self.pc + 2
            }
            0x9cdd | 0x9ce1 | 0x9ce5 => {
                let (taken, target) = match self.pc {
                    0x9cdd => (w.cpu.p & ZERO == 0, 0x9cbe),
                    0x9ce1 => (w.cpu.p & NEGATIVE != 0, 0x9d03),
                    _ => (w.cpu.p & NEGATIVE == 0, 0x9cbe),
                };
                let next = if w.branch(taken) { target } else { self.pc + 2 };
                if matches!(next, 0x9d03 | 0x9ce7) {
                    self.tail = Some(GroundSwordCapeTail::SwordSwingActive);
                }
                next
            }
            0x9cbe => {
                w.rts();
                assert_eq!(w.cpu.pc, 0x9ae0);
                w.cpu.pc
            }
            0x9ae5 => {
                w.rts();
                assert_eq!(w.cpu.pc, 0x81d2);
                w.cpu.pc
            }
            0xae8b => {
                w.mask(0x80, 1);
                0xae8d
            }
            0xae9a => {
                w.drain(16);
                let value = w.read(w.absolute(0x303), 1);
                let left = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value) & 255, 1);
                0xae9d
            }
            0xae9f => {
                w.dp_increment(0x4c, 1, false);
                0xaea1
            }
            0xaea9 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                0xaeaa
            }
            0xaeaa => {
                w.drain(16);
                let value = w.read(w.absolute(0xadbe) + u32::from(w.cpu.y), 1);
                w.load(value, 1);
                0xaead
            }
            0xaead => {
                w.dp_store(0x4c, w.cpu.a & 255, 1);
                0xaeaf
            }
            0xaeb5 => {
                w.drain(6);
                let value = w.cpu.a.wrapping_sub(1) & 255;
                w.load(value, 1);
                0xaeb6
            }
            0xaeb6 => {
                w.drain(24);
                w.write(0x7ef36e, w.cpu.a & 255, 1);
                0xaeba
            }
            0xae87 | 0xaebf => {
                w.rts();
                assert_eq!(w.cpu.pc, 0x81e2);
                self.tail = Some(GroundSwordCapeTail::Movement);
                w.cpu.pc
            }
            _ => unreachable!("native sword/cape return {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
