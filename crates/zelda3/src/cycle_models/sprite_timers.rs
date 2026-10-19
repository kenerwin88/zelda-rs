//! Sprite_ExecuteSingle_ entry and timer/OAM call, with a retained recoil callee.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
    sprite_slot::{SlotChild, SpriteSlot},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpriteTimersTail {
    Returned,
    Recoil(super::sprite_recoil::RecoilTail),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpriteTimers {
    state: u8,
    slot: Option<SpriteSlot>,
    message: Option<MessageMinimal>,
    recoil: Option<super::sprite_recoil::SpriteRecoil>,
    tail: Option<SpriteTimersTail>,
}
impl SpriteTimers {
    pub(crate) fn cached() -> Self {
        Self {
            state: 0,
            slot: None,
            message: None,
            recoil: None,
            tail: None,
        }
    }
    pub(crate) fn after_entry(state: u8) -> Self {
        Self::after_entry_with_slot(state, Default::default())
    }
    pub(crate) fn after_entry_with_slot(state: u8, slot: SpriteSlot) -> Self {
        assert_ne!(state, 0);
        Self {
            state,
            slot: Some(slot),
            message: None,
            recoil: None,
            tail: None,
        }
    }
    pub(crate) fn take_slot_body(&mut self) -> SpriteSlot {
        assert_eq!(self.tail, Some(SpriteTimersTail::Returned));
        self.slot.take().expect("retained timer slot").with_inactive_zelda()
    }
    pub(crate) fn state(&self) -> u8 {
        self.state
    }
    pub(crate) fn tail(&self) -> Option<SpriteTimersTail> {
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
        if let Some(recoil) = &mut self.recoil {
            let cycles = recoil.step(cpu, clock, bus);
            if let Some(tail) = recoil.tail() {
                if tail == super::sprite_recoil::RecoilTail::Returned {
                    assert_eq!((cpu.bank, cpu.pc), (6, 0x8481));
                    self.slot.as_mut().unwrap().return_from_child(cpu);
                    self.recoil = None;
                } else {
                    self.tail = Some(SpriteTimersTail::Recoil(tail));
                }
            }
            return cycles;
        }
        if let Some(message) = &mut self.message {
            let cycles = message.step(cpu, clock, bus);
            if message.complete {
                assert_eq!((cpu.bank, cpu.pc), (6, 0x8479));
                self.slot.as_mut().unwrap().return_from_child(cpu);
                self.message = None;
            }
            return cycles;
        }
        if let Some(work) = &mut self.slot {
            let cycles = work.step(cpu, clock, bus);
            self.tail = match work.child() {
                Some(SlotChild::AgahnimBalls) => {
                    self.message = Some(MessageMinimal::default());
                    None
                }
                Some(SlotChild::Recoil) => {
                    self.recoil = Some(Default::default());
                    None
                }
                Some(child) => unreachable!("timer helper {child:?}"),
                None if (cpu.bank, cpu.pc) == (6, 0x84eb) => Some(SpriteTimersTail::Returned),
                None => None,
            };
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.d, cpu.p & 0x30), (6, 0, 0x30));
        assert!(cpu.x < 16);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = w.cpu.pc;
        w.fetch();
        w.cpu.pc = match pc {
            0x84da | 0x84db => {
                w.drain(6);
                w.push(
                    u16::from(if pc == 0x84da { w.cpu.db } else { w.cpu.bank }),
                    1,
                );
                pc + 1
            }
            0x84dc => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0x84dd => {
                w.operand_then_internal(2);
                w.push(0x84df, 2);
                0x84e2
            }
            0x84e2 => {
                w.drain(16);
                let value = w.read(w.absolute(0xdd0 + w.cpu.x), 1);
                w.load(value, 1);
                self.state = value as u8;
                assert_ne!(self.state, 0);
                pc + 3
            }
            0x84e5 => {
                assert_ne!(self.state, 0);
                w.branch(false);
                self.slot = Some(Default::default());
                0x84e7
            }
            _ => unreachable!("cached sprite wrapper {pc:04x}"),
        };
        Some(w.cycles)
    }
}

/// Sprite_ShowMessageMinimal: fixed native instructions, including real RTL.
#[derive(Clone, Debug, PartialEq, Eq)]
struct MessageMinimal {
    pc: u16,
    complete: bool,
}
impl Default for MessageMinimal {
    fn default() -> Self {
        Self {
            pc: 0xfa8e,
            complete: false,
        }
    }
}
impl MessageMinimal {
    fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x30),
            (5, self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xfa8e | 0xfa91 | 0xfa9a => {
                w.drain(16);
                let (address, value) = match self.pc {
                    0xfa8e => (0x223, 0),
                    0xfa91 => (0x1cd8, 0),
                    _ => (0x10c, w.cpu.a & 255),
                };
                w.write(w.absolute(address), value, 1);
                self.pc + 3
            }
            0xfa94 | 0xfa9d => {
                w.immediate(if self.pc == 0xfa94 { 2 } else { 14 }, 1);
                self.pc + 2
            }
            0xfa96 | 0xfa9f => {
                w.dp_store(
                    if self.pc == 0xfa96 { 0x11 } else { 0x10 },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0xfa98 => {
                let value = w.dp_read(0x10, 1);
                w.load(value, 1);
                0xfa9a
            }
            0xfaa1 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("message minimal continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], slot: usize, case: usize) {
    r[0xee] = (case / 4 % 4) as u8;
    r[0xf20 + slot] = (case / 8 % 4) as u8;
    r[0x11] = [0, 1, 14, 17][case / 16 % 4];
    r[0xfc1] = [0, 128][case / 64 % 2];
    r[0xfb3] = (case / 128 % 2) as u8;
    r[0xe40 + slot] = (case * 7) as u8;
    r[0xe20 + slot] = [0x73, 0x7a][case / 256 % 2];
    r[0xfff] = [0, 1][case / 512 % 2];
    r[0xe50 + slot] = [0, 10, 20, 255][case / 1024 % 4];
    r[0xce2 + slot] = [0, 10, 20, 250, 251, 255][case / 2048 % 6];
    for (i, a) in [0xdf0, 0xe00, 0xe10, 0xee0, 0xf10].into_iter().enumerate() {
        r[a + slot] = [0, 1, 128, 255][(case + i) % 4];
    }
    r[0xef0 + slot] = [0, 1, 24, 31, 128, 152, 159, 255][case / 2 % 8];
    r[0xfe0..0xff8].fill(0);
    r[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
    r[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_timer_return_matches_source_despite_module_change() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..1024usize {
            let k = case % 16;
            let st = [9, 10, 11][case / 16 % 3];
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, byte) in machine.ram.iter_mut().enumerate() {
                *byte = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, k, case);
            machine.ram[0x10] = 7;
            machine.ram[0x11] = 0;
            machine.ram[0xfc1] = 0;
            machine.ram[0xfff] = 0;
            machine.ram[0xdd0 + k] = st;
            machine.ram[0xe20 + k] = 0x7a;
            machine.ram[0xe50 + k] = 0;
            machine.ram[0xce2 + k] = [0, 10, 250, 251, 254, 255][case / 48 % 6];
            machine.ram[0xef0 + k] = 31;
            machine.ram[0xf10 + k] = [0, 1, 128, 255][case / 288 % 4];
            let mut cpu = NativeNmiContext {
                a: 0xab00 | u16::from(st),
                x: k as u16,
                y: 7,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x84e7,
                sp: 0x1ee,
                p: 0x30 | (case as u8 & 0xc3),
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
            let initial = ram.clone();
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(76, 1200),
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
            let mut work = SpriteTimers::after_entry(st);
            while work.tail().is_none() {
                let pc = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
                visited.insert(pc);
                super::super::native_work::compare_instruction(
                    &format!("message timer return case={case} pc={pc:06x}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                instructions += 1;
            }
            assert_eq!(work.tail(), Some(SpriteTimersTail::Returned));
            assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (6, 0x84eb, 0x1ed, 6));
            assert_eq!(ram[0x1ee], st);
            assert_eq!((ram[0x10], ram[0x11], ram[0x10c]), (14, 2, 7));
            assert_eq!(ram[0xef0 + k], 30);
            assert_eq!(ram[0xf10 + k], initial[0xf10 + k].saturating_sub(1));
            assert!(work
                .step(&mut cpu, &mut clock, |_, _| panic!(
                    "completed timer must not access the bus"
                ))
                .is_none());
        }
        for pc in [
            0x05fa8e, 0x05faa1, 0x068479, 0x06847a, 0x06847c, 0x068481, 0x068486, 0x068488,
            0x06848e, 0x068491, 0x06849c, 0x06849f, 0x0684a1, 0x0684a4, 0x0684aa, 0x0684b5,
            0x0684b8,
        ] {
            assert!(
                visited.contains(&pc),
                "post-message Source instruction {pc:06x}"
            );
        }
        println!("native message timer return:1024 cases, {instructions} Source instructions, {} PCs, actual RTL then RTS, module changes preserved",visited.len());
    }

    #[test]
    fn sprite_message_minimal_matches_source_through_actual_long_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        for case in 0..512usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, byte) in machine.ram.iter_mut().enumerate() {
                *byte = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            let sp = 0x1c0 + (case % 32) as u16;
            machine.ram[usize::from(sp + 1)..usize::from(sp + 4)].copy_from_slice(&[0x78, 0x84, 6]);
            let mut cpu = NativeNmiContext {
                a: (case as u16).wrapping_mul(131),
                x: (case % 16) as u16,
                y: (case % 256) as u16,
                d: 0,
                db: [0, 6][case % 2],
                bank: 5,
                pc: 0xfa8e,
                sp,
                p: 0x30 | (case as u8 & 0xc3),
            };
            let original = cpu.clone();
            let saved_module = machine.ram[0x10];
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
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(76, 1200),
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
            let mut work = MessageMinimal::default();
            let mut owned = 0;
            while !work.complete {
                let pc = cpu.pc;
                super::super::native_work::compare_instruction(
                    &format!("message case={case} pc={pc:04x}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        owned += work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                instructions += 1;
            }
            assert_eq!(owned, 244);
            assert_eq!(
                (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.x, cpu.y),
                (6, 0x8479, sp + 3, original.db, original.x, original.y)
            );
            assert_eq!(cpu.a, (original.a & 0xff00) | 14);
            assert_eq!(
                (ram[0x10], ram[0x11], ram[0x10c], ram[0x223], ram[0x1cd8]),
                (14, 2, saved_module, 0, 0)
            );
        }
        assert_eq!(instructions, 4608);
        println!("native message minimal:512 cases, {instructions} Source instructions, actual long return,244 cycles each");
    }
}
