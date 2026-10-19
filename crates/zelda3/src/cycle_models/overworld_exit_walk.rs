//! Module09's two exit walks on the caller's real CPU and clock. Camera and
//! map-scroll calls expose their actual near frames to their subsequent owners.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
    player_animation::PlayerAnimation,
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitWalkDirection {
    Down,
    Up,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitWalkChild {
    Camera,
    MapScroll,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OverworldExitWalk {
    pc: u16,
    animation: Option<PlayerAnimation>,
    child: Option<ExitWalkChild>,
    return_sp: u16,
    complete: bool,
}
impl OverworldExitWalk {
    pub(crate) fn new(direction: ExitWalkDirection) -> Self {
        Self {
            pc: match direction {
                ExitWalkDirection::Down => 0xac8f,
                ExitWalkDirection::Up => 0xacc2,
            },
            animation: None,
            child: None,
            return_sp: 0,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn child(&self) -> Option<ExitWalkChild> {
        self.child
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        let child = self.child.take().expect("exit walk has a real near call");
        let target = match child {
            ExitWalkChild::Camera => 0xacb9,
            ExitWalkChild::MapScroll => 0xacc1,
        };
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, target, self.return_sp));
        self.pc = target;
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
        if let Some(animation) = &mut self.animation {
            let cycles = animation.step(cpu, clock, bus);
            if animation.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, self.pc, self.return_sp));
                self.animation = None;
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (2, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xac8f | 0xacc2 => {
                let entry = if pc == 0xac8f { 0xe69d } else { 0xe6a6 };
                self.return_sp = w.cpu.sp;
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 7;
                w.cpu.pc = entry;
                self.animation = Some(if pc == 0xac8f {
                    PlayerAnimation::facing_down()
                } else {
                    PlayerAnimation::default()
                });
                pc + 4
            }
            0xac93 | 0xaca6 | 0xacc6 => {
                w.width(0x20, true);
                pc + 2
            }
            0xac9d | 0xacb0 | 0xacd0 => {
                w.width(0x20, false);
                pc + 2
            }
            0xac95 | 0xaca8 | 0xacc8 => {
                let value = w.dp_read(0x20, m);
                w.load(value, m);
                pc + 2
            }
            0xac97 | 0xacaa => {
                w.drain(6);
                w.cpu.p &= !CARRY;
                pc + 1
            }
            0xacca => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xac98 | 0xacab => {
                w.drain(8 * u32::from(m));
                w.adc(if pc == 0xac98 { 1 } else { 3 });
                pc + 3
            }
            0xaccb => {
                w.drain(8 * u32::from(m));
                w.sbc(1, m);
                pc + 3
            }
            0xac9b | 0xacae | 0xacce => {
                w.dp_store(0x20, w.cpu.a & mask, m);
                pc + 2
            }
            0xac9f | 0xacd2 => {
                w.drain(16);
                let address = w.absolute(0x069a);
                let value = w.read(address, m).wrapping_sub(1) & mask;
                w.drain(6);
                w.flags(value, m);
                w.write_high_low(address, value, m);
                pc + 3
            }
            0xaca2 | 0xacd5 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    if pc == 0xaca2 {
                        0xacc1
                    } else {
                        0xacd9
                    }
                } else {
                    pc + 2
                }
            }
            0xaca4 | 0xacd7 => {
                w.dp_store(0x11, 0, m);
                pc + 2
            }
            0xacb2 => {
                w.immediate(3, m);
                pc + 2
            }
            0xacb4 => {
                w.dp_store(0x30, w.cpu.a & mask, m);
                pc + 2
            }
            0xacb6 | 0xacbe => {
                self.return_sp = w.cpu.sp;
                w.operand_then_internal(2);
                w.push(pc + 2, 2);
                self.child = Some(if pc == 0xacb6 {
                    ExitWalkChild::Camera
                } else {
                    ExitWalkChild::MapScroll
                });
                if pc == 0xacb6 {
                    0xbb90
                } else {
                    0xf273
                }
            }
            0xacb9 => {
                w.drain(16);
                let value = w.read(w.absolute(0x0416), m);
                w.load(value, m);
                pc + 3
            }
            0xacbc => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0xacc1
                } else {
                    0xacbe
                }
            }
            0xacc1 | 0xacd9 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("exit walk PC {pc:04x}"),
        };
        if self.animation.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exit_walk_matches_source_animation_stack_and_countdown_branches() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut camera_calls = 0;
        let mut map_calls = 0;
        for direction in [ExitWalkDirection::Down, ExitWalkDirection::Up] {
            for countdown in [0, 1, 2, 255] {
                for case in 0..64usize {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    machine.ram[0x11] = match direction {
                        ExitWalkDirection::Down => 10,
                        ExitWalkDirection::Up => 11,
                    };
                    machine.ram[0x20..0x22]
                        .copy_from_slice(&[0u16, 1, 255, 65535][case % 4].to_le_bytes());
                    machine.ram[0x069a] = countdown;
                    machine.ram[0x0416] = [0, 8, 5][case % 3];
                    machine.ram[0x67] = [0, 4, 8, 12][case / 4 % 4];
                    machine.ram[0x2f] = (case / 16 % 4 * 2) as u8;
                    machine.ram[0x2e] = (case % 8) as u8;
                    machine.ram[0x2b] = (case % 4) as u8;
                    // Module09's actual $A47B JSR ($A40D,X) frame.
                    machine.ram[0x1fb..0x1fd].copy_from_slice(&[0x7d, 0xa4]);
                    machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
                    let mut cpu = NativeNmiContext {
                        a: 0x1200,
                        x: 3,
                        y: 7,
                        d: 0,
                        db: [0, 2, 0x80, 0x82][case / 16 % 4],
                        bank: 2,
                        pc: match direction {
                            ExitWalkDirection::Down => 0xac8f,
                            ExitWalkDirection::Up => 0xacc2,
                        },
                        sp: 0x1fa,
                        p: 0x30 | [0, 1, 0x41, 0xc3][case % 4],
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
                    let mut work = OverworldExitWalk::new(direction);
                    for _ in 0..5000 {
                        if work.complete() || work.child().is_some() {
                            break;
                        }
                        super::super::native_work::compare_instruction_with_devices(
                            &format!("exit walk {direction:?} countdown={countdown} case={case} pc={:02x}:{:04x}",cpu.bank,cpu.pc),
                            &mut cpu,&mut clock,&mut source,&mut ram,&rom,None,
                            |_,access| match access {
                                super::super::intro_clear::Access::Read(address,width) if address&0x8000!=0&&address<0x7e0000=> {
                                    let read=|a|super::super::player_animation::table_byte(a).unwrap();
                                    Some(if width==1 {u16::from(read(address))} else {u16::from_le_bytes([read(address),read(address+1)])})
                                }
                                _=>None,
                            },
                            |cpu,clock,bus| {work.step(cpu,clock,bus).unwrap();});
                        work = work.clone();
                        instructions += 1;
                    }
                    assert_eq!(ram, source.snes().ram);
                    if direction == ExitWalkDirection::Down && countdown == 1 {
                        camera_calls += 1;
                        assert_eq!(work.child(), Some(ExitWalkChild::Camera));
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0xbb90, 0x1f8));
                        let mut camera = super::super::overworld_camera::OverworldCamera::default();
                        for _ in 0..10000 {
                            if camera.complete() {
                                break;
                            }
                            super::super::native_work::compare_instruction_with_devices(
                                &format!("exit walk camera case={case} pc={:04x}", cpu.pc),
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
                                            super::super::overworld_camera::table_byte(a).unwrap()
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
                                    camera.step(cpu, clock, bus).unwrap();
                                },
                            );
                            camera = camera.clone();
                            instructions += 1;
                        }
                        assert!(camera.complete());
                        work.return_from_child(&cpu);
                        assert_eq!(work.child(), None);
                        assert!(!work.complete());
                        for _ in 0..10 {
                            if work.complete() || work.child().is_some() {
                                break;
                            }
                            super::super::native_work::compare_instruction(
                                &format!("exit walk camera return case={case} pc={:04x}", cpu.pc),
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
                    }
                    assert_eq!(ram, source.snes().ram);
                    if work.child() == Some(ExitWalkChild::MapScroll) {
                        map_calls += 1;
                        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0xf273, 0x1f8));
                        let mut map =
                            super::super::overworld_map_scroll::OverworldMapScroll::default();
                        let mut stripes = None::<super::super::overworld_stripes::OverworldStripes>;
                        for _ in 0..10000 {
                            if map.complete() {
                                break;
                            }
                            if stripes.is_none() {
                                if let Some(child) = map.child() {
                                    stripes = Some(
                                        super::super::overworld_stripes::OverworldStripes::new(
                                            child, &cpu,
                                        ),
                                    );
                                }
                            }
                            super::super::native_work::compare_instruction_with_devices(
                                &format!("exit walk map case={case} pc={:04x}", cpu.pc),
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
                                            super::super::overworld_map_scroll::dispatch_byte(a)
                                                .or_else(|| {
                                                    super::super::overworld_stripes::table_byte(
                                                        a,
                                                        &rom[0x78000..0x80000],
                                                    )
                                                })
                                                .or_else(|| {
                                                    a.checked_sub(0x02f88d).and_then(|offset| {
                                                        rom[0x1788d..0x1790d]
                                                            .get(offset as usize)
                                                            .copied()
                                                    })
                                                })
                                                .expect("only immutable map tables")
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
                                    if let Some(child) = &mut stripes {
                                        child.step(cpu, clock, bus).unwrap();
                                    } else {
                                        map.step(cpu, clock, bus).unwrap();
                                    }
                                },
                            );
                            if let Some(child) = &mut stripes {
                                *child = child.clone();
                                if child.complete() {
                                    map.return_from_child(&cpu);
                                    stripes = None;
                                }
                            }
                            map = map.clone();
                            instructions += 1;
                        }
                        assert!(map.complete());
                        work.return_from_child(&cpu);
                        super::super::native_work::compare_instruction(
                            &format!("exit walk final RTS case={case}"),
                            &mut cpu,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        instructions += 1;
                    }
                    assert!(work.complete());
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0xa47e, 0x1fc));
                    assert_eq!(ram, source.snes().ram);
                }
            }
        }
        assert_eq!(map_calls, 42);
        eprintln!("512 complete native exit walks, {instructions} Source instruction boundaries, {camera_calls} native camera calls, {map_calls} complete native map-scroll calls; no behavioral ROM reads or Source child state imports");
    }
}
