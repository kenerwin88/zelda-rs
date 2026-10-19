//! Camera entry and page-delta checks on the ground caller's live stack.
//! Scrolling and door children retain explicit semantic handoffs.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroundCameraTail {
    Returned,
    Doors,
    Scroll,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GroundCameraReturn {
    pc: Option<u16>,
    tail: Option<GroundCameraTail>,
}
impl GroundCameraReturn {
    pub(crate) fn tail(&self) -> Option<GroundCameraTail> {
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
        let pc = self.pc.unwrap_or(0xe8f0);
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x38),
            (7, pc, 0, 7, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0xe8f0 | 0xe8f4 | 0xe9d6 | 0xe9dd | 0xe9e4 | 0xe9f4 => {
                let addr = match pc {
                    0xe8f0 => 0x1b,
                    0xe8f4 => 0x6c,
                    0xe9d6 => 0x21,
                    0xe9dd => 0x23,
                    0xe9e4 => 0x69,
                    _ => 0x68,
                };
                let v = w.dp_read(addr, 1);
                w.load(v, 1);
                pc + 2
            }
            0xe8f2 | 0xe8f6 | 0xe9e6 | 0xe9f6 => {
                let target = match pc {
                    0xe8f2 => 0xe900,
                    0xe8f6 => 0xe8fc,
                    0xe9e6 => 0xe9f4,
                    _ => 0xea04,
                };
                if w.branch(w.cpu.p & ZERO != 0) {
                    target
                } else {
                    pc + 2
                }
            }
            0xe8f8 => {
                w.drain(24);
                self.tail = Some(GroundCameraTail::Doors);
                0xe901
            }
            0xe8fc => {
                w.operand_then_internal(3);
                w.push(w.cpu.bank as u16, 1);
                w.push(pc + 3, 2);
                0xe9d3
            }
            0xe9d3 | 0xe9d4 => {
                w.drain(6);
                w.push(
                    u16::from(if pc == 0xe9d3 { w.cpu.db } else { w.cpu.bank }),
                    1,
                );
                pc + 1
            }
            0xe9d5 | 0xea04 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(w.cpu.db as u16, 1);
                pc + 1
            }
            0xe9d8 | 0xe9df => {
                w.drain(6);
                w.cpu.p |= 1;
                pc + 1
            }
            0xe9d9 | 0xe9e0 => {
                let v = w.dp_read(if pc == 0xe9d9 { 0x40 } else { 0x41 }, 1);
                w.sbc(v, 1);
                pc + 2
            }
            0xe9db | 0xe9e2 => {
                w.dp_store(if pc == 0xe9db { 0x68 } else { 0x69 }, w.cpu.a & 255, 1);
                pc + 2
            }
            0xe9e8 | 0xe9f8 => unreachable!("scroll handoff must precede its instruction"),
            0xea05 => {
                w.drain(12);
                w.cpu.pc = w.pull(2).wrapping_add(1);
                w.cpu.bank = w.pull(1) as u8;
                w.cpu.pc
            }
            0xe900 => {
                w.rts();
                w.cpu.pc
            }
            0x82d8 => {
                w.clear_carry();
                pc + 1
            }
            0x82d9 => {
                w.rts();
                self.tail = Some(GroundCameraTail::Returned);
                w.cpu.pc
            }
            _ => panic!("ground camera PC {pc:04x}"),
        };
        if matches!(next, 0xe9e8 | 0xe9f8) {
            self.tail = Some(GroundCameraTail::Scroll);
        }
        self.pc = Some(next);
        w.cpu.pc = next;
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
    fn ground_camera_return_matches_source_all_page_delta_handoffs() {
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
                r[0x1b] = u8::from(variant & 1 != 0);
                r[0x6c] = (variant / 2) % 3;
                r[0x21] = timer;
                r[0x23] = timer.wrapping_add(variant);
                r[0x40] = timer.wrapping_add((variant >> 3) & 3);
                r[0x41] = timer.wrapping_add(variant).wrapping_sub((variant >> 5) & 1);
                // Independent zero, positive and negative X deltas.
                r[0x41] = r[0x23].wrapping_add([0, 1, 255][(variant as usize / 4) % 3]);
                r[0x1f3..0x1f7].copy_from_slice(&[0xd7, 0x82, 0x1b, 0x80]);
                let pc = 0xe8f0;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: variant as u16,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: 0x1f2,
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
                let mut work = GroundCameraReturn::default();
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
                let tail=work.tail().unwrap();
                tails[match tail {GroundCameraTail::Returned=>0,GroundCameraTail::Doors=>1,GroundCameraTail::Scroll=>2}]+=1;
                if tail==GroundCameraTail::Returned {assert_eq!((cpu.pc,cpu.sp,cpu.p&1),(0x801c,0x1f6,0));}
                assert_eq!((cpu.bank, cpu.db), (7, 7));
            }
        }
        assert!(tails.iter().all(|n|*n>0),"missing handoff: {tails:?}");
        println!("ground camera return: 8192 cases, {instructions} source instructions, outdoor return, doors, signed page deltas and ground return");
    }
}
