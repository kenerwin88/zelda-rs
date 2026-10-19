//! The real Module07 return and main caller through its branch to the wait loop.
//! Each child retires its own stack frame. The saved scrolls, status flags and
//! synchronous clock remain live across all four owners.

use super::{
    dungeon_common::DungeonCommon,
    native_work::WorkBus,
    nmi_stack::NativeNmiContext,
    oam_pack::CallerPacking,
    sprite_dma::{MainSpriteTail, SourceWords},
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    super::dungeon_common::table_byte(address)
        .or_else(|| crate::zelda_rtl::link_dma_data_byte(address))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonMainReturn {
    common: DungeonCommon,
    packing: CallerPacking,
    sources: SourceWords,
    tail: MainSpriteTail,
    source_master_cycles: u64,
}

impl Default for DungeonMainReturn {
    fn default() -> Self {
        Self {
            common: DungeonCommon::after_sprites(),
            packing: CallerPacking::default(),
            sources: SourceWords::default(),
            tail: MainSpriteTail::default(),
            source_master_cycles: 0,
        }
    }
}

impl DungeonMainReturn {
    pub(crate) fn sprite_preparation_master_cycles(&self) -> u64 {
        self.packing.body_master_cycles()
            + self.source_master_cycles
            + self.tail.body_master_cycles()
    }
    pub(crate) fn complete(&self) -> bool {
        self.tail.complete()
    }

    pub(crate) fn routine(&self) -> u32 {
        if !self.common.complete() {
            self.common.routine()
        } else if self.packing.pending_call() {
            0x008034
        } else {
            0x0085fc
        }
    }

    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<(u64, u32)> {
        if self.complete() {
            return None;
        }
        let routine = self.routine();
        let cycles = if !self.common.complete() {
            let cycles = self.common.step(cpu, clock, bus);
            assert!(
                self.common.child().is_none(),
                "dungeon return owns every drawing child"
            );
            if self.common.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (0, 0x805a, 0x1ff));
            }
            cycles
        } else if !self.packing.complete() {
            self.packing.step_with_bus(cpu, clock, bus)
        } else if !self.sources.is_complete() {
            let cycles = self.sources.step_clocked(cpu, clock, bus);
            self.source_master_cycles += cycles.unwrap_or(0);
            cycles
        } else {
            return self.tail.step(cpu, clock, bus);
        };
        cycles.map(|cycles| (cycles, routine))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cycle_models::{
        intro_clear::Access,
        main_wait::{MainWait, WaitAccess},
        native_work::{compare_instruction, compare_instruction_with_devices},
    };

    #[test]
    fn dungeon_return_preserves_source_stack_bus_and_clock_to_main_wait() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut wait_instructions = 0;
        let mut table_reads = 0;
        for case in 0..32usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            let ram = &mut machine.ram;
            ram[0x1b] = 1;
            ram[0x5fc] = 1;
            ram[0x5fd] = 2;
            ram[0x500] = [0, 1, 2, 4][case % 4];
            ram[0x540..0x542].copy_from_slice(&0x280u16.to_le_bytes());
            ram[0x474] = (case % 4 * 2) as u8;
            ram[0x2f] = (case % 4 * 2) as u8;
            ram[0x11] = if case & 1 != 0 { 15 } else { 0 };
            ram[0x6c] = u8::from(case & 2 != 0);
            ram[0x454] = u8::from(case & 4 != 0);
            ram[0x428] = u8::from(case & 8 != 0);
            ram[0x478] = 2;
            ram[0x458] = (case & 1) as u8;
            // The Sprite_Main RTL has consumed its frame. Four 16-bit scrolls
            // and the actual main caller's far return remain above SP=$1f4.
            for (i, at) in [0x1f5, 0x1f7, 0x1f9, 0x1fb].into_iter().enumerate() {
                let word = (case as u16)
                    .wrapping_mul(997)
                    .wrapping_add((i * 71) as u16);
                ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
            }
            ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
            for (i, byte) in ram[0xa20..0xaa0].iter_mut().enumerate() {
                *byte = (i as u8).wrapping_mul(37) ^ (case as u8);
            }
            let mut cpu = NativeNmiContext {
                a: 0x1200,
                x: 3,
                y: 7,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x8842,
                sp: 0x1f4,
                p: 0x30 | [0, 1, 0x41, 0xc3][case % 4],
            };
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.pc = cpu.pc;
            machine.cpu.k = cpu.bank;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut ram = machine.ram.clone();
            let timeline = CpuMasterTimeline::at_raster(
                4586 + case as u64,
                snes::CpuRasterPosition::new(
                    [76, 224, 261, 170][case % 4],
                    [468, 1300, 1350, 0][case % 4],
                ),
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
            let mut work = DungeonMainReturn::default();
            for _ in 0..20000 {
                if work.complete() {
                    break;
                }
                compare_instruction_with_devices(
                    &format!(
                        "dungeon return case={case} pc={:02x}:{:04x}",
                        cpu.bank, cpu.pc
                    ),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    None,
                    |_, access| match access {
                        Access::Read(address, width)
                            if address & 0x8000 != 0 && address < 0x7e0000 =>
                        {
                            table_reads += 1;
                            let read = |address| {
                                table_byte(address).unwrap_or_else(|| {
                                    panic!("native dungeon data missing at {address:06x}")
                                })
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
                instructions += 1;
                work = work.clone();
            }
            assert!(work.complete(), "dungeon return case={case}");
            assert_eq!(
                (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.p & 0x30),
                (0, 0x8034, 0x1ff, 0, 0x30)
            );
            assert_eq!(ram[0x12], 0);
            assert_eq!(ram, source.snes().ram);
            // Keep the returned clock's event cursor. The deadline is the
            // next NMI acceptance boundary, never a reconstructed beam seed.
            let mut field = clock.field_index();
            let mut deadline =
                clock.master_cycles_at_raster(field, snes::CpuRasterPosition::new(225, 12));
            if deadline <= clock.clock_master_cycles() {
                field += 1;
                deadline =
                    clock.master_cycles_at_raster(field, snes::CpuRasterPosition::new(225, 12));
            }
            let mut wait = MainWait::default();
            for _ in 0..20000 {
                if clock.clock_master_cycles() >= deadline {
                    break;
                }
                compare_instruction(
                    &format!("dungeon main wait case={case} pc={:04x}", cpu.pc),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        wait.step_clocked(
                            cpu,
                            clock,
                            |at, access| match access {
                                WaitAccess::ReadLatch => bus(at, Access::Read(0x12, 1)) as u8,
                                WaitAccess::Vblank { .. } => 0,
                            },
                            |_, _| 0,
                        );
                    },
                );
                wait_instructions += 1;
            }
            assert!(
                clock.clock_master_cycles() >= deadline,
                "dungeon wait case={case}"
            );
            assert_eq!(clock.field_index(), field);
            assert!(matches!(cpu.pc, 0x8034 | 0x8036));
            assert_eq!(cpu.sp, 0x1ff);
            assert_eq!(ram, source.snes().ram);
        }
        assert!(table_reads > 0);
        eprintln!("32 complete dungeon returns: {instructions} return instructions, {wait_instructions} wait instructions, {table_reads} native table reads; every boundary matches Source");
    }
}
