//! Cycle-sequenced Sony SPC700 instruction core.
//!
//! Adapted from snes-apu 0.1.12's BSD-2-Clause SMP implementation. This
//! module intentionally contains no DSP renderer: its bus supplies timing,
//! memory, timer, and DSP-register-write behavior to the native Rust audio
//! path. See the snes-apu notice in the repository's `NOTICE.md`.

pub(crate) trait SmpBus {
    fn cycles(&mut self, cycles: i32);
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn read_cycle(&mut self, address: u16) -> u8 {
        self.cycles(1);
        self.read(address)
    }

    fn write_cycle(&mut self, address: u16, value: u8) {
        self.cycles(1);
        self.write(address, value);
    }

    fn read_stack_cycle(&mut self, address: u16) -> u8 {
        self.read_cycle(address)
    }

    fn write_stack_cycle(&mut self, address: u16, value: u8) {
        self.write_cycle(address, value);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SmpState {
    pub(crate) pc: u16,
    pub(crate) a: u8,
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) sp: u8,
    pub(crate) c: bool,
    pub(crate) z: bool,
    pub(crate) h: bool,
    pub(crate) p: bool,
    pub(crate) v: bool,
    pub(crate) n: bool,
    pub(crate) i: bool,
    pub(crate) b: bool,
    pub(crate) stopped: bool,
}

/// Persistent state at a Snes9x SMP coroutine yield boundary.
///
/// Snes9x does not make every SPC700 instruction atomic. `op_step` retains the
/// decoded opcode, its current pseudo-op case, and these scratch registers, and
/// yields once the requested SMP clock has been reached. Keeping the complete
/// generic scratch shape here lets individual opcodes be added without an
/// address- or value-specific continuation model.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SmpCoroutineState {
    pub(crate) enabled: bool,
    pub(crate) opcode: Option<u8>,
    pub(crate) opcode_cycle: u8,
    pub(crate) rd: u16,
    pub(crate) wr: u16,
    pub(crate) dp: u16,
    pub(crate) sp: u16,
    pub(crate) ya: u16,
    pub(crate) bit: u16,
}

/// Separately versionable continuation state for an opt-in Snes9x timing
/// shadow. This deliberately does not participate in `ApuState`/`Snes` serde.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Snes9xSmpCoroutineCheckpoint {
    opcode: Option<u8>,
    opcode_cycle: u8,
    rd: u16,
    wr: u16,
    dp: u16,
    sp: u16,
    ya: u16,
    bit: u16,
    pub(crate) dsp_clock: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Snes9xSmpCoroutineCheckpointError {
    IdleOpcodeCycle {
        opcode_cycle: u8,
    },
    UnsupportedOpcode {
        opcode: u8,
    },
    InvalidOpcodeCycle {
        opcode: u8,
        opcode_cycle: u8,
        stage_count: u8,
    },
}

impl std::fmt::Display for Snes9xSmpCoroutineCheckpointError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IdleOpcodeCycle { opcode_cycle } => {
                write!(formatter, "idle SMP checkpoint has opcode cycle {opcode_cycle}")
            }
            Self::UnsupportedOpcode { opcode } => {
                write!(formatter, "SMP checkpoint opcode ${opcode:02x} is not source-split")
            }
            Self::InvalidOpcodeCycle {
                opcode,
                opcode_cycle,
                stage_count,
            } => write!(
                formatter,
                "SMP checkpoint opcode ${opcode:02x} has non-resumable cycle {opcode_cycle} for {stage_count} stages"
            ),
        }
    }
}

impl std::error::Error for Snes9xSmpCoroutineCheckpointError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xStoreValue {
    A,
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xStoreIndex {
    None,
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xStorePlan {
    Direct {
        value: Snes9xStoreValue,
        index: Snes9xStoreIndex,
    },
    Absolute {
        value: Snes9xStoreValue,
        index: Snes9xStoreIndex,
    },
    IndirectX,
    AutoIncrementX,
    IndirectDpX,
    IndirectDpY,
    DirectWord,
    ImmediateDirect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xLoadValue {
    A,
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xLoadIndex {
    None,
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xAbsoluteOperandPlan {
    LowThenHigh,
    Combined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xLoadPlan {
    Direct {
        value: Snes9xLoadValue,
        index: Snes9xLoadIndex,
    },
    Absolute {
        value: Snes9xLoadValue,
        index: Snes9xLoadIndex,
        operands: Snes9xAbsoluteOperandPlan,
    },
    IndirectX {
        increment: bool,
    },
    IndirectDpX,
    IndirectDpY,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xBitPlan {
    ReadCarry,
    WriteCarry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xSplitPlan {
    /// An already source-checked non-store implementation below.
    ImplementedNonStore,
    Store(Snes9xStorePlan),
    Load(Snes9xLoadPlan),
    DirectCopy,
    Bit(Snes9xBitPlan),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Snes9xOpcodePlan {
    Atomic,
    Split(Snes9xSplitPlan),
}

// Pinned Snes9x 1.63 has exactly these 37 nested
// `switch(++opcode_cycle)` cases in `apu/bapu/smp/core/oppseudo_*.cpp`.
// Every other opcode is one atomic `op_step`, even when it consumes multiple
// SMP cycles. Keep this invocation as the single production authority for the
// split classifier; the macro also records which source stages are ported.
macro_rules! define_snes9x_opcode_plans {
    ($($opcode:literal => $plan:expr),+ $(,)?) => {
        const SNES9X_OPCODE_PLANS: [Snes9xOpcodePlan; 256] = {
            let mut plans = [Snes9xOpcodePlan::Atomic; 256];
            $(plans[$opcode] = Snes9xOpcodePlan::Split($plan);)+
            plans
        };
    };
}

define_snes9x_opcode_plans! {
    0x7e => Snes9xSplitPlan::ImplementedNonStore,
    0x8f => Snes9xSplitPlan::Store(Snes9xStorePlan::ImmediateDirect),
    0xaa => Snes9xSplitPlan::Bit(Snes9xBitPlan::ReadCarry),
    0xaf => Snes9xSplitPlan::Store(Snes9xStorePlan::AutoIncrementX),
    0xba => Snes9xSplitPlan::ImplementedNonStore,
    0xbf => Snes9xSplitPlan::Load(Snes9xLoadPlan::IndirectX { increment: true }),
    0xc4 => Snes9xSplitPlan::Store(Snes9xStorePlan::Direct {
        value: Snes9xStoreValue::A,
        index: Snes9xStoreIndex::None,
    }),
    0xc5 => Snes9xSplitPlan::Store(Snes9xStorePlan::Absolute {
        value: Snes9xStoreValue::A,
        index: Snes9xStoreIndex::None,
    }),
    0xc6 => Snes9xSplitPlan::Store(Snes9xStorePlan::IndirectX),
    0xc7 => Snes9xSplitPlan::Store(Snes9xStorePlan::IndirectDpX),
    0xc9 => Snes9xSplitPlan::Store(Snes9xStorePlan::Absolute {
        value: Snes9xStoreValue::X,
        index: Snes9xStoreIndex::None,
    }),
    0xca => Snes9xSplitPlan::Bit(Snes9xBitPlan::WriteCarry),
    0xcb => Snes9xSplitPlan::Store(Snes9xStorePlan::Direct {
        value: Snes9xStoreValue::Y,
        index: Snes9xStoreIndex::None,
    }),
    0xcc => Snes9xSplitPlan::Store(Snes9xStorePlan::Absolute {
        value: Snes9xStoreValue::Y,
        index: Snes9xStoreIndex::None,
    }),
    0xd4 => Snes9xSplitPlan::Store(Snes9xStorePlan::Direct {
        value: Snes9xStoreValue::A,
        index: Snes9xStoreIndex::X,
    }),
    0xd5 => Snes9xSplitPlan::Store(Snes9xStorePlan::Absolute {
        value: Snes9xStoreValue::A,
        index: Snes9xStoreIndex::X,
    }),
    0xd6 => Snes9xSplitPlan::Store(Snes9xStorePlan::Absolute {
        value: Snes9xStoreValue::A,
        index: Snes9xStoreIndex::Y,
    }),
    0xd7 => Snes9xSplitPlan::Store(Snes9xStorePlan::IndirectDpY),
    0xd8 => Snes9xSplitPlan::Store(Snes9xStorePlan::Direct {
        value: Snes9xStoreValue::X,
        index: Snes9xStoreIndex::None,
    }),
    0xd9 => Snes9xSplitPlan::Store(Snes9xStorePlan::Direct {
        value: Snes9xStoreValue::X,
        index: Snes9xStoreIndex::Y,
    }),
    0xda => Snes9xSplitPlan::Store(Snes9xStorePlan::DirectWord),
    0xdb => Snes9xSplitPlan::Store(Snes9xStorePlan::Direct {
        value: Snes9xStoreValue::Y,
        index: Snes9xStoreIndex::X,
    }),
    0xe4 => Snes9xSplitPlan::ImplementedNonStore,
    0xe5 => Snes9xSplitPlan::Load(Snes9xLoadPlan::Absolute {
        value: Snes9xLoadValue::A,
        index: Snes9xLoadIndex::None,
        operands: Snes9xAbsoluteOperandPlan::LowThenHigh,
    }),
    0xe6 => Snes9xSplitPlan::Load(Snes9xLoadPlan::IndirectX { increment: false }),
    0xe7 => Snes9xSplitPlan::Load(Snes9xLoadPlan::IndirectDpX),
    0xe9 => Snes9xSplitPlan::Load(Snes9xLoadPlan::Absolute {
        value: Snes9xLoadValue::X,
        index: Snes9xLoadIndex::None,
        operands: Snes9xAbsoluteOperandPlan::Combined,
    }),
    0xeb => Snes9xSplitPlan::ImplementedNonStore,
    0xec => Snes9xSplitPlan::Load(Snes9xLoadPlan::Absolute {
        value: Snes9xLoadValue::Y,
        index: Snes9xLoadIndex::None,
        operands: Snes9xAbsoluteOperandPlan::Combined,
    }),
    0xf4 => Snes9xSplitPlan::Load(Snes9xLoadPlan::Direct {
        value: Snes9xLoadValue::A,
        index: Snes9xLoadIndex::X,
    }),
    0xf5 => Snes9xSplitPlan::Load(Snes9xLoadPlan::Absolute {
        value: Snes9xLoadValue::A,
        index: Snes9xLoadIndex::X,
        operands: Snes9xAbsoluteOperandPlan::Combined,
    }),
    0xf6 => Snes9xSplitPlan::Load(Snes9xLoadPlan::Absolute {
        value: Snes9xLoadValue::A,
        index: Snes9xLoadIndex::Y,
        operands: Snes9xAbsoluteOperandPlan::Combined,
    }),
    0xf7 => Snes9xSplitPlan::Load(Snes9xLoadPlan::IndirectDpY),
    0xf8 => Snes9xSplitPlan::Load(Snes9xLoadPlan::Direct {
        value: Snes9xLoadValue::X,
        index: Snes9xLoadIndex::None,
    }),
    0xf9 => Snes9xSplitPlan::Load(Snes9xLoadPlan::Direct {
        value: Snes9xLoadValue::X,
        index: Snes9xLoadIndex::Y,
    }),
    0xfa => Snes9xSplitPlan::DirectCopy,
    0xfb => Snes9xSplitPlan::Load(Snes9xLoadPlan::Direct {
        value: Snes9xLoadValue::Y,
        index: Snes9xLoadIndex::X,
    }),
}

impl SmpCoroutineState {
    pub(crate) fn enabled() -> Self {
        Self {
            enabled: true,
            ..Self::default()
        }
    }

    pub(crate) fn is_enabled(self) -> bool {
        self.enabled
    }

    pub(crate) fn opcode(self) -> Option<u8> {
        self.opcode
    }

    pub(crate) fn is_idle(self) -> bool {
        self.opcode.is_none()
    }

    pub(crate) fn checkpoint(self) -> Option<Snes9xSmpCoroutineCheckpoint> {
        self.enabled.then_some(Snes9xSmpCoroutineCheckpoint {
            opcode: self.opcode,
            opcode_cycle: self.opcode_cycle,
            rd: self.rd,
            wr: self.wr,
            dp: self.dp,
            sp: self.sp,
            ya: self.ya,
            bit: self.bit,
            dsp_clock: 0,
        })
    }

    pub(crate) fn validate_checkpoint(
        checkpoint: &Snes9xSmpCoroutineCheckpoint,
    ) -> Result<(), Snes9xSmpCoroutineCheckpointError> {
        let Some(opcode) = checkpoint.opcode else {
            return if checkpoint.opcode_cycle == 0 {
                Ok(())
            } else {
                Err(Snes9xSmpCoroutineCheckpointError::IdleOpcodeCycle {
                    opcode_cycle: checkpoint.opcode_cycle,
                })
            };
        };
        let Some(stage_count) = Smp::snes9x_split_stage_count(opcode) else {
            return Err(Snes9xSmpCoroutineCheckpointError::UnsupportedOpcode { opcode });
        };
        if checkpoint.opcode_cycle == 0 || checkpoint.opcode_cycle >= stage_count {
            return Err(Snes9xSmpCoroutineCheckpointError::InvalidOpcodeCycle {
                opcode,
                opcode_cycle: checkpoint.opcode_cycle,
                stage_count,
            });
        }
        Ok(())
    }

    pub(crate) fn restore(
        checkpoint: Snes9xSmpCoroutineCheckpoint,
    ) -> Result<Self, Snes9xSmpCoroutineCheckpointError> {
        Self::validate_checkpoint(&checkpoint)?;
        Ok(Self {
            enabled: true,
            opcode: checkpoint.opcode,
            opcode_cycle: checkpoint.opcode_cycle,
            rd: checkpoint.rd,
            wr: checkpoint.wr,
            dp: checkpoint.dp,
            sp: checkpoint.sp,
            ya: checkpoint.ya,
            bit: checkpoint.bit,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmpMicroStepResult {
    InProgress { opcode: u8, opcode_cycle: u8 },
    InstructionComplete { opcode: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("SPC opcode ${opcode:02x} at ${pc:04x} has no resumable micro-step implementation")]
pub struct UnsupportedSmpMicroStep {
    pub opcode: u8,
    pub pc: u16,
}

pub(crate) struct Smp<'a> {
    bus: &'a mut dyn SmpBus,

    pub reg_pc: u16,
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_sp: u8,

    psw_c: bool,
    psw_z: bool,
    psw_h: bool,
    psw_p: bool,
    psw_v: bool,
    psw_n: bool,
    psw_i: bool,
    psw_b: bool,

    is_stopped: bool,

    // Pinned Snes9x keeps these address/data temporaries as machine state,
    // including across atomic `op_step` calls. The legacy complete-instruction
    // API discards them; the opt-in coroutine API imports and exports them.
    scratch_rd: u16,
    scratch_wr: u16,
    scratch_dp: u16,
    scratch_sp: u16,
    scratch_ya: u16,
    scratch_bit: u16,

    pub(crate) cycle_count: i32,
}

impl<'a> Smp<'a> {
    pub(crate) fn new(bus: &'a mut dyn SmpBus, state: SmpState) -> Self {
        Self {
            bus,
            reg_pc: state.pc,
            reg_a: state.a,
            reg_x: state.x,
            reg_y: state.y,
            reg_sp: state.sp,
            psw_c: state.c,
            psw_z: state.z,
            psw_h: state.h,
            psw_p: state.p,
            psw_v: state.v,
            psw_n: state.n,
            psw_i: state.i,
            psw_b: state.b,
            is_stopped: state.stopped,
            scratch_rd: 0,
            scratch_wr: 0,
            scratch_dp: 0,
            scratch_sp: 0,
            scratch_ya: 0,
            scratch_bit: 0,
            cycle_count: 0,
        }
    }

    fn import_coroutine_scratch(&mut self, coroutine: &SmpCoroutineState) {
        self.scratch_rd = coroutine.rd;
        self.scratch_wr = coroutine.wr;
        self.scratch_dp = coroutine.dp;
        self.scratch_sp = coroutine.sp;
        self.scratch_ya = coroutine.ya;
        self.scratch_bit = coroutine.bit;
    }

    fn export_coroutine_scratch(&self, coroutine: &mut SmpCoroutineState) {
        coroutine.rd = self.scratch_rd;
        coroutine.wr = self.scratch_wr;
        coroutine.dp = self.scratch_dp;
        coroutine.sp = self.scratch_sp;
        coroutine.ya = self.scratch_ya;
        coroutine.bit = self.scratch_bit;
    }

    pub(crate) fn state(&self) -> SmpState {
        SmpState {
            pc: self.reg_pc,
            a: self.reg_a,
            x: self.reg_x,
            y: self.reg_y,
            sp: self.reg_sp,
            c: self.psw_c,
            z: self.psw_z,
            h: self.psw_h,
            p: self.psw_p,
            v: self.psw_v,
            n: self.psw_n,
            i: self.psw_i,
            b: self.psw_b,
            stopped: self.is_stopped,
        }
    }

    pub(crate) fn run_instruction(&mut self) -> i32 {
        let start = self.cycle_count;
        self.run(start + 1);
        self.cycle_count - start
    }

    const fn snes9x_opcode_plan(opcode: u8) -> Snes9xOpcodePlan {
        SNES9X_OPCODE_PLANS[opcode as usize]
    }

    const fn snes9x_split_stage_count(opcode: u8) -> Option<u8> {
        match Self::snes9x_opcode_plan(opcode) {
            Snes9xOpcodePlan::Atomic => None,
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::ImplementedNonStore) => match opcode {
                0x7e | 0xe4 | 0xeb => Some(2),
                0xba => Some(3),
                _ => None,
            },
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::Store(plan)) => Some(match plan {
                Snes9xStorePlan::Direct { .. } | Snes9xStorePlan::ImmediateDirect => 3,
                Snes9xStorePlan::Absolute {
                    index: Snes9xStoreIndex::None,
                    ..
                }
                | Snes9xStorePlan::DirectWord => 4,
                Snes9xStorePlan::Absolute { .. } | Snes9xStorePlan::IndirectX => 3,
                Snes9xStorePlan::AutoIncrementX => 2,
                Snes9xStorePlan::IndirectDpX | Snes9xStorePlan::IndirectDpY => 5,
            }),
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::Load(plan)) => Some(match plan {
                Snes9xLoadPlan::Direct { .. } | Snes9xLoadPlan::IndirectX { .. } => 2,
                Snes9xLoadPlan::Absolute {
                    operands: Snes9xAbsoluteOperandPlan::LowThenHigh,
                    ..
                } => 3,
                Snes9xLoadPlan::Absolute {
                    operands: Snes9xAbsoluteOperandPlan::Combined,
                    ..
                } => 2,
                Snes9xLoadPlan::IndirectDpX | Snes9xLoadPlan::IndirectDpY => 4,
            }),
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::DirectCopy) => Some(4),
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::Bit(plan)) => Some(match plan {
                Snes9xBitPlan::ReadCarry => 2,
                Snes9xBitPlan::WriteCarry => 3,
            }),
        }
    }

    /// Execute exactly one Snes9x `SMP::op_step` boundary.
    ///
    /// The opcode fetch belongs to the first step. A later call resumes the
    /// retained pseudo-op case without fetching or completing any future case.
    /// Opcode `$8f` follows pinned Snes9x 1.63's generic `MOV dp,#imm` cases:
    /// fetch+operands, dummy direct-page read, then direct-page write.
    pub(crate) fn run_resumable_micro_step(
        &mut self,
        coroutine: &mut SmpCoroutineState,
    ) -> Result<SmpMicroStepResult, UnsupportedSmpMicroStep> {
        let opcode = match coroutine.opcode {
            Some(opcode) => opcode,
            None => {
                let opcode = self.read_pc();
                coroutine.opcode = Some(opcode);
                opcode
            }
        };

        match Self::snes9x_opcode_plan(opcode) {
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::Store(plan)) => {
                return self.run_snes9x_store_micro_step(opcode, plan, coroutine);
            }
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::Load(plan)) => {
                return self.run_snes9x_load_micro_step(opcode, plan, coroutine);
            }
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::Bit(plan)) => {
                return self.run_snes9x_bit_micro_step(opcode, plan, coroutine);
            }
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::DirectCopy) => {
                return self.run_snes9x_direct_copy_micro_step(opcode, coroutine);
            }
            Snes9xOpcodePlan::Split(Snes9xSplitPlan::ImplementedNonStore) => {}
            Snes9xOpcodePlan::Atomic => {
                self.import_coroutine_scratch(coroutine);
                if matches!(opcode, 0xef | 0xff) {
                    // Pinned Snes9x keeps SLEEP/STOP opt-in execution live by
                    // returning to the same opcode on every `op_step`.
                    self.cycles(2);
                    self.reg_pc = self.reg_pc.wrapping_sub(1);
                } else {
                    self.execute_opcode(opcode);
                }
                self.export_coroutine_scratch(coroutine);
                coroutine.opcode = None;
                coroutine.opcode_cycle = 0;
                return Ok(SmpMicroStepResult::InstructionComplete { opcode });
            }
        }

        match (opcode, coroutine.opcode_cycle) {
            (0xba, 0) => {
                coroutine.sp = u16::from(self.read_pc());
                coroutine.opcode_cycle = 1;
                Ok(SmpMicroStepResult::InProgress {
                    opcode,
                    opcode_cycle: 1,
                })
            }
            (0xba, 1) => {
                self.reg_a = self.read_dp(coroutine.sp as u8);
                self.cycles(1);
                coroutine.opcode_cycle = 2;
                Ok(SmpMicroStepResult::InProgress {
                    opcode,
                    opcode_cycle: 2,
                })
            }
            (0xba, 2) => {
                self.reg_y = self.read_dp((coroutine.sp as u8).wrapping_add(1));
                self.psw_n = (self.get_reg_ya() & 0x8000) != 0;
                self.psw_z = self.get_reg_ya() == 0;
                Self::snes9x_complete_split(coroutine, opcode)
            }
            (0xeb | 0xe4, 0) => {
                coroutine.sp = u16::from(self.read_pc());
                coroutine.opcode_cycle = 1;
                Ok(SmpMicroStepResult::InProgress {
                    opcode,
                    opcode_cycle: 1,
                })
            }
            (0xeb, 1) => {
                self.reg_y = self.read_dp(coroutine.sp as u8);
                self.set_psw_n_z(u32::from(self.reg_y));
                Self::snes9x_complete_split(coroutine, opcode)
            }
            (0xe4, 1) => {
                self.reg_a = self.read_dp(coroutine.sp as u8);
                self.set_psw_n_z(u32::from(self.reg_a));
                Self::snes9x_complete_split(coroutine, opcode)
            }
            (0x7e, 0) => {
                coroutine.dp = u16::from(self.read_pc());
                coroutine.opcode_cycle = 1;
                Ok(SmpMicroStepResult::InProgress {
                    opcode,
                    opcode_cycle: 1,
                })
            }
            (0x7e, 1) => {
                coroutine.rd = u16::from(self.read_dp(coroutine.dp as u8));
                self.reg_y = self.cmp(self.reg_y, coroutine.rd as u8);
                Self::snes9x_complete_split(coroutine, opcode)
            }
            _ => Err(UnsupportedSmpMicroStep {
                opcode,
                pc: self.reg_pc,
            }),
        }
    }

    fn snes9x_store_value(&self, value: Snes9xStoreValue) -> u8 {
        match value {
            Snes9xStoreValue::A => self.reg_a,
            Snes9xStoreValue::X => self.reg_x,
            Snes9xStoreValue::Y => self.reg_y,
        }
    }

    fn snes9x_store_index(&self, index: Snes9xStoreIndex) -> u16 {
        match index {
            Snes9xStoreIndex::None => 0,
            Snes9xStoreIndex::X => u16::from(self.reg_x),
            Snes9xStoreIndex::Y => u16::from(self.reg_y),
        }
    }

    fn snes9x_advance_split_stage(
        coroutine: &mut SmpCoroutineState,
        opcode: u8,
        opcode_cycle: u8,
    ) -> Result<SmpMicroStepResult, UnsupportedSmpMicroStep> {
        coroutine.opcode_cycle = opcode_cycle;
        Ok(SmpMicroStepResult::InProgress {
            opcode,
            opcode_cycle,
        })
    }

    fn snes9x_complete_split(
        coroutine: &mut SmpCoroutineState,
        opcode: u8,
    ) -> Result<SmpMicroStepResult, UnsupportedSmpMicroStep> {
        coroutine.opcode = None;
        coroutine.opcode_cycle = 0;
        Ok(SmpMicroStepResult::InstructionComplete { opcode })
    }

    /// Execute one complete pinned-Snes9x store pseudo-case. A case may contain
    /// several hardware cycles; `SMP::enter` can yield only after the case
    /// returns, so none of these source groupings may be split further.
    fn run_snes9x_store_micro_step(
        &mut self,
        opcode: u8,
        plan: Snes9xStorePlan,
        coroutine: &mut SmpCoroutineState,
    ) -> Result<SmpMicroStepResult, UnsupportedSmpMicroStep> {
        match (plan, coroutine.opcode_cycle) {
            (Snes9xStorePlan::Direct { index, .. }, 0) => {
                coroutine.dp = u16::from(self.read_pc());
                if index != Snes9xStoreIndex::None {
                    self.cycles(1);
                    coroutine.dp = coroutine.dp.wrapping_add(self.snes9x_store_index(index));
                }
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xStorePlan::Direct { .. }, 1) => {
                self.read_dp(coroutine.dp as u8);
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xStorePlan::Direct { value, .. }, 2) => {
                let value = self.snes9x_store_value(value);
                self.write_dp(coroutine.dp as u8, value);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (
                Snes9xStorePlan::Absolute {
                    index: Snes9xStoreIndex::None,
                    ..
                },
                0,
            ) => {
                coroutine.dp = u16::from(self.read_pc());
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (
                Snes9xStorePlan::Absolute {
                    index: Snes9xStoreIndex::None,
                    ..
                },
                1,
            ) => {
                coroutine.dp |= u16::from(self.read_pc()) << 8;
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (
                Snes9xStorePlan::Absolute {
                    index: Snes9xStoreIndex::None,
                    ..
                },
                2,
            ) => {
                self.read(coroutine.dp);
                Self::snes9x_advance_split_stage(coroutine, opcode, 3)
            }
            (
                Snes9xStorePlan::Absolute {
                    value,
                    index: Snes9xStoreIndex::None,
                },
                3,
            ) => {
                let value = self.snes9x_store_value(value);
                self.write(coroutine.dp, value);
                Self::snes9x_complete_split(coroutine, opcode)
            }
            (Snes9xStorePlan::Absolute { index, .. }, 0) => {
                coroutine.dp = u16::from(self.read_pc());
                coroutine.dp |= u16::from(self.read_pc()) << 8;
                self.cycles(1);
                coroutine.dp = coroutine.dp.wrapping_add(self.snes9x_store_index(index));
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xStorePlan::Absolute { .. }, 1) => {
                self.read(coroutine.dp);
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xStorePlan::Absolute { value, .. }, 2) => {
                let value = self.snes9x_store_value(value);
                self.write(coroutine.dp, value);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (Snes9xStorePlan::IndirectX, 0) => {
                self.cycles(1);
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xStorePlan::IndirectX, 1) => {
                self.read_dp(self.reg_x);
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xStorePlan::IndirectX, 2) => {
                self.write_dp(self.reg_x, self.reg_a);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (Snes9xStorePlan::AutoIncrementX, 0) => {
                self.cycles(2);
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xStorePlan::AutoIncrementX, 1) => {
                let address = self.reg_x;
                self.reg_x = self.reg_x.wrapping_add(1);
                self.write_dp(address, self.reg_a);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (Snes9xStorePlan::IndirectDpX, 0) => {
                coroutine.sp = u16::from(self.read_pc());
                self.cycles(1);
                coroutine.sp = coroutine.sp.wrapping_add(u16::from(self.reg_x));
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xStorePlan::IndirectDpX, 1) => {
                coroutine.dp = u16::from(self.read_dp(coroutine.sp as u8));
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xStorePlan::IndirectDpX, 2) => {
                coroutine.dp |= u16::from(self.read_dp((coroutine.sp as u8).wrapping_add(1))) << 8;
                Self::snes9x_advance_split_stage(coroutine, opcode, 3)
            }
            (Snes9xStorePlan::IndirectDpX, 3) => {
                self.read(coroutine.dp);
                Self::snes9x_advance_split_stage(coroutine, opcode, 4)
            }
            (Snes9xStorePlan::IndirectDpX, 4) => {
                self.write(coroutine.dp, self.reg_a);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (Snes9xStorePlan::IndirectDpY, 0) => {
                coroutine.sp = u16::from(self.read_pc());
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xStorePlan::IndirectDpY, 1) => {
                coroutine.dp = u16::from(self.read_dp(coroutine.sp as u8));
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xStorePlan::IndirectDpY, 2) => {
                coroutine.dp |= u16::from(self.read_dp((coroutine.sp as u8).wrapping_add(1))) << 8;
                self.cycles(1);
                coroutine.dp = coroutine.dp.wrapping_add(u16::from(self.reg_y));
                Self::snes9x_advance_split_stage(coroutine, opcode, 3)
            }
            (Snes9xStorePlan::IndirectDpY, 3) => {
                self.read(coroutine.dp);
                Self::snes9x_advance_split_stage(coroutine, opcode, 4)
            }
            (Snes9xStorePlan::IndirectDpY, 4) => {
                self.write(coroutine.dp, self.reg_a);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (Snes9xStorePlan::DirectWord, 0) => {
                coroutine.dp = u16::from(self.read_pc());
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xStorePlan::DirectWord, 1) => {
                self.read_dp(coroutine.dp as u8);
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xStorePlan::DirectWord, 2) => {
                self.write_dp(coroutine.dp as u8, self.reg_a);
                Self::snes9x_advance_split_stage(coroutine, opcode, 3)
            }
            (Snes9xStorePlan::DirectWord, 3) => {
                self.write_dp((coroutine.dp as u8).wrapping_add(1), self.reg_y);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (Snes9xStorePlan::ImmediateDirect, 0) => {
                coroutine.rd = u16::from(self.read_pc());
                coroutine.dp = u16::from(self.read_pc());
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xStorePlan::ImmediateDirect, 1) => {
                self.read_dp(coroutine.dp as u8);
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xStorePlan::ImmediateDirect, 2) => {
                self.write_dp(coroutine.dp as u8, coroutine.rd as u8);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            _ => Err(UnsupportedSmpMicroStep {
                opcode,
                pc: self.reg_pc,
            }),
        }
    }

    fn snes9x_load_index(&self, index: Snes9xLoadIndex) -> u16 {
        match index {
            Snes9xLoadIndex::None => 0,
            Snes9xLoadIndex::X => u16::from(self.reg_x),
            Snes9xLoadIndex::Y => u16::from(self.reg_y),
        }
    }

    fn snes9x_finish_load(&mut self, value: Snes9xLoadValue, loaded: u8) {
        match value {
            Snes9xLoadValue::A => self.reg_a = loaded,
            Snes9xLoadValue::X => self.reg_x = loaded,
            Snes9xLoadValue::Y => self.reg_y = loaded,
        }
        self.set_psw_n_z(u32::from(loaded));
    }

    /// Execute one complete pinned-Snes9x load pseudo-case. Address operands,
    /// internal cycles, pointer reads, and the final data read stay grouped as
    /// the source's nested `switch(++opcode_cycle)` cases group them.
    fn run_snes9x_load_micro_step(
        &mut self,
        opcode: u8,
        plan: Snes9xLoadPlan,
        coroutine: &mut SmpCoroutineState,
    ) -> Result<SmpMicroStepResult, UnsupportedSmpMicroStep> {
        match (plan, coroutine.opcode_cycle) {
            (Snes9xLoadPlan::Direct { index, .. }, 0) => {
                coroutine.sp = u16::from(self.read_pc());
                if index != Snes9xLoadIndex::None {
                    self.cycles(1);
                }
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xLoadPlan::Direct { value, index }, 1) => {
                let address =
                    (coroutine.sp as u8).wrapping_add(self.snes9x_load_index(index) as u8);
                let loaded = self.read_dp(address);
                self.snes9x_finish_load(value, loaded);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (
                Snes9xLoadPlan::Absolute {
                    operands: Snes9xAbsoluteOperandPlan::LowThenHigh,
                    ..
                },
                0,
            ) => {
                coroutine.sp = u16::from(self.read_pc());
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (
                Snes9xLoadPlan::Absolute {
                    operands: Snes9xAbsoluteOperandPlan::LowThenHigh,
                    ..
                },
                1,
            ) => {
                coroutine.sp |= u16::from(self.read_pc()) << 8;
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (
                Snes9xLoadPlan::Absolute {
                    value,
                    index,
                    operands: Snes9xAbsoluteOperandPlan::LowThenHigh,
                },
                2,
            ) => {
                let loaded = self.read(coroutine.sp.wrapping_add(self.snes9x_load_index(index)));
                self.snes9x_finish_load(value, loaded);
                Self::snes9x_complete_split(coroutine, opcode)
            }
            (
                Snes9xLoadPlan::Absolute {
                    index,
                    operands: Snes9xAbsoluteOperandPlan::Combined,
                    ..
                },
                0,
            ) => {
                coroutine.sp = u16::from(self.read_pc());
                coroutine.sp |= u16::from(self.read_pc()) << 8;
                if index != Snes9xLoadIndex::None {
                    self.cycles(1);
                }
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (
                Snes9xLoadPlan::Absolute {
                    value,
                    index,
                    operands: Snes9xAbsoluteOperandPlan::Combined,
                },
                1,
            ) => {
                let loaded = self.read(coroutine.sp.wrapping_add(self.snes9x_load_index(index)));
                self.snes9x_finish_load(value, loaded);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (Snes9xLoadPlan::IndirectX { .. }, 0) => {
                self.cycles(1);
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xLoadPlan::IndirectX { increment }, 1) => {
                let address = self.reg_x;
                if increment {
                    self.reg_x = self.reg_x.wrapping_add(1);
                }
                let loaded = self.read_dp(address);
                if increment {
                    self.cycles(1);
                }
                self.snes9x_finish_load(Snes9xLoadValue::A, loaded);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (Snes9xLoadPlan::IndirectDpX, 0) => {
                coroutine.dp = u16::from(self.read_pc()).wrapping_add(u16::from(self.reg_x));
                self.cycles(1);
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xLoadPlan::IndirectDpX, 1) => {
                coroutine.sp = u16::from(self.read_dp(coroutine.dp as u8));
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xLoadPlan::IndirectDpX, 2) => {
                coroutine.sp |= u16::from(self.read_dp((coroutine.dp as u8).wrapping_add(1))) << 8;
                Self::snes9x_advance_split_stage(coroutine, opcode, 3)
            }
            (Snes9xLoadPlan::IndirectDpX, 3) => {
                let loaded = self.read(coroutine.sp);
                self.snes9x_finish_load(Snes9xLoadValue::A, loaded);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            (Snes9xLoadPlan::IndirectDpY, 0) => {
                coroutine.dp = u16::from(self.read_pc());
                self.cycles(1);
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xLoadPlan::IndirectDpY, 1) => {
                coroutine.sp = u16::from(self.read_dp(coroutine.dp as u8));
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xLoadPlan::IndirectDpY, 2) => {
                coroutine.sp |= u16::from(self.read_dp((coroutine.dp as u8).wrapping_add(1))) << 8;
                Self::snes9x_advance_split_stage(coroutine, opcode, 3)
            }
            (Snes9xLoadPlan::IndirectDpY, 3) => {
                let loaded = self.read(coroutine.sp.wrapping_add(u16::from(self.reg_y)));
                self.snes9x_finish_load(Snes9xLoadValue::A, loaded);
                Self::snes9x_complete_split(coroutine, opcode)
            }

            _ => Err(UnsupportedSmpMicroStep {
                opcode,
                pc: self.reg_pc,
            }),
        }
    }

    fn run_snes9x_direct_copy_micro_step(
        &mut self,
        opcode: u8,
        coroutine: &mut SmpCoroutineState,
    ) -> Result<SmpMicroStepResult, UnsupportedSmpMicroStep> {
        match coroutine.opcode_cycle {
            0 => {
                coroutine.sp = u16::from(self.read_pc());
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            1 => {
                coroutine.rd = u16::from(self.read_dp(coroutine.sp as u8));
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            2 => {
                coroutine.dp = u16::from(self.read_pc());
                Self::snes9x_advance_split_stage(coroutine, opcode, 3)
            }
            3 => {
                self.write_dp(coroutine.dp as u8, coroutine.rd as u8);
                Self::snes9x_complete_split(coroutine, opcode)
            }
            _ => Err(UnsupportedSmpMicroStep {
                opcode,
                pc: self.reg_pc,
            }),
        }
    }

    fn run_snes9x_bit_micro_step(
        &mut self,
        opcode: u8,
        plan: Snes9xBitPlan,
        coroutine: &mut SmpCoroutineState,
    ) -> Result<SmpMicroStepResult, UnsupportedSmpMicroStep> {
        match (plan, coroutine.opcode_cycle) {
            (Snes9xBitPlan::ReadCarry, 0) => {
                coroutine.sp = u16::from(self.read_pc());
                coroutine.sp |= u16::from(self.read_pc()) << 8;
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xBitPlan::ReadCarry, 1) => {
                coroutine.bit = coroutine.sp >> 13;
                coroutine.sp &= 0x1fff;
                coroutine.rd = u16::from(self.read(coroutine.sp));
                self.psw_c = coroutine.rd & (1 << coroutine.bit) != 0;
                Self::snes9x_complete_split(coroutine, opcode)
            }
            (Snes9xBitPlan::WriteCarry, 0) => {
                coroutine.dp = u16::from(self.read_pc());
                coroutine.dp |= u16::from(self.read_pc()) << 8;
                Self::snes9x_advance_split_stage(coroutine, opcode, 1)
            }
            (Snes9xBitPlan::WriteCarry, 1) => {
                coroutine.bit = coroutine.dp >> 13;
                coroutine.dp &= 0x1fff;
                coroutine.rd = u16::from(self.read(coroutine.dp));
                if self.psw_c {
                    coroutine.rd |= 1 << coroutine.bit;
                } else {
                    coroutine.rd &= !(1 << coroutine.bit);
                }
                self.cycles(1);
                Self::snes9x_advance_split_stage(coroutine, opcode, 2)
            }
            (Snes9xBitPlan::WriteCarry, 2) => {
                self.write(coroutine.dp, coroutine.rd as u8);
                Self::snes9x_complete_split(coroutine, opcode)
            }
            _ => Err(UnsupportedSmpMicroStep {
                opcode,
                pc: self.reg_pc,
            }),
        }
    }

    pub fn reset(&mut self) {
        self.reg_pc = 0xffc0;
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.reg_sp = 0xef;

        self.set_psw(0x02);

        self.is_stopped = false;
    }

    pub fn set_reg_ya(&mut self, value: u16) {
        self.reg_a = value as u8;
        self.reg_y = (value >> 8) as u8;
    }

    pub fn get_reg_ya(&self) -> u16 {
        ((self.reg_y as u16) << 8) | (self.reg_a as u16)
    }

    pub fn set_psw(&mut self, value: u8) {
        self.psw_c = (value & 0x01) != 0;
        self.psw_z = (value & 0x02) != 0;
        self.psw_i = (value & 0x04) != 0;
        self.psw_h = (value & 0x08) != 0;
        self.psw_b = (value & 0x10) != 0;
        self.psw_p = (value & 0x20) != 0;
        self.psw_v = (value & 0x40) != 0;
        self.psw_n = (value & 0x80) != 0;
    }

    pub fn get_psw(&self) -> u8 {
        ((if self.psw_n { 1 } else { 0 }) << 7)
            | ((if self.psw_v { 1 } else { 0 }) << 6)
            | ((if self.psw_p { 1 } else { 0 }) << 5)
            | ((if self.psw_b { 1 } else { 0 }) << 4)
            | ((if self.psw_h { 1 } else { 0 }) << 3)
            | ((if self.psw_i { 1 } else { 0 }) << 2)
            | ((if self.psw_z { 1 } else { 0 }) << 1)
            | (if self.psw_c { 1 } else { 0 })
    }

    fn is_negative(value: u32) -> bool {
        (value & 0x80) != 0
    }

    fn cycles(&mut self, num_cycles: i32) {
        self.bus.cycles(num_cycles);
        self.cycle_count += num_cycles;
    }

    fn read(&mut self, addr: u16) -> u8 {
        self.cycle_count += 1;
        self.bus.read_cycle(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.cycle_count += 1;
        self.bus.write_cycle(addr, value);
    }

    pub(crate) fn read_pc(&mut self) -> u8 {
        let addr = self.reg_pc;
        let ret = self.read(addr);
        self.reg_pc = self.reg_pc.wrapping_add(1);
        ret
    }

    fn read_sp(&mut self) -> u8 {
        self.reg_sp = self.reg_sp.wrapping_add(1);
        let addr = 0x0100 | (self.reg_sp as u16);
        self.cycle_count += 1;
        self.bus.read_stack_cycle(addr)
    }

    fn write_sp(&mut self, value: u8) {
        let addr = 0x0100 | (self.reg_sp as u16);
        self.reg_sp = self.reg_sp.wrapping_sub(1);
        self.cycle_count += 1;
        self.bus.write_stack_cycle(addr, value);
    }

    fn read_dp(&mut self, addr: u8) -> u8 {
        let addr = (if self.psw_p { 0x0100 } else { 0 }) | (addr as u16);
        self.read(addr)
    }

    fn write_dp(&mut self, addr: u8, value: u8) {
        let addr = (if self.psw_p { 0x0100 } else { 0 }) | (addr as u16);
        self.write(addr, value);
    }

    fn set_psw_n_z(&mut self, x: u32) {
        self.psw_n = Smp::is_negative(x);
        self.psw_z = x == 0;
    }

    fn adc(&mut self, x: u8, y: u8) -> u8 {
        let x = x as i32;
        let y = y as i32;
        let r = x + y + (if self.psw_c { 1 } else { 0 });
        self.psw_n = Smp::is_negative(r as u32);
        self.psw_v = (!(x ^ y) & (x ^ r) & 0x80) != 0;
        self.psw_h = ((x ^ y ^ r) & 0x10) != 0;
        self.psw_z = (r as u8) == 0;
        self.psw_c = r > 0xff;
        r as u8
    }

    fn and(&mut self, x: u8, y: u8) -> u8 {
        let ret = x & y;
        self.set_psw_n_z(ret as u32);
        ret
    }

    fn asl(&mut self, x: u8) -> u8 {
        self.psw_c = Smp::is_negative(x as u32);
        let ret = x << 1;
        self.set_psw_n_z(ret as u32);
        ret
    }

    fn cmp(&mut self, x: u8, y: u8) -> u8 {
        let r = (x as i32) - (y as i32);
        self.psw_n = (r & 0x80) != 0;
        self.psw_z = (r as u8) == 0;
        self.psw_c = r >= 0;
        x
    }

    fn dec(&mut self, x: u8) -> u8 {
        let ret = x.wrapping_sub(1);
        self.set_psw_n_z(ret as u32);
        ret
    }

    fn eor(&mut self, x: u8, y: u8) -> u8 {
        let ret = x ^ y;
        self.set_psw_n_z(ret as u32);
        ret
    }

    fn inc(&mut self, x: u8) -> u8 {
        let ret = x.wrapping_add(1);
        self.set_psw_n_z(ret as u32);
        ret
    }

    fn ld(&mut self, _: u8, y: u8) -> u8 {
        self.set_psw_n_z(y as u32);
        y
    }

    fn lsr(&mut self, x: u8) -> u8 {
        self.psw_c = (x & 0x01) != 0;
        let ret = x >> 1;
        self.set_psw_n_z(ret as u32);
        ret
    }

    fn or(&mut self, x: u8, y: u8) -> u8 {
        let ret = x | y;
        self.set_psw_n_z(ret as u32);
        ret
    }

    fn rol(&mut self, x: u8) -> u8 {
        let carry = if self.psw_c { 1 } else { 0 };
        self.psw_c = (x & 0x80) != 0;
        let ret = (x << 1) | carry;
        self.set_psw_n_z(ret as u32);
        ret
    }

    fn ror(&mut self, x: u8) -> u8 {
        let carry = if self.psw_c { 0x80 } else { 0 };
        self.psw_c = (x & 0x01) != 0;
        let ret = carry | (x >> 1);
        self.set_psw_n_z(ret as u32);
        ret
    }

    fn sbc(&mut self, x: u8, y: u8) -> u8 {
        self.adc(x, !y)
    }

    fn st(&self, _: u8, y: u8) -> u8 {
        y
    }

    fn adw(&mut self, x: u16, y: u16) -> u16 {
        self.psw_c = false;
        let mut ret = self.adc(x as u8, y as u8) as u16;
        ret |= (self.adc((x >> 8) as u8, (y >> 8) as u8) as u16) << 8;
        self.psw_z = ret == 0;
        ret
    }

    fn cpw(&mut self, x: u16, y: u16) -> u16 {
        let r = (x as i32) - (y as i32);
        self.psw_n = (r & 0x8000) != 0;
        self.psw_z = (r as u16) == 0;
        self.psw_c = r >= 0;
        x
    }

    fn ldw(&mut self, _: u16, y: u16) -> u16 {
        self.psw_n = (y & 0x8000) != 0;
        self.psw_z = y == 0;
        y
    }

    fn sbw(&mut self, x: u16, y: u16) -> u16 {
        self.psw_c = true;
        let mut ret = self.sbc(x as u8, y as u8) as u16;
        ret |= (self.sbc((x >> 8) as u8, (y >> 8) as u8) as u16) << 8;
        self.psw_z = ret == 0;
        ret
    }

    fn adjust_dpw(&mut self, x: u16) {
        self.scratch_dp = self.read_pc() as u16;
        self.scratch_rd = (self.read_dp(self.scratch_dp as u8) as u16).wrapping_add(x);
        self.write_dp(self.scratch_dp as u8, self.scratch_rd as u8);
        self.scratch_dp = (self.scratch_dp as u8).wrapping_add(1) as u16;
        self.scratch_rd = self
            .scratch_rd
            .wrapping_add((self.read_dp(self.scratch_dp as u8) as u16) << 8);
        self.write_dp(self.scratch_dp as u8, (self.scratch_rd >> 8) as u8);
        self.psw_n = (self.scratch_rd & 0x8000) != 0;
        self.psw_z = self.scratch_rd == 0;
    }

    fn branch(&mut self, cond: bool) {
        self.scratch_rd = self.read_pc() as u16;
        if !cond {
            return;
        }
        self.cycles(2);
        self.reg_pc = self
            .reg_pc
            .wrapping_add(((self.scratch_rd as u8 as i8) as i16) as u16);
    }

    fn branch_bit(&mut self, x: u8) {
        self.scratch_dp = self.read_pc() as u16;
        self.scratch_sp = self.read_dp(self.scratch_dp as u8) as u16;
        self.scratch_rd = self.read_pc() as u16;
        self.cycles(1);
        if ((self.scratch_sp & (1 << ((x as i32) >> 5))) != 0) == ((x & 0x10) != 0) {
            return;
        }
        self.cycles(2);
        self.reg_pc = self
            .reg_pc
            .wrapping_add(((self.scratch_rd as u8 as i8) as i16) as u16);
    }

    fn push(&mut self, x: u8) {
        self.cycles(2);
        self.write_sp(x);
    }

    fn set_addr_bit(&mut self, opcode: u8) {
        self.scratch_dp = self.read_pc() as u16;
        self.scratch_dp |= (self.read_pc() as u16) << 8;
        self.scratch_bit = self.scratch_dp >> 13;
        self.scratch_dp &= 0x1fff;
        self.scratch_rd = self.read(self.scratch_dp) as u16;
        match opcode >> 5 {
            0 | 1 => {
                // orc addr:bit; orc !addr:bit
                self.cycles(1);
                self.psw_c |=
                    ((self.scratch_rd & (1 << self.scratch_bit)) != 0) ^ ((opcode & 0x20) != 0);
            }
            2 | 3 => {
                // and addr:bit; and larrd:bit
                self.psw_c &=
                    ((self.scratch_rd & (1 << self.scratch_bit)) != 0) ^ ((opcode & 0x20) != 0);
            }
            4 => {
                // eor addr:bit
                self.cycles(1);
                self.psw_c ^= (self.scratch_rd & (1 << self.scratch_bit)) != 0;
            }
            5 => {
                // ldc addr:bit
                self.psw_c = (self.scratch_rd & (1 << self.scratch_bit)) != 0;
            }
            6 => {
                // stc addr:bit
                self.cycles(1);
                self.scratch_rd = (self.scratch_rd & !(1 << self.scratch_bit))
                    | ((if self.psw_c { 1 } else { 0 }) << self.scratch_bit);
                self.write(self.scratch_dp, self.scratch_rd as u8);
            }
            7 => {
                // not addr:bit
                self.scratch_rd ^= 1 << self.scratch_bit;
                self.write(self.scratch_dp, self.scratch_rd as u8);
            }
            _ => unreachable!(),
        }
    }

    fn set_bit(&mut self, opcode: u8) {
        self.scratch_dp = self.read_pc() as u16;
        self.scratch_rd = self.read_dp(self.scratch_dp as u8) as u16;
        let mask = 1 << (opcode >> 5);
        if opcode & 0x10 == 0 {
            self.scratch_rd |= mask;
        } else {
            self.scratch_rd &= !mask;
        }
        self.write_dp(self.scratch_dp as u8, self.scratch_rd as u8);
    }

    fn test_addr(&mut self, x: bool) {
        self.scratch_dp = self.read_pc() as u16;
        self.scratch_dp |= (self.read_pc() as u16) << 8;
        self.scratch_rd = self.read(self.scratch_dp) as u16;
        let reg_a = self.reg_a;
        self.set_psw_n_z(reg_a.wrapping_sub(self.scratch_rd as u8) as u32);
        self.read(self.scratch_dp);
        self.write(
            self.scratch_dp,
            if x {
                self.scratch_rd as u8 | reg_a
            } else {
                self.scratch_rd as u8 & !reg_a
            },
        );
    }

    fn bne_dp(&mut self) {
        self.scratch_dp = self.read_pc() as u16;
        self.scratch_sp = self.read_dp(self.scratch_dp as u8) as u16;
        self.scratch_rd = self.read_pc() as u16;
        self.cycles(1);
        if self.reg_a == self.scratch_sp as u8 {
            return;
        }
        self.cycles(2);
        self.reg_pc = self
            .reg_pc
            .wrapping_add(((self.scratch_rd as u8 as i8) as i16) as u16);
    }

    fn bne_dp_dec(&mut self) {
        self.scratch_dp = self.read_pc() as u16;
        self.scratch_wr = self.read_dp(self.scratch_dp as u8).wrapping_sub(1) as u16;
        self.write_dp(self.scratch_dp as u8, self.scratch_wr as u8);
        self.scratch_rd = self.read_pc() as u16;
        if self.scratch_wr == 0 {
            return;
        }
        self.cycles(2);
        self.reg_pc = self
            .reg_pc
            .wrapping_add(((self.scratch_rd as u8 as i8) as i16) as u16);
    }

    fn bne_dp_x(&mut self) {
        self.scratch_dp = self.read_pc() as u16;
        self.cycles(1);
        self.scratch_sp = self.read_dp((self.scratch_dp as u8).wrapping_add(self.reg_x)) as u16;
        self.scratch_rd = self.read_pc() as u16;
        self.cycles(1);
        if self.reg_a == self.scratch_sp as u8 {
            return;
        }
        self.cycles(2);
        self.reg_pc = self
            .reg_pc
            .wrapping_add(((self.scratch_rd as u8 as i8) as i16) as u16);
    }

    fn bne_y_dec(&mut self) {
        self.scratch_rd = self.read_pc() as u16;
        self.cycles(1);
        self.reg_y = self.reg_y.wrapping_sub(1);
        self.cycles(1);
        if self.reg_y == 0 {
            return;
        }
        self.cycles(2);
        self.reg_pc = self
            .reg_pc
            .wrapping_add(((self.scratch_rd as u8 as i8) as i16) as u16);
    }

    fn brk(&mut self) {
        self.scratch_rd = self.read(0xffde) as u16;
        self.scratch_rd |= (self.read(0xffdf) as u16) << 8;
        self.cycles(2);
        let reg_pc = self.reg_pc;
        self.write_sp((reg_pc >> 8) as u8);
        self.write_sp(reg_pc as u8);
        let psw = self.get_psw();
        self.write_sp(psw);
        self.reg_pc = self.scratch_rd;
        self.psw_b = true;
        self.psw_i = false;
    }

    fn clv(&mut self) {
        self.cycles(1);
        self.psw_v = false;
        self.psw_h = false;
    }

    fn cmc(&mut self) {
        self.cycles(2);
        self.psw_c = !self.psw_c;
    }

    fn daa(&mut self) {
        self.cycles(2);
        if self.psw_c || self.reg_a > 0x99 {
            self.reg_a = self.reg_a.wrapping_add(0x60);
            self.psw_c = true;
        }
        if self.psw_h || (self.reg_a & 0x0f) > 0x09 {
            self.reg_a = self.reg_a.wrapping_add(0x06);
        }
        let reg_a = self.reg_a;
        self.set_psw_n_z(reg_a as u32);
    }

    fn das(&mut self) {
        self.cycles(2);
        if !self.psw_c || self.reg_a > 0x99 {
            self.reg_a = self.reg_a.wrapping_sub(0x60);
            self.psw_c = false;
        }
        if !self.psw_h || (self.reg_a & 0x0f) > 0x09 {
            self.reg_a = self.reg_a.wrapping_sub(0x06);
        }
        let reg_a = self.reg_a;
        self.set_psw_n_z(reg_a as u32);
    }

    fn div_ya(&mut self) {
        self.cycles(11);
        self.scratch_ya = self.get_reg_ya();
        self.psw_v = self.reg_y >= self.reg_x;
        self.psw_h = (self.reg_y & 0x0f) >= (self.reg_x & 0x0f);
        let reg_x = self.reg_x as u16;
        if (self.reg_y as u16) < (reg_x << 1) {
            self.reg_a = (self.scratch_ya / reg_x) as u8;
            self.reg_y = (self.scratch_ya % reg_x) as u8;
        } else {
            self.reg_a = (255 - (self.scratch_ya - (reg_x << 9)) / (256 - reg_x)) as u8;
            self.reg_y = (reg_x + (self.scratch_ya - (reg_x << 9)) % (256 - reg_x)) as u8;
        }
        let reg_a = self.reg_a;
        self.set_psw_n_z(reg_a as u32);
    }

    fn jmp_addr(&mut self) {
        self.scratch_rd = self.read_pc() as u16;
        self.scratch_rd |= (self.read_pc() as u16) << 8;
        self.reg_pc = self.scratch_rd;
    }

    fn jmp_i_addr_x(&mut self) {
        self.scratch_dp = self.read_pc() as u16;
        self.scratch_dp |= (self.read_pc() as u16) << 8;
        self.cycles(1);
        self.scratch_dp = self.scratch_dp.wrapping_add(self.reg_x as u16);
        self.scratch_rd = self.read(self.scratch_dp) as u16;
        self.scratch_rd |= (self.read(self.scratch_dp.wrapping_add(1)) as u16) << 8;
        self.reg_pc = self.scratch_rd;
    }

    fn jsp_dp(&mut self) {
        self.scratch_rd = self.read_pc() as u16;
        self.cycles(2);
        let reg_pc = self.reg_pc;
        self.write_sp((reg_pc >> 8) as u8);
        self.write_sp(reg_pc as u8);
        self.reg_pc = 0xff00 | self.scratch_rd;
    }

    fn jsr_addr(&mut self) {
        self.scratch_rd = self.read_pc() as u16;
        self.scratch_rd |= (self.read_pc() as u16) << 8;
        self.cycles(3);
        let reg_pc = self.reg_pc;
        self.write_sp((reg_pc >> 8) as u8);
        self.write_sp(reg_pc as u8);
        self.reg_pc = self.scratch_rd;
    }

    fn jst(&mut self, opcode: u8) {
        self.scratch_dp = 0xffde - (((opcode >> 4) << 1) as u16);
        self.scratch_rd = self.read(self.scratch_dp) as u16;
        self.scratch_rd |= (self.read(self.scratch_dp.wrapping_add(1)) as u16) << 8;
        self.cycles(3);
        let reg_pc = self.reg_pc;
        self.write_sp((reg_pc >> 8) as u8);
        self.write_sp(reg_pc as u8);
        self.reg_pc = self.scratch_rd;
    }

    fn lda_i_x_inc(&mut self) {
        self.cycles(1);
        let reg_x = self.reg_x;
        self.reg_a = self.read_dp(reg_x);
        self.reg_x = self.reg_x.wrapping_add(1);
        self.cycles(1);
        let reg_a = self.reg_a;
        self.set_psw_n_z(reg_a as u32);
    }

    fn mul_ya(&mut self) {
        self.cycles(8);
        self.scratch_ya = (self.reg_y as u16) * (self.reg_a as u16);
        self.reg_a = self.scratch_ya as u8;
        self.reg_y = (self.scratch_ya >> 8) as u8;
        let reg_y = self.reg_y;
        self.set_psw_n_z(reg_y as u32);
    }

    fn nop(&mut self) {
        self.cycles(1);
    }

    fn plp(&mut self) {
        self.cycles(2);
        let psw = self.read_sp();
        self.set_psw(psw);
    }

    fn rti(&mut self) {
        let psw = self.read_sp();
        self.set_psw(psw);
        self.scratch_rd = self.read_sp() as u16;
        self.scratch_rd |= (self.read_sp() as u16) << 8;
        self.cycles(2);
        self.reg_pc = self.scratch_rd;
    }

    fn rts(&mut self) {
        self.scratch_rd = self.read_sp() as u16;
        self.scratch_rd |= (self.read_sp() as u16) << 8;
        self.cycles(2);
        self.reg_pc = self.scratch_rd;
    }

    fn sta_i_dp_x(&mut self) {
        let mut addr = self.read_pc() + self.reg_x;
        self.cycles(1);
        let mut addr2 = self.read_dp(addr) as u16;
        addr = addr.wrapping_add(1);
        addr2 |= (self.read_dp(addr) as u16) << 8;
        self.read(addr2);
        let reg_a = self.reg_a;
        self.write(addr2, reg_a);
    }

    fn sta_i_dp_y(&mut self) {
        let mut addr = self.read_pc();
        let mut addr2 = self.read_dp(addr) as u16;
        addr = addr.wrapping_add(1);
        addr2 |= (self.read_dp(addr) as u16) << 8;
        self.cycles(1);
        addr2 = addr2.wrapping_add(self.reg_y as u16);
        self.read(addr2);
        let reg_a = self.reg_a;
        self.write(addr2, reg_a);
    }

    fn sta_i_x(&mut self) {
        self.cycles(1);
        let reg_x = self.reg_x;
        self.read_dp(reg_x);
        let reg_a = self.reg_a;
        self.write_dp(reg_x, reg_a);
    }

    fn sta_i_x_inc(&mut self) {
        self.cycles(2);
        let reg_x = self.reg_x;
        let reg_a = self.reg_a;
        self.write_dp(reg_x, reg_a);
        self.reg_x = self.reg_x.wrapping_add(1);
    }

    fn stw_dp(&mut self) {
        let mut addr = self.read_pc();
        self.read_dp(addr);
        let reg_a = self.reg_a;
        self.write_dp(addr, reg_a);
        addr = addr.wrapping_add(1);
        let reg_y = self.reg_y;
        self.write_dp(addr, reg_y);
    }

    fn sleep_stop(&mut self) {
        self.cycles(2);
        self.is_stopped = true;
    }

    fn xcn(&mut self) {
        self.cycles(4);
        self.reg_a = (self.reg_a >> 4) | (self.reg_a << 4);
        let reg_a = self.reg_a;
        self.set_psw_n_z(reg_a as u32);
    }

    fn execute_opcode(&mut self, opcode: u8) {
        macro_rules! adjust {
            ($op:ident, $x:expr) => {{
                self.cycles(1);
                let temp = $x;
                $x = self.$op(temp);
            }};
        }

        macro_rules! adjust_addr {
            ($op:ident) => {{
                self.scratch_dp = self.read_pc() as u16;
                self.scratch_dp |= (self.read_pc() as u16) << 8;
                self.scratch_rd = self.read(self.scratch_dp) as u16;
                self.scratch_rd = self.$op(self.scratch_rd as u8) as u16;
                self.write(self.scratch_dp, self.scratch_rd as u8);
            }};
        }

        macro_rules! adjust_dp {
            ($op:ident) => {{
                self.scratch_dp = self.read_pc() as u16;
                self.scratch_rd = self.read_dp(self.scratch_dp as u8) as u16;
                self.scratch_rd = self.$op(self.scratch_rd as u8) as u16;
                self.write_dp(self.scratch_dp as u8, self.scratch_rd as u8);
            }};
        }

        macro_rules! adjust_dp_x {
            ($op:ident) => {{
                self.scratch_dp = self.read_pc() as u16;
                self.cycles(1);
                self.scratch_rd =
                    self.read_dp((self.scratch_dp as u8).wrapping_add(self.reg_x)) as u16;
                self.scratch_rd = self.$op(self.scratch_rd as u8) as u16;
                self.write_dp(
                    (self.scratch_dp as u8).wrapping_add(self.reg_x),
                    self.scratch_rd as u8,
                );
            }};
        }

        macro_rules! read_addr {
            ($op:ident, $x:expr) => {{
                self.scratch_dp = self.read_pc() as u16;
                self.scratch_dp |= (self.read_pc() as u16) << 8;
                self.scratch_rd = self.read(self.scratch_dp) as u16;
                let temp = $x;
                $x = self.$op(temp, self.scratch_rd as u8);
            }};
        }

        macro_rules! read_addr_i {
            ($op:ident, $x:expr) => {{
                self.scratch_dp = self.read_pc() as u16;
                self.scratch_dp |= (self.read_pc() as u16) << 8;
                self.cycles(1);
                let temp = $x;
                self.scratch_rd = self.read(self.scratch_dp.wrapping_add(temp as u16)) as u16;
                let reg_a = self.reg_a;
                self.reg_a = self.$op(reg_a, self.scratch_rd as u8);
            }};
        }

        macro_rules! read_const {
            ($op:ident, $x:expr) => {{
                self.scratch_rd = self.read_pc() as u16;
                let temp = $x;
                $x = self.$op(temp, self.scratch_rd as u8);
            }};
        }

        macro_rules! read_dp {
            ($op:ident, $x:expr) => {{
                self.scratch_dp = self.read_pc() as u16;
                self.scratch_rd = self.read_dp(self.scratch_dp as u8) as u16;
                let temp = $x;
                $x = self.$op(temp, self.scratch_rd as u8);
            }};
        }

        macro_rules! read_dp_i {
            ($op:ident, $x:expr, $y:expr) => {{
                self.scratch_dp = self.read_pc() as u16;
                self.cycles(1);
                let index = $y;
                self.scratch_rd = self.read_dp((self.scratch_dp as u8).wrapping_add(index)) as u16;
                let destination = $x;
                $x = self.$op(destination, self.scratch_rd as u8);
            }};
        }

        macro_rules! read_dpw {
            ($op:ident, $is_cpw:expr) => {{
                self.scratch_dp = self.read_pc() as u16;
                self.scratch_rd = self.read_dp(self.scratch_dp as u8) as u16;
                if !$is_cpw {
                    self.cycles(1);
                }
                self.scratch_rd |=
                    (self.read_dp((self.scratch_dp as u8).wrapping_add(1)) as u16) << 8;
                let ya = self.get_reg_ya();
                let ya = self.$op(ya, self.scratch_rd);
                self.set_reg_ya(ya);
            }};
        }

        macro_rules! read_i_dp_x {
            ($op:ident) => {{
                self.scratch_dp = self.read_pc().wrapping_add(self.reg_x) as u16;
                self.cycles(1);
                self.scratch_sp = self.read_dp(self.scratch_dp as u8) as u16;
                self.scratch_sp |=
                    (self.read_dp((self.scratch_dp as u8).wrapping_add(1)) as u16) << 8;
                self.scratch_rd = self.read(self.scratch_sp) as u16;
                let reg_a = self.reg_a;
                self.reg_a = self.$op(reg_a, self.scratch_rd as u8);
            }};
        }

        macro_rules! read_i_dp_y {
            ($op:ident) => {{
                self.scratch_dp = self.read_pc() as u16;
                self.cycles(1);
                self.scratch_sp = self.read_dp(self.scratch_dp as u8) as u16;
                self.scratch_sp |=
                    (self.read_dp((self.scratch_dp as u8).wrapping_add(1)) as u16) << 8;
                self.scratch_rd =
                    self.read(self.scratch_sp.wrapping_add(u16::from(self.reg_y))) as u16;
                let reg_a = self.reg_a;
                self.reg_a = self.$op(reg_a, self.scratch_rd as u8);
            }};
        }

        macro_rules! read_i_x {
            ($op:ident) => {{
                self.cycles(1);
                self.scratch_rd = self.read_dp(self.reg_x) as u16;
                let reg_a = self.reg_a;
                self.reg_a = self.$op(reg_a, self.scratch_rd as u8);
            }};
        }

        macro_rules! set_flag {
            ($x:expr, $y:expr, $is_dest_psw_i:expr) => {{
                self.cycles(if $is_dest_psw_i { 2 } else { 1 });
                $x = $y;
            }};
        }

        macro_rules! transfer {
            ($x:expr, $y:expr, $is_dest_reg_sp:expr) => {{
                self.cycles(1);
                $y = $x;
                if !$is_dest_reg_sp {
                    let temp = $y;
                    self.set_psw_n_z(temp as u32);
                }
            }};
        }

        macro_rules! write_dp_const {
            ($op:ident, $is_cmp:expr) => {{
                self.scratch_rd = self.read_pc() as u16;
                self.scratch_dp = self.read_pc() as u16;
                self.scratch_wr = self.read_dp(self.scratch_dp as u8) as u16;
                self.scratch_wr = self.$op(self.scratch_wr as u8, self.scratch_rd as u8) as u16;
                if !$is_cmp {
                    self.write_dp(self.scratch_dp as u8, self.scratch_wr as u8);
                } else {
                    self.cycles(1);
                }
            }};
        }

        macro_rules! write_dp_dp {
            ($op:ident, $is_cmp:expr, $is_st:expr) => {{
                self.scratch_sp = self.read_pc() as u16;
                self.scratch_rd = self.read_dp(self.scratch_sp as u8) as u16;
                self.scratch_dp = self.read_pc() as u16;
                self.scratch_wr = if !$is_st {
                    self.read_dp(self.scratch_dp as u8) as u16
                } else {
                    0
                };
                self.scratch_wr = self.$op(self.scratch_wr as u8, self.scratch_rd as u8) as u16;
                if !$is_cmp {
                    self.write_dp(self.scratch_dp as u8, self.scratch_wr as u8);
                } else {
                    self.cycles(1);
                }
            }};
        }

        macro_rules! write_i_x_i_y {
            ($op:ident, $is_cmp:expr) => {{
                self.cycles(1);
                self.scratch_rd = self.read_dp(self.reg_y) as u16;
                self.scratch_wr = self.read_dp(self.reg_x) as u16;
                self.scratch_wr = self.$op(self.scratch_wr as u8, self.scratch_rd as u8) as u16;
                if !$is_cmp {
                    self.write_dp(self.reg_x, self.scratch_wr as u8);
                } else {
                    self.cycles(1);
                }
            }};
        }

        macro_rules! pull {
            ($x:expr) => {{
                self.cycles(2);
                $x = self.read_sp();
            }};
        }

        macro_rules! write_dp_imm {
            ($x:expr) => {{
                let addr = self.read_pc();
                self.read_dp(addr);
                let temp = $x;
                self.write_dp(addr, temp);
            }};
        }

        macro_rules! write_dp_i {
            ($x:expr, $y:expr) => {{
                let addr = self.read_pc() + $y;
                self.cycles(1);
                self.read_dp(addr);
                let temp = $x;
                self.write_dp(addr, temp);
            }};
        }

        macro_rules! write_addr {
            ($x:expr) => {{
                let mut addr = self.read_pc() as u16;
                addr |= (self.read_pc() as u16) << 8;
                self.read(addr);
                let temp = $x;
                self.write(addr, temp);
            }};
        }

        macro_rules! write_addr_i {
            ($x:expr) => {{
                let mut addr = self.read_pc() as u16;
                addr |= (self.read_pc() as u16) << 8;
                self.cycles(1);
                addr = addr.wrapping_add($x as u16);
                self.read(addr);
                let reg_a = self.reg_a;
                self.write(addr, reg_a);
            }};
        }

        match opcode {
            0x00 => self.nop(),
            0x01 => self.jst(opcode),
            0x02 => self.set_bit(opcode),
            0x03 => self.branch_bit(opcode),
            0x04 => read_dp!(or, self.reg_a),
            0x05 => read_addr!(or, self.reg_a),
            0x06 => read_i_x!(or),
            0x07 => read_i_dp_x!(or),
            0x08 => read_const!(or, self.reg_a),
            0x09 => write_dp_dp!(or, false, false),
            0x0a => self.set_addr_bit(opcode),
            0x0b => adjust_dp!(asl),
            0x0c => adjust_addr!(asl),
            0x0d => {
                let psw = self.get_psw();
                self.push(psw);
            }
            0x0e => self.test_addr(true),
            0x0f => self.brk(),

            0x10 => {
                let psw_n = self.psw_n;
                self.branch(!psw_n);
            }
            0x11 => self.jst(opcode),
            0x12 => self.set_bit(opcode),
            0x13 => self.branch_bit(opcode),
            0x14 => read_dp_i!(or, self.reg_a, self.reg_x),
            0x15 => read_addr_i!(or, self.reg_x),
            0x16 => read_addr_i!(or, self.reg_y),
            0x17 => read_i_dp_y!(or),
            0x18 => write_dp_const!(or, false),
            0x19 => write_i_x_i_y!(or, false),
            0x1a => self.adjust_dpw(!0),
            0x1b => adjust_dp_x!(asl),
            0x1c => adjust!(asl, self.reg_a),
            0x1d => adjust!(dec, self.reg_x),
            0x1e => read_addr!(cmp, self.reg_x),
            0x1f => self.jmp_i_addr_x(),

            0x20 => set_flag!(self.psw_p, false, false),
            0x21 => self.jst(opcode),
            0x22 => self.set_bit(opcode),
            0x23 => self.branch_bit(opcode),
            0x24 => read_dp!(and, self.reg_a),
            0x25 => read_addr!(and, self.reg_a),
            0x26 => read_i_x!(and),
            0x27 => read_i_dp_x!(and),
            0x28 => read_const!(and, self.reg_a),
            0x29 => write_dp_dp!(and, false, false),
            0x2a => self.set_addr_bit(opcode),
            0x2b => adjust_dp!(rol),
            0x2c => adjust_addr!(rol),
            0x2d => {
                let reg_a = self.reg_a;
                self.push(reg_a);
            }
            0x2e => self.bne_dp(),
            0x2f => self.branch(true),

            0x30 => {
                let psw_n = self.psw_n;
                self.branch(psw_n);
            }
            0x31 => self.jst(opcode),
            0x32 => self.set_bit(opcode),
            0x33 => self.branch_bit(opcode),
            0x34 => read_dp_i!(and, self.reg_a, self.reg_x),
            0x35 => read_addr_i!(and, self.reg_x),
            0x36 => read_addr_i!(and, self.reg_y),
            0x37 => read_i_dp_y!(and),
            0x38 => write_dp_const!(and, false),
            0x39 => write_i_x_i_y!(and, false),
            0x3a => self.adjust_dpw(1),
            0x3b => adjust_dp_x!(rol),
            0x3c => adjust!(rol, self.reg_a),
            0x3d => adjust!(inc, self.reg_x),
            0x3e => read_dp!(cmp, self.reg_x),
            0x3f => self.jsr_addr(),

            0x40 => set_flag!(self.psw_p, true, false),
            0x41 => self.jst(opcode),
            0x42 => self.set_bit(opcode),
            0x43 => self.branch_bit(opcode),
            0x44 => read_dp!(eor, self.reg_a),
            0x45 => read_addr!(eor, self.reg_a),
            0x46 => read_i_x!(eor),
            0x47 => read_i_dp_x!(eor),
            0x48 => read_const!(eor, self.reg_a),
            0x49 => write_dp_dp!(eor, false, false),
            0x4a => self.set_addr_bit(opcode),
            0x4b => adjust_dp!(lsr),
            0x4c => adjust_addr!(lsr),
            0x4d => {
                let reg_x = self.reg_x;
                self.push(reg_x);
            }
            0x4e => self.test_addr(false),
            0x4f => self.jsp_dp(),

            0x50 => {
                let psw_v = self.psw_v;
                self.branch(!psw_v);
            }
            0x51 => self.jst(opcode),
            0x52 => self.set_bit(opcode),
            0x53 => self.branch_bit(opcode),
            0x54 => read_dp_i!(eor, self.reg_a, self.reg_x),
            0x55 => read_addr_i!(eor, self.reg_x),
            0x56 => read_addr_i!(eor, self.reg_y),
            0x57 => read_i_dp_y!(eor),
            0x58 => write_dp_const!(eor, false),
            0x59 => write_i_x_i_y!(eor, false),
            0x5a => read_dpw!(cpw, true),
            0x5b => adjust_dp_x!(lsr),
            0x5c => adjust!(lsr, self.reg_a),
            0x5d => transfer!(self.reg_a, self.reg_x, false),
            0x5e => read_addr!(cmp, self.reg_y),
            0x5f => self.jmp_addr(),

            0x60 => set_flag!(self.psw_c, false, false),
            0x61 => self.jst(opcode),
            0x62 => self.set_bit(opcode),
            0x63 => self.branch_bit(opcode),
            0x64 => read_dp!(cmp, self.reg_a),
            0x65 => read_addr!(cmp, self.reg_a),
            0x66 => read_i_x!(cmp),
            0x67 => read_i_dp_x!(cmp),
            0x68 => read_const!(cmp, self.reg_a),
            0x69 => write_dp_dp!(cmp, true, false),
            0x6a => self.set_addr_bit(opcode),
            0x6b => adjust_dp!(ror),
            0x6c => adjust_addr!(ror),
            0x6d => {
                let reg_y = self.reg_y;
                self.push(reg_y);
            }
            0x6e => self.bne_dp_dec(),
            0x6f => self.rts(),

            0x70 => {
                let psw_v = self.psw_v;
                self.branch(psw_v);
            }
            0x71 => self.jst(opcode),
            0x72 => self.set_bit(opcode),
            0x73 => self.branch_bit(opcode),
            0x74 => read_dp_i!(cmp, self.reg_a, self.reg_x),
            0x75 => read_addr_i!(cmp, self.reg_x),
            0x76 => read_addr_i!(cmp, self.reg_y),
            0x77 => read_i_dp_y!(cmp),
            0x78 => write_dp_const!(cmp, true),
            0x79 => write_i_x_i_y!(cmp, true),
            0x7a => read_dpw!(adw, false),
            0x7b => adjust_dp_x!(ror),
            0x7c => adjust!(ror, self.reg_a),
            0x7d => transfer!(self.reg_x, self.reg_a, false),
            0x7e => read_dp!(cmp, self.reg_y),
            0x7f => self.rti(),

            0x80 => set_flag!(self.psw_c, true, false),
            0x81 => self.jst(opcode),
            0x82 => self.set_bit(opcode),
            0x83 => self.branch_bit(opcode),
            0x84 => read_dp!(adc, self.reg_a),
            0x85 => read_addr!(adc, self.reg_a),
            0x86 => read_i_x!(adc),
            0x87 => read_i_dp_x!(adc),
            0x88 => read_const!(adc, self.reg_a),
            0x89 => write_dp_dp!(adc, false, false),
            0x8a => self.set_addr_bit(opcode),
            0x8b => adjust_dp!(dec),
            0x8c => adjust_addr!(dec),
            0x8d => {
                self.reg_y = self.read_pc();
                self.set_psw_n_z(u32::from(self.reg_y));
            }
            0x8e => self.plp(),
            0x8f => write_dp_const!(st, false),

            0x90 => {
                let psw_c = self.psw_c;
                self.branch(!psw_c);
            }
            0x91 => self.jst(opcode),
            0x92 => self.set_bit(opcode),
            0x93 => self.branch_bit(opcode),
            0x94 => read_dp_i!(adc, self.reg_a, self.reg_x),
            0x95 => read_addr_i!(adc, self.reg_x),
            0x96 => read_addr_i!(adc, self.reg_y),
            0x97 => read_i_dp_y!(adc),
            0x98 => write_dp_const!(adc, false),
            0x99 => write_i_x_i_y!(adc, false),
            0x9a => read_dpw!(sbw, false),
            0x9b => adjust_dp_x!(dec),
            0x9c => adjust!(dec, self.reg_a),
            0x9d => transfer!(self.reg_sp, self.reg_x, false),
            0x9e => self.div_ya(),
            0x9f => self.xcn(),

            0xa0 => set_flag!(self.psw_i, true, true),
            0xa1 => self.jst(opcode),
            0xa2 => self.set_bit(opcode),
            0xa3 => self.branch_bit(opcode),
            0xa4 => read_dp!(sbc, self.reg_a),
            0xa5 => read_addr!(sbc, self.reg_a),
            0xa6 => read_i_x!(sbc),
            0xa7 => read_i_dp_x!(sbc),
            0xa8 => read_const!(sbc, self.reg_a),
            0xa9 => write_dp_dp!(sbc, false, false),
            0xaa => self.set_addr_bit(opcode),
            0xab => adjust_dp!(inc),
            0xac => adjust_addr!(inc),
            0xad => read_const!(cmp, self.reg_y),
            0xae => pull!(self.reg_a),
            0xaf => self.sta_i_x_inc(),

            0xb0 => {
                let psw_c = self.psw_c;
                self.branch(psw_c);
            }
            0xb1 => self.jst(opcode),
            0xb2 => self.set_bit(opcode),
            0xb3 => self.branch_bit(opcode),
            0xb4 => read_dp_i!(sbc, self.reg_a, self.reg_x),
            0xb5 => read_addr_i!(sbc, self.reg_x),
            0xb6 => read_addr_i!(sbc, self.reg_y),
            0xb7 => read_i_dp_y!(sbc),
            0xb8 => write_dp_const!(sbc, false),
            0xb9 => write_i_x_i_y!(sbc, false),
            0xba => read_dpw!(ldw, false),
            0xbb => adjust_dp_x!(inc),
            0xbc => adjust!(inc, self.reg_a),
            0xbd => transfer!(self.reg_x, self.reg_sp, true),
            0xbe => self.das(),
            0xbf => self.lda_i_x_inc(),

            0xc0 => set_flag!(self.psw_i, false, true),
            0xc1 => self.jst(opcode),
            0xc2 => self.set_bit(opcode),
            0xc3 => self.branch_bit(opcode),
            0xc4 => write_dp_imm!(self.reg_a),
            0xc5 => write_addr!(self.reg_a),
            0xc6 => self.sta_i_x(),
            0xc7 => self.sta_i_dp_x(),
            0xc8 => read_const!(cmp, self.reg_x),
            0xc9 => write_addr!(self.reg_x),
            0xca => self.set_addr_bit(opcode),
            0xcb => write_dp_imm!(self.reg_y),
            0xcc => write_addr!(self.reg_y),
            0xcd => {
                self.reg_x = self.read_pc();
                self.set_psw_n_z(u32::from(self.reg_x));
            }
            0xce => pull!(self.reg_x),
            0xcf => self.mul_ya(),

            0xd0 => {
                let psw_z = self.psw_z;
                self.branch(!psw_z);
            }
            0xd1 => self.jst(opcode),
            0xd2 => self.set_bit(opcode),
            0xd3 => self.branch_bit(opcode),
            0xd4 => write_dp_i!(self.reg_a, self.reg_x),
            0xd5 => write_addr_i!(self.reg_x),
            0xd6 => write_addr_i!(self.reg_y),
            0xd7 => self.sta_i_dp_y(),
            0xd8 => write_dp_imm!(self.reg_x),
            0xd9 => write_dp_i!(self.reg_x, self.reg_y),
            0xda => self.stw_dp(),
            0xdb => write_dp_i!(self.reg_y, self.reg_x),
            0xdc => adjust!(dec, self.reg_y),
            0xdd => transfer!(self.reg_y, self.reg_a, false),
            0xde => self.bne_dp_x(),
            0xdf => self.daa(),

            0xe0 => self.clv(),
            0xe1 => self.jst(opcode),
            0xe2 => self.set_bit(opcode),
            0xe3 => self.branch_bit(opcode),
            0xe4 => read_dp!(ld, self.reg_a),
            0xe5 => read_addr!(ld, self.reg_a),
            0xe6 => read_i_x!(ld),
            0xe7 => read_i_dp_x!(ld),
            0xe8 => {
                self.reg_a = self.read_pc();
                self.set_psw_n_z(u32::from(self.reg_a));
            }
            0xe9 => read_addr!(ld, self.reg_x),
            0xea => self.set_addr_bit(opcode),
            0xeb => read_dp!(ld, self.reg_y),
            0xec => read_addr!(ld, self.reg_y),
            0xed => self.cmc(),
            0xee => pull!(self.reg_y),
            0xef => self.sleep_stop(),

            0xf0 => {
                let psw_z = self.psw_z;
                self.branch(psw_z);
            }
            0xf1 => self.jst(opcode),
            0xf2 => self.set_bit(opcode),
            0xf3 => self.branch_bit(opcode),
            0xf4 => read_dp_i!(ld, self.reg_a, self.reg_x),
            0xf5 => read_addr_i!(ld, self.reg_x),
            0xf6 => read_addr_i!(ld, self.reg_y),
            0xf7 => read_i_dp_y!(ld),
            0xf8 => read_dp!(ld, self.reg_x),
            0xf9 => read_dp_i!(ld, self.reg_x, self.reg_y),
            0xfa => write_dp_dp!(st, false, true),
            0xfb => read_dp_i!(ld, self.reg_y, self.reg_x),
            0xfc => adjust!(inc, self.reg_y),
            0xfd => transfer!(self.reg_a, self.reg_y, false),
            0xfe => self.bne_y_dec(),
            0xff => self.sleep_stop(),
        }
    }

    pub fn run(&mut self, target_cycles: i32) -> i32 {
        self.cycle_count = 0;
        while self.cycle_count < target_cycles {
            if !self.is_stopped {
                let opcode = self.read_pc();
                self.execute_opcode(opcode);
            } else {
                self.cycles(2);
            }
        }

        self.cycle_count
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::Path;

    const OPCODE_LEDGER_PATH: &str =
        "../../external/snes9x-libretro/fixtures/snes9x-spc700-op-step-ledger.jsonl";

    fn decode_base64_bytes(encoded: &str) -> Vec<u8> {
        assert_eq!(encoded.len() % 4, 0);
        let value = |byte: u8| match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => 0,
            _ => panic!("invalid fixture base64 digit"),
        };
        let mut decoded = Vec::with_capacity(encoded.len() / 4 * 3);
        for chunk in encoded.as_bytes().chunks_exact(4) {
            let bits = u32::from(value(chunk[0])) << 18
                | u32::from(value(chunk[1])) << 12
                | u32::from(value(chunk[2])) << 6
                | u32::from(value(chunk[3]));
            decoded.push((bits >> 16) as u8);
            if chunk[2] != b'=' {
                decoded.push((bits >> 8) as u8);
            }
            if chunk[3] != b'=' {
                decoded.push(bits as u8);
            }
        }
        decoded
    }

    fn read_unsigned_varint(bytes: &[u8], index: &mut usize) -> u64 {
        let mut value = 0u64;
        let mut shift = 0;
        loop {
            let byte = bytes[*index];
            *index += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return value;
            }
            shift += 7;
            assert!(shift < 64);
        }
    }

    pub(crate) fn expand_ledger_sequence(sequence: &Value) -> Vec<Vec<i64>> {
        assert_eq!(
            sequence["encoding"],
            "columnar-signed-delta-zero-rle-varint-zstd-base64-v1"
        );
        let field_count = sequence["fields"].as_array().unwrap().len();
        let record_count = sequence["record_count"].as_u64().unwrap() as usize;
        let compressed = decode_base64_bytes(sequence["data_base64"].as_str().unwrap());
        let bytes = zstd::stream::decode_all(compressed.as_slice()).unwrap();
        let mut columns = Vec::with_capacity(field_count);
        let mut byte_index = 0;
        for _ in 0..field_count {
            let column_length = read_unsigned_varint(&bytes, &mut byte_index) as usize;
            let column_end = byte_index + column_length;
            let mut column = Vec::with_capacity(record_count);
            let mut previous = 0i64;
            while byte_index < column_end {
                let code = read_unsigned_varint(&bytes, &mut byte_index);
                if code == 0 {
                    let run_length = read_unsigned_varint(&bytes, &mut byte_index) as usize;
                    column.extend(std::iter::repeat_n(previous, run_length));
                } else {
                    let zigzag = code - 1;
                    let delta = ((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64);
                    previous += delta;
                    column.push(previous);
                }
            }
            assert_eq!(byte_index, column_end);
            assert_eq!(column.len(), record_count);
            columns.push(column);
        }
        assert_eq!(byte_index, bytes.len());

        let mut expanded = Vec::with_capacity(record_count * field_count * 8);
        let rows = (0..record_count)
            .map(|row| {
                (0..field_count)
                    .map(|field| {
                        let value = columns[field][row];
                        expanded.extend_from_slice(&value.to_le_bytes());
                        value
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let digest = format!("{:x}", Sha256::digest(expanded));
        assert_eq!(digest, sequence["expanded_sha256"].as_str().unwrap());
        rows
    }

    pub(crate) fn opcode_ledger() -> (Value, Value) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(OPCODE_LEDGER_PATH);
        let fixture =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let mut records = fixture
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap());
        let provenance = records.next().unwrap();
        let ledger = records.next().unwrap();
        assert!(records.next().is_none());
        assert_eq!(
            provenance["schema"],
            "pinned-snes9x-spc700-op-step-ledger-v1"
        );
        assert_eq!(provenance["classifier"]["atomic_opcode_count"], 219);
        assert_eq!(provenance["classifier"]["split_opcode_count"], 37);
        assert_eq!(provenance["classifier"]["total_case_count"], 327);
        assert_eq!(provenance["classifier"]["total_stage_count"], 439);
        assert_eq!(ledger["cases"].as_array().unwrap().len(), 327);
        (provenance, ledger)
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum BusAccess {
        Read(u16),
        Write(u16, u8),
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct LedgerBusEvent {
        kind: i64,
        address: i64,
        value: i64,
        clocks: i64,
        clock_before: i64,
        clock_after: i64,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestBus {
        memory: Vec<u8>,
        cycles: u32,
        accesses: Vec<BusAccess>,
        events: Vec<LedgerBusEvent>,
        read_overrides: Vec<(u16, u8)>,
    }

    impl Default for TestBus {
        fn default() -> Self {
            Self {
                memory: vec![0; 0x10000],
                cycles: 0,
                accesses: Vec::new(),
                events: Vec::new(),
                read_overrides: Vec::new(),
            }
        }
    }

    impl SmpBus for TestBus {
        fn cycles(&mut self, cycles: i32) {
            let before = self.cycles;
            self.cycles += cycles as u32;
            self.events.push(LedgerBusEvent {
                kind: 0,
                address: -1,
                value: -1,
                clocks: i64::from(cycles),
                clock_before: i64::from(before),
                clock_after: i64::from(self.cycles),
            });
        }

        fn read(&mut self, address: u16) -> u8 {
            self.accesses.push(BusAccess::Read(address));
            self.read_overrides
                .iter()
                .rev()
                .find_map(|&(override_address, value)| {
                    (override_address == address).then_some(value)
                })
                .unwrap_or(self.memory[usize::from(address)])
        }

        fn write(&mut self, address: u16, value: u8) {
            self.accesses.push(BusAccess::Write(address, value));
            self.memory[usize::from(address)] = value;
        }

        fn read_cycle(&mut self, address: u16) -> u8 {
            self.read_kind_cycle(1, address)
        }

        fn write_cycle(&mut self, address: u16, value: u8) {
            self.write_kind_cycle(2, address, value);
        }

        fn read_stack_cycle(&mut self, address: u16) -> u8 {
            self.read_kind_cycle(3, address)
        }

        fn write_stack_cycle(&mut self, address: u16, value: u8) {
            self.write_kind_cycle(4, address, value);
        }
    }

    impl TestBus {
        fn read_kind_cycle(&mut self, kind: i64, address: u16) -> u8 {
            let before = self.cycles;
            self.cycles += 1;
            let value = self.read(address);
            self.events.push(LedgerBusEvent {
                kind,
                address: i64::from(address),
                value: i64::from(value),
                clocks: 1,
                clock_before: i64::from(before),
                clock_after: i64::from(self.cycles),
            });
            value
        }

        fn write_kind_cycle(&mut self, kind: i64, address: u16, value: u8) {
            let before = self.cycles;
            self.cycles += 1;
            self.write(address, value);
            self.events.push(LedgerBusEvent {
                kind,
                address: i64::from(address),
                value: i64::from(value),
                clocks: 1,
                clock_before: i64::from(before),
                clock_after: i64::from(self.cycles),
            });
        }
    }

    #[derive(Clone, Debug)]
    struct StoreCase {
        opcode: u8,
        operands: &'static [u8],
        stage_cycles: &'static [u32],
        writes: &'static [(u16, u8)],
    }

    fn store_cases() -> Vec<StoreCase> {
        const DIRECT: &[u32] = &[2, 1, 1];
        const DIRECT_INDEXED: &[u32] = &[3, 1, 1];
        const ABSOLUTE: &[u32] = &[2, 1, 1, 1];
        const ABSOLUTE_INDEXED: &[u32] = &[4, 1, 1];
        vec![
            StoreCase {
                opcode: 0xc4,
                operands: &[0x30],
                stage_cycles: DIRECT,
                writes: &[(0x0030, 0xa1)],
            },
            StoreCase {
                opcode: 0xd8,
                operands: &[0x30],
                stage_cycles: DIRECT,
                writes: &[(0x0030, 0x02)],
            },
            StoreCase {
                opcode: 0xcb,
                operands: &[0x30],
                stage_cycles: DIRECT,
                writes: &[(0x0030, 0x03)],
            },
            StoreCase {
                opcode: 0xd4,
                operands: &[0x30],
                stage_cycles: DIRECT_INDEXED,
                writes: &[(0x0032, 0xa1)],
            },
            StoreCase {
                opcode: 0xd9,
                operands: &[0x30],
                stage_cycles: DIRECT_INDEXED,
                writes: &[(0x0033, 0x02)],
            },
            StoreCase {
                opcode: 0xdb,
                operands: &[0x30],
                stage_cycles: DIRECT_INDEXED,
                writes: &[(0x0032, 0x03)],
            },
            StoreCase {
                opcode: 0xc5,
                operands: &[0x34, 0x12],
                stage_cycles: ABSOLUTE,
                writes: &[(0x1234, 0xa1)],
            },
            StoreCase {
                opcode: 0xc9,
                operands: &[0x34, 0x12],
                stage_cycles: ABSOLUTE,
                writes: &[(0x1234, 0x02)],
            },
            StoreCase {
                opcode: 0xcc,
                operands: &[0x34, 0x12],
                stage_cycles: ABSOLUTE,
                writes: &[(0x1234, 0x03)],
            },
            StoreCase {
                opcode: 0xd5,
                operands: &[0x34, 0x12],
                stage_cycles: ABSOLUTE_INDEXED,
                writes: &[(0x1236, 0xa1)],
            },
            StoreCase {
                opcode: 0xd6,
                operands: &[0x34, 0x12],
                stage_cycles: ABSOLUTE_INDEXED,
                writes: &[(0x1237, 0xa1)],
            },
            StoreCase {
                opcode: 0xc6,
                operands: &[],
                stage_cycles: &[2, 1, 1],
                writes: &[(0x0002, 0xa1)],
            },
            StoreCase {
                opcode: 0xaf,
                operands: &[],
                stage_cycles: &[3, 1],
                writes: &[(0x0002, 0xa1)],
            },
            StoreCase {
                opcode: 0xc7,
                operands: &[0x30],
                stage_cycles: &[3, 1, 1, 1, 1],
                writes: &[(0x2000, 0xa1)],
            },
            StoreCase {
                opcode: 0xd7,
                operands: &[0x30],
                stage_cycles: &[2, 1, 2, 1, 1],
                writes: &[(0x1237, 0xa1)],
            },
            StoreCase {
                opcode: 0xda,
                operands: &[0x30],
                stage_cycles: &[2, 1, 1, 1],
                writes: &[(0x0030, 0xa1), (0x0031, 0x03)],
            },
            StoreCase {
                opcode: 0x8f,
                operands: &[0x5a, 0x30],
                stage_cycles: &[3, 1, 1],
                writes: &[(0x0030, 0x5a)],
            },
        ]
    }

    fn setup_case(case: &StoreCase) -> (TestBus, SmpState, SmpCoroutineState) {
        let mut bus = TestBus::default();
        let pc = 0x0200usize;
        bus.memory[pc] = case.opcode;
        bus.memory[pc + 1..pc + 1 + case.operands.len()].copy_from_slice(case.operands);
        bus.memory[0x0030] = 0x34;
        bus.memory[0x0031] = 0x12;
        bus.memory[0x0032] = 0x00;
        bus.memory[0x0033] = 0x20;
        let state = SmpState {
            pc: pc as u16,
            a: 0xa1,
            x: 0x02,
            y: 0x03,
            sp: 0xef,
            ..SmpState::default()
        };
        (bus, state, SmpCoroutineState::enabled())
    }

    const LEDGER_COVERED_SPLIT_OPCODES: [u8; 20] = [
        0x7e, 0xaa, 0xba, 0xbf, 0xca, 0xe4, 0xe5, 0xe6, 0xe7, 0xe9, 0xeb, 0xec, 0xf4, 0xf5, 0xf6,
        0xf7, 0xf8, 0xf9, 0xfa, 0xfb,
    ];

    fn setup_ledger_split_case(
        case: &Value,
        bus_rows: &[Vec<i64>],
    ) -> (TestBus, Vec<u8>, SmpState, SmpCoroutineState) {
        let case_id = case["id"].as_i64().unwrap();
        let seed = case["seed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_i64().unwrap())
            .collect::<Vec<_>>();
        let mut bus = TestBus::default();
        for (address, value) in bus.memory.iter_mut().enumerate() {
            *value =
                ((address * 37 + (address >> 8) * 13 + case_id as usize * 17 + 0x5a) & 0xff) as u8;
        }
        for override_ in case["ram_overrides"].as_array().unwrap() {
            let override_ = override_.as_array().unwrap();
            bus.memory[override_[0].as_u64().unwrap() as usize] =
                override_[1].as_u64().unwrap() as u8;
        }
        if seed[6] != 0 {
            for row in bus_rows
                .iter()
                .filter(|row| row[0] == case_id && row[2] == 1 && row[3] >= 0xffc0)
            {
                let entry = (row[3] as u16, row[4] as u8);
                if !bus.read_overrides.contains(&entry) {
                    bus.read_overrides.push(entry);
                }
            }
        }
        let initial_ram = bus.memory.clone();
        let state = SmpState {
            pc: seed[0] as u16,
            a: seed[1] as u8,
            x: seed[2] as u8,
            y: seed[3] as u8,
            sp: seed[4] as u8,
            c: seed[5] & 0x01 != 0,
            z: seed[5] & 0x02 != 0,
            i: seed[5] & 0x04 != 0,
            h: seed[5] & 0x08 != 0,
            b: seed[5] & 0x10 != 0,
            p: seed[5] & 0x20 != 0,
            v: seed[5] & 0x40 != 0,
            n: seed[5] & 0x80 != 0,
            stopped: false,
        };
        let coroutine = SmpCoroutineState {
            enabled: true,
            opcode: None,
            opcode_cycle: 0,
            rd: 0xa101,
            wr: 0xa202,
            dp: 0xa303,
            sp: 0xa404,
            ya: 0xa505,
            bit: 0xa606,
        };
        (bus, initial_ram, state, coroutine)
    }

    #[test]
    fn canonical_snes9x_ledger_uses_persistent_atomic_scratch_owner() {
        let (provenance, ledger) = opcode_ledger();
        let state_rows = expand_ledger_sequence(&ledger["state_sequence"]);
        let dsp_rows = expand_ledger_sequence(&ledger["dsp_state_sequence"]);
        let bus_rows = expand_ledger_sequence(&ledger["bus_event_sequence"]);
        let ram_diff_rows = expand_ledger_sequence(&ledger["ram_diff_sequence"]);
        let dsp_diff_rows = expand_ledger_sequence(&ledger["dsp_diff_sequence"]);
        assert_eq!(state_rows.len(), 439);
        assert_eq!(dsp_rows.len(), 439 * 128);
        assert_eq!(
            provenance["classifier"]["split_opcodes_and_stages"]
                .as_array()
                .unwrap()
                .len(),
            37
        );
        assert!(!bus_rows.is_empty());
        assert!(!ram_diff_rows.is_empty());
        assert!(!dsp_diff_rows.is_empty());

        let cases = ledger["cases"].as_array().unwrap();
        let mut state_index = 0;
        for case in cases {
            let case_id = case["id"].as_i64().unwrap();
            let opcode = case["opcode"].as_u64().unwrap() as u8;
            let expected_stages = case["expected_stages"].as_u64().unwrap() as usize;
            assert_eq!(
                matches!(Smp::snes9x_opcode_plan(opcode), Snes9xOpcodePlan::Split(_)),
                expected_stages > 1,
                "fixture and production classifier disagree for ${opcode:02x}"
            );
            for stage in 0..expected_stages {
                let row = &state_rows[state_index];
                assert_eq!(row[0], case_id);
                assert_eq!(row[1], stage as i64);
                state_index += 1;
            }
        }
        assert_eq!(state_index, state_rows.len());

        // Canonical atomic TCALL 0 ($01) is the first ledger case whose
        // source-visible scratch changes. The instruction itself is already
        // implemented, but Rust's atomic engine keeps addressing temporaries
        // in local variables and therefore cannot publish the pinned
        // `rd/wr/dp/sp/ya/bit` continuation state.
        let case = &cases[1];
        assert_eq!(case["opcode"], 0x01);
        assert_eq!(case["expected_stages"], 1);
        let expected = state_rows
            .iter()
            .find(|row| row[0] == 1 && row[1] == 0)
            .unwrap();
        let seed = case["seed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_i64().unwrap())
            .collect::<Vec<_>>();
        let mut bus = TestBus::default();
        for (address, value) in bus.memory.iter_mut().enumerate() {
            *value = ((address * 37 + (address >> 8) * 13 + 17 + 0x5a) & 0xff) as u8;
        }
        for override_ in case["ram_overrides"].as_array().unwrap() {
            let override_ = override_.as_array().unwrap();
            bus.memory[override_[0].as_u64().unwrap() as usize] =
                override_[1].as_u64().unwrap() as u8;
        }
        let mut coroutine = SmpCoroutineState {
            enabled: true,
            opcode: None,
            opcode_cycle: 0,
            rd: 0xa101,
            wr: 0xa202,
            dp: 0xa303,
            sp: 0xa404,
            ya: 0xa505,
            bit: 0xa606,
        };
        let mut smp = Smp::new(
            &mut bus,
            SmpState {
                pc: seed[0] as u16,
                a: seed[1] as u8,
                x: seed[2] as u8,
                y: seed[3] as u8,
                sp: seed[4] as u8,
                c: seed[5] & 0x01 != 0,
                z: seed[5] & 0x02 != 0,
                i: seed[5] & 0x04 != 0,
                h: seed[5] & 0x08 != 0,
                b: seed[5] & 0x10 != 0,
                p: seed[5] & 0x20 != 0,
                v: seed[5] & 0x40 != 0,
                n: seed[5] & 0x80 != 0,
                stopped: false,
            },
        );
        assert_eq!(
            smp.run_resumable_micro_step(&mut coroutine).unwrap(),
            SmpMicroStepResult::InstructionComplete { opcode: 0x01 }
        );
        assert_eq!(smp.cycle_count, expected[17] as i32);
        let actual_state = smp.state();
        assert_eq!(actual_state.pc, expected[3] as u16);
        assert_eq!(actual_state.sp, expected[7] as u8);

        let expected_scratch = (
            expected[11] as u16,
            expected[12] as u16,
            expected[13] as u16,
            expected[14] as u16,
            expected[15] as u16,
            expected[16] as u16,
        );
        assert_eq!(
            (
                coroutine.rd,
                coroutine.wr,
                coroutine.dp,
                coroutine.sp,
                coroutine.ya,
                coroutine.bit,
            ),
            expected_scratch,
            "generic atomic fallback cannot be enabled until the atomic core publishes source scratch"
        );
    }

    #[test]
    fn canonical_snes9x_atomic_opcode_ledger_matches_exactly() {
        let (_, ledger) = opcode_ledger();
        let state_rows = expand_ledger_sequence(&ledger["state_sequence"]);
        let bus_rows = expand_ledger_sequence(&ledger["bus_event_sequence"]);
        let ram_diff_rows = expand_ledger_sequence(&ledger["ram_diff_sequence"]);
        let cases = ledger["cases"].as_array().unwrap();
        let mut state_index = 0;

        for case in cases.iter().take(292) {
            let case_id = case["id"].as_i64().unwrap();
            let opcode = case["opcode"].as_u64().unwrap() as u8;
            let expected_stages = case["expected_stages"].as_u64().unwrap() as usize;
            let expected = &state_rows[state_index];
            state_index += expected_stages;
            if !matches!(Smp::snes9x_opcode_plan(opcode), Snes9xOpcodePlan::Atomic) {
                continue;
            }

            let seed = case["seed"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_i64().unwrap())
                .collect::<Vec<_>>();
            let mut bus = TestBus::default();
            for (address, value) in bus.memory.iter_mut().enumerate() {
                *value = ((address * 37 + (address >> 8) * 13 + case_id as usize * 17 + 0x5a)
                    & 0xff) as u8;
            }
            for override_ in case["ram_overrides"].as_array().unwrap() {
                let override_ = override_.as_array().unwrap();
                bus.memory[override_[0].as_u64().unwrap() as usize] =
                    override_[1].as_u64().unwrap() as u8;
            }
            let initial_ram = bus.memory.clone();
            let mut coroutine = SmpCoroutineState {
                enabled: true,
                opcode: None,
                opcode_cycle: 0,
                rd: 0xa101,
                wr: 0xa202,
                dp: 0xa303,
                sp: 0xa404,
                ya: 0xa505,
                bit: 0xa606,
            };
            let (actual_state, actual_psw, actual_cycles, result) = {
                let mut smp = Smp::new(
                    &mut bus,
                    SmpState {
                        pc: seed[0] as u16,
                        a: seed[1] as u8,
                        x: seed[2] as u8,
                        y: seed[3] as u8,
                        sp: seed[4] as u8,
                        c: seed[5] & 0x01 != 0,
                        z: seed[5] & 0x02 != 0,
                        i: seed[5] & 0x04 != 0,
                        h: seed[5] & 0x08 != 0,
                        b: seed[5] & 0x10 != 0,
                        p: seed[5] & 0x20 != 0,
                        v: seed[5] & 0x40 != 0,
                        n: seed[5] & 0x80 != 0,
                        stopped: false,
                    },
                );
                let result = smp.run_resumable_micro_step(&mut coroutine).unwrap();
                (smp.state(), smp.get_psw(), smp.cycle_count, result)
            };
            assert_eq!(
                result,
                SmpMicroStepResult::InstructionComplete { opcode },
                "case {case_id} ${opcode:02x}"
            );
            assert_eq!(
                [
                    i64::from(actual_state.pc),
                    i64::from(actual_state.a),
                    i64::from(actual_state.x),
                    i64::from(actual_state.y),
                    i64::from(actual_state.sp),
                    i64::from(actual_psw),
                    i64::from(opcode),
                    0,
                    i64::from(coroutine.rd),
                    i64::from(coroutine.wr),
                    i64::from(coroutine.dp),
                    i64::from(coroutine.sp),
                    i64::from(coroutine.ya),
                    i64::from(coroutine.bit),
                    i64::from(actual_cycles),
                ],
                expected[3..18],
                "state mismatch in case {case_id} ${opcode:02x} {}",
                case["variant"].as_str().unwrap()
            );

            let expected_bus = bus_rows
                .iter()
                .filter(|row| row[0] == case_id && row[1] == 0)
                .map(|row| row[2..8].to_vec())
                .collect::<Vec<_>>();
            let actual_bus = bus
                .events
                .iter()
                .map(|event| {
                    vec![
                        event.kind,
                        event.address,
                        event.value,
                        event.clocks,
                        event.clock_before,
                        event.clock_after,
                    ]
                })
                .collect::<Vec<_>>();
            assert_eq!(
                actual_bus, expected_bus,
                "bus mismatch in case {case_id} ${opcode:02x}"
            );

            let expected_ram = ram_diff_rows
                .iter()
                .filter(|row| row[0] == case_id && row[1] == 0)
                .map(|row| (row[2] as usize, row[3] as u8))
                .collect::<Vec<_>>();
            let actual_ram = bus
                .memory
                .iter()
                .zip(&initial_ram)
                .enumerate()
                .filter_map(|(address, (&actual, &before))| {
                    (actual != before).then_some((address, actual))
                })
                .collect::<Vec<_>>();
            assert_eq!(
                actual_ram, expected_ram,
                "RAM mismatch in case {case_id} ${opcode:02x}"
            );
        }
    }

    #[test]
    fn ledger_covered_split_plans_match_every_controlled_ledger_stage() {
        let (_, ledger) = opcode_ledger();
        let state_rows = expand_ledger_sequence(&ledger["state_sequence"]);
        let bus_rows = expand_ledger_sequence(&ledger["bus_event_sequence"]);
        let ram_diff_rows = expand_ledger_sequence(&ledger["ram_diff_sequence"]);
        let cases = ledger["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|case| {
                LEDGER_COVERED_SPLIT_OPCODES.contains(&(case["opcode"].as_u64().unwrap() as u8))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            cases.len(),
            22,
            "canonical cases plus carry and IPL variants"
        );

        for case in cases {
            let case_id = case["id"].as_i64().unwrap();
            let opcode = case["opcode"].as_u64().unwrap() as u8;
            let expected_stages = case["expected_stages"].as_u64().unwrap() as usize;
            let (mut bus, initial_ram, mut state, mut coroutine) =
                setup_ledger_split_case(case, &bus_rows);

            for stage in 0..expected_stages {
                let event_start = bus.events.len();
                let (result, _) = step(&mut bus, &mut state, &mut coroutine);
                let expected_result = if stage + 1 == expected_stages {
                    SmpMicroStepResult::InstructionComplete { opcode }
                } else {
                    SmpMicroStepResult::InProgress {
                        opcode,
                        opcode_cycle: (stage + 1) as u8,
                    }
                };
                assert_eq!(
                    result, expected_result,
                    "case {case_id} ${opcode:02x} stage {stage} result"
                );

                let expected = state_rows
                    .iter()
                    .find(|row| row[0] == case_id && row[1] == stage as i64)
                    .unwrap();
                let actual_psw = {
                    let smp = Smp::new(&mut bus, state);
                    smp.get_psw()
                };
                assert_eq!(
                    [
                        i64::from(state.pc),
                        i64::from(state.a),
                        i64::from(state.x),
                        i64::from(state.y),
                        i64::from(state.sp),
                        i64::from(actual_psw),
                        i64::from(opcode),
                        i64::from(coroutine.opcode_cycle),
                        i64::from(coroutine.rd),
                        i64::from(coroutine.wr),
                        i64::from(coroutine.dp),
                        i64::from(coroutine.sp),
                        i64::from(coroutine.ya),
                        i64::from(coroutine.bit),
                        i64::from(bus.cycles),
                    ],
                    expected[3..18],
                    "case {case_id} ${opcode:02x} stage {stage} state/scratch"
                );

                let expected_bus = bus_rows
                    .iter()
                    .filter(|row| row[0] == case_id && row[1] == stage as i64)
                    .map(|row| row[2..8].to_vec())
                    .collect::<Vec<_>>();
                let actual_bus = bus.events[event_start..]
                    .iter()
                    .map(|event| {
                        vec![
                            event.kind,
                            event.address,
                            event.value,
                            event.clocks,
                            event.clock_before,
                            event.clock_after,
                        ]
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    actual_bus, expected_bus,
                    "case {case_id} ${opcode:02x} stage {stage} bus order"
                );

                let expected_ram = ram_diff_rows
                    .iter()
                    .filter(|row| row[0] == case_id && row[1] == stage as i64)
                    .map(|row| (row[2] as usize, row[3] as u8))
                    .collect::<Vec<_>>();
                let actual_ram = bus
                    .memory
                    .iter()
                    .zip(&initial_ram)
                    .enumerate()
                    .filter_map(|(address, (&actual, &before))| {
                        (actual != before).then_some((address, actual))
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    actual_ram, expected_ram,
                    "case {case_id} ${opcode:02x} stage {stage} RAM"
                );
            }
            assert!(coroutine.is_idle(), "case {case_id} ${opcode:02x}");
        }
    }

    #[test]
    fn every_ledger_covered_split_continuation_round_trips_without_replay() {
        let (_, ledger) = opcode_ledger();
        let bus_rows = expand_ledger_sequence(&ledger["bus_event_sequence"]);
        let cases = ledger["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|case| {
                LEDGER_COVERED_SPLIT_OPCODES.contains(&(case["opcode"].as_u64().unwrap() as u8))
            })
            .collect::<Vec<_>>();

        for case in cases {
            let case_id = case["id"].as_i64().unwrap();
            let opcode = case["opcode"].as_u64().unwrap() as u8;
            let expected_stages = case["expected_stages"].as_u64().unwrap() as usize;
            let (mut expected_bus, _, mut expected_state, mut expected_coroutine) =
                setup_ledger_split_case(case, &bus_rows);
            run_to_completion(
                &mut expected_bus,
                &mut expected_state,
                &mut expected_coroutine,
            );

            for steps_before_checkpoint in 1..expected_stages {
                let (mut bus, _, mut state, mut coroutine) =
                    setup_ledger_split_case(case, &bus_rows);
                for _ in 0..steps_before_checkpoint {
                    assert!(matches!(
                        step(&mut bus, &mut state, &mut coroutine).0,
                        SmpMicroStepResult::InProgress { .. }
                    ));
                }
                let prefix_events = bus.events.clone();
                let checkpoint: Snes9xSmpCoroutineCheckpoint = serde_json::from_slice(
                    &serde_json::to_vec(&coroutine.checkpoint().unwrap()).unwrap(),
                )
                .unwrap();
                let mut restored = SmpCoroutineState::restore(checkpoint).unwrap();
                run_to_completion(&mut bus, &mut state, &mut restored);

                assert_eq!(
                    &bus.events[..prefix_events.len()],
                    prefix_events,
                    "case {case_id} ${opcode:02x} replayed prefix after stage {steps_before_checkpoint}"
                );
                assert_eq!(
                    bus, expected_bus,
                    "case {case_id} ${opcode:02x} bus after stage {steps_before_checkpoint}"
                );
                assert_eq!(
                    state, expected_state,
                    "case {case_id} ${opcode:02x} state after stage {steps_before_checkpoint}"
                );
                assert_eq!(
                    restored, expected_coroutine,
                    "case {case_id} ${opcode:02x} continuation after stage {steps_before_checkpoint}"
                );
            }
        }
    }

    #[test]
    fn split_completion_checkpoint_retains_scratch_for_the_next_opcode() {
        let (_, ledger) = opcode_ledger();
        let bus_rows = expand_ledger_sequence(&ledger["bus_event_sequence"]);
        let case = ledger["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == 170)
            .unwrap();
        assert_eq!(case["opcode"], 0xaa);
        let (mut bus, _, mut state, mut coroutine) = setup_ledger_split_case(case, &bus_rows);
        run_to_completion(&mut bus, &mut state, &mut coroutine);
        assert!(coroutine.is_idle());
        assert_eq!(
            (
                coroutine.rd,
                coroutine.wr,
                coroutine.dp,
                coroutine.sp,
                coroutine.ya,
                coroutine.bit,
            ),
            (0x0042, 0xa202, 0xa303, 0x0340, 0xa505, 0)
        );

        let checkpoint: Snes9xSmpCoroutineCheckpoint =
            serde_json::from_slice(&serde_json::to_vec(&coroutine.checkpoint().unwrap()).unwrap())
                .unwrap();
        let mut restored = SmpCoroutineState::restore(checkpoint).unwrap();
        let retained_scratch = (
            restored.rd,
            restored.wr,
            restored.dp,
            restored.sp,
            restored.ya,
            restored.bit,
        );
        bus.memory[usize::from(state.pc)] = 0xe6;
        let event_count = bus.events.len();
        assert_eq!(
            step(&mut bus, &mut state, &mut restored).0,
            SmpMicroStepResult::InProgress {
                opcode: 0xe6,
                opcode_cycle: 1,
            }
        );
        assert_eq!(bus.events.len(), event_count + 2, "fetch and source op_io");
        assert_eq!(
            (
                restored.rd,
                restored.wr,
                restored.dp,
                restored.sp,
                restored.ya,
                restored.bit,
            ),
            retained_scratch,
            "the next source pseudo-case does not replay or clear prior scratch"
        );
        assert_eq!(
            step(&mut bus, &mut state, &mut restored).0,
            SmpMicroStepResult::InstructionComplete { opcode: 0xe6 }
        );
        assert!(restored.is_idle());
        assert_eq!(
            (
                restored.rd,
                restored.wr,
                restored.dp,
                restored.sp,
                restored.ya,
                restored.bit,
            ),
            retained_scratch
        );
    }

    fn step(
        bus: &mut TestBus,
        state: &mut SmpState,
        coroutine: &mut SmpCoroutineState,
    ) -> (SmpMicroStepResult, u32) {
        let before = bus.cycles;
        let (next_state, result) = {
            let mut smp = Smp::new(bus, *state);
            let result = smp.run_resumable_micro_step(coroutine).unwrap();
            (smp.state(), result)
        };
        *state = next_state;
        (result, bus.cycles - before)
    }

    fn run_to_completion(
        bus: &mut TestBus,
        state: &mut SmpState,
        coroutine: &mut SmpCoroutineState,
    ) {
        for _ in 0..8 {
            if matches!(
                step(bus, state, coroutine).0,
                SmpMicroStepResult::InstructionComplete { .. }
            ) {
                return;
            }
        }
        panic!("store plan did not complete");
    }

    #[test]
    fn snes9x_classifier_has_exact_source_split_set_and_store_descriptions() {
        let (provenance, _) = opcode_ledger();
        let expected_split = [
            0x7e, 0x8f, 0xaa, 0xaf, 0xba, 0xbf, 0xc4, 0xc5, 0xc6, 0xc7, 0xc9, 0xca, 0xcb, 0xcc,
            0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xe4, 0xe5, 0xe6, 0xe7, 0xe9, 0xeb,
            0xec, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb,
        ];
        let mut expected_stores = store_cases()
            .into_iter()
            .map(|case| case.opcode)
            .collect::<Vec<_>>();
        expected_stores.sort_unstable();
        let actual_split = (0u8..=u8::MAX)
            .filter(|opcode| matches!(Smp::snes9x_opcode_plan(*opcode), Snes9xOpcodePlan::Split(_)))
            .collect::<Vec<_>>();
        let actual_stores = (0u8..=u8::MAX)
            .filter(|opcode| {
                matches!(
                    Smp::snes9x_opcode_plan(*opcode),
                    Snes9xOpcodePlan::Split(Snes9xSplitPlan::Store(_))
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(actual_split, expected_split);
        assert_eq!(actual_stores, expected_stores);
        let source_stage_counts = provenance["classifier"]["split_opcodes_and_stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                let entry = entry.as_array().unwrap();
                (
                    entry[0].as_u64().unwrap() as u8,
                    entry[1].as_u64().unwrap() as u8,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual_split
                .iter()
                .map(|&opcode| (opcode, Smp::snes9x_split_stage_count(opcode).unwrap()))
                .collect::<Vec<_>>(),
            source_stage_counts
        );
    }

    #[test]
    fn malformed_coroutine_checkpoints_are_rejected_before_restore() {
        let idle = SmpCoroutineState::enabled().checkpoint().unwrap();
        assert_eq!(
            SmpCoroutineState::restore(idle).unwrap(),
            SmpCoroutineState::enabled()
        );

        let mut idle_cycle_255 = idle;
        idle_cycle_255.opcode_cycle = u8::MAX;
        assert_eq!(
            SmpCoroutineState::restore(idle_cycle_255).unwrap_err(),
            Snes9xSmpCoroutineCheckpointError::IdleOpcodeCycle {
                opcode_cycle: u8::MAX
            }
        );

        let mut atomic = idle;
        atomic.opcode = Some(0xe8);
        atomic.opcode_cycle = 1;
        assert_eq!(
            SmpCoroutineState::restore(atomic).unwrap_err(),
            Snes9xSmpCoroutineCheckpointError::UnsupportedOpcode { opcode: 0xe8 }
        );

        let mut first_case_not_completed = idle;
        first_case_not_completed.opcode = Some(0x8f);
        assert_eq!(
            SmpCoroutineState::restore(first_case_not_completed).unwrap_err(),
            Snes9xSmpCoroutineCheckpointError::InvalidOpcodeCycle {
                opcode: 0x8f,
                opcode_cycle: 0,
                stage_count: 3,
            }
        );

        let mut terminal = first_case_not_completed;
        terminal.opcode_cycle = 3;
        assert_eq!(
            SmpCoroutineState::restore(terminal).unwrap_err(),
            Snes9xSmpCoroutineCheckpointError::InvalidOpcodeCycle {
                opcode: 0x8f,
                opcode_cycle: 3,
                stage_count: 3,
            }
        );

        terminal.opcode_cycle = u8::MAX;
        assert_eq!(
            SmpCoroutineState::restore(terminal).unwrap_err(),
            Snes9xSmpCoroutineCheckpointError::InvalidOpcodeCycle {
                opcode: 0x8f,
                opcode_cycle: u8::MAX,
                stage_count: 3,
            }
        );

        for opcode_cycle in [1, 2] {
            let mut valid = first_case_not_completed;
            valid.opcode_cycle = opcode_cycle;
            assert_eq!(
                SmpCoroutineState::restore(valid).unwrap().opcode_cycle,
                opcode_cycle
            );
        }
    }

    #[test]
    fn every_store_plan_preserves_source_op_step_stage_shapes() {
        for case in store_cases() {
            let (mut bus, mut state, mut coroutine) = setup_case(&case);
            let mut observed_cycles = Vec::new();
            for stage in 0..case.stage_cycles.len() {
                let (result, cycles) = step(&mut bus, &mut state, &mut coroutine);
                observed_cycles.push(cycles);
                if stage + 1 == case.stage_cycles.len() {
                    assert_eq!(
                        result,
                        SmpMicroStepResult::InstructionComplete {
                            opcode: case.opcode
                        },
                        "${:02x} final stage",
                        case.opcode
                    );
                } else {
                    assert_eq!(
                        result,
                        SmpMicroStepResult::InProgress {
                            opcode: case.opcode,
                            opcode_cycle: (stage + 1) as u8,
                        },
                        "${:02x} stage {stage}",
                        case.opcode
                    );
                }
            }

            assert_eq!(observed_cycles, case.stage_cycles, "${:02x}", case.opcode);
            let writes = bus
                .accesses
                .iter()
                .filter_map(|access| match access {
                    BusAccess::Write(address, value) => Some((*address, *value)),
                    BusAccess::Read(_) => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(writes, case.writes, "${:02x}", case.opcode);
            assert_eq!(
                state.pc,
                0x0201 + case.operands.len() as u16,
                "${:02x}",
                case.opcode
            );
            assert_eq!(
                state.x,
                if case.opcode == 0xaf { 0x03 } else { 0x02 },
                "${:02x}",
                case.opcode
            );
            assert!(coroutine.is_idle(), "${:02x}", case.opcode);
        }
    }

    #[test]
    fn every_store_continuation_stage_round_trips_and_resumes_once() {
        for case in store_cases() {
            let (mut expected_bus, mut expected_state, mut expected_coroutine) = setup_case(&case);
            run_to_completion(
                &mut expected_bus,
                &mut expected_state,
                &mut expected_coroutine,
            );

            for steps_before_checkpoint in 1..case.stage_cycles.len() {
                let (mut bus, mut state, mut coroutine) = setup_case(&case);
                for _ in 0..steps_before_checkpoint {
                    assert!(matches!(
                        step(&mut bus, &mut state, &mut coroutine).0,
                        SmpMicroStepResult::InProgress { .. }
                    ));
                }

                let checkpoint = coroutine.checkpoint().unwrap();
                let checkpoint: Snes9xSmpCoroutineCheckpoint =
                    serde_json::from_slice(&serde_json::to_vec(&checkpoint).unwrap()).unwrap();
                let mut restored = SmpCoroutineState::restore(checkpoint).unwrap();
                run_to_completion(&mut bus, &mut state, &mut restored);

                assert_eq!(
                    bus, expected_bus,
                    "${:02x} after stage {steps_before_checkpoint}",
                    case.opcode
                );
                assert_eq!(
                    state, expected_state,
                    "${:02x} after stage {steps_before_checkpoint}",
                    case.opcode
                );
                assert_eq!(
                    restored, expected_coroutine,
                    "${:02x} after stage {steps_before_checkpoint}",
                    case.opcode
                );
            }
        }
    }
}
