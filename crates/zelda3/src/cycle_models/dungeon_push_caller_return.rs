//! Dungeon movement return, object-index reset and push-block call.
use super::{
    dungeon_push_blocks::{DungeonPushBlocks, PushBlockChild},
    native_work::{Work, WorkBus},
    nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PushCallerTail {
    /// The real JSL has retired; the handler's effects remain pending.
    Handler,
    Returned,
    RoomTransition(u32),
}

#[cfg(test)]
mod tests {
    use super::super::{dungeon_push_blocks, native_work::compare_instruction};
    use super::*;

    #[test]
    fn dungeon_push_caller_return_matches_source_full_handler_and_handoffs() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 3];
        for mode in 0..7 {
            for case in 0..256usize {
                for deferred in [false, true] {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    for (i, v) in machine.ram.iter_mut().enumerate() {
                        *v = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
                    }
                    dungeon_push_blocks::seed_case(&mut machine.ram, case, mode);
                    if mode == 6 {
                        // A horizontal doorway edge reaches the real room
                        // transition child without colliding with the block.
                        machine.ram[0x1b] = 1;
                        machine.ram[0x6c] = 2;
                        machine.ram[0x26] = 1;
                        machine.ram[0x67] = 1;
                        machine.ram[0x20..0x24].copy_from_slice(&[0, 0, 0xeb, 1]);
                    }
                    machine.ram[0x42c] = case as u8;
                    if deferred {
                        machine.ram[0x42d] = case as u8;
                    }
                    machine.ram[0x1fb..0x1fd].copy_from_slice(&[0xae, 0x87]);
                    let mut cpu = NativeNmiContext {
                        a: 0xab01,
                        x: 3,
                        y: 0,
                        d: 0,
                        db: 0,
                        bank: 2,
                        pc: 0x894b,
                        sp: 0x1fa,
                        p: 0x30 | (case as u8 & 0xc3),
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let ram_before = machine.ram.clone();
                    let mut ram = ram_before.clone();
                    let timeline = snes::CpuMasterTimeline::at_raster(
                        4586,
                        snes::CpuRasterPosition::new(224, 1300),
                        snes::CpuBusWorkload::default(),
                        snes::CpuFieldTiming::non_interlace(case & 1 != 0),
                    );
                    let mut source = snes::RomCpuTimingProbe::new(
                        machine,
                        timeline,
                        snes::SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    let mut clock = source.timeline().clone();
                    let mut work = DungeonPushCallerReturn::with_handler_deferred(deferred);
                    for _ in 0..20000 {
                        if work.tail().is_some() {
                            break;
                        }
                        compare_instruction(&format!("push caller mode={mode} case={case} deferred={deferred} pc={:02x}:{:04x}",cpu.bank,cpu.pc),&mut cpu,&mut clock,&mut source,&mut ram,&rom,|cpu,clock,bus|{work.step(cpu,clock,bus).unwrap();});
                        work = work.clone();
                        count += 1;
                    }
                    assert_eq!(ram, source.snes().ram);
                    match work.tail().expect("push caller did not reach a boundary") {
                        PushCallerTail::Handler => {
                            assert!(deferred);
                            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (1, 0xd81b, 0x1f9));
                            assert_eq!(&ram[0x42c..0x42e], &[0, ram_before[0x42d]]);
                            assert_eq!(&ram[0x500..0x600], &ram_before[0x500..0x600]);
                            tails[0] += 1;
                        }
                        PushCallerTail::Returned => {
                            assert!(!deferred);
                            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87b6, 0x1fc));
                            tails[1] += 1;
                        }
                        PushCallerTail::RoomTransition(target) => {
                            assert!(!deferred);
                            assert_eq!(mode, 6);
                            assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), target);
                            tails[2] += 1;
                        }
                    }
                }
            }
        }
        assert!(tails.iter().all(|v| *v > 0), "missing handoff: {tails:?}");
        println!("dungeon push caller return: 3584 cases, {count} source instructions, full block effects and explicit handler/room-transition boundaries {tails:?}");
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DungeonPushCallerReturn {
    pc: Option<u16>,
    blocks: Option<DungeonPushBlocks>,
    defer_handler: bool,
    tail: Option<PushCallerTail>,
}

impl DungeonPushCallerReturn {
    pub(crate) fn with_handler_deferred(defer_handler: bool) -> Self {
        Self {
            defer_handler,
            ..Self::default()
        }
    }
    pub(crate) fn tail(&self) -> Option<PushCallerTail> {
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
        if let Some(blocks) = &mut self.blocks {
            let cycles = blocks.step(cpu, clock, bus);
            if blocks.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x87b6, 0x1fc));
                self.tail = Some(PushCallerTail::Returned);
            } else if let Some(PushBlockChild::RoomTransition(target)) = blocks.child() {
                self.tail = Some(PushCallerTail::RoomTransition(target));
            }
            return cycles;
        }
        let pc = self.pc.unwrap_or(0x894b);
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x38),
            (2, pc, 0, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match pc {
            0x894b => {
                w.drain(12);
                let target = w.pull(2).wrapping_add(1);
                w.drain(6);
                assert_eq!((target, w.cpu.sp), (0x87af, 0x1fc));
                target
            }
            0x87af => {
                w.drain(16);
                w.write(w.absolute(0x42c), 0, 1);
                0x87b2
            }
            0x87b2 => {
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(pc + 3, 2);
                w.cpu.bank = 1;
                if self.defer_handler {
                    self.tail = Some(PushCallerTail::Handler);
                } else {
                    self.blocks = Some(DungeonPushBlocks::default());
                }
                0xd81b
            }
            _ => panic!("dungeon push caller PC {pc:04x}"),
        };
        w.cpu.pc = next;
        self.pc = Some(next);
        Some(w.cycles)
    }
}
