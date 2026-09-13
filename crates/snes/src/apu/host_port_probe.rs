//! Explicit native-APU ownership at a complete instruction boundary.
//! This is not an exact cold CPU/APU checkpoint or a CPU-to-SMP clock seed.

use super::{ApuState, SmpCoroutineState, SmpMicroStepResult, UnsupportedSmpMicroStep};

#[derive(Debug, thiserror::Error)]
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

    pub fn machine(&self) -> &ApuState { &self.apu }

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
