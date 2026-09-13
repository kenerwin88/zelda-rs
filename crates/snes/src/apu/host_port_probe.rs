//! Explicit native-APU ownership at a complete instruction boundary.
//! This is not an exact cold CPU/APU checkpoint or a CPU-to-SMP clock seed.

use super::{ApuState, SmpCoroutineState, SmpMicroStepResult, UnsupportedSmpMicroStep};
use crate::snes9x_apu_clock::{Snes9xApuClockCheckpoint, Snes9xApuClockError, Snes9xApuClockState};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApuHostPortProbeError {
    #[error("host-port probe requires a completed legacy APU instruction and no exact DSP owner")]
    InvalidBoundary,
    #[error("host-port probe is poisoned")]
    Poisoned,
    #[error("invalid CPU APU port {0}")]
    InvalidPort(u8),
    #[error(transparent)]
    Step(#[from] UnsupportedSmpMicroStep),
}

/// Owns a native APU and its retained SPC instruction while a CPU timing
/// owner synchronizes individual port accesses. No host window, clock ratio,
/// reset, or audio lookahead is invented here. The caller supplies the
/// synchronization deadline and advances through source pseudo-op boundaries.
/// Deliberately not serializable: dropping a continuation into a legacy
/// APU snapshot would lose the pending instruction.
#[derive(Clone)]
pub struct ApuHostPortProbe {
    apu: ApuState,
    instruction_start: u32,
    poisoned: bool,
}

impl ApuHostPortProbe {
    pub fn new(mut apu: ApuState) -> Result<Self, ApuHostPortProbeError> {
        if apu.snes9x_dsp.is_some()
            || apu.smp_coroutine.is_enabled()
            || !apu.smp_coroutine.is_idle()
            || apu.cpu_cycles_left != 0
        {
            return Err(ApuHostPortProbeError::InvalidBoundary);
        }
        // Begin the next opcode from the actual retained native machine;
        // preserve all RAM, timers, port latches and scheduled input events.
        apu.smp_coroutine = SmpCoroutineState::enabled();
        Ok(Self { instruction_start: apu.cycles, apu, poisoned: false })
    }

    /// Adopt a machine that is already in pinned Snes9x coroutine form at an
    /// idle pseudo-op boundary (a cold `reset_snes9x_coroutine` seed or a
    /// restored coroutine checkpoint). Nothing is reset or re-enabled, and an
    /// exact DSP owner is preserved: [`ApuHostPortTiming::end_scanline_at`]
    /// drains it exactly where `S9xAPUEndScanline` does.
    pub fn from_snes9x_coroutine(apu: ApuState) -> Result<Self, ApuHostPortProbeError> {
        if !apu.smp_coroutine.is_enabled()
            || !apu.smp_coroutine.is_idle()
            || apu.cpu_cycles_left != 0
        {
            return Err(ApuHostPortProbeError::InvalidBoundary);
        }
        Ok(Self { instruction_start: apu.cycles, apu, poisoned: false })
    }

    pub fn machine(&self) -> &ApuState { &self.apu }

    /// Pinned `S9xAPUEndScanline`'s `SNES::dsp.synchronize()` half. A machine
    /// without the exact DSP owner has nothing to drain.
    pub(crate) fn synchronize_dsp(&mut self) {
        self.apu.synchronize_snes9x_dsp();
    }

    pub fn at_instruction_boundary(&self) -> bool {
        !self.poisoned && self.apu.smp_coroutine.is_idle()
    }

    pub fn step(&mut self) -> Result<SmpMicroStepResult, ApuHostPortProbeError> {
        if self.poisoned { return Err(ApuHostPortProbeError::Poisoned); }
        if self.apu.smp_coroutine.is_idle() {
            self.instruction_start = self.apu.cycles;
            self.apu.record_spc_instruction_trace();
        }
        match self.apu.run_snes9x_micro_step_without_dsp() {
            Ok(result) => {
                if matches!(result, SmpMicroStepResult::InstructionComplete { .. }) {
                    self.apu.spc.cycles_used = self.apu.cycles.wrapping_sub(self.instruction_start) as u8;
                }
                Ok(result)
            }
            Err(error) => {
                self.poisoned = true;
                Err(error.into())
            }
        }
    }

    /// Publish at a CPU access boundary already selected by the caller.
    pub fn write_cpu_port(&mut self, port: u8, value: u8) -> Result<(), ApuHostPortProbeError> {
        if self.poisoned { return Err(ApuHostPortProbeError::Poisoned); }
        if port >= 4 { return Err(ApuHostPortProbeError::InvalidPort(port)); }
        self.apu.in_ports[usize::from(port)] = value;
        Ok(())
    }

    /// Retain the whole probe on refusal so a suspended instruction cannot
    /// accidentally be resumed by the legacy complete-instruction executor.
    pub fn into_machine(self) -> Result<ApuState, Self> {
        if !self.at_instruction_boundary() { return Err(self); }
        let mut apu = self.apu;
        apu.smp_coroutine = SmpCoroutineState::default();
        Ok(apu)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApuHostPortTimingError {
    #[error("a retained SPC continuation must be adopted at a complete instruction boundary")]
    SuspendedInstruction,
    #[error("a complete instruction boundary cannot owe {0} SMP cycles")]
    RetainedSmpDebt(i64),
    #[error(transparent)]
    Clock(#[from] Snes9xApuClockError),
    #[error(transparent)]
    Probe(#[from] ApuHostPortProbeError),
}

/// Pinned `apu/apu.cpp` CPU-to-SMP synchronization over a retained native SPC
/// continuation.
///
/// `S9xAPUExecute` converts the absolute CPU master clock through
/// [`Snes9xApuClockState`] — remainder and signed SMP credit/debt included —
/// and runs source pseudo-steps until the SMP has caught up; only then does the
/// port access happen. Nothing here resets the APU, invents a clock ratio,
/// seeds an audio lookahead, or opens a host output window: the caller supplies
/// the machine, its clock provenance, and every absolute access timestamp.
///
/// The machine arrives through [`ApuHostPortProbe`], which accepts exactly two
/// provenances: a retained native SPC continuation adopted at a completed
/// legacy instruction boundary, or a machine already in pinned Snes9x coroutine
/// form. Only the second carries an exact DSP owner; the first publishes no
/// samples, and `end_scanline_at` then has nothing to drain.
pub struct ApuHostPortTiming {
    probe: ApuHostPortProbe,
    clock: Snes9xApuClockState,
}

impl ApuHostPortTiming {
    /// Adopt a retained continuation with explicit clock provenance.
    ///
    /// A completed instruction boundary has already been executed, so pinned
    /// `SNES::smp.clock` is non-negative there; a negative seed would claim the
    /// SMP still owes execution that the boundary says it finished.
    pub fn new(
        probe: ApuHostPortProbe,
        clock: Snes9xApuClockCheckpoint,
    ) -> Result<Self, ApuHostPortTimingError> {
        if !probe.at_instruction_boundary() {
            return Err(ApuHostPortTimingError::SuspendedInstruction);
        }
        if clock.smp_clock() < 0 {
            return Err(ApuHostPortTimingError::RetainedSmpDebt(clock.smp_clock()));
        }
        Ok(Self {
            probe,
            clock: Snes9xApuClockState::from_checkpoint(clock)?,
        })
    }

    pub fn machine(&self) -> &ApuState {
        self.probe.machine()
    }

    pub fn clock(&self) -> &Snes9xApuClockState {
        &self.clock
    }

    /// Return the retained continuation to its caller. A suspended or poisoned
    /// probe keeps the whole owner, exactly as [`ApuHostPortProbe`] does.
    pub fn into_probe(self) -> Result<ApuHostPortProbe, Self> {
        if self.probe.at_instruction_boundary() {
            Ok(self.probe)
        } else {
            Err(self)
        }
    }

    /// Pinned `S9xAPUExecute`: convert the elapsed CPU master clock, then run
    /// source pseudo-steps until the SMP clock is no longer in debt.
    ///
    /// An unsupported SMP opcode leaves the converted debt recorded in the
    /// clock, so retrying the same timestamp resumes instead of converting the
    /// same CPU interval twice.
    pub fn sync_to(&mut self, cpu_master_cycles: u64) -> Result<(), ApuHostPortTimingError> {
        let probe = &mut self.probe;
        let mut failure = None;
        let clock_result = self.clock.sync_to(cpu_master_cycles, || {
            let before = probe.machine().cycles;
            match probe.step() {
                Ok(_) => probe.machine().cycles.wrapping_sub(before),
                Err(error) => {
                    failure = Some(error);
                    0
                }
            }
        });
        if let Some(error) = failure {
            return Err(error.into());
        }
        Ok(clock_result?)
    }

    /// Pinned `S9xAPUReadPort`: synchronize, then sample the SMP output port.
    pub fn read_cpu_port_at(
        &mut self,
        cpu_master_cycles: u64,
        port: u8,
    ) -> Result<u8, ApuHostPortTimingError> {
        self.sync_to(cpu_master_cycles)?;
        Ok(self.probe.machine().read_snes_port(port))
    }

    /// Pinned `S9xAPUWritePort`: synchronize, then publish the CPU input latch.
    /// An output-window boundary can never cancel this publication.
    pub fn write_cpu_port_at(
        &mut self,
        cpu_master_cycles: u64,
        port: u8,
        value: u8,
    ) -> Result<(), ApuHostPortTimingError> {
        self.sync_to(cpu_master_cycles)?;
        self.probe.write_cpu_port(port & 3, value)?;
        Ok(())
    }

    /// Pinned `S9xAPUEndScanline` at `cpuexec.cpp:HC_HCOUNTER_MAX_EVENT`:
    /// execute the SMP through HMax, then drain the DSP clock that execution
    /// accumulated. A retained native continuation carries no exact DSP owner,
    /// so there is nothing to drain and no sample is published.
    pub fn end_scanline_at(
        &mut self,
        cpu_master_cycles: u64,
    ) -> Result<(), ApuHostPortTimingError> {
        self.sync_to(cpu_master_cycles)?;
        self.probe.synchronize_dsp();
        Ok(())
    }
}
