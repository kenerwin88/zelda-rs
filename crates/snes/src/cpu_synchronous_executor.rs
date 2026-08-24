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
    SNES9X_NMI_GENERAL_DMA_DELAY_MASTER_CYCLES,
};
use crate::dma::{SynchronousGeneralDmaByte, SynchronousGeneralDmaUnsupported};
use crate::snes::Snes;
use crate::snes9x_apu_clock::{Snes9xApuClockError, Snes9xApuClockState};

mod source_cpu;
pub use source_cpu::{
    Snes9xColdCpuExecutor, Snes9xCpuQuiescentCheckpoint, Snes9xCpuQuiescentCheckpointError,
    Snes9xMainLoopReceipt, SourceCpuBusAccess, SourceCpuBusAccessKind, SourceCpuError,
    SourceCpuStepReceipt, SourceCpuTransaction, SourceCpuTransactionKind,
};

/// A CPU bus semantic which has committed exactly once and whose access charge
/// is waiting for a fallible hardware-event drain to finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CpuSynchronousCompletion {
    Read(u8),
    ReadWord(u16),
    Write,
    WriteWord,
    GeneralDmaWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SynchronousGeneralDmaPhase {
    FindChannel,
    TransferByte {
        channel: u8,
        b_phase: u8,
    },
    DrainCommittedByte {
        channel: u8,
        b_phase: u8,
        channel_complete: bool,
    },
    DrainOuterWrite {
        started_at: CpuMasterTimestamp,
        start_wram_refresh_position: u16,
        charged: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SynchronousGeneralDmaContinuation {
    mask: u8,
    next_channel: u8,
    phase: SynchronousGeneralDmaPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SynchronousGeneralDmaWriteReceipt {
    pub outer_started_at: CpuMasterTimestamp,
    pub outer_ended_at: CpuMasterTimestamp,
    pub outer_start_wram_refresh_position: u16,
    pub outer_end_wram_refresh_position: u16,
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
    pending_general_dma: Option<SynchronousGeneralDmaContinuation>,
    /// Source-owned VMAIN remap capability. The legacy PPU port supports only
    /// linear VRAM addressing, so nonzero FullGraphicCount is fail-closed for
    /// the fast DMA path rather than approximated.
    source_vmain_full_graphic_count_nonzero: bool,
    /// Pinned `CPU.NMIPending` plus its absolute `NMITriggerPos`. Interrupt
    /// entry remains instruction-boundary owned by the source CPU executor.
    nmi_acceptance_not_before: Option<CpuMasterTimestamp>,
    /// `$4200` low-to-high NMI edges are published by `CHECK_FOR_IRQ_CHANGE`
    /// after the writing instruction, not by the register semantic itself.
    deferred_nmi_enable_edge: bool,
    /// Absolute source `Timings.NextIRQTimer` for the audited vertical-only
    /// IRQ mode. `None` is Snes9x's disabled/out-of-range sentinel.
    irq_timer_at: Option<CpuMasterTimestamp>,
    /// Pinned `SCAN_KEYS_FLAG`, with the exact VBlank H-event timestamp carried
    /// as its one-shot host-return receipt. Repeated VBlank events cannot
    /// replace an unconsumed return request.
    main_loop_return_pending: Option<CpuMasterTimestamp>,
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
            pending_general_dma: None,
            source_vmain_full_graphic_count_nonzero: false,
            nmi_acceptance_not_before: None,
            deferred_nmi_enable_edge: false,
            irq_timer_at: None,
            main_loop_return_pending: None,
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
            | CpuSynchronousCompletion::WriteWord
            | CpuSynchronousCompletion::GeneralDmaWrite => {
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
        if self.pending_general_dma.is_some() {
            self.run_pending_general_dma()?;
            debug_assert!(self.pending_general_dma.is_none());
            debug_assert_eq!(
                self.take_pending_completion(),
                CpuSynchronousCompletion::GeneralDmaWrite
            );
            return Ok(CpuSynchronousCompletion::Write);
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

    /// Execute the semantic nested inside a nonzero `$420b` CPU write. Pinned
    /// Snes9x performs the whole DMA before charging the enclosing six-cycle
    /// getset access; this method returns the exact outer transaction bounds.
    pub(crate) fn write_general_dma_control(
        &mut self,
        mask: u8,
    ) -> Result<SynchronousGeneralDmaWriteReceipt, CpuSynchronousMachineError> {
        debug_assert_ne!(mask, 0);
        self.reject_new_semantic_while_pending()?;
        if self.snes.dma.dma_busy
            || self
                .snes
                .dma
                .channel
                .iter()
                .any(|channel| channel.dma_active)
        {
            return Err(CpuSynchronousMachineError::GeneralDmaAlreadyActive);
        }
        if let Some(channel) = self
            .snes
            .dma
            .channel
            .iter()
            .position(|channel| channel.hdma_active)
        {
            return Err(CpuSynchronousMachineError::GeneralDmaActiveHdma {
                channel: channel as u8,
            });
        }
        let bus = self.timeline.bus_workload();
        if bus.dynamic_hdma() || bus.hdma_stall_master_cycles() != 0 {
            return Err(CpuSynchronousMachineError::GeneralDmaDynamicHdma);
        }
        // Capability checks precede `$420b` mutation and every timing charge.
        self.snes
            .validate_synchronous_general_dma(mask)
            .map_err(|error| match error {
                SynchronousGeneralDmaUnsupported::ActiveHdma { channel } => {
                    CpuSynchronousMachineError::GeneralDmaActiveHdma { channel }
                }
                SynchronousGeneralDmaUnsupported::ApuBus { channel } => {
                    CpuSynchronousMachineError::GeneralDmaApuBus { channel }
                }
                SynchronousGeneralDmaUnsupported::CpuOrPpuRegisterBus { channel } => {
                    CpuSynchronousMachineError::GeneralDmaCpuOrPpuRegisterBus { channel }
                }
            })?;
        self.validate_general_dma_fast_vram_path(mask)?;
        self.snes.synchronous_general_dma_begin(mask);
        self.pending_completion = Some(CpuSynchronousCompletion::GeneralDmaWrite);
        self.pending_general_dma = Some(SynchronousGeneralDmaContinuation {
            mask,
            next_channel: 0,
            phase: SynchronousGeneralDmaPhase::FindChannel,
        });
        // ppu.cpp:S9xSetCPU charges DMACPUSync without processing events.
        self.advance_source_non_draining_master_cycles(18);
        self.run_pending_general_dma()
    }

    fn run_pending_general_dma(
        &mut self,
    ) -> Result<SynchronousGeneralDmaWriteReceipt, CpuSynchronousMachineError> {
        loop {
            let continuation = self
                .pending_general_dma
                .expect("a general-DMA completion owns its continuation");
            match continuation.phase {
                SynchronousGeneralDmaPhase::FindChannel => {
                    let Some(channel) = (continuation.next_channel..8)
                        .find(|channel| continuation.mask & (1 << channel) != 0)
                    else {
                        self.snes.synchronous_general_dma_finish_all();
                        self.pending_general_dma.as_mut().unwrap().phase =
                            SynchronousGeneralDmaPhase::DrainOuterWrite {
                                started_at: self.timestamp(),
                                start_wram_refresh_position: self.timeline.wram_refresh_cycle()
                                    as u16,
                                charged: false,
                            };
                        continue;
                    };
                    // S9xDoDMA owns per-channel InDMA flags before its raw
                    // eight-clock setup; no other selected channel is exposed.
                    self.snes.synchronous_general_dma_begin_channel(channel);
                    self.advance_source_non_draining_master_cycles(8);
                    let state = self.pending_general_dma.as_mut().unwrap();
                    state.next_channel = channel;
                    state.phase = SynchronousGeneralDmaPhase::TransferByte {
                        channel,
                        b_phase: 0,
                    };
                }
                SynchronousGeneralDmaPhase::TransferByte { channel, b_phase } => {
                    let byte = self
                        .snes
                        .synchronous_general_dma_next_byte(channel, b_phase);
                    self.execute_general_dma_bus_semantic(byte);
                    // Source UPDATE_COUNTERS commits before the fallible event drain.
                    let channel_complete = self.snes.synchronous_general_dma_commit_byte(channel);
                    self.pending_general_dma.as_mut().unwrap().phase =
                        SynchronousGeneralDmaPhase::DrainCommittedByte {
                            channel,
                            b_phase,
                            channel_complete,
                        };
                    self.drain_add_cycles_after_committed_semantic(8)?;
                    self.finish_drained_general_dma_byte(channel, b_phase, channel_complete);
                }
                SynchronousGeneralDmaPhase::DrainCommittedByte {
                    channel,
                    b_phase,
                    channel_complete,
                } => {
                    // The byte semantic, counters, and eight clocks already committed.
                    self.drain_add_cycles_after_committed_semantic(0)?;
                    self.finish_drained_general_dma_byte(channel, b_phase, channel_complete);
                }
                SynchronousGeneralDmaPhase::DrainOuterWrite {
                    started_at,
                    start_wram_refresh_position,
                    charged,
                } => {
                    if !charged {
                        self.pending_general_dma.as_mut().unwrap().phase =
                            SynchronousGeneralDmaPhase::DrainOuterWrite {
                                started_at,
                                start_wram_refresh_position,
                                charged: true,
                            };
                        self.drain_add_cycles_after_committed_semantic(6)?;
                    } else {
                        self.drain_add_cycles_after_committed_semantic(0)?;
                    }
                    let receipt = SynchronousGeneralDmaWriteReceipt {
                        outer_started_at: started_at,
                        outer_ended_at: self.timestamp(),
                        outer_start_wram_refresh_position: start_wram_refresh_position,
                        outer_end_wram_refresh_position: self.timeline.wram_refresh_cycle() as u16,
                    };
                    self.pending_general_dma = None;
                    return Ok(receipt);
                }
            }
        }
    }

    fn finish_drained_general_dma_byte(
        &mut self,
        channel: u8,
        b_phase: u8,
        channel_complete: bool,
    ) {
        if channel_complete {
            // dma.cpp retimes a pending NMI while the channel still owns the
            // source InDMA flags, then releases those flags on return.
            let skips_nmi_retime = self.snes.synchronous_general_dma_skips_nmi_retime(channel);
            if !skips_nmi_retime
                && self.snes.cpu.nmi_wanted
                && self.nmi_acceptance_not_before.is_some()
            {
                self.nmi_acceptance_not_before = Some(CpuMasterTimestamp::new(
                    self.timestamp().master_cycles() + SNES9X_NMI_GENERAL_DMA_DELAY_MASTER_CYCLES,
                ));
            }
            self.snes.synchronous_general_dma_finish_channel(channel);
            let state = self.pending_general_dma.as_mut().unwrap();
            state.next_channel = channel + 1;
            state.phase = SynchronousGeneralDmaPhase::FindChannel;
        } else {
            self.pending_general_dma.as_mut().unwrap().phase =
                SynchronousGeneralDmaPhase::TransferByte {
                    channel,
                    b_phase: b_phase.wrapping_add(1) & 3,
                };
        }
    }

    fn execute_general_dma_bus_semantic(&mut self, byte: SynchronousGeneralDmaByte) {
        let old_open_bus = self.snes.open_bus;
        if (0x80..=0x83).contains(&byte.b_address) && byte.a_bus_is_wram {
            if byte.from_b {
                // S9xGetPPU($2180) returns global OpenBus without incrementing
                // PPU.WRAM while InWRAMDMAorHDMA is set.
                self.write_general_dma_a_bus(byte.a_address, old_open_bus);
            }
            // Forward WRAM->$2180 is a timed drop. Neither direction touches
            // the WRAM data-port pointer.
        } else if byte.from_b {
            let value = self.read_general_dma_b_bus(byte.b_address);
            self.write_general_dma_a_bus(byte.a_address, value);
        } else {
            let value = self.read_general_dma_a_bus(byte.a_address);
            self.write_general_dma_b_bus(byte.b_address, value);
        }
        // CPU store macros own the final OpenBus publication after the full
        // nested DMA and enclosing write charge.
        if !(byte.publishes_fast_vram_high_open_bus
            && self.general_dma_a_bus_has_direct_base_pointer(byte.a_address))
        {
            self.snes.open_bus = old_open_bus;
        }
    }

    fn general_dma_a_bus_has_direct_base_pointer(&self, address: u32) -> bool {
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        matches!(bank, 0x7e | 0x7f)
            || (bank & 0x40 == 0 && adr < 0x2000)
            || (matches!(self.snes.cart.kind, crate::cart::CartType::LoRom) && adr >= 0x8000)
    }

    fn validate_general_dma_fast_vram_path(
        &self,
        mask: u8,
    ) -> Result<(), CpuSynchronousMachineError> {
        for channel in 0..8u8 {
            if mask & (1 << channel) == 0 {
                continue;
            }
            let dma = &self.snes.dma.channel[usize::from(channel)];
            let count = if dma.size == 0 {
                u32::from(u16::MAX) + 1
            } else {
                u32::from(dma.size)
            };
            let mut address = dma.a_adr;
            for transfer_index in 0..count {
                let fast_vram_high = !dma.from_b
                    && matches!(dma.mode, 1 | 5)
                    && dma.b_adr == 0x18
                    && transfer_index & 1 != 0;
                if fast_vram_high {
                    if self.source_vmain_full_graphic_count_nonzero {
                        return Err(CpuSynchronousMachineError::GeneralDmaNonlinearVram {
                            channel,
                        });
                    }
                    let full_address = (u32::from(dma.a_bank) << 16) | u32::from(address);
                    if !self.general_dma_a_bus_has_direct_base_pointer(full_address) {
                        return Err(CpuSynchronousMachineError::GeneralDmaUnprovenFastMap {
                            channel,
                            address: full_address,
                        });
                    }
                }
                if !dma.fixed {
                    address = if dma.decrement {
                        address.wrapping_sub(1)
                    } else {
                        address.wrapping_add(1)
                    };
                }
            }
        }
        Ok(())
    }

    fn read_general_dma_a_bus(&mut self, address: u32) -> u8 {
        debug_assert!(Snes::synchronous_cpu_apu_port(address).is_none());
        self.snes.read(address)
    }

    fn write_general_dma_a_bus(&mut self, address: u32, value: u8) {
        debug_assert!(Snes::synchronous_cpu_apu_port(address).is_none());
        self.snes.write(address, value);
    }

    fn read_general_dma_b_bus(&mut self, address: u8) -> u8 {
        debug_assert!(!(0x40..0x80).contains(&address));
        self.snes.read_b_bus(address)
    }

    fn write_general_dma_b_bus(&mut self, address: u8, value: u8) {
        debug_assert!(!(0x40..0x80).contains(&address));
        self.snes.write_b_bus(address, value);
    }

    fn advance_source_non_draining_master_cycles(&mut self, master_cycles: u8) {
        // This timeline primitive is the exact raw `CPU.Cycles += n` operation:
        // it advances physical time while retaining every due event for the
        // next draining source transaction.
        self.timeline
            .advance_synchronous_pcbase_opcode_fetch(master_cycles);
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
        let main_loop_return_pending = &mut self.main_loop_return_pending;
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
                        // cpuexec.cpp sets SCAN_KEYS_FLAG at the same VBlank
                        // H-event. S9xMainLoop consumes it only after the
                        // crossing instruction and any then-due interrupt
                        // boundary have completed.
                        main_loop_return_pending.get_or_insert(event_timestamp);
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
    #[error("general DMA channel {channel} collides with active HDMA")]
    GeneralDmaActiveHdma { channel: u8 },
    #[error("general DMA cannot nest while DMA state is already active")]
    GeneralDmaAlreadyActive,
    #[error("general DMA cannot run while dynamic or fixed HDMA timing is enabled")]
    GeneralDmaDynamicHdma,
    #[error("general DMA channel {channel} touches the fallible APUI synchronization path")]
    GeneralDmaApuBus { channel: u8 },
    #[error(
        "general DMA channel {channel} maps its A-bus through an unimplemented low-bank I/O range"
    )]
    GeneralDmaCpuOrPpuRegisterBus { channel: u8 },
    #[error("general DMA channel {channel} requires unsupported non-linear VMAIN remapping")]
    GeneralDmaNonlinearVram { channel: u8 },
    #[error("general DMA channel {channel} fast path has no proven direct map at ${address:06x}")]
    GeneralDmaUnprovenFastMap { channel: u8, address: u32 },
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
            pending_general_dma: None,
            source_vmain_full_graphic_count_nonzero: false,
            nmi_acceptance_not_before: None,
            deferred_nmi_enable_edge: false,
            irq_timer_at: None,
            main_loop_return_pending: None,
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

    fn configure_wram_to_b_bus_dma(
        machine: &mut CpuSynchronousMachine,
        channel: usize,
        source: u16,
        size: u16,
        mode: u8,
        b_address: u8,
    ) {
        let dma = &mut machine.snes.dma.channel[channel];
        dma.a_bank = 0x7e;
        dma.a_adr = source;
        dma.size = size;
        dma.mode = mode;
        dma.b_adr = b_address;
        dma.fixed = false;
        dma.decrement = false;
        dma.from_b = false;
        dma.hdma_active = false;
    }

    #[test]
    fn general_dma_owns_source_setup_channel_byte_and_outer_write_order() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        machine.snes.ram[0] = 0x5a;
        configure_wram_to_b_bus_dma(&mut machine, 0, 0, 1, 0, 0x18);

        let receipt = machine.write_general_dma_control(1).unwrap();

        assert_eq!(receipt.outer_started_at, CpuMasterTimestamp::new(34));
        assert_eq!(receipt.outer_ended_at, CpuMasterTimestamp::new(40));
        assert_eq!(machine.snes.ppu.vram[0], 0x005a);
        assert_eq!(machine.snes.ppu.vram_pointer, 1);
        assert_eq!(machine.snes.dma.channel[0].a_adr, 1);
        assert_eq!(machine.snes.dma.channel[0].size, 0);
        assert!(!machine.snes.dma.channel[0].dma_active);
        assert!(!machine.snes.dma.dma_busy);
        assert_eq!(
            machine.take_pending_completion(),
            CpuSynchronousCompletion::GeneralDmaWrite
        );
    }

    #[test]
    fn general_dma_byte_drain_processes_refresh_after_the_semantic() {
        let start = 504;
        let clock = Snes9xApuClockState::from_checkpoint(
            Snes9xApuClockCheckpoint::new(start, 0, 0).unwrap(),
        )
        .unwrap();
        let mut machine =
            machine_at_source_checkpoint(CpuRasterPosition::new(0, start as u16), clock);
        machine.snes.ram[0] = 0x44;
        configure_wram_to_b_bus_dma(&mut machine, 0, 0, 1, 0, 0x18);

        let receipt = machine.write_general_dma_control(1).unwrap();

        // T504 +18 +8 => semantic T530; +8 reaches refresh H538, whose
        // source handler adds 40, then the enclosing write adds six.
        assert_eq!(receipt.outer_started_at, CpuMasterTimestamp::new(578));
        assert_eq!(receipt.outer_ended_at, CpuMasterTimestamp::new(584));
        assert_eq!(machine.snes.ppu.vram[0], 0x0044);
    }

    #[test]
    fn general_dma_crossing_hmax_resumes_committed_byte_without_replay() {
        let start = 1_330;
        let clock = Snes9xApuClockState::from_checkpoint(
            Snes9xApuClockCheckpoint::new(start, 250_000, 0).unwrap(),
        )
        .unwrap();
        let mut machine =
            machine_at_source_checkpoint(CpuRasterPosition::new(0, start as u16), clock);
        machine.force_zero_cycle_smp_step = true;
        machine.snes.ram[0] = 0x66;
        configure_wram_to_b_bus_dma(&mut machine, 0, 0, 1, 0, 0x18);

        assert_eq!(
            machine.write_general_dma_control(1),
            Err(zero_cycle_smp_step())
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(1_364));
        assert_eq!(machine.snes.ppu.vram[0], 0x0066);
        assert_eq!(machine.snes.ppu.vram_pointer, 1);
        assert_eq!(machine.snes.dma.channel[0].a_adr, 1);
        assert_eq!(machine.snes.dma.channel[0].size, 0);
        assert!(machine.snes.dma.channel[0].dma_active);
        assert!(machine.snes.dma.dma_busy);
        assert_eq!(
            machine.pending_completion(),
            Some(CpuSynchronousCompletion::GeneralDmaWrite)
        );

        machine.force_zero_cycle_smp_step = false;
        assert_eq!(
            machine.resume_pending_completion().unwrap(),
            CpuSynchronousCompletion::Write
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(1_370));
        assert_eq!(machine.snes.ppu.vram_pointer, 1);
        assert_eq!(machine.snes.dma.channel[0].a_adr, 1);
        assert_eq!(machine.pending_completion(), None);
    }

    #[test]
    fn general_dma_160_byte_transfer_uses_live_counters_and_one_byte_drains() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        for (index, byte) in machine.snes.ram[..160].iter_mut().enumerate() {
            *byte = index as u8;
        }
        configure_wram_to_b_bus_dma(&mut machine, 0, 0, 160, 0, 0x18);

        let receipt = machine.write_general_dma_control(1).unwrap();

        // Raw setup is 26, byte charges are 1280, line-0 refresh is 40, and
        // the enclosing write is 6. The transfer remains before HMax.
        assert_eq!(receipt.outer_ended_at, CpuMasterTimestamp::new(1_352));
        assert_eq!(machine.snes.dma.channel[0].a_adr, 160);
        assert_eq!(machine.snes.dma.channel[0].size, 0);
        assert_eq!(machine.snes.ppu.vram_pointer, 160);
        for index in 0..160usize {
            assert_eq!(machine.snes.ppu.vram[index] as u8, index as u8);
        }
    }

    #[test]
    fn general_dma_mask_runs_selected_channels_in_source_order() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        machine.snes.ram[0] = 0x11;
        machine.snes.ram[1] = 0x22;
        configure_wram_to_b_bus_dma(&mut machine, 0, 0, 1, 0, 0x18);
        configure_wram_to_b_bus_dma(&mut machine, 2, 1, 1, 0, 0x18);

        let receipt = machine.write_general_dma_control(0b0000_0101).unwrap();

        assert_eq!(receipt.outer_ended_at, CpuMasterTimestamp::new(56));
        assert_eq!(machine.snes.ppu.vram[0], 0x0011);
        assert_eq!(machine.snes.ppu.vram[1], 0x0022);
        assert!(!machine.snes.dma.channel[0].dma_active);
        assert!(!machine.snes.dma.channel[2].dma_active);
    }

    #[test]
    fn completed_general_dma_retimes_pending_nmi_from_dma_end() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        machine.snes.cpu.nmi_wanted = true;
        machine.nmi_acceptance_not_before = Some(CpuMasterTimestamp::new(12));
        machine.snes.ram[0] = 0x77;
        configure_wram_to_b_bus_dma(&mut machine, 0, 0, 1, 0, 0x18);

        let receipt = machine.write_general_dma_control(1).unwrap();

        assert_eq!(receipt.outer_started_at, CpuMasterTimestamp::new(34));
        assert_eq!(
            machine.nmi_acceptance_not_before,
            Some(CpuMasterTimestamp::new(58))
        );
    }

    #[test]
    fn forward_wram_to_2180_is_timed_drop_without_wram_port_increment() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        machine.snes.ram_adr = 0x1234;
        machine.snes.ram[0] = 0xa5;
        configure_wram_to_b_bus_dma(&mut machine, 0, 0, 2, 1, 0x80);
        machine.snes.dma.channel[0].fixed = true;
        machine.snes.dma.channel[0].decrement = true;

        let receipt = machine.write_general_dma_control(1).unwrap();

        assert_eq!(receipt.outer_ended_at, CpuMasterTimestamp::new(48));
        assert_eq!(machine.snes.ram_adr, 0x1234);
        assert_eq!(machine.snes.ram[0x1234], 0);
        // The source's invalid 7e/7f branch increments A unconditionally.
        assert_eq!(machine.snes.dma.channel[0].a_adr, 2);
        assert!(!machine.snes.dma.channel[0].dma_active);

        let mut mirror = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        mirror.snes.ram_adr = 0x4567;
        mirror.snes.ram[0x100] = 0x7b;
        let dma = &mut mirror.snes.dma.channel[0];
        dma.a_bank = 0;
        dma.a_adr = 0x0100;
        dma.size = 1;
        dma.mode = 0;
        dma.b_adr = 0x80;
        dma.fixed = false;
        dma.decrement = false;
        dma.from_b = false;
        mirror.write_general_dma_control(1).unwrap();
        assert_eq!(mirror.snes.ram_adr, 0x4567);
        assert_eq!(mirror.snes.dma.channel[0].a_adr, 0x0101);

        let mut mode4 = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        mode4.snes.ram_adr = 0x2222;
        configure_wram_to_b_bus_dma(&mut mode4, 0, 0x0100, 4, 4, 0x80);
        mode4.write_general_dma_control(1).unwrap();
        assert_eq!(mode4.snes.ram_adr, 0x2222);
        assert_eq!(mode4.snes.dma.channel[0].a_adr, 0x0104);
    }

    #[test]
    fn reverse_general_dma_reads_b_bus_then_writes_a_bus() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        machine.snes.ram_adr = 0;
        machine.snes.open_bus = 0x5a;
        machine.snes.ram[0] = 0xa5;
        let dma = &mut machine.snes.dma.channel[0];
        dma.a_bank = 0x7f;
        dma.a_adr = 0x0010;
        dma.size = 1;
        dma.mode = 0;
        dma.b_adr = 0x80;
        dma.fixed = false;
        dma.decrement = false;
        dma.from_b = true;

        machine.write_general_dma_control(1).unwrap();

        assert_eq!(machine.snes.ram[0x1_0010], 0x5a);
        assert_eq!(machine.snes.ram_adr, 0);
        assert_eq!(machine.snes.dma.channel[0].a_adr, 0x0011);
    }

    #[test]
    fn full_bank_wram_2180_skips_nmi_retime_but_low_mirror_retimes() {
        let mut full = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        full.snes.cpu.nmi_wanted = true;
        full.nmi_acceptance_not_before = Some(CpuMasterTimestamp::new(12));
        configure_wram_to_b_bus_dma(&mut full, 0, 0, 2, 1, 0x80);
        let receipt = full.write_general_dma_control(1).unwrap();
        assert_eq!(receipt.outer_started_at, CpuMasterTimestamp::new(42));
        assert_eq!(
            full.nmi_acceptance_not_before,
            Some(CpuMasterTimestamp::new(12))
        );

        let mut mirror = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        mirror.snes.cpu.nmi_wanted = true;
        mirror.nmi_acceptance_not_before = Some(CpuMasterTimestamp::new(12));
        let dma = &mut mirror.snes.dma.channel[0];
        dma.a_bank = 0;
        dma.a_adr = 0x0100;
        dma.size = 1;
        dma.mode = 0;
        dma.b_adr = 0x80;
        dma.fixed = false;
        dma.decrement = false;
        dma.from_b = false;
        let receipt = mirror.write_general_dma_control(1).unwrap();
        assert_eq!(receipt.outer_started_at, CpuMasterTimestamp::new(34));
        assert_eq!(
            mirror.nmi_acceptance_not_before,
            Some(CpuMasterTimestamp::new(58))
        );
    }

    #[test]
    fn apui_general_dma_is_rejected_before_setup_or_semantics() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        let dma = &mut machine.snes.dma.channel[0];
        dma.a_bank = 0;
        dma.a_adr = 0x2140;
        dma.size = 1;
        dma.mode = 0;
        dma.b_adr = 0x18;
        dma.fixed = true;
        dma.from_b = false;

        assert_eq!(
            machine.write_general_dma_control(1),
            Err(CpuSynchronousMachineError::GeneralDmaApuBus { channel: 0 })
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(0));
        assert_eq!(machine.snes.apu.out_ports[0], 0);
        assert!(!machine.snes.dma.channel[0].dma_active);
        assert!(!machine.snes.dma.dma_busy);
    }

    #[test]
    fn general_dma_hdma_collision_is_rejected_before_timing_or_flag_mutation() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        configure_wram_to_b_bus_dma(&mut machine, 0, 0, 1, 0, 0x18);
        machine.snes.dma.channel[0].hdma_active = true;

        assert_eq!(
            machine.write_general_dma_control(1),
            Err(CpuSynchronousMachineError::GeneralDmaActiveHdma { channel: 0 })
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(0));
        assert!(!machine.snes.dma.channel[0].dma_active);
        assert!(machine.snes.dma.channel[0].hdma_active);
        assert!(!machine.snes.dma.dma_busy);
    }

    #[test]
    fn general_dma_rejects_unselected_and_dynamic_hdma_owners() {
        let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        configure_wram_to_b_bus_dma(&mut machine, 0, 0, 1, 0, 0x18);
        machine.snes.dma.channel[7].hdma_active = true;
        assert_eq!(
            machine.write_general_dma_control(1),
            Err(CpuSynchronousMachineError::GeneralDmaActiveHdma { channel: 7 })
        );
        assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(0));

        let mut dynamic = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        let mut timeline = CpuMasterTimeline::new(
            0,
            CpuBusWorkload::with_dynamic_hdma(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline.begin_synchronous_timeline().unwrap();
        dynamic.timeline = timeline;
        configure_wram_to_b_bus_dma(&mut dynamic, 0, 0, 1, 0, 0x18);
        assert_eq!(
            dynamic.write_general_dma_control(1),
            Err(CpuSynchronousMachineError::GeneralDmaDynamicHdma)
        );
        assert_eq!(dynamic.timestamp(), CpuMasterTimestamp::new(0));
    }

    #[test]
    fn general_dma_rejects_nested_and_dma_specific_a_bus_register_maps() {
        let mut nested = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        configure_wram_to_b_bus_dma(&mut nested, 0, 0, 1, 0, 0x18);
        nested.snes.dma.dma_busy = true;
        assert_eq!(
            nested.write_general_dma_control(1),
            Err(CpuSynchronousMachineError::GeneralDmaAlreadyActive)
        );
        assert_eq!(nested.timestamp(), CpuMasterTimestamp::new(0));

        for (a_bank, a_address) in [
            (0u8, 0x2000u16),
            (0, 0x3fff),
            (0, 0x4000),
            (0, 0x5fff),
            (0x80, 0x2000),
            (0xbf, 0x5fff),
        ] {
            let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
            let dma = &mut machine.snes.dma.channel[0];
            dma.a_bank = a_bank;
            dma.a_adr = a_address;
            dma.size = 1;
            dma.mode = 0;
            dma.b_adr = 0x18;
            dma.fixed = true;
            dma.from_b = false;
            assert_eq!(
                machine.write_general_dma_control(1),
                Err(CpuSynchronousMachineError::GeneralDmaCpuOrPpuRegisterBus { channel: 0 })
            );
            assert_eq!(machine.timestamp(), CpuMasterTimestamp::new(0));
        }

        for a_address in [0x1fffu16, 0x6000] {
            let mut machine = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
            let dma = &mut machine.snes.dma.channel[0];
            dma.a_bank = 0;
            dma.a_adr = a_address;
            dma.size = 1;
            dma.mode = 0;
            dma.b_adr = 0x18;
            dma.fixed = true;
            dma.from_b = false;
            assert_eq!(machine.snes.validate_synchronous_general_dma(1), Ok(()));
        }
    }

    #[test]
    fn fast_vram_high_requires_linear_vmain_and_proven_direct_map() {
        let mut nonlinear = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        configure_wram_to_b_bus_dma(&mut nonlinear, 0, 0, 2, 1, 0x18);
        nonlinear.source_vmain_full_graphic_count_nonzero = true;
        assert_eq!(
            nonlinear.write_general_dma_control(1),
            Err(CpuSynchronousMachineError::GeneralDmaNonlinearVram { channel: 0 })
        );
        assert_eq!(nonlinear.timestamp(), CpuMasterTimestamp::new(0));

        let mut mapped = CpuSynchronousMachine::from_snes9x_apu_reset_seed();
        mapped.snes.cart.kind = crate::cart::CartType::LoRom;
        let dma = &mut mapped.snes.dma.channel[0];
        dma.a_bank = 0x70;
        dma.a_adr = 0x1000;
        dma.size = 2;
        dma.mode = 1;
        dma.b_adr = 0x18;
        dma.fixed = false;
        dma.decrement = false;
        dma.from_b = false;
        assert_eq!(
            mapped.write_general_dma_control(1),
            Err(CpuSynchronousMachineError::GeneralDmaUnprovenFastMap {
                channel: 0,
                address: 0x70_1001,
            })
        );
        assert_eq!(mapped.timestamp(), CpuMasterTimestamp::new(0));
    }
}
