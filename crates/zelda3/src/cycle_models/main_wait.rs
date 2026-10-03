//! Native continuation of ZeldaRunGameLoop's `LDA $12 : BEQ` wait.
//!
//! NMI can interrupt either instruction. Returning from NMI must preserve
//! the branch's already-read latch, rather than testing the new latch early.
//! Instruction costs are slow-ROM fetches, WRAM accesses, and internal work;
//! the caller's timeline owns refresh, HDMA, and interrupt acceptance.

/// ZeldaRunGameLoop's own work after the wait exits: CLI/BRA, INC $1A,
/// three call instructions, STZ $12, and BRA back to the wait. Callee work
/// belongs to its own native owner. Each BRA has two slow-ROM fetches and
/// one internal cycle (8 + 8 + 6); unlike a conditional branch it has no
/// additional taken-branch cycle.
#[cfg(test)]
pub(crate) const MAIN_LOOP_OWN_MASTER_CYCLES: u32 = 14 + 22 + 38 + 46 + 62 + 46 + 24 + 22;

/// Caller work must follow its callees in execution order. In particular,
/// entering Module_MainRouting does not execute the later sprite-preparation
/// call or clear the NMI latch. A suspended module retains that caller debt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum MainLoopCallerPhase {
    #[default]
    Waiting,
    ModuleRouting,
    SpritePreparation,
    ReturnBranch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CallerReturnStep {
    pub master_cycles: u32,
    pub clear_latch: bool,
}

impl MainLoopCallerPhase {
    pub(crate) fn enter_sprite_preparation(&mut self) -> u32 {
        if *self == Self::ModuleRouting {
            *self = Self::SpritePreparation;
            46 // $805A JSR NMI_PrepareSprites
        } else {
            0 // A resumed callee or a call outside a tracked main iteration.
        }
    }

    pub(crate) fn step_return_to_wait(&mut self) -> Option<CallerReturnStep> {
        match *self {
            Self::SpritePreparation => {
                *self = Self::ReturnBranch;
                Some(CallerReturnStep {
                    master_cycles: 24,
                    clear_latch: true,
                })
            }
            Self::ReturnBranch => {
                *self = Self::Waiting;
                Some(CallerReturnStep {
                    master_cycles: 22,
                    clear_latch: false,
                })
            }
            Self::Waiting => None,
            Self::ModuleRouting => {
                panic!("main caller returned before its sprite-preparation call")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum MainWait {
    #[default]
    LoadLatch,
    TestLatch {
        value: u8,
    },
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MainWaitStep {
    pub(crate) master_cycles: u32,
    pub(crate) loaded_latch: Option<u8>,
}

pub(crate) enum WaitAccess {
    ReadLatch,
    Vblank { field: u64, at: u64 },
}

impl MainWait {
    /// Retire the live bus transaction and preserve the loaded branch operand
    /// across host returns and NMI. Fetch leaves a due event for the operand
    /// access, matching the source CPU's instruction boundary.
    pub(crate) fn step_clocked(
        &mut self,
        cpu: &mut super::nmi_stack::NativeNmiContext,
        clock: &mut snes::CpuMasterTimeline,
        mut bus: impl FnMut(u64, WaitAccess) -> u8,
    ) -> u32 {
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 0x30),
            (0, self.next_pc() as u16, 0, 0x30));
        clock.advance_synchronous_pcbase_opcode_fetch(8);
        let drain = |clock: &mut snes::CpuMasterTimeline, cycles, bus: &mut dyn FnMut(u64, WaitAccess) -> u8| {
            clock.advance_synchronous_after_semantics_with(cycles, |event, _| {
                if let snes::cpu_timeline::CpuSynchronousTimelineEvent::HMax {
                    completed_field_index, completed_scanline: 224, event_timestamp, ..
                } = event {
                    let at = event_timestamp.master_cycles();
                    bus(at, WaitAccess::Vblank { field: completed_field_index, at });
                }
                Ok::<u32, String>(0)
            }).expect("native main wait clock");
        };
        drain(clock, 8, &mut bus);
        let value = match *self {
            Self::LoadLatch => {
                let value = bus(clock.clock_master_cycles(), WaitAccess::ReadLatch);
                cpu.load_a_byte(value);
                drain(clock, 8, &mut bus);
                value
            }
            Self::TestLatch { value } => {
                assert_eq!(cpu.p & super::nmi_stack::ZERO != 0, value == 0);
                if value == 0 { drain(clock, 6, &mut bus); }
                value
            }
            Self::Complete => panic!("main wait already complete"),
        };
        let cycles = self.step(value).master_cycles;
        cpu.pc = self.next_pc() as u16;
        cycles
    }

    pub(crate) fn next_pc(self) -> u32 {
        match self {
            Self::LoadLatch => 0x00_8034,
            Self::TestLatch { .. } => 0x00_8036,
            Self::Complete => 0x00_8038,
        }
    }

    /// Retire one indivisible instruction. In particular, a suspended BEQ
    /// consumes its saved load result even if NMI has since changed `$12`.
    pub(crate) fn step(&mut self, current_latch: u8) -> MainWaitStep {
        match *self {
            Self::LoadLatch => {
                *self = Self::TestLatch {
                    value: current_latch,
                };
                MainWaitStep {
                    master_cycles: 24,
                    loaded_latch: Some(current_latch),
                }
            }
            Self::TestLatch { value } => {
                *self = if value == 0 {
                    Self::LoadLatch
                } else {
                    Self::Complete
                };
                MainWaitStep {
                    master_cycles: if value == 0 { 22 } else { 16 },
                    loaded_latch: None,
                }
            }
            Self::Complete => panic!("main wait already returned to its caller"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};

    #[test]
    fn caller_retains_future_work_until_callee_return() {
        let mut caller = MainLoopCallerPhase::ModuleRouting;
        let prefix = 14 + 22 + 38 + 46 + 62;
        assert_eq!(MAIN_LOOP_OWN_MASTER_CYCLES - prefix, 92);
        let call = caller.enter_sprite_preparation();
        assert_eq!(call, 46);
        // Suspension and re-entry cannot execute the JSR twice.
        assert_eq!(caller.enter_sprite_preparation(), 0);
        let store = caller.step_return_to_wait().unwrap();
        assert_eq!(caller, MainLoopCallerPhase::ReturnBranch);
        assert_eq!(
            store,
            CallerReturnStep {
                master_cycles: 24,
                clear_latch: true
            }
        );
        let branch = caller.step_return_to_wait().unwrap();
        assert_eq!(
            branch,
            CallerReturnStep {
                master_cycles: 22,
                clear_latch: false
            }
        );
        assert_eq!(
            prefix + call + store.master_cycles + branch.master_cycles,
            MAIN_LOOP_OWN_MASTER_CYCLES
        );
        assert_eq!(caller, MainLoopCallerPhase::Waiting);
        assert_eq!(caller.step_return_to_wait(), None);
    }

    #[test]
    fn main_loop_own_work_matches_rom_instructions() {
        let path = std::env::var_os("ZELDA3_ROM")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")
            });
        let Ok(mut rom) = std::fs::read(path) else {
            return;
        };
        if rom.len() % 0x400 == 0x200 {
            rom.drain(..0x200);
        }
        // Enter each caller instruction independently so the three callee
        // bodies cannot contaminate the caller's work measurement.
        for frame in [0, 0xff] {
            let mut total = 0;
            for (entry_pc, next_pc) in [
                (0x00_8038, 0x00_8039),
                (0x00_8039, 0x00_8051),
                (0x00_8051, 0x00_8053),
                (0x00_8053, 0x00_841e),
                (0x00_8056, 0x00_80b5),
                (0x00_805a, 0x00_85fc),
                (0x00_805d, 0x00_805f),
                (0x00_805f, 0x00_8034),
            ] {
                let mut ram = vec![0; 0x20000];
                ram[0x12] = 1;
                ram[0x1a] = frame;
                let checkpoint = RomCpuCheckpoint {
                    entry_pc,
                    stop_pc: next_pc,
                    a: 0xabcd,
                    x: 0x31,
                    y: 0x42,
                    sp: 0x01ff,
                    dp: 0,
                    db: 0,
                    carry: true,
                    zero: false,
                    overflow: true,
                    negative: true,
                    interrupt_disable: true,
                    decimal: false,
                    accumulator_is_8_bit: true,
                    index_is_8_bit: true,
                    emulation: false,
                    waiting: false,
                    stack_address: 0,
                    stack_bytes: &[],
                };
                let mut run = RomCpuTimingRun::new(
                    &rom,
                    &ram,
                    &[0; 0x2000],
                    &snes::PpuState::default(),
                    &snes::DmaState::default(),
                    [0; 4],
                    checkpoint,
                )
                .unwrap();
                let step = run.step();
                assert_eq!(run.pc(), next_pc, "caller instruction {entry_pc:06x}");
                if matches!(entry_pc, 0x00_8039 | 0x00_805f) {
                    assert_eq!(step.master_cycles, 22, "BRA {entry_pc:06x}");
                }
                total += step.master_cycles;
            }
            assert_eq!(total, MAIN_LOOP_OWN_MASTER_CYCLES);
        }
    }

    #[test]
    fn held_branch_observes_the_pre_interrupt_load_until_the_next_lda() {
        let mut wait = MainWait::default();
        assert_eq!(wait.step(0).loaded_latch, Some(0));
        // NMI sets $12 while the game thread is suspended at BEQ.
        assert_eq!(wait.step(1).master_cycles, 22);
        assert_eq!(wait, MainWait::LoadLatch);
        assert_eq!(wait.step(1).loaded_latch, Some(1));
        assert_eq!(wait.step(0).master_cycles, 16);
        assert_eq!(wait, MainWait::Complete);
    }

    #[test]
    fn wait_instruction_state_and_cost_match_rom_for_every_latch_value() {
        let path = std::env::var_os("ZELDA3_ROM")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")
            });
        let Ok(mut rom) = std::fs::read(path) else {
            return;
        };
        if rom.len() % 0x400 == 0x200 {
            rom.drain(..0x200);
        }
        for latch in 0..=u8::MAX {
            let mut ram = vec![0; 0x20000];
            ram[0x12] = latch;
            let checkpoint = RomCpuCheckpoint {
                entry_pc: 0x00_8034,
                stop_pc: 0x00_8038,
                a: 0xabcd,
                x: 0x31,
                y: 0x42,
                sp: 0x01ff,
                dp: 0,
                db: 0,
                carry: true,
                zero: false,
                overflow: true,
                negative: true,
                interrupt_disable: false,
                decimal: false,
                accumulator_is_8_bit: true,
                index_is_8_bit: true,
                emulation: false,
                waiting: false,
                stack_address: 0,
                stack_bytes: &[],
            };
            let mut run = RomCpuTimingRun::new(
                &rom,
                &ram,
                &[0; 0x2000],
                &snes::PpuState::default(),
                &snes::DmaState::default(),
                [0; 4],
                checkpoint,
            )
            .unwrap();
            let mut wait = MainWait::default();
            for _ in 0..2 {
                let native = wait.step(latch);
                assert_eq!(
                    run.step().master_cycles,
                    native.master_cycles,
                    "latch {latch}"
                );
                assert_eq!(run.pc(), wait.next_pc(), "latch {latch}");
                assert_eq!(run.accumulator(), 0xab00 | u16::from(latch));
            }
        }
    }
}
