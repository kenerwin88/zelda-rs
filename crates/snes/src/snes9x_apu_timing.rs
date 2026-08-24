//! Opt-in Snes9x-compatible ownership of the APU machine and CPU/APUI clock.
//!
//! This wrapper is deliberately separate from [`crate::Snes`]. Native C-port
//! execution keeps its existing APU behavior unless a caller explicitly owns
//! this timing shadow.

use crate::apu::{
    ApuState, Snes9xApuCoroutineCheckpoint, Snes9xApuCoroutineCheckpointError,
    Snes9xDspSampleReceipt, UnsupportedSmpMicroStep,
};
use crate::snes9x_apu_clock::{Snes9xApuClockCheckpoint, Snes9xApuClockError, Snes9xApuClockState};

/// Serializable timing sidecars for a separately serialized APU machine.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Snes9xApuTimingCheckpoint {
    pub clock: Snes9xApuClockCheckpoint,
    pub coroutine: Snes9xApuCoroutineCheckpoint,
}

/// Opt-in APU timing owner following pinned Snes9x 1.63 synchronization.
#[derive(Clone, Debug)]
pub struct Snes9xApuTiming {
    apu: ApuState,
    clock: Snes9xApuClockState,
}

impl Default for Snes9xApuTiming {
    fn default() -> Self {
        Self::new()
    }
}

impl Snes9xApuTiming {
    pub fn new() -> Self {
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        Self {
            apu,
            clock: Snes9xApuClockState::new(),
        }
    }

    pub fn reset(&mut self) {
        self.apu.reset_snes9x_coroutine();
        self.clock = Snes9xApuClockState::new();
    }

    /// Restore separately persisted machine and timing-sidecar state.
    pub fn from_machine_and_checkpoint(
        mut apu: ApuState,
        checkpoint: Snes9xApuTimingCheckpoint,
    ) -> Result<Self, Snes9xApuTimingError> {
        apu.restore_snes9x_apu_coroutine_checkpoint(checkpoint.coroutine)?;
        Ok(Self {
            apu,
            clock: Snes9xApuClockState::from_checkpoint(checkpoint.clock)?,
        })
    }

    pub fn checkpoint(&self) -> Snes9xApuTimingCheckpoint {
        Snes9xApuTimingCheckpoint {
            clock: self.clock.checkpoint(),
            coroutine: self
                .apu
                .capture_snes9x_apu_coroutine_checkpoint()
                .expect("Snes9x APU timing owner always enables its coroutine"),
        }
    }

    /// Atomically clone the stable machine and its identity-bound exact timing
    /// sidecar from one immutable owner snapshot.
    pub fn machine_and_checkpoint(&self) -> (ApuState, Snes9xApuTimingCheckpoint) {
        (self.apu.clone(), self.checkpoint())
    }

    pub const fn apu(&self) -> &ApuState {
        &self.apu
    }

    pub const fn clock(&self) -> &Snes9xApuClockState {
        &self.clock
    }

    /// Transfer samples emitted by exact DSP synchronizations exactly once.
    pub fn take_dsp_samples(&mut self) -> Snes9xDspSampleReceipt {
        self.apu.take_snes9x_dsp_samples()
    }

    /// Bring the SMP forward to an absolute CPU master-clock timestamp.
    ///
    /// An unsupported opcode returns without fetching it. The clock owner has
    /// already retained the newly converted SMP debt, so retrying this exact
    /// timestamp resumes rather than converting the CPU interval twice.
    pub fn sync_to(&mut self, cpu_master_cycles: u64) -> Result<(), Snes9xApuTimingError> {
        self.sync_to_with_step(cpu_master_cycles, |apu| {
            let before = apu.cycles;
            apu.run_snes9x_micro_step_without_dsp()?;
            Ok(apu.cycles.wrapping_sub(before))
        })
    }

    fn sync_to_with_step(
        &mut self,
        cpu_master_cycles: u64,
        mut step: impl FnMut(&mut ApuState) -> Result<u32, UnsupportedSmpMicroStep>,
    ) -> Result<(), Snes9xApuTimingError> {
        let apu = &mut self.apu;
        let mut unsupported = None;
        let clock_result = self.clock.sync_to(cpu_master_cycles, || match step(apu) {
            Ok(consumed) => consumed,
            Err(error) => {
                unsupported = Some(error);
                0
            }
        });

        if let Some(error) = unsupported {
            return Err(Snes9xApuTimingError::UnsupportedSmpOpcode(error));
        }
        clock_result.map_err(Snes9xApuTimingError::Clock)
    }

    /// Synchronize first, then sample the SMP-to-CPU output port.
    pub fn read_cpu_port_at(
        &mut self,
        cpu_master_cycles: u64,
        port: u8,
    ) -> Result<u8, Snes9xApuTimingError> {
        self.sync_to(cpu_master_cycles)?;
        Ok(self.apu.read_snes_port(port))
    }

    /// Synchronize first, then commit the CPU-to-SMP input port write.
    pub fn write_cpu_port_at(
        &mut self,
        cpu_master_cycles: u64,
        port: u8,
        value: u8,
    ) -> Result<(), Snes9xApuTimingError> {
        self.sync_to(cpu_master_cycles)?;
        self.apu.write_snes_port(port, value);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Snes9xApuTimingError {
    #[error(transparent)]
    Clock(#[from] Snes9xApuClockError),
    #[error(transparent)]
    UnsupportedSmpOpcode(UnsupportedSmpMicroStep),
    #[error(transparent)]
    CoroutineCheckpoint(#[from] Snes9xApuCoroutineCheckpointError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bootstrap_fixture::{cpu_apu_accesses, records, split_first_cc_cpu_accesses};

    #[test]
    fn cpu_port_timeline_matches_cold_snes9x_fixture_through_cc() {
        let fixture = records();
        let events = &fixture[2];
        let mut timing = Snes9xApuTiming::new();

        let accesses = cpu_apu_accesses(events);
        let (reset_writes, handshake) = split_first_cc_cpu_accesses(&accesses);
        for event in reset_writes {
            timing
                .write_cpu_port_at(event.absolute_master_cycle(), event.port, event.value)
                .unwrap();
            assert_eq!(timing.apu().cycles, event.apu_cycle_after);
        }

        let first_time = handshake[0].absolute_master_cycle();
        let second_time = handshake[1].absolute_master_cycle();
        assert_eq!(handshake[0].program_counter, handshake[1].program_counter);
        assert_eq!(second_time, first_time + 6);
        assert_eq!(handshake[0].value, 0);
        assert_eq!(handshake[1].value, 0);
        assert_eq!(handshake[0].apu_cycle_after, 2_397);
        assert_eq!(handshake[1].apu_cycle_after, 2_398);

        for event in handshake {
            let timestamp = event.absolute_master_cycle();
            if event.is_read {
                assert_eq!(
                    timing.read_cpu_port_at(timestamp, event.port).unwrap(),
                    event.value
                );
            } else {
                let before_input = timing.apu().in_ports[usize::from(event.port & 3)];
                timing
                    .write_cpu_port_at(timestamp, event.port, event.value)
                    .unwrap();
                assert_eq!(
                    timing.apu().in_ports[usize::from(event.port & 3)],
                    event.value
                );

                // The CC write occurs after synchronization. Consequently the
                // just-completed CMP/branch still observed AA and returned to
                // the polling PC before CC becomes visible to the SMP.
                if event.port == 0 && event.value == 0xcc {
                    assert_eq!(before_input, 0);
                    assert_eq!(timing.apu().cycles, 2_431);
                    assert_eq!(timing.apu().spc.pc, 0xffcf);
                    assert!(!timing.apu().spc.z);
                }
            }
            assert_eq!(timing.apu().cycles, event.apu_cycle_after);
        }

        assert_eq!(timing.apu().cycles, 2_461);
        assert_eq!(timing.apu().spc.pc, 0xfff7);
        assert_eq!(timing.apu().out_ports, [0xcc, 0xbb, 0, 0]);
    }

    #[test]
    fn unsupported_opcode_retains_debt_for_same_timestamp_retry() {
        let mut timing = Snes9xApuTiming::new();
        timing.apu.rom_readable = false;
        timing.apu.spc.pc = 0x0200;
        timing.apu.ram[0x0200] = 0x00;

        let error = timing.sync_to(21).unwrap_err();
        assert_eq!(
            error,
            Snes9xApuTimingError::UnsupportedSmpOpcode(UnsupportedSmpMicroStep {
                opcode: 0,
                pc: 0x0200,
            })
        );
        assert_eq!(timing.apu.cycles, 0);
        assert_eq!(timing.apu.spc.pc, 0x0200);
        assert_eq!(timing.clock.checkpoint().cpu_reference_master_cycles(), 21);
        assert_eq!(timing.clock.checkpoint().smp_clock(), -1);

        timing.apu.ram[0x0200..0x0202].copy_from_slice(&[0xcd, 0x7f]);
        timing.sync_to(21).unwrap();
        assert_eq!(timing.apu.cycles, 2);
        assert_eq!(timing.apu.spc.pc, 0x0202);
        assert_eq!(timing.clock.checkpoint().smp_clock(), 1);
    }

    #[test]
    fn zero_cycle_step_retains_debt_for_same_timestamp_retry() {
        let mut timing = Snes9xApuTiming::new();

        assert_eq!(
            timing.sync_to_with_step(21, |_| Ok(0)).unwrap_err(),
            Snes9xApuTimingError::Clock(Snes9xApuClockError::ZeroCycleSmpStep)
        );
        assert_eq!(timing.apu.cycles, 0);
        assert_eq!(timing.apu.spc.pc, 0xffc0);
        assert_eq!(timing.clock.checkpoint().cpu_reference_master_cycles(), 21);
        assert_eq!(timing.clock.checkpoint().smp_clock(), -1);

        timing.sync_to(21).unwrap();
        assert_eq!(timing.apu.cycles, 2);
        assert_eq!(timing.apu.spc.pc, 0xffc2);
        assert_eq!(timing.clock.checkpoint().smp_clock(), 1);
    }

    #[test]
    fn machine_and_timing_sidecars_restore_separately_mid_instruction() {
        let mut timing = Snes9xApuTiming::new();
        // Fixture T=v36:H1126 suspends the first $8f after pseudo-case 1.
        timing.sync_to(36 * 1_364 + 1_126).unwrap();
        timing.apu.dsp_adr = 0x7c;
        let _ = timing.apu.cpu_read(0xf3);
        let (machine, checkpoint) = timing.machine_and_checkpoint();
        let coroutine_json = serde_json::to_value(&checkpoint.coroutine).unwrap();
        assert_eq!(coroutine_json["smp"]["opcode"], 0x8f);
        assert_eq!(coroutine_json["smp"]["opcode_cycle"], 1);
        let machine_json = serde_json::to_vec(&machine).unwrap();
        let checkpoint_json = serde_json::to_vec(&checkpoint).unwrap();

        let machine: ApuState = serde_json::from_slice(&machine_json).unwrap();
        let checkpoint: Snes9xApuTimingCheckpoint =
            serde_json::from_slice(&checkpoint_json).unwrap();
        let mut restored =
            Snes9xApuTiming::from_machine_and_checkpoint(machine, checkpoint).unwrap();

        timing.sync_to(50_300).unwrap();
        restored.sync_to(50_300).unwrap();
        assert_eq!(restored.apu().cycles, timing.apu().cycles);
        assert_eq!(restored.apu().spc.pc, timing.apu().spc.pc);
        assert_eq!(restored.apu().ram, timing.apu().ram);
        assert_eq!(restored.apu().out_ports, timing.apu().out_ports);
        assert_eq!(restored.clock().checkpoint(), timing.clock().checkpoint());
        assert_eq!(restored.take_dsp_samples(), timing.take_dsp_samples());
    }
}
