//! Exact CPU-master-clock to SMP-clock conversion used by Snes9x 1.63.
//!
//! This module is intentionally only a pure timing foundation. It does not
//! attach itself to the SNES bus or advance an [`crate::apu::ApuState`]. A
//! future bus-timed owner supplies the absolute CPU master clock and a callback
//! that executes one Snes9x-compatible SMP pseudo-step.

use serde::{Deserialize, Serialize};

/// Pinned Snes9x NTSC CPU-master-clock to SMP-clock numerator.
pub const SNES9X_NTSC_APU_CLOCK_NUMERATOR: u32 = 15_664;

/// Pinned Snes9x NTSC CPU-master-clock to SMP-clock denominator.
pub const SNES9X_NTSC_APU_CLOCK_DENOMINATOR: u32 = 328_125;

/// Serializable timing fields required to resume the clock conversion exactly.
///
/// The SMP clock is signed because Snes9x retains pseudo-step overshoot as
/// positive credit and subtracts newly requested SMP cycles from that credit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snes9xApuClockCheckpoint {
    cpu_reference_master_cycles: u64,
    remainder: u32,
    smp_clock: i64,
}

impl Snes9xApuClockCheckpoint {
    /// Construct a validated checkpoint.
    pub fn new(
        cpu_reference_master_cycles: u64,
        remainder: u32,
        smp_clock: i64,
    ) -> Result<Self, Snes9xApuClockError> {
        if remainder >= SNES9X_NTSC_APU_CLOCK_DENOMINATOR {
            return Err(Snes9xApuClockError::InvalidRemainder { remainder });
        }
        Ok(Self {
            cpu_reference_master_cycles,
            remainder,
            smp_clock,
        })
    }

    pub const fn cpu_reference_master_cycles(self) -> u64 {
        self.cpu_reference_master_cycles
    }

    pub const fn remainder(self) -> u32 {
        self.remainder
    }

    pub const fn smp_clock(self) -> i64 {
        self.smp_clock
    }
}

/// Pure, persistable Snes9x CPU-master-clock to SMP-clock synchronizer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snes9xApuClockState {
    checkpoint: Snes9xApuClockCheckpoint,
}

impl Snes9xApuClockState {
    pub const fn new() -> Self {
        Self {
            checkpoint: Snes9xApuClockCheckpoint {
                cpu_reference_master_cycles: 0,
                remainder: 0,
                smp_clock: 0,
            },
        }
    }

    pub fn from_checkpoint(
        checkpoint: Snes9xApuClockCheckpoint,
    ) -> Result<Self, Snes9xApuClockError> {
        if checkpoint.remainder >= SNES9X_NTSC_APU_CLOCK_DENOMINATOR {
            return Err(Snes9xApuClockError::InvalidRemainder {
                remainder: checkpoint.remainder,
            });
        }
        Ok(Self { checkpoint })
    }

    pub const fn checkpoint(&self) -> Snes9xApuClockCheckpoint {
        self.checkpoint
    }

    /// Synchronize the SMP to an absolute CPU master-clock timestamp.
    ///
    /// `step_smp` executes one Snes9x-compatible SMP pseudo-step and returns
    /// the nonzero number of SMP cycles it consumed. Positive pseudo-step
    /// overshoot remains in `smp_clock` as credit for the next synchronization.
    ///
    /// A zero-cycle callback leaves the requested clock conversion and its SMP
    /// debt recorded in this state. Calling `sync_to` again at the same CPU
    /// timestamp can therefore resume without replaying converted CPU time.
    pub fn sync_to(
        &mut self,
        cpu_master_cycles: u64,
        mut step_smp: impl FnMut() -> u32,
    ) -> Result<(), Snes9xApuClockError> {
        if self.checkpoint.remainder >= SNES9X_NTSC_APU_CLOCK_DENOMINATOR {
            return Err(Snes9xApuClockError::InvalidRemainder {
                remainder: self.checkpoint.remainder,
            });
        }
        if cpu_master_cycles < self.checkpoint.cpu_reference_master_cycles {
            return Err(Snes9xApuClockError::CpuClockMovedBackwards {
                reference: self.checkpoint.cpu_reference_master_cycles,
                requested: cpu_master_cycles,
            });
        }

        let elapsed = cpu_master_cycles - self.checkpoint.cpu_reference_master_cycles;
        let scaled = u128::from(elapsed) * u128::from(SNES9X_NTSC_APU_CLOCK_NUMERATOR)
            + u128::from(self.checkpoint.remainder);
        let denominator = u128::from(SNES9X_NTSC_APU_CLOCK_DENOMINATOR);
        let requested_smp_cycles = scaled / denominator;
        let requested_smp_cycles = i64::try_from(requested_smp_cycles)
            .expect("the pinned NTSC ratio maps every u64 CPU timestamp delta into i64 SMP cycles");

        let smp_clock = self
            .checkpoint
            .smp_clock
            .checked_sub(requested_smp_cycles)
            .ok_or(Snes9xApuClockError::SmpClockOverflow)?;
        self.checkpoint.cpu_reference_master_cycles = cpu_master_cycles;
        self.checkpoint.remainder = (scaled % denominator) as u32;
        self.checkpoint.smp_clock = smp_clock;

        while self.checkpoint.smp_clock < 0 {
            let consumed = step_smp();
            if consumed == 0 {
                return Err(Snes9xApuClockError::ZeroCycleSmpStep);
            }
            self.checkpoint.smp_clock += i64::from(consumed);
        }
        Ok(())
    }
}

/// Invalid input or callback behavior rejected by [`Snes9xApuClockState`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Snes9xApuClockError {
    #[error("Snes9x APU clock remainder {remainder} is not below 328125")]
    InvalidRemainder { remainder: u32 },
    #[error("CPU master clock moved backwards from {reference} to {requested}")]
    CpuClockMovedBackwards { reference: u64, requested: u64 },
    #[error("SMP clock credit/debt overflowed")]
    SmpClockOverflow,
    #[error("SMP pseudo-step consumed zero cycles")]
    ZeroCycleSmpStep,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_cycle() -> u32 {
        1
    }

    #[test]
    fn split_and_combined_synchronization_are_exactly_equivalent() {
        let mut split = Snes9xApuClockState::new();
        let mut split_steps = 0u64;
        split
            .sync_to(12_345, || {
                split_steps += 1;
                1
            })
            .unwrap();
        split
            .sync_to(67_890, || {
                split_steps += 1;
                1
            })
            .unwrap();

        let mut combined = Snes9xApuClockState::new();
        let mut combined_steps = 0u64;
        combined
            .sync_to(67_890, || {
                combined_steps += 1;
                1
            })
            .unwrap();

        assert_eq!(split, combined);
        assert_eq!(split_steps, combined_steps);
    }

    #[test]
    fn six_master_cycles_cross_the_ratio_boundary_from_remainder_250000() {
        let checkpoint = Snes9xApuClockCheckpoint::new(0, 250_000, 0).unwrap();
        let mut clock = Snes9xApuClockState::from_checkpoint(checkpoint).unwrap();
        let mut steps = 0;

        clock
            .sync_to(6, || {
                steps += 1;
                1
            })
            .unwrap();

        assert_eq!(steps, 1);
        assert_eq!(clock.checkpoint().remainder(), 15_859);
        assert_eq!(clock.checkpoint().smp_clock(), 0);
    }

    #[test]
    fn absolute_clock_remains_exact_beyond_24000_ntsc_fields() {
        const LONG_FIELD_MASTER_CYCLES: u64 = 262 * 1_364;
        const SHORT_FIELD_MASTER_CYCLES: u64 = LONG_FIELD_MASTER_CYCLES - 4;
        const FIELD_COUNT: u64 = 24_001;
        const STEP_CYCLES: u32 = 1_000_000;

        let mut clock = Snes9xApuClockState::new();
        let mut cpu_now = 0u64;
        let mut consumed = 0u64;
        for field in 0..FIELD_COUNT {
            cpu_now += if field & 1 == 0 {
                LONG_FIELD_MASTER_CYCLES
            } else {
                SHORT_FIELD_MASTER_CYCLES
            };
            clock
                .sync_to(cpu_now, || {
                    consumed += u64::from(STEP_CYCLES);
                    STEP_CYCLES
                })
                .unwrap();
        }

        let scaled = u128::from(cpu_now) * u128::from(SNES9X_NTSC_APU_CLOCK_NUMERATOR);
        let expected_cycles = (scaled / u128::from(SNES9X_NTSC_APU_CLOCK_DENOMINATOR)) as u64;
        let expected_remainder = (scaled % u128::from(SNES9X_NTSC_APU_CLOCK_DENOMINATOR)) as u32;
        assert!(cpu_now > u64::from(u32::MAX));
        assert_eq!(clock.checkpoint().cpu_reference_master_cycles(), cpu_now);
        assert_eq!(clock.checkpoint().remainder(), expected_remainder);
        assert_eq!(
            clock.checkpoint().smp_clock(),
            (consumed - expected_cycles) as i64
        );
    }

    #[test]
    fn pseudo_step_overshoot_is_retained_as_signed_clock_credit() {
        let checkpoint = Snes9xApuClockCheckpoint::new(0, 0, 2).unwrap();
        let mut clock = Snes9xApuClockState::from_checkpoint(checkpoint).unwrap();
        let mut steps = 0;

        clock
            .sync_to(105, || {
                steps += 1;
                4
            })
            .unwrap();

        assert_eq!(steps, 1);
        assert_eq!(clock.checkpoint().remainder(), 4_095);
        assert_eq!(clock.checkpoint().smp_clock(), 1);
    }

    #[test]
    fn backwards_cpu_clock_is_rejected_without_mutation_or_callback() {
        let checkpoint = Snes9xApuClockCheckpoint::new(100, 12_345, 7).unwrap();
        let mut clock = Snes9xApuClockState::from_checkpoint(checkpoint).unwrap();
        let before = clock.clone();
        let mut called = false;

        assert_eq!(
            clock.sync_to(99, || {
                called = true;
                1
            }),
            Err(Snes9xApuClockError::CpuClockMovedBackwards {
                reference: 100,
                requested: 99,
            })
        );
        assert!(!called);
        assert_eq!(clock, before);
    }

    #[test]
    fn zero_cycle_step_fails_with_resumable_debt() {
        let mut clock = Snes9xApuClockState::new();

        assert_eq!(
            clock.sync_to(105, || 0),
            Err(Snes9xApuClockError::ZeroCycleSmpStep)
        );
        assert_eq!(clock.checkpoint().cpu_reference_master_cycles(), 105);
        assert_eq!(clock.checkpoint().remainder(), 4_095);
        assert_eq!(clock.checkpoint().smp_clock(), -5);

        clock.sync_to(105, || 5).unwrap();
        assert_eq!(clock.checkpoint().smp_clock(), 0);
    }

    #[test]
    fn state_and_checkpoint_clone_and_serde_roundtrip() {
        let checkpoint = Snes9xApuClockCheckpoint::new(123_456, 234_567, 3).unwrap();
        let state = Snes9xApuClockState::from_checkpoint(checkpoint).unwrap();

        assert_eq!(state.clone(), state);
        assert_eq!(
            serde_json::from_slice::<Snes9xApuClockCheckpoint>(
                &serde_json::to_vec(&checkpoint).unwrap()
            )
            .unwrap(),
            checkpoint
        );
        assert_eq!(
            serde_json::from_slice::<Snes9xApuClockState>(&serde_json::to_vec(&state).unwrap())
                .unwrap(),
            state
        );
    }

    #[test]
    fn zero_elapsed_time_does_not_step_the_smp() {
        let mut clock = Snes9xApuClockState::new();
        clock.sync_to(0, one_cycle).unwrap();
        assert_eq!(clock, Snes9xApuClockState::new());
    }
}
