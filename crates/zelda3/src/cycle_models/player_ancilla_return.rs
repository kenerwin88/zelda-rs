//! Player ancilla guards and descending slot scans on the retained live stack.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayerAncillaTail {
    Returned,
    Grave(u8),
    Somaria(u8),
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PlayerAncillaReturn {
    pc: Option<u16>,
    tail: Option<PlayerAncillaTail>,
}
impl PlayerAncillaReturn {
    pub(crate) fn tail(&self) -> Option<PlayerAncillaTail> {
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
        let pc = self.pc.unwrap_or(0x801c);
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38), (7, pc, 0, 0x30));
        if pc != 0x8020 {
            assert_eq!(cpu.db, 7);
        }
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0x801c => {
                w.item_call();
                0x8689
            }
            0x8689 => {
                let v = w.dp_read(0x1b, 1);
                w.load(v, 1);
                pc + 2
            }
            0x868d => {
                w.drain(16);
                let v = w.read(w.absolute(0x3e9), 1);
                w.load(v, 1);
                pc + 3
            }
            0x868b | 0x8690 | 0x8699 | 0x86a0 | 0x86a9 | 0x86b2 => {
                let (take, target) = match pc {
                    0x868b => (w.cpu.p & ZERO == 0, 0x86a2),
                    0x8690 => (w.cpu.p & ZERO != 0, 0x86a2),
                    0x8699 => (w.cpu.p & ZERO == 0, 0x869f),
                    0x86a9 => (w.cpu.p & ZERO == 0, 0x86b1),
                    0x86a0 => (w.cpu.p & NEGATIVE == 0, 0x8694),
                    _ => (w.cpu.p & NEGATIVE == 0, 0x86a4),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0x8692 | 0x86a2 => {
                w.drain(8);
                w.cpu.x = 4;
                w.flags(4, 1);
                pc + 2
            }
            0x8694 | 0x86a4 => {
                w.drain(16);
                let v = w.read(w.absolute(0xc4a) + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                pc + 3
            }
            0x8697 | 0x86a7 => {
                w.compare_immediate(if pc == 0x8697 { 0x24 } else { 0x2c }, 1);
                pc + 2
            }
            0x869f | 0x86b1 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0x869b | 0x86ab => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                self.tail = Some(if pc == 0x869b {
                    PlayerAncillaTail::Grave(w.cpu.x as u8)
                } else {
                    PlayerAncillaTail::Somaria(w.cpu.x as u8)
                });
                w.cpu.bank = 8;
                if pc == 0x869b {
                    0xed89
                } else {
                    0xe7e6
                }
            }
            0x86b4 => {
                w.rts();
                w.cpu.pc
            }
            0x801f => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(w.cpu.db as u16, 1);
                pc + 1
            }
            0x8020 => {
                w.drain(12);
                let ret = w.pull(2).wrapping_add(1);
                w.cpu.bank = w.pull(1) as u8;
                self.tail = Some(PlayerAncillaTail::Returned);
                ret
            }
            _ => panic!("player ancilla return PC {pc:04x}"),
        };
        self.pc = Some(next);
        w.cpu.pc = next;
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize, variant: usize) {
    r[0x1b] = (variant & 1) as u8;
    r[0x3e9] = ((variant >> 1) & 1) as u8;
    let mut pattern = case % 243;
    for slot in 0..5 {
        r[0xc4a + slot] = [0x20, 0x24, 0x2c][pattern % 3];
        pattern /= 3;
    }
    r[0x1f7..0x1fb].copy_from_slice(&[[0, 1, 127, 255][variant / 8 % 4], 0x4a, 0x89, 2]);
}
#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn player_ancilla_return_matches_source_slot_order_and_child_stack() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut tails = [0usize; 3];

        for timer in 0..=255u8 {
            for variant in 0..32u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7) ^ timer as usize) as u8;
                }
                seed_case(r, timer as usize, variant as usize);
                let pc = 0x801c;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: variant as u16,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: 0x1f6,
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
                let mut work = PlayerAncillaReturn::default();
                while work.tail().is_none() {
                    let label = format!(
                        "velocity controls={timer} variant={variant} pc={:04x}",
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
                            if let super::super::intro_clear::Access::Read(a, width) = access {
                                let byte =
                                    |address| super::super::player_animation::table_byte(address);
                                let low = byte(a)?;
                                Some(if width == 1 {
                                    u16::from(low)
                                } else {
                                    u16::from_le_bytes([low, byte(a + 1)?])
                                })
                            } else {
                                None
                            }
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                let tail = work.tail().unwrap();
                tails[match tail {
                    PlayerAncillaTail::Returned => 0,
                    PlayerAncillaTail::Grave(_) => 1,
                    PlayerAncillaTail::Somaria(_) => 2,
                }] += 1;
                match tail {
                    PlayerAncillaTail::Returned => assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.db),
                        (2, 0x894b, 0x1fa, [0, 1, 127, 255][variant as usize / 8 % 4])
                    ),
                    PlayerAncillaTail::Grave(slot) => assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.x),
                        (8, 0xed89, 0x1f1, slot as u16)
                    ),
                    PlayerAncillaTail::Somaria(slot) => assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.x),
                        (8, 0xe7e6, 0x1f1, slot as u16)
                    ),
                }
            }
        }
        assert!(tails.iter().all(|n| *n > 0), "missing handoff: {tails:?}");
        println!("player ancilla return: 8192 cases, {instructions} source instructions, grave gate, descending slot order, selected child stack and player far return");
    }
}
