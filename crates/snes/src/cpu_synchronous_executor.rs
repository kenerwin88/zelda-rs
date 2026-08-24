//! Opt-in synchronous S-CPU/APU ownership on one physical master timeline.
//!
//! The legacy C-port CPU remains unchanged. This machine is an isolated
//! foundation for source-ordered original-ROM execution: its SNES machine,
//! timeline cursor, APU clock debt, and any post-semantic completion always
//! move together.

use crate::apu::{Snes9xDspSampleReceipt, UnsupportedSmpMicroStep};
use crate::cpu_timeline::{
    CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuMasterTimestamp,
    CpuSynchronousTimelineEvent, NMI_SCANLINE, SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES,
};
use crate::snes::Snes;
use crate::snes9x_apu_clock::{Snes9xApuClockError, Snes9xApuClockState};

mod source_cpu;
pub use source_cpu::{
    Snes9xColdCpuExecutor, SourceCpuBusAccess, SourceCpuBusAccessKind, SourceCpuError,
    SourceCpuStepReceipt, SourceCpuTransaction, SourceCpuTransactionKind,
};

/// A CPU bus semantic which has committed exactly once and whose access charge
/// is waiting for a fallible hardware-event drain to finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuSynchronousCompletion {
    Read(u8),
    ReadWord(u16),
    Write,
    WriteWord,
}

/// Coherent opt-in owner for the modeled synchronous S-CPU/APUI subset.
///
/// There is deliberately no public `from_parts` or raw cycle-advance API.
/// Future CPU execution must call the private transaction drain from the exact
/// AddCycles boundaries in the pinned Snes9x source. The only public seed is
/// the source-exact T=0 Snes9x SMP coroutine reset used by this subset.
pub struct CpuSynchronousMachine {
    snes: Snes,
    timeline: CpuMasterTimeline,
    apu_clock: Snes9xApuClockState,
    pending_completion: Option<CpuSynchronousCompletion>,
    /// Pinned `CPU.NMIPending` plus its absolute `NMITriggerPos`. Interrupt
    /// entry remains instruction-boundary owned by the source CPU executor.
    nmi_acceptance_not_before: Option<CpuMasterTimestamp>,
    /// `$4200` low-to-high NMI edges are published by `CHECK_FOR_IRQ_CHANGE`
    /// after the writing instruction, not by the register semantic itself.
    deferred_nmi_enable_edge: bool,
    #[cfg(test)]
    force_zero_cycle_smp_step: bool,
}

impl CpuSynchronousMachine {
    pub fn from_snes9x_apu_reset_seed() -> Self {
        let mut snes = Snes::new();
        snes.apu.reset_snes9x_coroutine();
        let mut timeline = CpuMasterTimeline::new(
            0,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline
            .begin_synchronous_timeline()
            .expect("the Snes9x reset seed starts before the first bus event");
        Self {
            snes,
            timeline,
            apu_clock: Snes9xApuClockState::new(),
            pending_completion: None,
            nmi_acceptance_not_before: None,
            deferred_nmi_enable_edge: false,
            #[cfg(test)]
            force_zero_cycle_smp_step: false,
        }
    }

    pub fn timestamp(&self) -> CpuMasterTimestamp {
        self.timeline.timestamp()
    }

    pub const fn snes(&self) -> &Snes {
        &self.snes
    }

    pub const fn pending_completion(&self) -> Option<CpuSynchronousCompletion> {
        self.pending_completion
    }

    /// Transfer samples emitted by scanline-end/F3 DSP synchronization exactly
    /// once from the machine that owns the physical CPU/APU timeline.
    pub fn take_dsp_samples(&mut self) -> Snes9xDspSampleReceipt {
        self.snes.apu.take_snes9x_dsp_samples()
    }

    /// Read one CPU-visible APU port alias. If a post-semantic event cannot
    /// finish, the sampled byte remains owned by `pending_completion` and is
    /// returned only by `resume_pending_completion`; the port is not reread.
    pub fn read_apu_port_alias(&mut self, full_adr: u32) -> Result<u8, CpuSynchronousMachineError> {
        self.reject_new_semantic_while_pending()?;
        let port = Snes::synchronous_cpu_apu_port(full_adr)
            .ok_or(CpuSynchronousMachineError::UnsupportedCpuBusAddress { full_adr })?;
        let timestamp = self.timestamp();
        let force_zero_cycle_smp_step = self.force_zero_cycle_smp_step();
        Self::synchronize_apu(
            &mut self.snes,
            &mut self.apu_clock,
            timestamp,
            force_zero_cycle_smp_step,
        )?;
        let value = self.snes.synchronous_cpu_read_apu_port_semantic(port);
        self.pending_completion = Some(CpuSynchronousCompletion::Read(value));
        let access_master_cycles = u32::from(self.snes.hardware_access_time(full_adr));
        self.drain_add_cycles_after_committed_semantic(access_master_cycles)?;
        match self.take_pending_completion() {
            CpuSynchronousCompletion::Read(value) => Ok(value),
            CpuSynchronousCompletion::ReadWord(_)
            | CpuSynchronousCompletion::Write
            | CpuSynchronousCompletion::WriteWord => {
                unreachable!("read installed a byte-read completion")
            }
        }
    }

    /// Write one CPU-visible APU port alias. If a post-semantic event cannot
    /// finish, the input latch remains committed and only its pending access
    /// completion is resumed; the write semantic is never replayed.
    pub fn write_apu_port_alias(
        &mut self,
        full_adr: u32,
        value: u8,
    ) -> Result<(), CpuSynchronousMachineError> {
        self.reject_new_semantic_while_pending()?;
        let port = Snes::synchronous_cpu_apu_port(full_adr)
            .ok_or(CpuSynchronousMachineError::UnsupportedCpuBusAddress { full_adr })?;
        let timestamp = self.timestamp();
        let force_zero_cycle_smp_step = self.force_zero_cycle_smp_step();
        Self::synchronize_apu(
            &mut self.snes,
            &mut self.apu_clock,
            timestamp,
            force_zero_cycle_smp_step,
        )?;
        self.snes
            .synchronous_cpu_write_apu_port_semantic(full_adr, port, value);
        self.pending_completion = Some(CpuSynchronousCompletion::Write);
        let access_master_cycles = u32::from(self.snes.hardware_access_time(full_adr));
        self.drain_add_cycles_after_committed_semantic(access_master_cycles)?;
        debug_assert_eq!(
            self.take_pending_completion(),
            CpuSynchronousCompletion::Write
        );
        Ok(())
    }

    /// Resume the already-charged event drain for one committed bus semantic.
    /// The returned completion is the retained original result; no bus semantic
    /// and no access duration is executed a second time.
    pub fn resume_pending_completion(
        &mut self,
    ) -> Result<CpuSynchronousCompletion, CpuSynchronousMachineError> {
        if self.pending_completion.is_none() {
            return Err(CpuSynchronousMachineError::NoPendingCompletion);
        }
        self.drain_add_cycles_after_committed_semantic(0)?;
        Ok(self.take_pending_completion())
    }

    fn reject_new_semantic_while_pending(&self) -> Result<(), CpuSynchronousMachineError> {
        if let Some(completion) = self.pending_completion {
            return Err(CpuSynchronousMachineError::PendingCompletionMustResume { completion });
        }
        Ok(())
    }

    fn take_pending_completion(&mut self) -> CpuSynchronousCompletion {
        self.pending_completion
            .take()
            .expect("a committed semantic owns its completion until drain succeeds")
    }

    /// One post-semantic pinned-Snes9x AddCycles transaction. This remains
    /// private until the source CPU executor can supply exact transaction
    /// boundaries; it must never be driven per logical byte or receipt.
    fn drain_add_cycles_after_committed_semantic(
        &mut self,
        master_cycles: u32,
    ) -> Result<(), CpuSynchronousMachineError> {
        let force_zero_cycle_smp_step = self.force_zero_cycle_smp_step();
        let snes = &mut self.snes;
        let apu_clock = &mut self.apu_clock;
        let nmi_acceptance_not_before = &mut self.nmi_acceptance_not_before;
        self.timeline
            .advance_synchronous_after_semantics_with(master_cycles, |event, timestamp| match event
            {
                CpuSynchronousTimelineEvent::HMax {
                    completed_field_index: _,
                    completed_scanline,
                    event_timestamp,
                    ..
                } => {
                    Self::synchronize_apu(snes, apu_clock, timestamp, force_zero_cycle_smp_step)?;
                    // Pinned `S9xAPUEndScanline`: execute the SMP through HMax,
                    // then drain the DSP clock accumulated by that execution.
                    snes.apu.synchronize_snes9x_dsp();
                    let next_scanline = if completed_scanline + 1
                        == crate::cpu_timeline::NTSC_SCANLINES_PER_FIELD as u16
                    {
                        0
                    } else {
                        completed_scanline + 1
                    };
                    if u32::from(next_scanline) == NMI_SCANLINE {
                        // cpuexec.cpp publishes RDNMI before scheduling an
                        // enabled NMI for H=12 of the new VBlank scanline.
                        snes.in_vblank = true;
                        snes.in_nmi = true;
                        if snes.nmi_enabled {
                            snes.cpu.nmi_wanted = true;
                            let deadline = CpuMasterTimestamp::new(
                                event_timestamp.master_cycles()
                                    + SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES,
                            );
                            *nmi_acceptance_not_before = Some(deadline);
                        }
                    } else if next_scanline == 0 {
                        snes.in_vblank = false;
                        // cpuexec.cpp returns RDNMI to the model's low
                        // `_5A22` state at the start of the new field.
                        snes.in_nmi = false;
                    }
                    // Pinned `S9xMainLoop`: after the scanline increment,
                    // auto-read both joypad ports at ScreenHeight + 3 (V228)
                    // when JOYSER0 is enabled by NMITIMEN bit 0.
                    if next_scanline == 228 && snes.auto_joy_read {
                        snes.do_auto_joypad();
                    }
                    Ok(0)
                }
                CpuSynchronousTimelineEvent::Bus(_) => Ok(0),
            })
    }

    fn synchronize_apu(
        snes: &mut Snes,
        apu_clock: &mut Snes9xApuClockState,
        timestamp: CpuMasterTimestamp,
        force_zero_cycle_smp_step: bool,
    ) -> Result<(), CpuSynchronousMachineError> {
        let apu = &mut snes.apu;
        let mut unsupported = None;
        let clock_result = apu_clock.sync_to(timestamp.master_cycles(), || {
            if force_zero_cycle_smp_step {
                return 0;
            }
            let before = apu.cycles;
            match apu.run_snes9x_micro_step_without_dsp() {
                Ok(_) => apu.cycles.wrapping_sub(before),
                Err(error) => {
                    unsupported = Some(error);
                    0
                }
            }
        });
        if let Some(error) = unsupported {
            return Err(CpuSynchronousMachineError::UnsupportedSmpOpcode(error));
        }
        clock_result.map_err(CpuSynchronousMachineError::ApuClock)
    }

    #[cfg(test)]
    const fn force_zero_cycle_smp_step(&self) -> bool {
        self.force_zero_cycle_smp_step
    }

    #[cfg(not(test))]
    const fn force_zero_cycle_smp_step(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CpuSynchronousMachineError {
    #[error("synchronous CPU bus address ${full_adr:06x} is not an APU port alias")]
    UnsupportedCpuBusAddress { full_adr: u32 },
    #[error("the committed {completion:?} bus semantic must be resumed before a new semantic")]
    PendingCompletionMustResume {
        completion: CpuSynchronousCompletion,
    },
    #[error("there is no committed CPU bus completion to resume")]
    NoPendingCompletion,
    #[error(transparent)]
    ApuClock(#[from] Snes9xApuClockError),
    #[error(transparent)]
    UnsupportedSmpOpcode(UnsupportedSmpMicroStep),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_timeline::{CpuRasterPosition, MASTER_CYCLES_PER_SCANLINE};
    use crate::Snes9xApuClockCheckpoint;

    fn machine_at_source_checkpoint(
        raster: CpuRasterPosition,
        clock: Snes9xApuClockState,
    ) -> CpuSynchronousMachine {
        let mut snes = Snes::new();
        snes.apu.reset_snes9x_coroutine();
        let mut timeline = CpuMasterTimeline::at_raster(
            0,
            raster,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline.begin_synchronous_timeline().unwrap();
        CpuSynchronousMachine {
            snes,
            timeline,
            apu_clock: clock,
            pending_completion: None,
            nmi_acceptance_not_before: None,
            deferred_nmi_enable_edge: false,
            force_zero_cycle_smp_step: false,
        }
    }

    fn post_semantic_hmax_failure_machine() -> CpuSynchronousMachine {
        let checkpoint = Snes9xApuClockCheckpoint::new(1_358, 250_000, 0).unwrap();
        let mut machine = machine_at_source_checkpoint(
            CpuRasterPosition::new(0, 1_358),
            Snes9xApuClockState::from_checkpoint(checkpoint).unwrap(),
        );
        machine.force_zero_cycle_smp_step = true;
        machine
    }

    fn zero_cycle_smp_step() -> CpuSynchronousMachineError {
        CpuSynchronousMachineError::ApuClock(Snes9xApuClockError::ZeroCycleSmpStep)
    }

    #[test]
    fn owned_machine_starts_from_one_coherent_snes9x_apu_reset_seed() {
        let machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(0));
        assert_eq!(
            machine.timeline.raster_position(),
            CpuRasterPosition::new(0, 0)
        );
        assert_eq!(machine.apu_clock, Snes9xApuClockState::new());
        assert_eq!(machine.snes.apu.spc.pc, 0xffc0);
        assert_eq!(machine.snes.apu.spc.sp, 0xef);
        assert!(machine.snes.apu.spc.z);
        assert_eq!(machine.snes.apu.cpu_cycles_left, 0);
        assert!(machine
            .snes
            .apu
            .capture_snes9x_coroutine_checkpoint()
            .is_some());
        assert_eq!(machine.pending_completion(), None);
    }

    #[test]
    fn field_wrap_clears_vblank_and_rdnmi_latches_to_model_low_state() {
        let raster = CpuRasterPosition::new(261, 1_358);
        let start = CpuFieldTiming::NON_INTERLACE_EVEN.master_cycles_at(0, raster);
        let clock = Snes9xApuClockState::from_checkpoint(
            Snes9xApuClockCheckpoint::new(start, 0, 0).unwrap(),
        )
        .unwrap();
        let mut machine = machine_at_source_checkpoint(raster, clock);
        machine.snes.in_vblank = true;
        machine.snes.in_nmi = true;

        machine
            .drain_add_cycles_after_committed_semantic(6)
            .unwrap();

        assert_eq!(
            machine.timeline.raster_position(),
            CpuRasterPosition::new(0, 0)
        );
        assert!(!machine.snes.in_vblank);
        assert!(!machine.snes.in_nmi);
        assert!(!machine.snes.cpu.nmi_wanted);
        assert_eq!(machine.nmi_acceptance_not_before, None);
    }

    #[test]
    fn hmax_at_v228_runs_auto_joypad_only_when_nmitimen_bit_zero_is_enabled() {
        let raster = CpuRasterPosition::new(227, 1_358);
        let start = CpuFieldTiming::NON_INTERLACE_EVEN.master_cycles_at(0, raster);
        let clock = || {
            Snes9xApuClockState::from_checkpoint(
                Snes9xApuClockCheckpoint::new(start, 0, 0).unwrap(),
            )
            .unwrap()
        };

        let mut enabled = machine_at_source_checkpoint(raster, clock());
        enabled.snes.auto_joy_read = true;
        enabled.snes.input1.current_state = 0x1234;
        enabled.snes.input2.current_state = 0xabcd;
        enabled.snes.port_auto_read = [0xffff; 4];
        enabled
            .drain_add_cycles_after_committed_semantic(6)
            .unwrap();

        assert_eq!(
            enabled.timeline.raster_position(),
            CpuRasterPosition::new(228, 0)
        );
        assert_eq!(enabled.snes.port_auto_read, [0x2c48, 0xb3d5, 0, 0]);
        assert!(!enabled.snes.input1.latch_line);
        assert!(!enabled.snes.input2.latch_line);

        let mut disabled = machine_at_source_checkpoint(raster, clock());
        disabled.snes.auto_joy_read = false;
        disabled.snes.input1.current_state = 0x1234;
        disabled.snes.input2.current_state = 0xabcd;
        disabled.snes.port_auto_read = [0x1111, 0x2222, 0x3333, 0x4444];
        disabled
            .drain_add_cycles_after_committed_semantic(6)
            .unwrap();

        assert_eq!(
            disabled.timeline.raster_position(),
            CpuRasterPosition::new(228, 0)
        );
        assert_eq!(
            disabled.snes.port_auto_read,
            [0x1111, 0x2222, 0x3333, 0x4444]
        );
    }

    #[test]
    fn reset_seed_apui_semantics_own_their_six_cycle_transactions() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        assert_eq!(machine.read_apu_port_alias(0x002140).unwrap(), 0);
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(6));
        machine.write_apu_port_alias(0x002140, 0x77).unwrap();
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(12));
        assert_eq!(machine.snes.apu.in_ports[0], 0x77);
        assert_eq!(machine.pending_completion(), None);
    }

    #[test]
    fn pre_semantic_sync_failure_commits_neither_semantic_nor_duration() {
        let mut machine =
            machine_at_source_checkpoint(CpuRasterPosition::new(0, 21), Snes9xApuClockState::new());
        machine.force_zero_cycle_smp_step = true;

        assert_eq!(
            machine.write_apu_port_alias(0x002140, 0x77),
            Err(zero_cycle_smp_step())
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(21));
        assert_eq!(machine.snes.apu.in_ports[0], 0);
        assert_eq!(machine.pending_completion(), None);
    }

    #[test]
    fn failed_post_write_hmax_drain_resumes_without_replaying_write_or_duration() {
        let mut machine = post_semantic_hmax_failure_machine();
        assert_eq!(
            machine.write_apu_port_alias(0x002140, 0x77),
            Err(zero_cycle_smp_step())
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(1_364));
        assert_eq!(machine.timeline.wram_refresh_cycle(), 538);
        assert_eq!(machine.snes.apu.in_ports[0], 0x77);
        assert_eq!(
            machine.pending_completion(),
            Some(CpuSynchronousCompletion::Write)
        );

        assert_eq!(
            machine.write_apu_port_alias(0x002140, 0x88),
            Err(CpuSynchronousMachineError::PendingCompletionMustResume {
                completion: CpuSynchronousCompletion::Write,
            })
        );
        assert_eq!(machine.snes.apu.in_ports[0], 0x77);
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(1_364));
        assert_eq!(machine.timeline.wram_refresh_cycle(), 538);

        machine.force_zero_cycle_smp_step = false;
        assert_eq!(
            machine.resume_pending_completion().unwrap(),
            CpuSynchronousCompletion::Write
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(1_364));
        assert_eq!(machine.timeline.wram_refresh_cycle(), 534);
        assert_eq!(machine.snes.apu.in_ports[0], 0x77);
        assert_eq!(machine.pending_completion(), None);
    }

    #[test]
    fn failed_post_read_hmax_drain_returns_retained_value_without_rereading() {
        let mut machine = post_semantic_hmax_failure_machine();
        machine.snes.apu.out_ports[0] = 0x5a;
        assert_eq!(
            machine.read_apu_port_alias(0x002140),
            Err(zero_cycle_smp_step())
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(1_364));
        assert_eq!(
            machine.pending_completion(),
            Some(CpuSynchronousCompletion::Read(0x5a))
        );

        machine.snes.apu.out_ports[0] = 0xa5;
        assert_eq!(
            machine.read_apu_port_alias(0x002140),
            Err(CpuSynchronousMachineError::PendingCompletionMustResume {
                completion: CpuSynchronousCompletion::Read(0x5a),
            })
        );
        machine.force_zero_cycle_smp_step = false;
        assert_eq!(
            machine.resume_pending_completion().unwrap(),
            CpuSynchronousCompletion::Read(0x5a)
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(1_364));
        assert_eq!(machine.snes.apu.out_ports[0], 0xa5);
        assert_eq!(machine.pending_completion(), None);
    }

    #[test]
    fn apui_low_ending_at_refresh_places_the_next_semantic_after_the_stall() {
        let checkpoint = Snes9xApuClockCheckpoint::new(532, 0, 0).unwrap();
        let mut machine = machine_at_source_checkpoint(
            CpuRasterPosition::new(0, 532),
            Snes9xApuClockState::from_checkpoint(checkpoint).unwrap(),
        );
        assert_eq!(machine.read_apu_port_alias(0x002140).unwrap(), 0);
        assert_eq!(
            machine.timeline.raster_position(),
            CpuRasterPosition::new(0, 578)
        );
        assert_eq!(machine.read_apu_port_alias(0x002141).unwrap(), 0);
        assert_eq!(
            machine.timeline.raster_position(),
            CpuRasterPosition::new(0, 584)
        );
        assert_eq!(
            machine.apu_clock.checkpoint().cpu_reference_master_cycles(),
            578
        );
    }

    #[test]
    fn six_cycle_apui_access_drains_hmax_at_its_exact_endpoint() {
        let checkpoint = Snes9xApuClockCheckpoint::new(1_358, 250_000, 0).unwrap();
        let mut machine = machine_at_source_checkpoint(
            CpuRasterPosition::new(0, 1_358),
            Snes9xApuClockState::from_checkpoint(checkpoint).unwrap(),
        );
        assert_eq!(machine.read_apu_port_alias(0x002140).unwrap(), 0);
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(1_364));
        assert_eq!(
            machine.timeline.raster_position(),
            CpuRasterPosition::new(1, 0)
        );
        assert_eq!(
            machine.apu_clock.checkpoint().cpu_reference_master_cycles(),
            1_364
        );
        assert_eq!(machine.snes.apu.cycles, 2);
        assert_eq!(
            machine
                .snes
                .apu
                .capture_snes9x_apu_coroutine_checkpoint()
                .unwrap()
                .pending_dsp_clocks(),
            0
        );
        assert!(machine.take_dsp_samples().samples.is_empty());
        assert_eq!(MASTER_CYCLES_PER_SCANLINE, 1_364);
    }

    #[test]
    fn resume_without_owned_completion_is_rejected() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        assert_eq!(
            machine.resume_pending_completion(),
            Err(CpuSynchronousMachineError::NoPendingCompletion)
        );
    }
}
