//! Opt-in, source-ordered subset of the pinned Snes9x 65816 executor.
//!
//! This is intentionally separate from the legacy C-port interpreter. Each
//! helper below corresponds to a concrete Snes9x `PCBase`, `Immediate*`,
//! `S9xGet*`, `S9xSet*`, or `AddCycles` transaction boundary.

use super::{CpuSynchronousCompletion, CpuSynchronousMachine, CpuSynchronousMachineError};
use crate::apu::{Snes9xApuCoroutineCheckpoint, Snes9xApuCoroutineCheckpointError};
use crate::cart::CartType;
use crate::cpu_timeline::{
    CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuMasterTimestamp,
    CpuSynchronousTimelineCheckpointError,
};
use crate::snes::Snes;
use crate::snes9x_apu_clock::{Snes9xApuClockCheckpoint, Snes9xApuClockError, Snes9xApuClockState};

const ONE_CYCLE: u32 = 6;
const TWO_CYCLES: u32 = 12;
const RESET_CPU_MASTER_CYCLE: u64 = 182;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCpuBusAccessKind {
    OpcodeFetch {
        value: u8,
    },
    Read {
        value: u16,
        width: u8,
    },
    /// One direct `READ_3WORD(PCBase + PC)` operand semantic followed by a
    /// single source `AddCycles(MemSpeedx2 + MemSpeed)` transaction.
    ReadLong {
        value: u32,
    },
    Write {
        value: u16,
        width: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceCpuBusAccess {
    pub address: u32,
    pub timestamp: CpuMasterTimestamp,
    pub charged_master_cycles: u8,
    pub kind: SourceCpuBusAccessKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCpuTransactionKind {
    FastPcBaseOpcodeFetchNonDraining,
    CpuOpsAddCyclesDraining,
    GetSetMemoryAccessAfterSemanticDraining,
    GetSetMemoryAccessX2AfterSemanticDraining,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceCpuTransaction {
    pub kind: SourceCpuTransactionKind,
    pub duration_master_cycles: u8,
    pub origin_pc: u32,
    pub opcode: u8,
    pub started_at: CpuMasterTimestamp,
    pub ended_at: CpuMasterTimestamp,
    pub cpu_model_identity: u8,
    pub cpu_model_5a22: u8,
    pub start_wram_refresh_position: u16,
    pub end_wram_refresh_position: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceCpuStepReceipt {
    pub origin_pc: u32,
    pub opcode: u8,
    pub started_at: CpuMasterTimestamp,
    pub ended_at: CpuMasterTimestamp,
    pub accesses: Vec<SourceCpuBusAccess>,
    pub transactions: Vec<SourceCpuTransaction>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SourceCpuError {
    #[error("LoROM reset seed requires a non-empty power-of-two ROM")]
    InvalidLoRom,
    #[error("LoROM reset seed requires exactly {expected} SRAM bytes, received {actual}")]
    InvalidLoRomSram { expected: usize, actual: usize },
    #[error("unsupported source CPU opcode ${opcode:02x} at ${pc:06x}")]
    UnsupportedOpcode { pc: u32, opcode: u8 },
    #[error("source CPU bus map at ${address:06x} is outside the audited cold subset")]
    UnsupportedBusMap { address: u32 },
    #[error("source CPU executor is poisoned by an earlier partial instruction")]
    Poisoned,
    #[error("source CPU execution encountered an unaudited IRQ, DMA, or HDMA path")]
    UnexpectedInterruptOrDma,
    #[error("source CPU NMI entry is currently proven only for native, non-WAI execution")]
    UnsupportedNmiEntryState,
    #[error(transparent)]
    Machine(#[from] CpuSynchronousMachineError),
}

/// Isolated pinned-Snes9x cold CPU/APU executor for the audited opcode subset.
pub struct Snes9xColdCpuExecutor {
    machine: CpuSynchronousMachine,
    active_trace: Option<SourceCpuInstructionTrace>,
    poisoned: bool,
}

/// Versioned, standalone in-memory checkpoint of one quiescent source-exact
/// CPU owner.
///
/// `Snes` keeps its legacy serialization unchanged. The exact SMP coroutine,
/// DSP pipeline/RAM/sample publication state, physical CPU timeline cursor,
/// and interrupt deadlines live alongside that stable machine in this one
/// atomic sidecar. This intentionally contains the full ROM and duplicates APU
/// RAM; a later persisted `.z3timing` wire format must instead be ROM-free and
/// bind explicitly to its external machine/base-state identity.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Snes9xCpuQuiescentCheckpoint {
    version: u8,
    snes: Snes,
    timeline: CpuMasterTimeline,
    apu_clock: Snes9xApuClockCheckpoint,
    apu_exact: Snes9xApuCoroutineCheckpoint,
    pending_completion: Option<CpuSynchronousCompletion>,
    #[serde(default)]
    source_vmain_full_graphic_count_nonzero: bool,
    nmi_acceptance_not_before: Option<CpuMasterTimestamp>,
    deferred_nmi_enable_edge: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Snes9xCpuQuiescentCheckpointError {
    #[error("source CPU executor is poisoned by a partial instruction")]
    Poisoned,
    #[error("source CPU executor has an active instruction trace")]
    ActiveInstruction,
    #[error("source CPU checkpoint has committed pending bus completion {completion:?}")]
    PendingCompletion {
        completion: CpuSynchronousCompletion,
    },
    #[error("source CPU checkpoint has an unpublished deferred NMI enable edge")]
    DeferredNmiEnableEdge,
    #[error("source CPU checkpoint contains transient debug/instruction state")]
    TransientDebugState,
    #[error("source CPU checkpoint is outside the audited IRQ/DMA/HDMA scope")]
    UnsupportedExecutionScope,
    #[error("source CPU checkpoint cannot resume WAI/STP state")]
    UnsupportedPowerState,
    #[error("source CPU checkpoint does not contain an exact APU sidecar")]
    MissingApuSidecar,
    #[error("unsupported source CPU checkpoint version {version}")]
    Version { version: u8 },
    #[error("APU clock reference {reference} is after CPU timeline clock {timeline}")]
    ApuClockAfterTimeline { reference: u64, timeline: u64 },
    #[error("source CPU checkpoint NMI pending bit and acceptance deadline disagree")]
    NmiOwnershipMismatch,
    #[error("source CPU checkpoint NMI deadline {deadline} is not in ({timeline}, {latest}]")]
    NmiDeadline {
        deadline: u64,
        timeline: u64,
        latest: u64,
    },
    #[error(transparent)]
    Timeline(#[from] CpuSynchronousTimelineCheckpointError),
    #[error(transparent)]
    ApuClock(#[from] Snes9xApuClockError),
    #[error(transparent)]
    Apu(#[from] Snes9xApuCoroutineCheckpointError),
}

struct SourceCpuInstructionTrace {
    origin_pc: u32,
    opcode: Option<u8>,
    memory_speed: Option<u8>,
    transactions: Vec<SourceCpuTransaction>,
}

impl Snes9xColdCpuExecutor {
    /// Construct the exact cold CPU subset seed, then perform Snes9x's reset
    /// vector `S9xGetWord($00fffc)` transaction at T=182..198.
    pub fn from_lorom_reset(rom: &[u8]) -> Result<Self, SourceCpuError> {
        Self::from_lorom_reset_with_sram(rom, None)
    }

    /// Construct the same cold seed with the exact externally supplied save
    /// RAM image. Without a file, pinned Snes9x `ClearSRAM` uses its generic
    /// `$60` initial value; supplied bytes are copied without interpretation.
    pub fn from_lorom_reset_with_sram(
        rom: &[u8],
        initial_sram: Option<&[u8]>,
    ) -> Result<Self, SourceCpuError> {
        if rom.is_empty() || !rom.len().is_power_of_two() {
            return Err(SourceCpuError::InvalidLoRom);
        }

        let mut snes = Snes::new();
        snes.cart.load(CartType::LoRom, rom, 0x2000);
        match initial_sram {
            Some(initial_sram) if initial_sram.len() != snes.cart.ram.len() => {
                return Err(SourceCpuError::InvalidLoRomSram {
                    expected: snes.cart.ram.len(),
                    actual: initial_sram.len(),
                });
            }
            Some(initial_sram) => snes.cart.ram.copy_from_slice(initial_sram),
            None => snes.cart.ram.fill(0x60),
        }
        snes.ram.fill(0x55);
        snes.apu.reset_snes9x_coroutine();
        snes.cpu.a = 0;
        snes.cpu.x = 0;
        snes.cpu.y = 0;
        snes.cpu.sp = 0x01ff;
        snes.cpu.pc = 0;
        snes.cpu.dp = 0;
        snes.cpu.k = 0;
        snes.cpu.db = 0;
        snes.cpu.c = false;
        snes.cpu.z = false;
        snes.cpu.v = false;
        snes.cpu.n = false;
        snes.cpu.i = true;
        snes.cpu.d = false;
        snes.cpu.xf = true;
        snes.cpu.mf = true;
        snes.cpu.e = true;
        snes.open_bus = 0;

        let mut timeline = CpuMasterTimeline::new(
            RESET_CPU_MASTER_CYCLE,
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline
            .begin_synchronous_timeline()
            .expect("the reset vector starts outside a consumed event window");
        let machine = CpuSynchronousMachine {
            snes,
            timeline,
            apu_clock: Snes9xApuClockState::new(),
            pending_completion: None,
            pending_general_dma: None,
            source_vmain_full_graphic_count_nonzero: false,
            nmi_acceptance_not_before: None,
            deferred_nmi_enable_edge: false,
            #[cfg(test)]
            force_zero_cycle_smp_step: false,
        };
        let mut this = Self {
            machine,
            active_trace: None,
            poisoned: false,
        };
        let mut accesses = Vec::new();
        let reset_pc = this.read_word(0x00_fffc, WordWrap::Bank, &mut accesses)?;
        this.machine.snes.cpu.pc = reset_pc;
        // cpu.cpp:S9xSoftResetCPU explicitly publishes `Registers.PCh` after
        // the direct reset-vector read, whose direct-memory path does not own
        // an OpenBus update.
        this.machine.snes.open_bus = (reset_pc >> 8) as u8;
        debug_assert_eq!(this.machine.timestamp().master_cycles(), 198);
        Ok(this)
    }

    pub const fn machine(&self) -> &CpuSynchronousMachine {
        &self.machine
    }

    pub fn capture_quiescent_checkpoint(
        &self,
    ) -> Result<Snes9xCpuQuiescentCheckpoint, Snes9xCpuQuiescentCheckpointError> {
        if self.poisoned {
            return Err(Snes9xCpuQuiescentCheckpointError::Poisoned);
        }
        if self.active_trace.is_some() {
            return Err(Snes9xCpuQuiescentCheckpointError::ActiveInstruction);
        }
        Self::validate_quiescent_machine(&self.machine)?;
        let apu_exact = self
            .machine
            .snes
            .apu
            .capture_snes9x_apu_coroutine_checkpoint()
            .ok_or(Snes9xCpuQuiescentCheckpointError::MissingApuSidecar)?;
        Ok(Snes9xCpuQuiescentCheckpoint {
            version: 2,
            snes: self.machine.snes.clone(),
            timeline: self.machine.timeline.clone(),
            apu_clock: self.machine.apu_clock.checkpoint(),
            apu_exact,
            pending_completion: self.machine.pending_completion,
            source_vmain_full_graphic_count_nonzero: self
                .machine
                .source_vmain_full_graphic_count_nonzero,
            nmi_acceptance_not_before: self.machine.nmi_acceptance_not_before,
            deferred_nmi_enable_edge: self.machine.deferred_nmi_enable_edge,
        })
    }

    pub fn from_quiescent_checkpoint(
        checkpoint: Snes9xCpuQuiescentCheckpoint,
    ) -> Result<Self, Snes9xCpuQuiescentCheckpointError> {
        if checkpoint.version != 2 {
            return Err(Snes9xCpuQuiescentCheckpointError::Version {
                version: checkpoint.version,
            });
        }
        let apu_clock = Snes9xApuClockState::from_checkpoint(checkpoint.apu_clock)?;
        let mut snes = checkpoint.snes;
        snes.apu
            .restore_snes9x_apu_coroutine_checkpoint(checkpoint.apu_exact)?;
        let machine = CpuSynchronousMachine {
            snes,
            timeline: checkpoint.timeline,
            apu_clock,
            pending_completion: checkpoint.pending_completion,
            pending_general_dma: None,
            source_vmain_full_graphic_count_nonzero: checkpoint
                .source_vmain_full_graphic_count_nonzero,
            nmi_acceptance_not_before: checkpoint.nmi_acceptance_not_before,
            deferred_nmi_enable_edge: checkpoint.deferred_nmi_enable_edge,
            #[cfg(test)]
            force_zero_cycle_smp_step: false,
        };
        Self::validate_quiescent_machine(&machine)?;
        Ok(Self {
            machine,
            active_trace: None,
            poisoned: false,
        })
    }

    pub fn restore_quiescent_checkpoint(
        &mut self,
        checkpoint: Snes9xCpuQuiescentCheckpoint,
    ) -> Result<(), Snes9xCpuQuiescentCheckpointError> {
        let candidate = Self::from_quiescent_checkpoint(checkpoint)?;
        *self = candidate;
        Ok(())
    }

    fn validate_quiescent_machine(
        machine: &CpuSynchronousMachine,
    ) -> Result<(), Snes9xCpuQuiescentCheckpointError> {
        if let Some(completion) = machine.pending_completion {
            return Err(Snes9xCpuQuiescentCheckpointError::PendingCompletion { completion });
        }
        if machine.pending_general_dma.is_some() || machine.snes.dma.dma_busy {
            return Err(Snes9xCpuQuiescentCheckpointError::UnsupportedExecutionScope);
        }
        if machine.deferred_nmi_enable_edge {
            return Err(Snes9xCpuQuiescentCheckpointError::DeferredNmiEnableEdge);
        }
        if machine.snes.has_transient_synchronous_debug_state() {
            return Err(Snes9xCpuQuiescentCheckpointError::TransientDebugState);
        }
        #[cfg(test)]
        if machine.force_zero_cycle_smp_step {
            return Err(Snes9xCpuQuiescentCheckpointError::TransientDebugState);
        }
        if machine.snes.h_irq_enabled
            || machine.snes.v_irq_enabled
            || machine.snes.cpu.irq_wanted
            || machine
                .snes
                .dma
                .channel
                .iter()
                .any(|channel| channel.dma_active || channel.hdma_active)
        {
            return Err(Snes9xCpuQuiescentCheckpointError::UnsupportedExecutionScope);
        }
        if machine.snes.cpu.waiting || machine.snes.cpu.stopped {
            return Err(Snes9xCpuQuiescentCheckpointError::UnsupportedPowerState);
        }
        machine
            .timeline
            .validate_quiescent_synchronous_checkpoint()?;
        let apu_reference = machine.apu_clock.checkpoint().cpu_reference_master_cycles();
        let timeline_clock = machine.timestamp().master_cycles();
        if apu_reference > timeline_clock {
            return Err(Snes9xCpuQuiescentCheckpointError::ApuClockAfterTimeline {
                reference: apu_reference,
                timeline: timeline_clock,
            });
        }
        if machine.snes.cpu.nmi_wanted != machine.nmi_acceptance_not_before.is_some() {
            return Err(Snes9xCpuQuiescentCheckpointError::NmiOwnershipMismatch);
        }
        if let Some(deadline) = machine.nmi_acceptance_not_before {
            let deadline = deadline.master_cycles();
            let latest = timeline_clock
                .saturating_add(crate::cpu_timeline::SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES);
            if deadline <= timeline_clock || deadline > latest {
                return Err(Snes9xCpuQuiescentCheckpointError::NmiDeadline {
                    deadline,
                    timeline: timeline_clock,
                    latest,
                });
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<SourceCpuStepReceipt, SourceCpuError> {
        if self.poisoned {
            return Err(SourceCpuError::Poisoned);
        }
        self.assert_source_execution_scope()?;
        if let Some(completion) = self.machine.pending_completion() {
            return Err(
                CpuSynchronousMachineError::PendingCompletionMustResume { completion }.into(),
            );
        }

        let started_at = self.machine.timestamp();
        let origin_pc = self.program_address();
        self.active_trace = Some(SourceCpuInstructionTrace {
            origin_pc,
            opcode: None,
            memory_speed: None,
            transactions: Vec::new(),
        });
        let mut accesses = Vec::new();
        let result = (|| {
            let opcode = self.fetch_opcode(&mut accesses)?;
            self.execute(opcode, origin_pc, &mut accesses)?;
            self.finish_instruction_interrupt_boundary(&mut accesses)?;
            self.assert_source_execution_scope()?;
            Ok(opcode)
        })();
        let opcode = match result {
            Ok(opcode) => opcode,
            Err(error) => {
                self.poisoned = true;
                self.active_trace = None;
                return Err(error);
            }
        };
        let trace = self
            .active_trace
            .take()
            .expect("a source CPU step owns one transaction trace");
        debug_assert_eq!(trace.origin_pc, origin_pc);
        debug_assert_eq!(trace.opcode, Some(opcode));
        Ok(SourceCpuStepReceipt {
            origin_pc,
            opcode,
            started_at,
            ended_at: self.machine.timestamp(),
            accesses,
            transactions: trace.transactions,
        })
    }

    pub const fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    fn assert_source_execution_scope(&self) -> Result<(), SourceCpuError> {
        let snes = &self.machine.snes;
        if snes.h_irq_enabled
            || snes.v_irq_enabled
            || snes.cpu.irq_wanted
            || snes
                .dma
                .channel
                .iter()
                .any(|channel| channel.dma_active || channel.hdma_active)
        {
            return Err(SourceCpuError::UnexpectedInterruptOrDma);
        }
        Ok(())
    }

    fn publish_deferred_nmi_enable_edge(&mut self) {
        if self.machine.deferred_nmi_enable_edge {
            self.machine.deferred_nmi_enable_edge = false;
            self.machine.snes.cpu.nmi_wanted = true;
            self.machine.nmi_acceptance_not_before = Some(CpuMasterTimestamp::new(
                self.machine.timestamp().master_cycles() + ONE_CYCLE as u64,
            ));
        }
    }

    /// Finish the pinned `S9xMainLoop` boundary after one complete opcode.
    /// A pre-existing due NMI is selected and cleared first; then
    /// `CHECK_FOR_IRQ_CHANGE` can publish a new `$4200` low-to-high edge before
    /// the selected interrupt is entered. The newly queued NMI therefore
    /// survives that entry and is considered only after the handler executes
    /// its first complete opcode.
    fn finish_instruction_interrupt_boundary(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let old_nmi_is_due = self.machine.snes.cpu.nmi_wanted
            && self
                .machine
                .nmi_acceptance_not_before
                .is_some_and(|deadline| self.machine.timestamp() >= deadline);
        if !old_nmi_is_due {
            self.publish_deferred_nmi_enable_edge();
            return Ok(());
        }
        if self.machine.snes.cpu.e || self.machine.snes.cpu.waiting {
            return Err(SourceCpuError::UnsupportedNmiEntryState);
        }

        // cpuexec.cpp clears the old pending latch, applies
        // CHECK_FOR_IRQ_CHANGE, then performs the selected native NMI entry.
        self.machine.snes.cpu.nmi_wanted = false;
        self.machine.nmi_acceptance_not_before = None;
        self.publish_deferred_nmi_enable_edge();
        let memory_speed = self
            .active_trace
            .as_ref()
            .and_then(|trace| trace.memory_speed)
            .expect("NMI entry follows one source-owned opcode fetch");
        self.add_cycles(u32::from(memory_speed) + ONE_CYCLE)?;
        let program_bank = self.machine.snes.cpu.k;
        self.push_byte(program_bank, accesses)?;
        let program_counter = self.machine.snes.cpu.pc;
        self.push_word(program_counter, accesses)?;
        let status = self.machine.snes.cpu.pack_flags();
        self.push_byte(status, accesses)?;
        self.machine.snes.open_bus = status;
        self.machine.snes.cpu.d = false;
        self.machine.snes.cpu.i = true;
        let vector = self.read_word(0x00_ffea, WordWrap::Bank, accesses)?;
        self.machine.snes.open_bus = (vector >> 8) as u8;
        self.machine.snes.cpu.k = 0;
        self.machine.snes.cpu.pc = vector;
        Ok(())
    }

    fn program_address(&self) -> u32 {
        (u32::from(self.machine.snes.cpu.k) << 16) | u32::from(self.machine.snes.cpu.pc)
    }

    fn fetch_opcode(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        let address = self.program_address();
        let timestamp = self.machine.timestamp();
        let start_wram_refresh_position = self.machine.timeline.wram_refresh_cycle() as u16;
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        let memory_speed = self.machine.snes.hardware_access_time(address);
        if adr < 0x8000 || !matches!(memory_speed, 6 | 8) {
            return Err(SourceCpuError::UnsupportedOpcode {
                pc: address,
                opcode: self.machine.snes.open_bus,
            });
        }
        // cpuexec.cpp reads direct `PCBase` without changing OpenBus or
        // draining a due event, then adds MemSpeed directly.
        let opcode = self
            .machine
            .snes
            .cart
            .read(bank, adr, self.machine.snes.open_bus);
        self.active_trace
            .as_mut()
            .expect("opcode fetch requires an active instruction trace")
            .opcode = Some(opcode);
        self.active_trace
            .as_mut()
            .expect("opcode fetch requires an active instruction trace")
            .memory_speed = Some(memory_speed);
        let ended_at;
        self.machine
            .timeline
            .advance_synchronous_pcbase_opcode_fetch(memory_speed);
        ended_at = self.machine.timestamp();
        self.machine.snes.cpu.pc = self.machine.snes.cpu.pc.wrapping_add(1);
        self.record_transaction(
            SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
            memory_speed,
            timestamp,
            ended_at,
            start_wram_refresh_position,
            start_wram_refresh_position,
        );
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: memory_speed,
            kind: SourceCpuBusAccessKind::OpcodeFetch { value: opcode },
        });
        Ok(opcode)
    }

    fn execute(
        &mut self,
        opcode: u8,
        origin_pc: u32,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        match opcode {
            0x0a => {
                self.add_cycles(ONE_CYCLE)?;
                self.asl_accumulator();
            }
            0x0b => {
                self.add_cycles(ONE_CYCLE)?;
                let direct_page = self.machine.snes.cpu.dp;
                self.push_word(direct_page, accesses)?;
                self.machine.snes.open_bus = direct_page as u8;
            }
            0x08 => {
                let flags = self.machine.snes.cpu.pack_flags();
                self.add_cycles(ONE_CYCLE)?;
                self.push_byte(flags, accesses)?;
                self.machine.snes.open_bus = flags;
            }
            0x18 => {
                self.machine.snes.cpu.c = false;
                self.add_cycles(ONE_CYCLE)?;
            }
            0x1a => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.machine.snes.cpu.mf {
                    let value = (self.machine.snes.cpu.a as u8).wrapping_add(1);
                    self.machine.snes.cpu.a = (self.machine.snes.cpu.a & 0xff00) | u16::from(value);
                    u16::from(value)
                } else {
                    self.machine.snes.cpu.a = self.machine.snes.cpu.a.wrapping_add(1);
                    self.machine.snes.cpu.a
                };
                self.set_zn(value, self.machine.snes.cpu.mf);
            }
            0x1b => {
                self.add_cycles(ONE_CYCLE)?;
                self.machine.snes.cpu.sp = self.machine.snes.cpu.a;
                self.fix_emulation_stack();
            }
            0x20 => {
                let target = self.immediate16(false, accesses)?;
                self.add_cycles(ONE_CYCLE)?;
                let return_pc = self.machine.snes.cpu.pc.wrapping_sub(1);
                self.push_word(return_pc, accesses)?;
                self.machine.snes.cpu.pc = target;
            }
            0x25 => {
                let address = self.direct_address(true, accesses)?;
                let operand = self.read_by_m(address, WordWrap::Bank, accesses)?;
                if self.machine.snes.cpu.mf {
                    let result = u16::from((self.machine.snes.cpu.a as u8) & operand as u8);
                    self.machine.snes.cpu.a = (self.machine.snes.cpu.a & 0xff00) | result;
                    self.set_zn(result, true);
                } else {
                    self.machine.snes.cpu.a &= operand;
                    self.set_zn(self.machine.snes.cpu.a, false);
                }
            }
            0x28 => {
                self.add_cycles(TWO_CYCLES)?;
                let flags = self.pull_byte(accesses)?;
                self.machine.snes.open_bus = flags;
                self.machine.snes.cpu.unpack_flags(flags);
                self.fix_status_widths();
            }
            0x29 => {
                let operand = self.immediate_by_m(accesses)?;
                if self.machine.snes.cpu.mf {
                    let result = u16::from((self.machine.snes.cpu.a as u8) & operand as u8);
                    self.machine.snes.cpu.a = (self.machine.snes.cpu.a & 0xff00) | result;
                    self.set_zn(result, true);
                } else {
                    self.machine.snes.cpu.a &= operand;
                    self.set_zn(self.machine.snes.cpu.a, false);
                }
            }
            0x2a => {
                self.add_cycles(ONE_CYCLE)?;
                self.rol_accumulator();
            }
            0x2b => {
                self.add_cycles(TWO_CYCLES)?;
                let direct_page = self.pull_word_bank(accesses)?;
                self.machine.snes.cpu.dp = direct_page;
                self.set_zn(direct_page, false);
                self.machine.snes.open_bus = (direct_page >> 8) as u8;
                if self.machine.snes.cpu.e {
                    // PLD uses PullW even in emulation mode, then repairs SH.
                    self.machine.snes.cpu.sp = 0x0100 | (self.machine.snes.cpu.sp & 0x00ff);
                }
            }
            0x45 => {
                let address = self.direct_address(true, accesses)?;
                let operand = self.read_by_m(address, WordWrap::Bank, accesses)?;
                if self.machine.snes.cpu.mf {
                    let result = u16::from((self.machine.snes.cpu.a as u8) ^ operand as u8);
                    self.machine.snes.cpu.a = (self.machine.snes.cpu.a & 0xff00) | result;
                    self.set_zn(result, true);
                } else {
                    self.machine.snes.cpu.a ^= operand;
                    self.set_zn(self.machine.snes.cpu.a, false);
                }
            }
            0x48 => {
                self.add_cycles(ONE_CYCLE)?;
                let a = self.machine.snes.cpu.a;
                if self.machine.snes.cpu.mf {
                    self.push_byte(a as u8, accesses)?;
                } else {
                    self.push_word(a, accesses)?;
                }
                self.machine.snes.open_bus = a as u8;
            }
            0x4b => {
                self.add_cycles(ONE_CYCLE)?;
                let program_bank = self.machine.snes.cpu.k;
                self.push_byte(program_bank, accesses)?;
                self.machine.snes.open_bus = program_bank;
            }
            0x58 => {
                self.add_cycles(ONE_CYCLE)?;
                self.machine.snes.cpu.i = false;
            }
            0x5a => {
                self.add_cycles(ONE_CYCLE)?;
                let y = self.machine.snes.cpu.y;
                if self.machine.snes.cpu.xf {
                    self.push_byte(y as u8, accesses)?;
                } else {
                    self.push_word(y, accesses)?;
                }
                self.machine.snes.open_bus = y as u8;
            }
            0x5b => {
                self.add_cycles(ONE_CYCLE)?;
                self.machine.snes.cpu.dp = self.machine.snes.cpu.a;
                self.set_zn(self.machine.snes.cpu.dp, false);
            }
            0x60 => {
                self.add_cycles(TWO_CYCLES)?;
                let target = self.pull_word(accesses)?;
                self.add_cycles(ONE_CYCLE)?;
                self.machine.snes.cpu.pc = target.wrapping_add(1);
            }
            0x64 => {
                let address = self.direct_address(false, accesses)?;
                if self.machine.snes.cpu.mf {
                    self.write_byte(address, 0, accesses)?;
                } else {
                    self.write_word(
                        address,
                        0,
                        WordWrap::Bank,
                        WordWriteOrder::LowHigh,
                        accesses,
                    )?;
                }
                self.machine.snes.open_bus = 0;
            }
            0x68 => {
                self.add_cycles(TWO_CYCLES)?;
                let value = if self.machine.snes.cpu.mf {
                    let value = self.pull_byte(accesses)?;
                    self.machine.snes.cpu.a = (self.machine.snes.cpu.a & 0xff00) | u16::from(value);
                    u16::from(value)
                } else {
                    let value = self.pull_word(accesses)?;
                    self.machine.snes.cpu.a = value;
                    value
                };
                self.machine.snes.open_bus = if self.machine.snes.cpu.mf {
                    value as u8
                } else {
                    (value >> 8) as u8
                };
                self.set_zn(value, self.machine.snes.cpu.mf);
            }
            0x69 => {
                let value = self.immediate_by_m(accesses)?;
                self.adc(value);
            }
            0x70 => self.branch(self.machine.snes.cpu.v, accesses)?,
            0x78 => {
                self.add_cycles(ONE_CYCLE)?;
                self.machine.snes.cpu.i = true;
            }
            0x7a => {
                let value = self.pull_by_x(accesses)?;
                self.machine.snes.cpu.y = value;
                self.set_zn(value, self.machine.snes.cpu.xf);
                self.machine.snes.open_bus = if self.machine.snes.cpu.xf {
                    value as u8
                } else {
                    (value >> 8) as u8
                };
            }
            0x7c => {
                let operand = self.immediate16_slow(true, accesses)?;
                self.add_cycles(ONE_CYCLE)?;
                let pointer = operand.wrapping_add(self.machine.snes.cpu.x);
                let address = (u32::from(self.machine.snes.cpu.k) << 16) | u32::from(pointer);
                let target = self.read_word(address, WordWrap::Bank, accesses)?;
                self.machine.snes.open_bus = (target >> 8) as u8;
                self.machine.snes.cpu.pc = target;
            }
            0x80 => self.branch(true, accesses)?,
            0x84 => {
                let address = self.direct_address(false, accesses)?;
                let value = self.machine.snes.cpu.y;
                self.store_index_register(address, value, accesses)?;
            }
            0x85 => {
                let address = self.direct_address(false, accesses)?;
                self.store_accumulator(address, accesses)?;
            }
            0x8c => {
                let address = self.absolute_address(false, accesses)?;
                let value = self.machine.snes.cpu.y;
                self.store_index_register(address, value, accesses)?;
            }
            0x8d => {
                let address = self.absolute_address(false, accesses)?;
                self.store_accumulator(address, accesses)?;
            }
            0x8e => {
                let address = self.absolute_address(false, accesses)?;
                let value = self.machine.snes.cpu.x;
                self.store_index_register(address, value, accesses)?;
            }
            0x8f => {
                let address = self.absolute_long_address(false, accesses)?;
                self.store_accumulator(address, accesses)?;
            }
            0x8b => {
                self.add_cycles(ONE_CYCLE)?;
                let data_bank = self.machine.snes.cpu.db;
                self.push_byte(data_bank, accesses)?;
                self.machine.snes.open_bus = data_bank;
            }
            0x9d => {
                let address = self.absolute_address(false, accesses)?;
                // cpuaddr.h:AbsoluteIndexedXX0 always charges this cycle;
                // AbsoluteIndexedXX1 also always charges it for writes.
                self.add_cycles(ONE_CYCLE)?;
                let address =
                    address.wrapping_add(u32::from(self.machine.snes.cpu.x)) & 0x00ff_ffff;
                self.store_accumulator(address, accesses)?;
            }
            0x9c => {
                let address = self.absolute_address(false, accesses)?;
                self.write_by_m(address, 0, WordWriteOrder::LowHigh, accesses)?;
                self.machine.snes.open_bus = 0;
            }
            0xa0 => {
                let value = self.immediate_by_x(accesses)?;
                if self.machine.snes.cpu.xf {
                    self.machine.snes.cpu.y = value & 0xff;
                } else {
                    self.machine.snes.cpu.y = value;
                }
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xa2 => {
                let value = self.immediate_by_x(accesses)?;
                self.machine.snes.cpu.x = if self.machine.snes.cpu.xf {
                    value & 0xff
                } else {
                    value
                };
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xa4 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.machine.snes.cpu.y = value;
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xa5 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.load_accumulator(value);
            }
            0xa6 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.machine.snes.cpu.x = value;
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xa8 => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.machine.snes.cpu.xf {
                    self.machine.snes.cpu.a & 0x00ff
                } else {
                    self.machine.snes.cpu.a
                };
                self.machine.snes.cpu.y = value;
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xa9 => {
                let value = self.immediate_by_m(accesses)?;
                if self.machine.snes.cpu.mf {
                    self.machine.snes.cpu.a = (self.machine.snes.cpu.a & 0xff00) | (value & 0xff);
                } else {
                    self.machine.snes.cpu.a = value;
                }
                self.set_zn(value, self.machine.snes.cpu.mf);
            }
            0xaa => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.machine.snes.cpu.xf {
                    self.machine.snes.cpu.a & 0xff
                } else {
                    self.machine.snes.cpu.a
                };
                self.machine.snes.cpu.x = value;
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xac => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.machine.snes.cpu.y = value;
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xae => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.machine.snes.cpu.x = value;
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xad => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.load_accumulator(value);
            }
            0xaf => {
                let address = self.absolute_long_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.load_accumulator(value);
            }
            0xab => {
                self.add_cycles(TWO_CYCLES)?;
                let data_bank = self.pull_byte(accesses)?;
                self.machine.snes.cpu.db = data_bank;
                self.set_zn(u16::from(data_bank), true);
                self.machine.snes.open_bus = data_bank;
            }
            0xb7 => {
                let pointer = self.direct_address(true, accesses)?;
                let low = self.read_word(pointer, WordWrap::None, accesses)?;
                self.machine.snes.open_bus = (low >> 8) as u8;
                let bank = self.read_byte(pointer.wrapping_add(2), accesses)?;
                self.machine.snes.open_bus = bank;
                let address = ((u32::from(bank) << 16) | u32::from(low))
                    .wrapping_add(u32::from(self.machine.snes.cpu.y))
                    & 0x00ff_ffff;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.load_accumulator(value);
            }
            0xc0 => {
                let value = self.immediate_by_x(accesses)?;
                self.compare(self.machine.snes.cpu.y, value, self.machine.snes.cpu.xf);
            }
            0xc9 => {
                let value = self.immediate_by_m(accesses)?;
                self.compare(self.machine.snes.cpu.a, value, self.machine.snes.cpu.mf);
            }
            0xc2 => {
                let mask = self.immediate8(true, accesses)?;
                let flags = self.machine.snes.cpu.pack_flags() & !mask;
                self.machine.snes.cpu.unpack_flags(flags);
                self.add_cycles(ONE_CYCLE)?;
                self.fix_status_widths();
            }
            0xc8 => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.machine.snes.cpu.xf {
                    self.machine.snes.cpu.y =
                        u16::from((self.machine.snes.cpu.y as u8).wrapping_add(1));
                    self.machine.snes.cpu.y
                } else {
                    self.machine.snes.cpu.y = self.machine.snes.cpu.y.wrapping_add(1);
                    self.machine.snes.cpu.y
                };
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xca => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.machine.snes.cpu.xf {
                    self.machine.snes.cpu.x =
                        u16::from((self.machine.snes.cpu.x as u8).wrapping_sub(1));
                    self.machine.snes.cpu.x
                } else {
                    self.machine.snes.cpu.x = self.machine.snes.cpu.x.wrapping_sub(1);
                    self.machine.snes.cpu.x
                };
                self.set_zn(value, self.machine.snes.cpu.xf);
            }
            0xcd => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.compare(self.machine.snes.cpu.a, value, self.machine.snes.cpu.mf);
            }
            0xd0 => self.branch(!self.machine.snes.cpu.z, accesses)?,
            0xda => {
                self.add_cycles(ONE_CYCLE)?;
                let x = self.machine.snes.cpu.x;
                if self.machine.snes.cpu.xf {
                    self.push_byte(x as u8, accesses)?;
                } else {
                    self.push_word(x, accesses)?;
                }
                self.machine.snes.open_bus = x as u8;
            }
            0xe0 => {
                let value = self.immediate_by_x(accesses)?;
                self.compare(self.machine.snes.cpu.x, value, self.machine.snes.cpu.xf);
            }
            0xe2 => {
                let mask = self.immediate8(true, accesses)?;
                let flags = self.machine.snes.cpu.pack_flags() | mask;
                self.machine.snes.cpu.unpack_flags(flags);
                self.add_cycles(ONE_CYCLE)?;
                self.fix_status_widths();
            }
            0xe6 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::Bank, accesses)?;
                let result = if self.machine.snes.cpu.mf {
                    u16::from((value as u8).wrapping_add(1))
                } else {
                    value.wrapping_add(1)
                };
                self.add_cycles(ONE_CYCLE)?;
                self.write_by_m(address, result, WordWriteOrder::HighLow, accesses)?;
                self.machine.snes.open_bus = result as u8;
                self.set_zn(result, self.machine.snes.cpu.mf);
            }
            0xeb => {
                let value = self.machine.snes.cpu.a;
                self.machine.snes.cpu.a = value.rotate_left(8);
                self.set_zn(self.machine.snes.cpu.a & 0xff, true);
                self.add_cycles(TWO_CYCLES)?;
            }
            0xf0 => self.branch(self.machine.snes.cpu.z, accesses)?,
            0xfa => {
                let value = self.pull_by_x(accesses)?;
                self.machine.snes.cpu.x = value;
                self.set_zn(value, self.machine.snes.cpu.xf);
                self.machine.snes.open_bus = if self.machine.snes.cpu.xf {
                    value as u8
                } else {
                    (value >> 8) as u8
                };
            }
            0xfb => {
                self.add_cycles(ONE_CYCLE)?;
                std::mem::swap(&mut self.machine.snes.cpu.c, &mut self.machine.snes.cpu.e);
                self.fix_status_widths();
                self.fix_emulation_stack();
            }
            _ => {
                return Err(SourceCpuError::UnsupportedOpcode {
                    pc: origin_pc,
                    opcode,
                })
            }
        }
        Ok(())
    }

    fn immediate_by_m(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        if self.machine.snes.cpu.mf {
            Ok(u16::from(self.immediate8(true, accesses)?))
        } else {
            self.immediate16(true, accesses)
        }
    }

    fn immediate_by_x(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        if self.machine.snes.cpu.xf {
            Ok(u16::from(self.immediate8(true, accesses)?))
        } else {
            self.immediate16(true, accesses)
        }
    }

    fn immediate8(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        let address = self.program_address();
        let timestamp = self.machine.timestamp();
        let value = self.machine.snes.cart.read(
            (address >> 16) as u8,
            address as u16,
            self.machine.snes.open_bus,
        );
        self.add_cycles(8)?;
        self.machine.snes.cpu.pc = self.machine.snes.cpu.pc.wrapping_add(1);
        if update_open_bus {
            self.machine.snes.open_bus = value;
        }
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: 8,
            kind: SourceCpuBusAccessKind::Read {
                value: u16::from(value),
                width: 1,
            },
        });
        Ok(value)
    }

    fn immediate16(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let address = self.program_address();
        let timestamp = self.machine.timestamp();
        let bank = (address >> 16) as u8;
        let low = self
            .machine
            .snes
            .cart
            .read(bank, address as u16, self.machine.snes.open_bus);
        let high = self.machine.snes.cart.read(
            bank,
            (address as u16).wrapping_add(1),
            self.machine.snes.open_bus,
        );
        let value = u16::from_le_bytes([low, high]);
        self.add_cycles(16)?;
        self.machine.snes.cpu.pc = self.machine.snes.cpu.pc.wrapping_add(2);
        if update_open_bus {
            self.machine.snes.open_bus = high;
        }
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: 16,
            kind: SourceCpuBusAccessKind::Read { value, width: 2 },
        });
        Ok(value)
    }

    fn immediate16_slow(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let address = self.program_address();
        let value = self.read_word(address, WordWrap::Bank, accesses)?;
        if update_open_bus {
            self.machine.snes.open_bus = (value >> 8) as u8;
        }
        self.machine.snes.cpu.pc = self.machine.snes.cpu.pc.wrapping_add(2);
        Ok(value)
    }

    fn direct_address(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        let operand = self.immediate8(update_open_bus, accesses)?;
        let dp = self.machine.snes.cpu.dp;
        if dp as u8 != 0 {
            self.add_cycles(ONE_CYCLE)?;
        }
        Ok(u32::from(dp.wrapping_add(u16::from(operand))))
    }

    fn absolute_address(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        let address = self.immediate16(update_open_bus, accesses)?;
        Ok((u32::from(self.machine.snes.cpu.db) << 16) | u32::from(address))
    }

    fn absolute_long_address(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        let address = self.program_address();
        let timestamp = self.machine.timestamp();
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        let low = self
            .machine
            .snes
            .cart
            .read(bank, adr, self.machine.snes.open_bus);
        let high =
            self.machine
                .snes
                .cart
                .read(bank, adr.wrapping_add(1), self.machine.snes.open_bus);
        let data_bank =
            self.machine
                .snes
                .cart
                .read(bank, adr.wrapping_add(2), self.machine.snes.open_bus);
        let value = u32::from(low) | (u32::from(high) << 8) | (u32::from(data_bank) << 16);
        // cpuaddr.h:AbsoluteLong uses one AddCycles transaction for all three
        // already-read direct PCBase bytes.
        self.add_cycles(24)?;
        self.machine.snes.cpu.pc = self.machine.snes.cpu.pc.wrapping_add(3);
        if update_open_bus {
            self.machine.snes.open_bus = data_bank;
        }
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: 24,
            kind: SourceCpuBusAccessKind::ReadLong { value },
        });
        Ok(value)
    }

    fn branch(
        &mut self,
        taken: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let offset = self.immediate8(true, accesses)? as i8;
        let old_pc = self.machine.snes.cpu.pc;
        let target = old_pc.wrapping_add_signed(i16::from(offset));
        if taken {
            self.add_cycles(ONE_CYCLE)?;
            if self.machine.snes.cpu.e && old_pc & 0xff00 != target & 0xff00 {
                self.add_cycles(ONE_CYCLE)?;
            }
            self.machine.snes.cpu.pc = target;
        }
        Ok(())
    }

    fn store_accumulator(
        &mut self,
        address: u32,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let value = self.machine.snes.cpu.a;
        self.write_by_m(address, value, WordWriteOrder::LowHigh, accesses)?;
        self.machine.snes.open_bus = if self.machine.snes.cpu.mf {
            value as u8
        } else {
            (value >> 8) as u8
        };
        Ok(())
    }

    fn store_index_register(
        &mut self,
        address: u32,
        value: u16,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        if self.machine.snes.cpu.xf {
            self.write_byte(address, value as u8, accesses)?;
            self.machine.snes.open_bus = value as u8;
        } else {
            self.write_word(
                address,
                value,
                WordWrap::Bank,
                WordWriteOrder::LowHigh,
                accesses,
            )?;
            self.machine.snes.open_bus = (value >> 8) as u8;
        }
        Ok(())
    }

    fn load_accumulator(&mut self, value: u16) {
        if self.machine.snes.cpu.mf {
            self.machine.snes.cpu.a = (self.machine.snes.cpu.a & 0xff00) | (value & 0xff);
        } else {
            self.machine.snes.cpu.a = value;
        }
        self.set_zn(value, self.machine.snes.cpu.mf);
    }

    fn read_by_m(
        &mut self,
        address: u32,
        wrap: WordWrap,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let value = if self.machine.snes.cpu.mf {
            u16::from(self.read_byte(address, accesses)?)
        } else {
            self.read_word(address, wrap, accesses)?
        };
        self.machine.snes.open_bus = if self.machine.snes.cpu.mf {
            value as u8
        } else {
            (value >> 8) as u8
        };
        Ok(value)
    }

    fn read_by_x(
        &mut self,
        address: u32,
        wrap: WordWrap,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let value = if self.machine.snes.cpu.xf {
            u16::from(self.read_byte(address, accesses)?)
        } else {
            self.read_word(address, wrap, accesses)?
        };
        self.machine.snes.open_bus = if self.machine.snes.cpu.xf {
            value as u8
        } else {
            (value >> 8) as u8
        };
        Ok(value)
    }

    fn write_by_m(
        &mut self,
        address: u32,
        value: u16,
        order: WordWriteOrder,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        if self.machine.snes.cpu.mf {
            self.write_byte(address, value as u8, accesses)
        } else {
            self.write_word(address, value, WordWrap::None, order, accesses)
        }
    }

    fn read_byte(
        &mut self,
        address: u32,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        let address = address & 0x00ff_ffff;
        let timestamp = self.machine.timestamp();
        let value = if let Some(port) = Snes::synchronous_cpu_apu_port(address) {
            let force_zero_cycle_smp_step = self.machine.force_zero_cycle_smp_step();
            CpuSynchronousMachine::synchronize_apu(
                &mut self.machine.snes,
                &mut self.machine.apu_clock,
                timestamp,
                force_zero_cycle_smp_step,
            )?;
            self.machine
                .snes
                .synchronous_cpu_read_apu_port_raw_semantic(port)
        } else {
            self.source_read_semantic(address)?
        };
        self.machine.pending_completion = Some(CpuSynchronousCompletion::Read(value));
        let duration = self.machine.snes.hardware_access_time(address);
        self.drain_and_record_transaction(
            SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
            u32::from(duration),
        )?;
        debug_assert_eq!(
            self.machine.take_pending_completion(),
            CpuSynchronousCompletion::Read(value)
        );
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: duration,
            kind: SourceCpuBusAccessKind::Read {
                value: u16::from(value),
                width: 1,
            },
        });
        Ok(value)
    }

    fn write_byte(
        &mut self,
        address: u32,
        value: u8,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let address = address & 0x00ff_ffff;
        self.guard_control_write(address, value)?;
        let timestamp = self.machine.timestamp();
        let mirrored_bank = (address >> 16) as u8 & 0x7f;
        if mirrored_bank < 0x40 && address as u16 == 0x420b && value != 0 {
            let receipt = self.machine.write_general_dma_control(value)?;
            self.record_transaction(
                SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                self.machine.snes.hardware_access_time(address),
                receipt.outer_started_at,
                receipt.outer_ended_at,
                receipt.outer_start_wram_refresh_position,
                receipt.outer_end_wram_refresh_position,
            );
            debug_assert_eq!(
                self.machine.take_pending_completion(),
                CpuSynchronousCompletion::GeneralDmaWrite
            );
        } else if let Some(port) = Snes::synchronous_cpu_apu_port(address) {
            let force_zero_cycle_smp_step = self.machine.force_zero_cycle_smp_step();
            CpuSynchronousMachine::synchronize_apu(
                &mut self.machine.snes,
                &mut self.machine.apu_clock,
                timestamp,
                force_zero_cycle_smp_step,
            )?;
            self.machine
                .snes
                .synchronous_cpu_write_apu_port_raw_semantic(address, port, value);
        } else {
            self.source_write_semantic(address, value)?;
        }
        if !(mirrored_bank < 0x40 && address as u16 == 0x420b && value != 0) {
            self.machine.pending_completion = Some(CpuSynchronousCompletion::Write);
            let duration = self.machine.snes.hardware_access_time(address);
            self.drain_and_record_transaction(
                SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                u32::from(duration),
            )?;
            debug_assert_eq!(
                self.machine.take_pending_completion(),
                CpuSynchronousCompletion::Write
            );
        }
        let duration = self.machine.snes.hardware_access_time(address);
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: duration,
            kind: SourceCpuBusAccessKind::Write {
                value: u16::from(value),
                width: 1,
            },
        });
        Ok(())
    }

    fn read_word(
        &mut self,
        address: u32,
        wrap: WordWrap,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let next = wrap.next(address);
        if self.word_is_direct_transaction(address, next, wrap) {
            let timestamp = self.machine.timestamp();
            let low = self.source_read_semantic(address)?;
            let high = self.source_read_semantic(next)?;
            let value = u16::from_le_bytes([low, high]);
            self.machine.pending_completion = Some(CpuSynchronousCompletion::ReadWord(value));
            let duration = self.machine.snes.hardware_access_time(address) * 2;
            self.drain_and_record_transaction(
                SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                u32::from(duration),
            )?;
            debug_assert_eq!(
                self.machine.take_pending_completion(),
                CpuSynchronousCompletion::ReadWord(value)
            );
            accesses.push(SourceCpuBusAccess {
                address,
                timestamp,
                charged_master_cycles: duration,
                kind: SourceCpuBusAccessKind::Read { value, width: 2 },
            });
            Ok(value)
        } else {
            let low = self.read_byte(address, accesses)?;
            if Self::word_crosses_wrap_boundary(address, wrap) {
                self.machine.snes.open_bus = low;
            }
            let high = self.read_byte(next, accesses)?;
            Ok(u16::from_le_bytes([low, high]))
        }
    }

    fn write_word(
        &mut self,
        address: u32,
        value: u16,
        wrap: WordWrap,
        order: WordWriteOrder,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let next = wrap.next(address);
        if self.word_is_direct_transaction(address, next, wrap) {
            let timestamp = self.machine.timestamp();
            let [low, high] = value.to_le_bytes();
            match order {
                WordWriteOrder::LowHigh => {
                    self.source_write_semantic(address, low)?;
                    self.source_write_semantic(next, high)?;
                }
                WordWriteOrder::HighLow => {
                    self.source_write_semantic(next, high)?;
                    self.source_write_semantic(address, low)?;
                }
            }
            self.machine.pending_completion = Some(CpuSynchronousCompletion::WriteWord);
            let duration = self.machine.snes.hardware_access_time(address) * 2;
            self.drain_and_record_transaction(
                SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                u32::from(duration),
            )?;
            debug_assert_eq!(
                self.machine.take_pending_completion(),
                CpuSynchronousCompletion::WriteWord
            );
            accesses.push(SourceCpuBusAccess {
                address,
                timestamp,
                charged_master_cycles: duration,
                kind: SourceCpuBusAccessKind::Write { value, width: 2 },
            });
            Ok(())
        } else {
            let [low, high] = value.to_le_bytes();
            match order {
                WordWriteOrder::LowHigh => {
                    self.write_byte(address, low, accesses)?;
                    self.write_byte(next, high, accesses)
                }
                WordWriteOrder::HighLow => {
                    self.write_byte(next, high, accesses)?;
                    self.write_byte(address, low, accesses)
                }
            }
        }
    }

    fn word_is_direct_transaction(&self, address: u32, next: u32, wrap: WordWrap) -> bool {
        if Self::word_crosses_wrap_boundary(address, wrap) {
            return false;
        }
        let direct = self.source_map_class(address);
        direct.is_some()
            && direct == self.source_map_class(next)
            && self.machine.snes.hardware_access_time(address)
                == self.machine.snes.hardware_access_time(next)
            && Snes::synchronous_cpu_apu_port(address).is_none()
    }

    fn word_crosses_wrap_boundary(address: u32, wrap: WordWrap) -> bool {
        let boundary_mask = match wrap {
            WordWrap::None => 0x00ff_ffff,
            WordWrap::Bank => 0x0000_ffff,
            WordWrap::Page => 0x0000_00ff,
        } & 0x0fff;
        address & boundary_mask == boundary_mask
    }

    fn source_map_class(&self, address: u32) -> Option<SourceCpuMapClass> {
        let address = address & 0x00ff_ffff;
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        if bank == 0x7e || bank == 0x7f || ((bank & 0x7f) < 0x40 && adr < 0x2000) {
            Some(SourceCpuMapClass::Wram)
        } else if (0x70..0x7e).contains(&bank) && adr < 0x8000 {
            Some(SourceCpuMapClass::LoRomSram)
        } else if adr >= 0x8000 {
            Some(SourceCpuMapClass::LoRom)
        } else {
            None
        }
    }

    fn source_read_semantic(&mut self, address: u32) -> Result<u8, SourceCpuError> {
        let address = address & 0x00ff_ffff;
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        if (bank & 0x7f) < 0x40 && adr == 0x4210 {
            // ppu.cpp:S9xGetCPU(RDNMI): sample and clear the latch before the
            // subsequent getset access charge. This does not cancel an NMI
            // which has already become CPU.NMIPending.
            let latched = self.machine.snes.in_nmi;
            self.machine.snes.in_nmi = false;
            return Ok((u8::from(latched) << 7) | (self.machine.snes.open_bus & 0x70) | 2);
        }
        if (bank & 0x7f) < 0x40 && (0x4218..=0x421f).contains(&adr) {
            let byte_index = usize::from(adr - 0x4218);
            let word = self.machine.snes.port_auto_read[byte_index / 2];
            return Ok(word.to_le_bytes()[byte_index & 1]);
        }
        match self.source_map_class(address) {
            Some(SourceCpuMapClass::Wram) => {
                let index = if bank == 0x7e || bank == 0x7f {
                    ((usize::from(bank) - 0x7e) << 16) | usize::from(adr)
                } else {
                    usize::from(adr)
                };
                Ok(self.machine.snes.ram[index])
            }
            Some(SourceCpuMapClass::LoRom) => {
                Ok(self
                    .machine
                    .snes
                    .cart
                    .read(bank, adr, self.machine.snes.open_bus))
            }
            Some(SourceCpuMapClass::LoRomSram) => {
                Ok(self
                    .machine
                    .snes
                    .cart
                    .read(bank, adr, self.machine.snes.open_bus))
            }
            None => Err(SourceCpuError::UnsupportedBusMap { address }),
        }
    }

    fn source_write_semantic(&mut self, address: u32, value: u8) -> Result<(), SourceCpuError> {
        let address = address & 0x00ff_ffff;
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        if (bank & 0x7f) < 0x40 && adr == 0x4016 {
            let latch_high = value & 1 != 0;
            let was_high = self.machine.snes.input1.latch_line;
            debug_assert_eq!(was_high, self.machine.snes.input2.latch_line);
            self.machine.snes.input1.latch_line = latch_high;
            self.machine.snes.input2.latch_line = latch_high;
            if latch_high && !was_high {
                self.machine.snes.input1.cycle();
                self.machine.snes.input2.cycle();
            }
            return Ok(());
        }
        if (bank & 0x7f) < 0x40 && adr == 0x4200 {
            if value & 0x30 != 0 {
                return Err(SourceCpuError::UnexpectedInterruptOrDma);
            }
            let was_nmi_enabled = self.machine.snes.nmi_enabled;
            let nmi_enabled = value & 0x80 != 0;
            if nmi_enabled
                && !was_nmi_enabled
                && self.machine.snes.in_vblank
                && self.machine.snes.in_nmi
            {
                self.machine.deferred_nmi_enable_edge = true;
            }
            self.machine.snes.nmi_enabled = nmi_enabled;
            self.machine.snes.auto_joy_read = value & 1 != 0;
            if !self.machine.snes.auto_joy_read {
                self.machine.snes.auto_joy_timer = 0;
            }
            return Ok(());
        }
        if (bank & 0x7f) < 0x40 && matches!(adr, 0x420b | 0x420c) {
            return Ok(());
        }
        if (bank & 0x7f) < 0x40 && (0x4300..=0x437f).contains(&adr) {
            // ppu.cpp:S9xSetCPU owns the DMA-channel register semantics, while
            // cpumacro.h's store operation publishes OpenBus only after the
            // complete byte/word write. Calling the register owner directly
            // preserves that source ordering.
            self.machine.snes.dma_write_reg(adr, value);
            return Ok(());
        }
        if (bank & 0x7f) < 0x40 && (0x2100..=0x21ff).contains(&adr) {
            if adr == 0x2115 {
                self.machine.source_vmain_full_graphic_count_nonzero = value & 0x0c != 0;
            }
            let open_bus = self.machine.snes.open_bus;
            self.machine.snes.write(address, value);
            self.machine.snes.open_bus = open_bus;
            return Ok(());
        }
        match self.source_map_class(address) {
            Some(SourceCpuMapClass::Wram) => {
                let index = if bank == 0x7e || bank == 0x7f {
                    ((usize::from(bank) - 0x7e) << 16) | usize::from(adr)
                } else {
                    usize::from(adr)
                };
                self.machine.snes.ram[index] = value;
                Ok(())
            }
            Some(SourceCpuMapClass::LoRomSram) => {
                self.machine.snes.cart.write(bank, adr, value);
                Ok(())
            }
            Some(SourceCpuMapClass::LoRom) | None => {
                Err(SourceCpuError::UnsupportedBusMap { address })
            }
        }
    }

    fn push_byte(
        &mut self,
        value: u8,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let address = u32::from(self.machine.snes.cpu.sp);
        self.write_byte(address, value, accesses)?;
        if self.machine.snes.cpu.e {
            let low = (self.machine.snes.cpu.sp as u8).wrapping_sub(1);
            self.machine.snes.cpu.sp = 0x0100 | u16::from(low);
        } else {
            self.machine.snes.cpu.sp = self.machine.snes.cpu.sp.wrapping_sub(1);
        }
        Ok(())
    }

    fn push_word(
        &mut self,
        value: u16,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let address = if self.machine.snes.cpu.e {
            0x0100 | u32::from((self.machine.snes.cpu.sp as u8).wrapping_sub(1))
        } else {
            u32::from(self.machine.snes.cpu.sp.wrapping_sub(1))
        };
        let wrap = if self.machine.snes.cpu.e {
            WordWrap::Page
        } else {
            WordWrap::Bank
        };
        self.write_word(address, value, wrap, WordWriteOrder::HighLow, accesses)?;
        if self.machine.snes.cpu.e {
            let low = (self.machine.snes.cpu.sp as u8).wrapping_sub(2);
            self.machine.snes.cpu.sp = 0x0100 | u16::from(low);
        } else {
            self.machine.snes.cpu.sp = self.machine.snes.cpu.sp.wrapping_sub(2);
        }
        Ok(())
    }

    fn pull_byte(&mut self, accesses: &mut Vec<SourceCpuBusAccess>) -> Result<u8, SourceCpuError> {
        if self.machine.snes.cpu.e {
            let low = (self.machine.snes.cpu.sp as u8).wrapping_add(1);
            self.machine.snes.cpu.sp = 0x0100 | u16::from(low);
        } else {
            self.machine.snes.cpu.sp = self.machine.snes.cpu.sp.wrapping_add(1);
        }
        self.read_byte(u32::from(self.machine.snes.cpu.sp), accesses)
    }

    fn pull_word(&mut self, accesses: &mut Vec<SourceCpuBusAccess>) -> Result<u16, SourceCpuError> {
        if self.machine.snes.cpu.e {
            let low = (self.machine.snes.cpu.sp as u8).wrapping_add(1);
            self.machine.snes.cpu.sp = 0x0100 | u16::from(low);
            let value = self.read_word(
                u32::from(self.machine.snes.cpu.sp),
                WordWrap::Page,
                accesses,
            )?;
            let low = (self.machine.snes.cpu.sp as u8).wrapping_add(1);
            self.machine.snes.cpu.sp = 0x0100 | u16::from(low);
            Ok(value)
        } else {
            let address = self.machine.snes.cpu.sp.wrapping_add(1);
            let value = self.read_word(u32::from(address), WordWrap::Bank, accesses)?;
            self.machine.snes.cpu.sp = self.machine.snes.cpu.sp.wrapping_add(2);
            Ok(value)
        }
    }

    fn pull_word_bank(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let address = self.machine.snes.cpu.sp.wrapping_add(1);
        let value = self.read_word(u32::from(address), WordWrap::Bank, accesses)?;
        self.machine.snes.cpu.sp = self.machine.snes.cpu.sp.wrapping_add(2);
        Ok(value)
    }

    fn pull_by_x(&mut self, accesses: &mut Vec<SourceCpuBusAccess>) -> Result<u16, SourceCpuError> {
        self.add_cycles(TWO_CYCLES)?;
        if self.machine.snes.cpu.xf {
            Ok(u16::from(self.pull_byte(accesses)?))
        } else {
            self.pull_word_bank(accesses)
        }
    }

    fn guard_control_write(&self, address: u32, value: u8) -> Result<(), SourceCpuError> {
        let mirrored = (address >> 16) as u8 & 0x7f;
        if mirrored < 0x40 {
            match address as u16 {
                0x4200 if value & 0x30 != 0 => {
                    return Err(SourceCpuError::UnexpectedInterruptOrDma)
                }
                0x420c if value != 0 => return Err(SourceCpuError::UnexpectedInterruptOrDma),
                _ => {}
            }
        }
        Ok(())
    }

    fn add_cycles(&mut self, cycles: u32) -> Result<(), SourceCpuError> {
        self.drain_and_record_transaction(SourceCpuTransactionKind::CpuOpsAddCyclesDraining, cycles)
    }

    fn drain_and_record_transaction(
        &mut self,
        kind: SourceCpuTransactionKind,
        duration_master_cycles: u32,
    ) -> Result<(), SourceCpuError> {
        let duration_master_cycles = u8::try_from(duration_master_cycles)
            .expect("audited source CPU transactions fit in one byte");
        let started_at = self.machine.timestamp();
        let start_wram_refresh_position = self.machine.timeline.wram_refresh_cycle() as u16;
        self.machine
            .drain_add_cycles_after_committed_semantic(u32::from(duration_master_cycles))?;
        if self.active_trace.is_some() {
            self.record_transaction(
                kind,
                duration_master_cycles,
                started_at,
                self.machine.timestamp(),
                start_wram_refresh_position,
                self.machine.timeline.wram_refresh_cycle() as u16,
            );
        }
        Ok(())
    }

    fn record_transaction(
        &mut self,
        kind: SourceCpuTransactionKind,
        duration_master_cycles: u8,
        started_at: CpuMasterTimestamp,
        ended_at: CpuMasterTimestamp,
        start_wram_refresh_position: u16,
        end_wram_refresh_position: u16,
    ) {
        let trace = self
            .active_trace
            .as_mut()
            .expect("source transaction requires an active instruction trace");
        trace.transactions.push(SourceCpuTransaction {
            kind,
            duration_master_cycles,
            origin_pc: trace.origin_pc,
            opcode: trace
                .opcode
                .expect("source transaction requires a fetched opcode"),
            started_at,
            ended_at,
            cpu_model_identity: 1,
            cpu_model_5a22: 2,
            start_wram_refresh_position,
            end_wram_refresh_position,
        });
    }

    fn compare(&mut self, register: u16, operand: u16, eight_bit: bool) {
        if eight_bit {
            let result = (register as u8).wrapping_sub(operand as u8);
            self.machine.snes.cpu.c = (register as u8) >= (operand as u8);
            self.set_zn(u16::from(result), true);
        } else {
            let result = register.wrapping_sub(operand);
            self.machine.snes.cpu.c = register >= operand;
            self.set_zn(result, false);
        }
    }

    fn adc(&mut self, operand: u16) {
        let eight_bit = self.machine.snes.cpu.mf;
        let mask = if eight_bit { 0x00ff } else { 0xffff };
        let sign = if eight_bit { 0x0080 } else { 0x8000 };
        let lhs = self.machine.snes.cpu.a & mask;
        let rhs = operand & mask;
        let carry = u32::from(self.machine.snes.cpu.c);
        let binary = u32::from(lhs) + u32::from(rhs) + carry;
        let mut result = binary;
        if self.machine.snes.cpu.d {
            result = bcd_add(lhs, rhs, carry, if eight_bit { 2 } else { 4 });
        }
        self.machine.snes.cpu.c = result > u32::from(mask);
        let narrowed = result as u16 & mask;
        self.machine.snes.cpu.v = (!(lhs ^ rhs) & (rhs ^ binary as u16) & sign) != 0;
        self.machine.snes.cpu.a = (self.machine.snes.cpu.a & !mask) | narrowed;
        self.set_zn(narrowed, eight_bit);
    }

    fn rol_accumulator(&mut self) {
        let eight_bit = self.machine.snes.cpu.mf;
        let mask = if eight_bit { 0x00ff } else { 0xffff };
        let carry_bit = if eight_bit { 0x0100 } else { 0x1_0000 };
        let result =
            (u32::from(self.machine.snes.cpu.a & mask) << 1) | u32::from(self.machine.snes.cpu.c);
        self.machine.snes.cpu.c = result & carry_bit != 0;
        let value = result as u16 & mask;
        self.machine.snes.cpu.a = (self.machine.snes.cpu.a & !mask) | value;
        self.set_zn(value, eight_bit);
    }

    fn asl_accumulator(&mut self) {
        let eight_bit = self.machine.snes.cpu.mf;
        let mask = if eight_bit { 0x00ff } else { 0xffff };
        let sign = if eight_bit { 0x0080 } else { 0x8000 };
        let value = self.machine.snes.cpu.a & mask;
        self.machine.snes.cpu.c = value & sign != 0;
        let result = value.wrapping_shl(1) & mask;
        self.machine.snes.cpu.a = (self.machine.snes.cpu.a & !mask) | result;
        self.set_zn(result, eight_bit);
    }

    fn set_zn(&mut self, value: u16, eight_bit: bool) {
        let value = if eight_bit { value & 0xff } else { value };
        self.machine.snes.cpu.z = value == 0;
        self.machine.snes.cpu.n = if eight_bit {
            value & 0x80 != 0
        } else {
            value & 0x8000 != 0
        };
    }

    fn fix_status_widths(&mut self) {
        if self.machine.snes.cpu.e {
            self.machine.snes.cpu.mf = true;
            self.machine.snes.cpu.xf = true;
        }
        if self.machine.snes.cpu.xf {
            self.machine.snes.cpu.x &= 0xff;
            self.machine.snes.cpu.y &= 0xff;
        }
    }

    fn fix_emulation_stack(&mut self) {
        if self.machine.snes.cpu.e {
            self.machine.snes.cpu.sp = 0x0100 | (self.machine.snes.cpu.sp & 0xff);
        }
    }
}

fn bcd_add(lhs: u16, rhs: u16, carry: u32, digits: usize) -> u32 {
    let mut result = 0u32;
    let mut carry = carry;
    for digit in 0..digits {
        let shift = digit * 4;
        let mut nibble = u32::from((lhs >> shift) & 0xf) + u32::from((rhs >> shift) & 0xf) + carry;
        if nibble > 9 {
            nibble += 6;
        }
        carry = u32::from(nibble > 0xf);
        result |= (nibble & 0xf) << shift;
    }
    result | (carry << (digits * 4))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceCpuMapClass {
    Wram,
    LoRom,
    LoRomSram,
}

#[derive(Clone, Copy)]
enum WordWrap {
    None,
    Bank,
    Page,
}

impl WordWrap {
    fn next(self, address: u32) -> u32 {
        match self {
            Self::None => address.wrapping_add(1) & 0x00ff_ffff,
            Self::Bank => (address & 0x00ff_0000) | u32::from((address as u16).wrapping_add(1)),
            Self::Page => (address & 0x00ff_ff00) | u32::from((address as u8).wrapping_add(1)),
        }
    }
}

#[derive(Clone, Copy)]
enum WordWriteOrder {
    LowHigh,
    HighLow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bootstrap_fixture::{
        cpu_apu_accesses, cpu_timing_transactions_through_first_cc, records,
        split_first_cc_cpu_accesses, visit_cpu_timing_transactions, CpuTimingTransaction,
    };
    use crate::Snes9xApuClockCheckpoint;
    use std::collections::VecDeque;

    const POST_HANDOFF_FIRST_NMI_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../external/snes9x-libretro/fixtures/zelda3-cold-apu-first-nmi.jsonl"
    ));
    const FIRST_NMI_DMA_SETUP_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma-setup.jsonl"
    ));

    fn synthetic_rom(program: &[u8]) -> Vec<u8> {
        let mut rom = vec![0xea; 0x8000];
        rom[..program.len()].copy_from_slice(program);
        rom[0x7ffc] = 0x00;
        rom[0x7ffd] = 0x80;
        rom
    }

    #[test]
    fn quiescent_checkpoint_roundtrips_every_exact_owner_without_changing_legacy_snes_serde() {
        let rom = synthetic_rom(&[0x18, 0x18]);
        let mut original = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        original.machine.snes.input1.current_state = 0x1234;
        original.machine.snes.input1.latched_state = 0xabcd;
        let legacy_before = serde_json::to_vec(original.machine.snes()).unwrap();

        let checkpoint = original.capture_quiescent_checkpoint().unwrap();
        assert_eq!(serde_json::to_vec(&checkpoint.snes).unwrap(), legacy_before);
        let encoded = serde_json::to_vec(&checkpoint).unwrap();
        let checkpoint: Snes9xCpuQuiescentCheckpoint = serde_json::from_slice(&encoded).unwrap();
        let mut restored = Snes9xColdCpuExecutor::from_quiescent_checkpoint(checkpoint).unwrap();

        assert_eq!(
            serde_json::to_vec(restored.machine.snes()).unwrap(),
            legacy_before
        );
        assert_eq!(restored.machine.timestamp(), original.machine.timestamp());
        assert_eq!(
            restored
                .machine
                .snes
                .apu
                .capture_snes9x_apu_coroutine_checkpoint(),
            original
                .machine
                .snes
                .apu
                .capture_snes9x_apu_coroutine_checkpoint()
        );
        assert_eq!(restored.step().unwrap(), original.step().unwrap());
    }

    #[test]
    fn checkpoint_v2_roundtrips_nonlinear_vmain_capability() {
        let rom = synthetic_rom(&[0x18]);
        let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        source.machine.source_vmain_full_graphic_count_nonzero = true;
        let encoded = serde_json::to_vec(&source.capture_quiescent_checkpoint().unwrap()).unwrap();
        let checkpoint: Snes9xCpuQuiescentCheckpoint = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(checkpoint.version, 2);
        let mut restored = Snes9xColdCpuExecutor::from_quiescent_checkpoint(checkpoint).unwrap();
        assert!(restored.machine.source_vmain_full_graphic_count_nonzero);

        let dma = &mut restored.machine.snes.dma.channel[0];
        dma.a_bank = 0x7e;
        dma.a_adr = 0;
        dma.size = 2;
        dma.mode = 1;
        dma.b_adr = 0x18;
        dma.fixed = false;
        dma.decrement = false;
        dma.from_b = false;
        assert_eq!(
            restored.machine.write_general_dma_control(1),
            Err(CpuSynchronousMachineError::GeneralDmaNonlinearVram { channel: 0 })
        );
        assert_eq!(restored.machine.timestamp(), source.machine.timestamp());
    }

    #[test]
    fn quiescent_checkpoint_rejects_nonquiescent_executor_state() {
        let rom = synthetic_rom(&[0x18]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.pending_completion = Some(CpuSynchronousCompletion::Read(0x5a));
        assert!(matches!(
            cpu.capture_quiescent_checkpoint(),
            Err(Snes9xCpuQuiescentCheckpointError::PendingCompletion {
                completion: CpuSynchronousCompletion::Read(0x5a)
            })
        ));

        cpu.machine.pending_completion = None;
        cpu.machine.deferred_nmi_enable_edge = true;
        assert_eq!(
            cpu.capture_quiescent_checkpoint().err().unwrap(),
            Snes9xCpuQuiescentCheckpointError::DeferredNmiEnableEdge
        );

        cpu.machine.deferred_nmi_enable_edge = false;
        cpu.active_trace = Some(SourceCpuInstructionTrace {
            origin_pc: 0x008000,
            opcode: None,
            memory_speed: None,
            transactions: Vec::new(),
        });
        assert_eq!(
            cpu.capture_quiescent_checkpoint().err().unwrap(),
            Snes9xCpuQuiescentCheckpointError::ActiveInstruction
        );

        cpu.active_trace = None;
        cpu.poisoned = true;
        assert_eq!(
            cpu.capture_quiescent_checkpoint().err().unwrap(),
            Snes9xCpuQuiescentCheckpointError::Poisoned
        );

        cpu.poisoned = false;
        cpu.machine.snes.debug_cpu_write_trace = Some(Vec::new());
        assert_eq!(
            cpu.capture_quiescent_checkpoint().err().unwrap(),
            Snes9xCpuQuiescentCheckpointError::TransientDebugState
        );

        cpu.machine.snes.debug_cpu_write_trace = None;
        cpu.machine.snes.cpu.waiting = true;
        assert_eq!(
            cpu.capture_quiescent_checkpoint().err().unwrap(),
            Snes9xCpuQuiescentCheckpointError::UnsupportedPowerState
        );
        cpu.machine.snes.cpu.waiting = false;
        cpu.machine.snes.cpu.stopped = true;
        assert_eq!(
            cpu.capture_quiescent_checkpoint().err().unwrap(),
            Snes9xCpuQuiescentCheckpointError::UnsupportedPowerState
        );
    }

    #[test]
    fn quiescent_checkpoint_retains_pending_nmi_deadline_and_input_shift_state() {
        let rom = synthetic_rom(&[0x18]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.nmi_wanted = true;
        let deadline = CpuMasterTimestamp::new(cpu.machine.timestamp().master_cycles() + 12);
        cpu.machine.nmi_acceptance_not_before = Some(deadline);
        cpu.machine.snes.input1.current_state = 0x1357;
        cpu.machine.snes.input1.latched_state = 0x2468;
        cpu.machine.snes.input1.latch_line = true;

        let encoded = serde_json::to_vec(&cpu.capture_quiescent_checkpoint().unwrap()).unwrap();
        let checkpoint: Snes9xCpuQuiescentCheckpoint = serde_json::from_slice(&encoded).unwrap();
        let restored = Snes9xColdCpuExecutor::from_quiescent_checkpoint(checkpoint).unwrap();

        assert!(restored.machine.snes.cpu.nmi_wanted);
        assert_eq!(restored.machine.nmi_acceptance_not_before, Some(deadline));
        assert_eq!(restored.machine.snes.input1.current_state, 0x1357);
        assert_eq!(restored.machine.snes.input1.latched_state, 0x2468);
        assert!(restored.machine.snes.input1.latch_line);
    }

    #[test]
    fn quiescent_checkpoint_restores_unpublished_dsp_samples_exactly_once() {
        let rom = synthetic_rom(&[0x18]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        while cpu.machine.snes.apu.cycles < 32 {
            cpu.machine
                .snes
                .apu
                .run_snes9x_micro_step_without_dsp()
                .unwrap();
        }
        cpu.machine.snes.apu.synchronize_snes9x_dsp();

        let encoded = serde_json::to_vec(&cpu.capture_quiescent_checkpoint().unwrap()).unwrap();
        let checkpoint: Snes9xCpuQuiescentCheckpoint = serde_json::from_slice(&encoded).unwrap();
        let mut restored = Snes9xColdCpuExecutor::from_quiescent_checkpoint(checkpoint).unwrap();
        let first = restored.machine.take_dsp_samples();
        assert!(!first.samples.is_empty());
        assert!(restored.machine.take_dsp_samples().samples.is_empty());
    }

    #[test]
    fn malformed_or_mixed_quiescent_checkpoint_never_mutates_restore_target() {
        let rom = synthetic_rom(&[0x18]);
        let source = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        let good = source.capture_quiescent_checkpoint().unwrap();

        let mut target = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        let target_before = serde_json::to_vec(target.machine.snes()).unwrap();
        let target_timestamp = target.machine.timestamp();
        let assert_unchanged = |target: &Snes9xColdCpuExecutor| {
            assert_eq!(
                serde_json::to_vec(target.machine.snes()).unwrap(),
                target_before
            );
            assert_eq!(target.machine.timestamp(), target_timestamp);
        };

        let mut legacy_v1 = serde_json::to_value(&good).unwrap();
        legacy_v1["version"] = serde_json::json!(1);
        legacy_v1
            .as_object_mut()
            .unwrap()
            .remove("source_vmain_full_graphic_count_nonzero");
        let malformed: Snes9xCpuQuiescentCheckpoint = serde_json::from_value(legacy_v1).unwrap();
        assert!(matches!(
            target.restore_quiescent_checkpoint(malformed),
            Err(Snes9xCpuQuiescentCheckpointError::Version { version: 1 })
        ));
        assert_unchanged(&target);

        let mut malformed = good.clone();
        malformed.timeline.corrupt_field_timing_for_test();
        assert!(matches!(
            target.restore_quiescent_checkpoint(malformed),
            Err(Snes9xCpuQuiescentCheckpointError::Timeline(
                CpuSynchronousTimelineCheckpointError::FieldTiming
            ))
        ));
        assert_unchanged(&target);

        let mut malformed = good.clone();
        malformed.timeline.corrupt_synchronous_cursor_for_test();
        assert!(matches!(
            target.restore_quiescent_checkpoint(malformed),
            Err(Snes9xCpuQuiescentCheckpointError::Timeline(
                CpuSynchronousTimelineCheckpointError::CursorMismatch
            ))
        ));
        assert_unchanged(&target);

        let mut mixed_owner = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        mixed_owner.machine.snes.apu.out_ports[0] = 1;
        let mixed_apu = mixed_owner
            .capture_quiescent_checkpoint()
            .unwrap()
            .apu_exact;
        let mut malformed = good.clone();
        malformed.apu_exact = mixed_apu;
        assert!(matches!(
            target.restore_quiescent_checkpoint(malformed),
            Err(Snes9xCpuQuiescentCheckpointError::Apu(
                Snes9xApuCoroutineCheckpointError::MachineIdentityMismatch
            ))
        ));
        assert_unchanged(&target);

        let mut malformed = good.clone();
        malformed.snes.cpu.nmi_wanted = true;
        assert_eq!(
            target.restore_quiescent_checkpoint(malformed).unwrap_err(),
            Snes9xCpuQuiescentCheckpointError::NmiOwnershipMismatch
        );
        assert_unchanged(&target);

        for deadline in [
            target_timestamp.master_cycles(),
            target_timestamp.master_cycles() + 13,
        ] {
            let mut malformed = good.clone();
            malformed.snes.cpu.nmi_wanted = true;
            malformed.nmi_acceptance_not_before = Some(CpuMasterTimestamp::new(deadline));
            assert!(matches!(
                target.restore_quiescent_checkpoint(malformed),
                Err(Snes9xCpuQuiescentCheckpointError::NmiDeadline { .. })
            ));
            assert_unchanged(&target);
        }
    }

    fn assert_timing_transaction(
        index: usize,
        actual: SourceCpuTransaction,
        expected: CpuTimingTransaction,
    ) {
        let actual_kind = match actual.kind {
            SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining => 0,
            SourceCpuTransactionKind::CpuOpsAddCyclesDraining => 1,
            SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining => 2,
            SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining => 3,
        };
        assert_eq!(
            actual_kind, expected.kind,
            "transaction {index}: actual={actual:?} expected={expected:?}"
        );
        assert_eq!(
            actual.duration_master_cycles, expected.duration,
            "transaction {index}"
        );
        assert_eq!(actual.origin_pc, expected.origin_pc, "transaction {index}");
        assert_eq!(actual.opcode, expected.opcode, "transaction {index}");
        assert_eq!(
            actual.started_at.master_cycles(),
            expected.absolute_start_master_cycle(),
            "transaction {index}"
        );
        assert_eq!(
            actual.ended_at.master_cycles(),
            expected.absolute_end_master_cycle(),
            "transaction {index}: {actual:?} != {expected:?}"
        );
        assert_eq!(
            actual.cpu_model_identity, expected.cpu_model_identity,
            "transaction {index}"
        );
        assert_eq!(
            actual.cpu_model_5a22, expected.cpu_model_5a22,
            "transaction {index}"
        );
        assert_eq!(
            actual.start_wram_refresh_position, expected.start_wram_refresh_position,
            "transaction {index}: {actual:?} != {expected:?}"
        );
        assert_eq!(
            actual.end_wram_refresh_position, expected.end_wram_refresh_position,
            "transaction {index}: {actual:?} != {expected:?}"
        );
    }

    fn post_handoff_first_nmi_record() -> serde_json::Value {
        POST_HANDOFF_FIRST_NMI_FIXTURE
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .find(|record| record["kind"] == "post-handoff-first-nmi")
            .expect("fixture omitted the post-handoff first-NMI receipt")
    }

    fn first_nmi_dma_setup_record() -> serde_json::Value {
        FIRST_NMI_DMA_SETUP_FIXTURE
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .find(|record| record["kind"] == "first-nmi-dma-setup")
            .expect("fixture omitted the first-NMI DMA setup receipt")
    }

    #[test]
    fn cold_reset_vector_and_fast_opcode_fetch_use_source_transactions() {
        let rom = synthetic_rom(&[0x18]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        assert_eq!(cpu.machine.timestamp().master_cycles(), 198);
        assert_eq!(cpu.machine.snes.cpu.pc, 0x8000);
        assert_eq!(cpu.machine.snes.cpu.sp, 0x01ff);
        assert_eq!(cpu.machine.snes.cpu.pack_flags(), 0x34);
        assert!(cpu.machine.snes.cpu.e);
        assert!(cpu.machine.snes.ram.iter().all(|byte| *byte == 0x55));
        assert_eq!(cpu.machine.snes.open_bus, 0x80);

        let receipt = cpu.step().unwrap();
        assert_eq!(receipt.origin_pc, 0x00_8000);
        assert_eq!(receipt.opcode, 0x18);
        assert_eq!(receipt.started_at.master_cycles(), 198);
        assert_eq!(receipt.ended_at.master_cycles(), 212);
        assert_eq!(receipt.accesses[0].timestamp.master_cycles(), 198);
        assert_eq!(receipt.accesses[0].charged_master_cycles, 8);
        assert_eq!(receipt.transactions.len(), 2);
        assert_eq!(
            receipt.transactions[0],
            SourceCpuTransaction {
                kind: SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                duration_master_cycles: 8,
                origin_pc: 0x00_8000,
                opcode: 0x18,
                started_at: CpuMasterTimestamp::new(198),
                ended_at: CpuMasterTimestamp::new(206),
                cpu_model_identity: 1,
                cpu_model_5a22: 2,
                start_wram_refresh_position: 538,
                end_wram_refresh_position: 538,
            }
        );
        assert_eq!(
            receipt.transactions[1].kind,
            SourceCpuTransactionKind::CpuOpsAddCyclesDraining
        );
        assert_eq!(receipt.transactions[1].started_at.master_cycles(), 206);
        assert_eq!(receipt.transactions[1].ended_at.master_cycles(), 212);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn reset_sram_seed_matches_snes9x_default_or_exact_supplied_image() {
        let rom = synthetic_rom(&[0x18]);
        let default = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        assert!(default
            .machine
            .snes
            .cart
            .ram
            .iter()
            .all(|byte| *byte == 0x60));

        let supplied = (0..0x2000).map(|index| index as u8).collect::<Vec<_>>();
        let seeded =
            Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&supplied)).unwrap();
        assert_eq!(seeded.machine.snes.cart.ram, supplied);
        assert!(matches!(
            Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&[0; 1])),
            Err(SourceCpuError::InvalidLoRomSram {
                expected: 0x2000,
                actual: 1,
            })
        ));
    }

    #[test]
    fn slowrom_native_nmi_entry_uses_current_eight_cycle_memory_speed() {
        let mut rom = synthetic_rom(&[0xf0, 0xfe]); // BEQ $8000
        rom[0x00c9] = 0x78; // first handler instruction at $80c9
        rom[0x7fea] = 0xc9;
        rom[0x7feb] = 0x80;
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.z = true;
        cpu.machine.snes.cpu.d = true;
        cpu.machine.snes.cpu.i = false;
        cpu.machine.snes.cpu.nmi_wanted = true;
        cpu.machine.nmi_acceptance_not_before = Some(cpu.machine.timestamp());
        let pushed_status = cpu.machine.snes.cpu.pack_flags();

        let receipt = cpu.step().unwrap();
        assert_eq!(receipt.origin_pc, 0x00_8000);
        assert_eq!(receipt.opcode, 0xf0);
        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 14),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert!(receipt
            .transactions
            .iter()
            .all(|transaction| transaction.origin_pc == 0x00_8000 && transaction.opcode == 0xf0));
        assert_eq!(cpu.machine.snes.ram[0x01ff], 0);
        assert_eq!(&cpu.machine.snes.ram[0x01fd..=0x01fe], &[0x00, 0x80]);
        assert_eq!(cpu.machine.snes.ram[0x01fc], pushed_status);
        assert_eq!(cpu.machine.snes.cpu.sp, 0x01fb);
        assert_eq!(cpu.machine.snes.cpu.pc, 0x80c9);
        assert_eq!(cpu.machine.snes.cpu.k, 0);
        assert!(cpu.machine.snes.cpu.i);
        assert!(!cpu.machine.snes.cpu.d);
        assert!(!cpu.machine.snes.cpu.nmi_wanted);
        assert_eq!(cpu.machine.nmi_acceptance_not_before, None);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn fastrom_native_nmi_entry_uses_current_six_cycle_memory_speed() {
        let mut rom = synthetic_rom(&[0x18]); // CLC
        rom[0x00c9] = 0x78;
        rom[0x7fea] = 0xc9;
        rom[0x7feb] = 0x80;
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.fast_mem = true;
        cpu.machine.snes.cpu.k = 0x80;
        cpu.machine.snes.cpu.pc = 0x8000;
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.nmi_wanted = true;
        cpu.machine.nmi_acceptance_not_before = Some(cpu.machine.timestamp());

        let receipt = cpu.step().unwrap();
        assert_eq!(receipt.origin_pc, 0x80_8000);
        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    6,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 12),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses[0].charged_master_cycles, 6);
        assert_eq!(cpu.machine.snes.ram[0x01ff], 0x80);
        assert_eq!(cpu.machine.snes.cpu.pc, 0x80c9);
        assert_eq!(cpu.machine.snes.cpu.k, 0);
    }

    #[test]
    fn vblank_publication_schedules_h12_but_finishes_the_active_opcode() {
        let mut rom = synthetic_rom(&[
            0xa5, 0x12, // LDA $12 crosses V224 HMax and ends at V225:H6
            0xf0, 0xfc, // BEQ $8000 runs atomically through H28
        ]);
        rom[0x00c9] = 0x78;
        rom[0x7fea] = 0xc9;
        rom[0x7feb] = 0x80;
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = true;
        cpu.machine.snes.cpu.xf = true;
        cpu.machine.snes.cpu.i = false;
        cpu.machine.snes.ram[0x12] = 0;
        cpu.machine.snes.nmi_enabled = true;
        cpu.machine.snes.auto_joy_read = true;
        cpu.machine.timeline = CpuMasterTimeline::at_raster(
            0,
            crate::CpuRasterPosition::new(224, 1_346),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        cpu.machine.timeline.begin_synchronous_timeline().unwrap();
        let start = cpu.machine.timestamp().master_cycles();
        cpu.machine.apu_clock = Snes9xApuClockState::from_checkpoint(
            Snes9xApuClockCheckpoint::new(start, 0, 0).unwrap(),
        )
        .unwrap();

        let load = cpu.step().unwrap();
        assert_source_transaction_shape(
            &load,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(225, 6)
        );
        assert!(cpu.machine.snes.in_vblank);
        assert!(cpu.machine.snes.in_nmi);
        assert!(cpu.machine.snes.cpu.nmi_wanted);
        assert_eq!(
            cpu.machine.nmi_acceptance_not_before,
            Some(CpuMasterTimestamp::new(
                CpuFieldTiming::NON_INTERLACE_EVEN
                    .master_cycles_at(0, crate::CpuRasterPosition::new(225, 12),)
            ))
        );
        assert_eq!(cpu.machine.snes.cpu.pc, 0x8002);

        let branch_and_nmi = cpu.step().unwrap();
        assert_source_transaction_shape(
            &branch_and_nmi,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 14),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(branch_and_nmi.ended_at.master_cycles(), start + 108);
        assert_eq!(cpu.machine.snes.cpu.pc, 0x80c9);
        assert!(!cpu.machine.snes.cpu.nmi_wanted);
        assert_eq!(cpu.machine.nmi_acceptance_not_before, None);
    }

    #[test]
    fn nmitimen_and_rdnmi_keep_source_latch_and_pending_ownership_separate() {
        let rom = synthetic_rom(&[
            0xa9, 0x81, // LDA #$81
            0x8d, 0x00, 0x42, // STA $4200
        ]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.step().unwrap();
        cpu.machine.snes.in_vblank = true;
        cpu.machine.snes.in_nmi = true;
        let write = cpu.step().unwrap();
        assert!(cpu.machine.snes.nmi_enabled);
        assert!(cpu.machine.snes.auto_joy_read);
        assert!(cpu.machine.snes.cpu.nmi_wanted);
        assert_eq!(
            cpu.machine.nmi_acceptance_not_before,
            Some(CpuMasterTimestamp::new(
                write.ended_at.master_cycles() + ONE_CYCLE as u64
            ))
        );

        let read_rom = synthetic_rom(&[0xad, 0x10, 0x42]); // LDA $4210
        let mut read_cpu = Snes9xColdCpuExecutor::from_lorom_reset(&read_rom).unwrap();
        read_cpu.machine.snes.in_vblank = true;
        read_cpu.machine.snes.in_nmi = true;
        read_cpu.machine.snes.cpu.nmi_wanted = true;
        read_cpu.machine.nmi_acceptance_not_before = Some(CpuMasterTimestamp::new(u64::MAX));
        let read = read_cpu.step().unwrap();
        assert_eq!(read_cpu.machine.snes.cpu.a as u8, 0xc2);
        assert!(!read_cpu.machine.snes.in_nmi);
        assert!(read_cpu.machine.snes.cpu.nmi_wanted);
        assert_eq!(
            read.accesses.last().unwrap().kind,
            SourceCpuBusAccessKind::Read {
                value: 0x00c2,
                width: 1,
            }
        );

        let disable_rom = synthetic_rom(&[
            0xa9, 0x00, // LDA #$00
            0x8d, 0x00, 0x42, // STA $4200
        ]);
        let mut disable = Snes9xColdCpuExecutor::from_lorom_reset(&disable_rom).unwrap();
        disable.machine.snes.nmi_enabled = true;
        disable.machine.snes.cpu.nmi_wanted = true;
        disable.machine.nmi_acceptance_not_before = Some(CpuMasterTimestamp::new(u64::MAX));
        disable.step().unwrap();
        disable.step().unwrap();
        assert!(!disable.machine.snes.nmi_enabled);
        assert!(disable.machine.snes.cpu.nmi_wanted);
        assert_eq!(
            disable.machine.nmi_acceptance_not_before,
            Some(CpuMasterTimestamp::new(u64::MAX))
        );
    }

    #[test]
    fn due_nmi_entry_preserves_new_deferred_nmitimen_edge() {
        let mut rom = synthetic_rom(&[0x8d, 0x00, 0x42]); // STA $4200
        rom[0x00c9] = 0x78;
        rom[0x7fea] = 0xc9;
        rom[0x7feb] = 0x80;
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = true;
        cpu.machine.snes.cpu.a = 0x80;
        cpu.machine.snes.in_vblank = true;
        cpu.machine.snes.in_nmi = true;
        cpu.machine.snes.cpu.nmi_wanted = true;
        cpu.machine.nmi_acceptance_not_before = Some(CpuMasterTimestamp::new(
            cpu.machine.timestamp().master_cycles() + 1,
        ));
        let instruction_boundary = cpu.machine.timestamp().master_cycles() + 8 + 16 + 6;

        let receipt = cpu.step().unwrap();
        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    6,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 14),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert!(cpu.machine.snes.nmi_enabled);
        assert!(cpu.machine.snes.cpu.nmi_wanted);
        assert_eq!(
            cpu.machine.nmi_acceptance_not_before,
            Some(CpuMasterTimestamp::new(
                instruction_boundary + ONE_CYCLE as u64
            ))
        );
        assert_eq!(cpu.machine.snes.cpu.pc, 0x80c9);
    }

    #[test]
    fn synthetic_native_stack_jsr_rts_and_width_changes_are_source_ordered() {
        let rom = synthetic_rom(&[
            0x18, // CLC
            0xfb, // XCE -> native
            0xc2, 0x30, // REP #$30
            0xa9, 0x34, 0x12, // LDA #$1234
            0x1b, // TCS
            0x20, 0x0f, 0x80, // JSR $800f
            0xe2, 0x20, // SEP #$20
            0x80, 0x03, // BRA $8012
            0x48, // $800f PHA (16-bit at call)
            0x68, // PLA
            0x60, // RTS
            0x18, // $8013 CLC
        ]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        for _ in 0..6 {
            cpu.step().unwrap();
        }
        assert!(!cpu.machine.snes.cpu.e);
        assert_eq!(cpu.machine.snes.cpu.sp, 0x1232);
        assert_eq!(cpu.machine.snes.cpu.pc, 0x800f);
        cpu.step().unwrap();
        cpu.step().unwrap();
        cpu.step().unwrap();
        assert_eq!(cpu.machine.snes.cpu.pc, 0x800b);
        assert_eq!(cpu.machine.snes.cpu.sp, 0x1234);
        assert_eq!(cpu.machine.snes.cpu.a, 0x1234);
    }

    #[test]
    fn emulation_pushwe_and_pullwe_wrap_inside_stack_page_one() {
        let rom = synthetic_rom(&[
            0x20, 0x05, 0x80, // JSR $8005
            0x18, 0x18, // return target
            0x60, // RTS
        ]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.sp = 0x0100;

        let jsr = cpu.step().unwrap();
        assert_eq!(cpu.machine.snes.cpu.sp, 0x01fe);
        assert_eq!(cpu.machine.snes.ram[0x0100], 0x80);
        assert_eq!(cpu.machine.snes.ram[0x01ff], 0x02);
        let writes = jsr
            .accesses
            .iter()
            .filter(|access| matches!(access.kind, SourceCpuBusAccessKind::Write { .. }))
            .map(|access| access.address)
            .collect::<Vec<_>>();
        assert_eq!(writes, [0x0100, 0x01ff]);

        cpu.machine.snes.open_bus = 0x5a;
        let rts = cpu.step().unwrap();
        assert_eq!(cpu.machine.snes.cpu.sp, 0x0100);
        assert_eq!(cpu.machine.snes.cpu.pc, 0x8003);
        let reads = rts
            .accesses
            .iter()
            .filter(|access| matches!(access.kind, SourceCpuBusAccessKind::Read { .. }))
            .map(|access| access.address)
            .collect::<Vec<_>>();
        assert_eq!(reads, [0x01ff, 0x0100]);
        assert_eq!(cpu.machine.snes.open_bus, 0x02);
    }

    fn install_hmax_smp_failure(cpu: &mut Snes9xColdCpuExecutor, start_cycle: u16) {
        cpu.machine.timeline = CpuMasterTimeline::at_raster(
            0,
            crate::CpuRasterPosition::new(0, start_cycle),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        cpu.machine.timeline.begin_synchronous_timeline().unwrap();
        cpu.machine.apu_clock = Snes9xApuClockState::from_checkpoint(
            Snes9xApuClockCheckpoint::new(1_358, 250_000, 0).unwrap(),
        )
        .unwrap();
        cpu.machine.force_zero_cycle_smp_step = true;
    }

    #[test]
    fn immediate_drain_failure_defers_pc_and_open_bus_then_poison_is_terminal() {
        let mut cpu =
            Snes9xColdCpuExecutor::from_lorom_reset(&synthetic_rom(&[0xc2, 0x20])).unwrap();
        install_hmax_smp_failure(&mut cpu, 1_350);
        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert!(cpu.is_poisoned());
        assert_eq!(cpu.machine.snes.cpu.pc, 0x8001);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
        let poisoned_at = cpu.machine.timestamp();
        assert!(matches!(cpu.step(), Err(SourceCpuError::Poisoned)));
        assert_eq!(cpu.machine.timestamp(), poisoned_at);
    }

    #[test]
    fn committed_apui_failure_poison_does_not_replay_semantic_or_duration() {
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&synthetic_rom(&[
            0x8d, 0x40, 0x21, // STA $2140
        ]))
        .unwrap();
        cpu.machine.snes.cpu.a = 0x77;
        install_hmax_smp_failure(&mut cpu, 1_334);
        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert_eq!(cpu.machine.snes.apu.in_ports[0], 0x77);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
        let poisoned_at = cpu.machine.timestamp();
        assert!(matches!(cpu.step(), Err(SourceCpuError::Poisoned)));
        assert_eq!(cpu.machine.timestamp(), poisoned_at);
        assert_eq!(cpu.machine.snes.apu.in_ports[0], 0x77);
    }

    #[test]
    fn apui_word_low_read_does_not_publish_open_bus_before_second_byte() {
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&synthetic_rom(&[
            0xcd, 0x40, 0x21, // CMP $2140
        ]))
        .unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = false;
        cpu.machine.snes.apu.out_ports[..2].copy_from_slice(&[0x12, 0x34]);
        install_hmax_smp_failure(&mut cpu, 1_334);
        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        // Absolute() published its high operand byte after its AddCycles.
        // The low APUI semantic committed, but mapped split reads do not run
        // S9xGetWord's true-wrap-boundary `OpenBus = low` statement.
        assert_eq!(cpu.machine.snes.open_bus, 0x21);
        assert_eq!(
            cpu.machine.pending_completion(),
            Some(CpuSynchronousCompletion::Read(0x12))
        );
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn unaudited_bus_map_fails_closed_and_poisons_the_partial_instruction() {
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&synthetic_rom(&[
            0xcd, 0x00, 0x40, // CMP $4000
        ]))
        .unwrap();
        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::UnsupportedBusMap { address: 0x4000 })
        ));
        assert!(cpu.is_poisoned());
        assert!(matches!(cpu.step(), Err(SourceCpuError::Poisoned)));
    }

    #[test]
    fn word_accesses_split_apui_but_combine_direct_wram_transactions() {
        let rom = synthetic_rom(&[
            0x18, // CLC
            0xfb, // XCE -> native
            0xc2, 0x20, // REP #$20
            0xa9, 0x34, 0x12, // LDA #$1234
            0x8d, 0x40, 0x21, // STA $2140
            0x8d, 0x00, 0x01, // STA $0100
        ]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        for _ in 0..4 {
            cpu.step().unwrap();
        }

        let apui = cpu.step().unwrap();
        let apui_writes = apui
            .accesses
            .iter()
            .filter(|access| matches!(access.kind, SourceCpuBusAccessKind::Write { .. }))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(apui_writes.len(), 2);
        assert_eq!(apui_writes[0].address, 0x2140);
        assert_eq!(apui_writes[0].timestamp.master_cycles(), 296);
        assert_eq!(apui_writes[0].charged_master_cycles, 6);
        assert_eq!(
            apui_writes[0].kind,
            SourceCpuBusAccessKind::Write {
                value: 0x34,
                width: 1
            }
        );
        assert_eq!(apui_writes[1].address, 0x2141);
        assert_eq!(apui_writes[1].timestamp.master_cycles(), 302);
        assert_eq!(apui_writes[1].charged_master_cycles, 6);
        assert_eq!(
            apui_writes[1].kind,
            SourceCpuBusAccessKind::Write {
                value: 0x12,
                width: 1
            }
        );

        let wram = cpu.step().unwrap();
        let wram_writes = wram
            .accesses
            .iter()
            .filter(|access| matches!(access.kind, SourceCpuBusAccessKind::Write { .. }))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(wram_writes.len(), 1);
        assert_eq!(wram_writes[0].address, 0x0100);
        assert_eq!(wram_writes[0].timestamp.master_cycles(), 332);
        assert_eq!(wram_writes[0].charged_master_cycles, 16);
        assert_eq!(
            wram_writes[0].kind,
            SourceCpuBusAccessKind::Write {
                value: 0x1234,
                width: 2
            }
        );
        assert_eq!(&cpu.machine.snes.ram[0x100..0x102], &[0x34, 0x12]);
    }

    fn assert_source_transaction_shape(
        receipt: &SourceCpuStepReceipt,
        expected: &[(SourceCpuTransactionKind, u8)],
    ) {
        assert_eq!(
            receipt
                .transactions
                .iter()
                .map(|transaction| (transaction.kind, transaction.duration_master_cycles))
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn and_direct_m8_charges_nonzero_dp_then_preserves_accumulator_b() {
        let rom = synthetic_rom(&[0x25, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.cpu.a = 0x12f0;
        cpu.machine.snes.ram[0x0200] = 0x80;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses[2].address, 0x00_0200);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(220));
        assert_eq!(cpu.machine.snes.cpu.a, 0x1280);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn and_direct_m16_uses_bank_wrapped_word_owner_and_publishes_high_byte() {
        let rom = synthetic_rom(&[0x25, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = false;
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.cpu.a = 0x9234;
        cpu.machine.snes.ram[0x0200] = 0xcb;
        cpu.machine.snes.ram[0x0201] = 0x6d;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses[2].address, 0x00_0200);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(220));
        assert_eq!(cpu.machine.snes.cpu.a, 0);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x6d);
    }

    #[test]
    fn and_direct_operand_failure_precedes_accumulator_and_flag_mutation() {
        let rom = synthetic_rom(&[0x25, 0x14]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.cpu.a = 0x12f0;
        cpu.machine.snes.cpu.n = false;
        cpu.machine.snes.cpu.z = true;
        cpu.machine.snes.ram[0x0115] = 0x80;
        install_hmax_smp_failure(&mut cpu, 1_334);

        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert_eq!(cpu.machine.snes.cpu.a, 0x12f0);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
        assert_eq!(cpu.program_address(), 0x00_8002);
        assert_eq!(cpu.machine.snes.open_bus, 0x14);
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn and_immediate_m8_preserves_accumulator_b_and_publishes_operand() {
        let rom = synthetic_rom(&[0x29, 0x80]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.a = 0x12f0;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
            ],
        );
        assert_eq!(receipt.accesses.len(), 2);
        assert_eq!(receipt.accesses[1].address, 0x00_8001);
        assert_eq!(receipt.accesses[1].timestamp, CpuMasterTimestamp::new(206));
        assert_eq!(cpu.machine.snes.cpu.a, 0x1280);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn and_immediate_m16_reads_one_source_operand_transaction_and_publishes_high_byte() {
        let rom = synthetic_rom(&[0x29, 0xcb, 0x6d]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = false;
        cpu.machine.snes.cpu.a = 0x9234;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
            ],
        );
        assert_eq!(receipt.accesses.len(), 2);
        assert_eq!(receipt.accesses[1].address, 0x00_8001);
        assert_eq!(receipt.accesses[1].timestamp, CpuMasterTimestamp::new(206));
        assert_eq!(
            receipt.accesses[1].kind,
            SourceCpuBusAccessKind::Read {
                value: 0x6dcb,
                width: 2,
            }
        );
        assert_eq!(cpu.machine.snes.cpu.a, 0);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x6d);
    }

    #[test]
    fn and_immediate_failure_defers_pc_open_bus_accumulator_and_flags() {
        let rom = synthetic_rom(&[0x29, 0x80]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.a = 0x12f0;
        cpu.machine.snes.cpu.n = false;
        cpu.machine.snes.cpu.z = true;
        cpu.machine.snes.open_bus = 0x5a;
        install_hmax_smp_failure(&mut cpu, 1_350);

        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert_eq!(cpu.program_address(), 0x00_8001);
        assert_eq!(cpu.machine.snes.open_bus, 0x5a);
        assert_eq!(cpu.machine.snes.cpu.a, 0x12f0);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn pld_native_adds_two_cycles_then_pulls_word_and_publishes_high_byte() {
        let rom = synthetic_rom(&[0x2b]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.sp = 0x01fd;
        cpu.machine.snes.cpu.dp = 0x7777;
        cpu.machine.snes.ram[0x01fe] = 0x34;
        cpu.machine.snes.ram[0x01ff] = 0x92;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 12),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 2);
        assert_eq!(receipt.accesses[1].address, 0x00_01fe);
        assert_eq!(receipt.accesses[1].timestamp, CpuMasterTimestamp::new(218));
        assert_eq!(cpu.machine.snes.cpu.sp, 0x01ff);
        assert_eq!(cpu.machine.snes.cpu.dp, 0x9234);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn pld_emulation_uses_pullw_bank_semantics_before_repairing_stack_high_byte() {
        let rom = synthetic_rom(&[0x2b]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.sp = 0x01ff;
        cpu.machine.snes.ram[0x0200] = 0xcd;
        cpu.machine.snes.ram[0x0201] = 0xab;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_eq!(receipt.accesses.len(), 2);
        assert_eq!(receipt.accesses[1].address, 0x00_0200);
        assert_eq!(
            receipt.accesses[1].kind,
            SourceCpuBusAccessKind::Read {
                value: 0xabcd,
                width: 2,
            }
        );
        assert_eq!(cpu.machine.snes.cpu.sp, 0x0101);
        assert_eq!(cpu.machine.snes.cpu.dp, 0xabcd);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0xab);
    }

    #[test]
    fn pld_pull_failure_retains_stack_dp_flags_and_open_bus_after_semantic() {
        let rom = synthetic_rom(&[0x2b]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.sp = 0x01fd;
        cpu.machine.snes.cpu.dp = 0x7777;
        cpu.machine.snes.cpu.n = false;
        cpu.machine.snes.cpu.z = true;
        cpu.machine.snes.ram[0x01fe] = 0x34;
        cpu.machine.snes.ram[0x01ff] = 0x92;
        cpu.machine.snes.open_bus = 0x5a;
        install_hmax_smp_failure(&mut cpu, 1_334);

        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert_eq!(cpu.machine.snes.cpu.sp, 0x01fd);
        assert_eq!(cpu.machine.snes.cpu.dp, 0x7777);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x5a);
        assert_eq!(
            cpu.machine.pending_completion(),
            Some(CpuSynchronousCompletion::ReadWord(0x9234))
        );
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn ply_x8_adds_two_cycles_then_pulls_page_wrapped_byte() {
        let rom = synthetic_rom(&[0x7a]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.sp = 0x01ff;
        cpu.machine.snes.cpu.y = 0x0077;
        cpu.machine.snes.ram[0x0100] = 0x80;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 12),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 2);
        assert_eq!(receipt.accesses[1].address, 0x00_0100);
        assert_eq!(receipt.accesses[1].timestamp, CpuMasterTimestamp::new(218));
        assert_eq!(cpu.machine.snes.cpu.sp, 0x0100);
        assert_eq!(cpu.machine.snes.cpu.y, 0x0080);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn ply_x16_pulls_bank_wrapped_word_and_publishes_high_byte() {
        let rom = synthetic_rom(&[0x7a]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.sp = 0x01fd;
        cpu.machine.snes.ram[0x01fe] = 0x34;
        cpu.machine.snes.ram[0x01ff] = 0x92;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 12),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses[1].address, 0x00_01fe);
        assert_eq!(cpu.machine.snes.cpu.sp, 0x01ff);
        assert_eq!(cpu.machine.snes.cpu.y, 0x9234);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn ply_byte_failure_retains_prepull_registers_but_owns_incremented_stack() {
        let rom = synthetic_rom(&[0x7a]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.sp = 0x01fe;
        cpu.machine.snes.cpu.y = 0x0077;
        cpu.machine.snes.cpu.n = false;
        cpu.machine.snes.cpu.z = true;
        cpu.machine.snes.ram[0x01ff] = 0x80;
        cpu.machine.snes.open_bus = 0x5a;
        install_hmax_smp_failure(&mut cpu, 1_338);

        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert_eq!(cpu.machine.snes.cpu.sp, 0x01ff);
        assert_eq!(cpu.machine.snes.cpu.y, 0x0077);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x5a);
        assert_eq!(
            cpu.machine.pending_completion(),
            Some(CpuSynchronousCompletion::Read(0x80))
        );
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn plx_x8_shares_page_wrapped_pull_and_flag_publication_order() {
        let rom = synthetic_rom(&[0xfa]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.sp = 0x01ff;
        cpu.machine.snes.cpu.x = 0x0077;
        cpu.machine.snes.ram[0x0100] = 0x80;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 12),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses[1].address, 0x00_0100);
        assert_eq!(cpu.machine.snes.cpu.sp, 0x0100);
        assert_eq!(cpu.machine.snes.cpu.x, 0x0080);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn plx_x16_shares_bank_word_pull_and_high_byte_publication_order() {
        let rom = synthetic_rom(&[0xfa]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.sp = 0x01fd;
        cpu.machine.snes.ram[0x01fe] = 0x34;
        cpu.machine.snes.ram[0x01ff] = 0x92;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 12),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses[1].address, 0x00_01fe);
        assert_eq!(cpu.machine.snes.cpu.sp, 0x01ff);
        assert_eq!(cpu.machine.snes.cpu.x, 0x9234);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn eor_direct_m8_charges_nonzero_dp_then_preserves_accumulator_b() {
        let rom = synthetic_rom(&[0x45, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.cpu.a = 0x12c0;
        cpu.machine.snes.ram[0x0200] = 0x40;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses[2].address, 0x00_0200);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(220));
        assert_eq!(cpu.machine.snes.cpu.a, 0x1280);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x40);
    }

    #[test]
    fn eor_direct_m16_reads_wrapped_bank_word_and_publishes_high_byte() {
        let rom = synthetic_rom(&[0x45, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = false;
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.cpu.a = 0x9234;
        cpu.machine.snes.ram[0x0200] = 0x34;
        cpu.machine.snes.ram[0x0201] = 0x92;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses[2].address, 0x00_0200);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(220));
        assert_eq!(cpu.machine.snes.cpu.a, 0);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn eor_direct_operand_failure_precedes_accumulator_and_flag_mutation() {
        let rom = synthetic_rom(&[0x45, 0x14]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.cpu.a = 0x12c0;
        cpu.machine.snes.cpu.n = false;
        cpu.machine.snes.cpu.z = true;
        cpu.machine.snes.ram[0x0115] = 0x40;
        install_hmax_smp_failure(&mut cpu, 1_334);

        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert_eq!(cpu.machine.snes.cpu.a, 0x12c0);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
        assert_eq!(cpu.program_address(), 0x00_8002);
        assert_eq!(cpu.machine.snes.open_bus, 0x14);
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn asl_accumulator_m8_preserves_b_and_sets_carry_and_sign() {
        let rom = synthetic_rom(&[0x0a]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.a = 0x12c1;
        cpu.machine.snes.cpu.c = false;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
            ],
        );
        assert_eq!(receipt.accesses.len(), 1);
        assert_eq!(cpu.machine.snes.cpu.a, 0x1282);
        assert!(cpu.machine.snes.cpu.c);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
    }

    #[test]
    fn asl_accumulator_m16_uses_bit_fifteen_and_wraps_to_zero() {
        let rom = synthetic_rom(&[0x0a]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = false;
        cpu.machine.snes.cpu.a = 0x8000;
        cpu.machine.snes.cpu.c = false;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
            ],
        );
        assert_eq!(receipt.accesses.len(), 1);
        assert_eq!(cpu.machine.snes.cpu.a, 0);
        assert!(cpu.machine.snes.cpu.c);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
    }

    #[test]
    fn tay_x8_transfers_low_accumulator_into_zero_extended_index_state() {
        let rom = synthetic_rom(&[0xa8]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.a = 0x12c1;
        cpu.machine.snes.cpu.y = 0x005a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
            ],
        );
        // CpuState maintains the architectural X8 invariant that index high
        // bytes are zero; this mirrors the existing TAX owner rather than
        // preserving a forged, noncanonical YH value.
        assert_eq!(cpu.machine.snes.cpu.y, 0x00c1);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
    }

    #[test]
    fn tay_x16_transfers_full_accumulator_and_sets_word_flags() {
        let rom = synthetic_rom(&[0xa8]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.a = 0x8000;
        cpu.machine.snes.cpu.y = 0x1234;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
            ],
        );
        assert_eq!(cpu.machine.snes.cpu.y, 0x8000);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
    }

    #[test]
    fn tay_event_failure_precedes_register_and_flag_mutation() {
        let rom = synthetic_rom(&[0xa8]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.a = 0x12c1;
        cpu.machine.snes.cpu.y = 0x005a;
        cpu.machine.snes.cpu.n = false;
        cpu.machine.snes.cpu.z = true;
        install_hmax_smp_failure(&mut cpu, 1_350);

        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert_eq!(cpu.machine.snes.cpu.a, 0x12c1);
        assert_eq!(cpu.machine.snes.cpu.y, 0x005a);
        assert!(!cpu.machine.snes.cpu.n);
        assert!(cpu.machine.snes.cpu.z);
        assert_eq!(cpu.program_address(), 0x00_8001);
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn jmp_absolute_indexed_indirect_uses_slow_operand_internal_and_pointer_transactions() {
        let mut rom = synthetic_rom(&[0x7c, 0x00, 0x81]);
        rom[0x0102] = 0x34;
        rom[0x0103] = 0x92;
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.x = 2;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 3);
        assert_eq!(receipt.accesses[1].address, 0x00_8001);
        assert_eq!(receipt.accesses[1].timestamp, CpuMasterTimestamp::new(206));
        assert_eq!(receipt.accesses[2].address, 0x00_8102);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(228));
        assert_eq!(cpu.program_address(), 0x00_9234);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn jmp_absolute_indexed_indirect_wraps_pointer_read_inside_program_bank() {
        let mut rom = synthetic_rom(&[0x7c, 0xfd, 0xff]);
        rom[0x7fff] = 0x34;
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.x = 2;
        cpu.machine.snes.ram[0] = 0x92;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 4);
        assert_eq!(receipt.accesses[2].address, 0x00_ffff);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(228));
        assert_eq!(receipt.accesses[3].address, 0x00_0000);
        assert_eq!(receipt.accesses[3].timestamp, CpuMasterTimestamp::new(236));
        assert_eq!(cpu.program_address(), 0x00_9234);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn jmp_absolute_indexed_indirect_exposes_sequential_pc_during_internal_cycle_event() {
        let mut rom = synthetic_rom(&[0x7c, 0x00, 0x81]);
        rom[0x0102] = 0x34;
        rom[0x0103] = 0x92;
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.x = 2;
        install_hmax_smp_failure(&mut cpu, 1338);

        assert!(matches!(cpu.step(), Err(SourceCpuError::Machine(_))));

        // Immediate16Slow increments PC after its mapped word transaction.
        // The following AddCycles(ONE_CYCLE) owns this failing HMax, before
        // the pointer read or target PC publication.
        assert_eq!(cpu.program_address(), 0x00_8003);
        assert_eq!(cpu.machine.snes.open_bus, 0x81);
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn joyser0_rising_edge_latches_both_inputs_before_drain_and_publishes_after_success() {
        let rom = synthetic_rom(&[0x8d, 0x16, 0x40]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.a = 1;
        cpu.machine.snes.input1.current_state = 0x1234;
        cpu.machine.snes.input2.current_state = 0xabcd;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    12,
                ),
            ],
        );
        assert!(cpu.machine.snes.input1.latch_line);
        assert!(cpu.machine.snes.input2.latch_line);
        assert_eq!(cpu.machine.snes.input1.latched_state, 0x1234);
        assert_eq!(cpu.machine.snes.input2.latched_state, 0xabcd);
        assert_eq!(cpu.machine.snes.open_bus, 1);

        let mut observed = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        observed.machine.snes.cpu.a = 1;
        observed.machine.snes.input1.current_state = 0x1234;
        observed.machine.snes.input2.current_state = 0xabcd;
        observed.machine.snes.open_bus = 0x5a;
        install_hmax_smp_failure(&mut observed, 1_334);
        assert!(matches!(
            observed.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert!(observed.machine.snes.input1.latch_line);
        assert!(observed.machine.snes.input2.latch_line);
        assert_eq!(observed.machine.snes.input1.latched_state, 0x1234);
        assert_eq!(observed.machine.snes.input2.latched_state, 0xabcd);
        assert_eq!(observed.machine.snes.open_bus, 0x5a);
        assert_eq!(
            observed.machine.pending_completion(),
            Some(CpuSynchronousCompletion::Write)
        );
    }

    #[test]
    fn joyser0_repeated_high_does_not_relatch_changed_current_state() {
        let rom = synthetic_rom(&[0x8d, 0x16, 0x40, 0x8d, 0x16, 0x40]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.a = 1;
        cpu.machine.snes.input1.current_state = 0x1234;
        cpu.machine.snes.input2.current_state = 0xabcd;
        cpu.step().unwrap();

        cpu.machine.snes.input1.current_state = 0x5678;
        cpu.machine.snes.input2.current_state = 0xef01;
        cpu.step().unwrap();

        assert!(cpu.machine.snes.input1.latch_line);
        assert!(cpu.machine.snes.input2.latch_line);
        assert_eq!(cpu.machine.snes.input1.latched_state, 0x1234);
        assert_eq!(cpu.machine.snes.input2.latched_state, 0xabcd);
    }

    #[test]
    fn joyser0_falling_edge_only_clears_both_latch_lines() {
        let rom = synthetic_rom(&[0x8d, 0x16, 0x40]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.a = 0;
        cpu.machine.snes.input1.latch_line = true;
        cpu.machine.snes.input2.latch_line = true;
        cpu.machine.snes.input1.latched_state = 0x1234;
        cpu.machine.snes.input2.latched_state = 0xabcd;

        cpu.step().unwrap();

        assert!(!cpu.machine.snes.input1.latch_line);
        assert!(!cpu.machine.snes.input2.latch_line);
        assert_eq!(cpu.machine.snes.input1.latched_state, 0x1234);
        assert_eq!(cpu.machine.snes.input2.latched_state, 0xabcd);
        assert_eq!(cpu.machine.snes.open_bus, 0);
    }

    #[test]
    fn auto_joy_result_m8_reads_all_four_words_in_little_endian_port_order() {
        let words = [0x1234, 0xabcd, 0x00ff, 0x8001];
        let expected = [0x34, 0x12, 0xcd, 0xab, 0xff, 0x00, 0x01, 0x80];

        for (offset, expected) in expected.into_iter().enumerate() {
            let address = 0x4218u16 + offset as u16;
            let [low, high] = address.to_le_bytes();
            let rom = synthetic_rom(&[0xad, low, high]);
            let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
            cpu.machine.snes.port_auto_read = words;

            let receipt = cpu.step().unwrap();

            assert_source_transaction_shape(
                &receipt,
                &[
                    (
                        SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                        8,
                    ),
                    (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                    (
                        SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                        6,
                    ),
                ],
            );
            assert_eq!(receipt.accesses[2].address, u32::from(address));
            assert_eq!(cpu.machine.snes.cpu.a as u8, expected);
            assert_eq!(cpu.machine.snes.open_bus, expected);
        }
    }

    #[test]
    fn auto_joy_result_m16_splits_cpu_register_semantics_and_publishes_only_final_high() {
        let rom = synthetic_rom(&[0xad, 0x1a, 0x42]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = false;
        cpu.machine.snes.cpu.a = 0x7777;
        cpu.machine.snes.port_auto_read[1] = 0xabcd;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    6,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    6,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 4);
        assert_eq!(receipt.accesses[2].address, 0x00_421a);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(222));
        assert_eq!(receipt.accesses[3].address, 0x00_421b);
        assert_eq!(receipt.accesses[3].timestamp, CpuMasterTimestamp::new(228));
        assert_eq!(cpu.machine.snes.cpu.a, 0xabcd);
        assert_eq!(cpu.machine.snes.open_bus, 0xab);

        let mut observed = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        observed.machine.snes.cpu.e = false;
        observed.machine.snes.cpu.mf = false;
        observed.machine.snes.cpu.a = 0x7777;
        observed.machine.snes.port_auto_read[1] = 0xabcd;
        install_hmax_smp_failure(&mut observed, 1_334);
        assert!(matches!(
            observed.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert_eq!(observed.machine.snes.open_bus, 0x42);
        assert_eq!(observed.machine.snes.cpu.a, 0x7777);
        assert_eq!(
            observed.machine.pending_completion(),
            Some(CpuSynchronousCompletion::Read(0xcd))
        );
    }

    #[test]
    fn ldx_absolute_x8_reads_after_operand_and_publishes_loaded_byte() {
        let rom = synthetic_rom(&[0xae, 0x34, 0x12]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.db = 0x7e;
        cpu.machine.snes.ram[0x1234] = 0x80;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 3);
        assert_eq!(receipt.accesses[2].address, 0x7e_1234);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(222));
        assert_eq!(cpu.machine.snes.cpu.x, 0x0080);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn ldx_absolute_x16_uses_bank_wrap_and_publishes_high_byte_last() {
        let rom = synthetic_rom(&[0xae, 0xff, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.db = 0x7e;
        cpu.machine.snes.ram[0xffff] = 0x34;
        cpu.machine.snes.ram[0x0000] = 0x92;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 4);
        assert_eq!(receipt.accesses[2].address, 0x7e_ffff);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(222));
        assert_eq!(receipt.accesses[3].address, 0x7e_0000);
        assert_eq!(receipt.accesses[3].timestamp, CpuMasterTimestamp::new(230));
        assert_eq!(cpu.machine.snes.cpu.x, 0x9234);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn stz_direct_m8_fetches_dp_operand_then_publishes_zero_after_store() {
        let rom = synthetic_rom(&[0x64, 0x15]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.ram[0x15] = 0xa5;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 3);
        assert_eq!(receipt.accesses[2].address, 0x00_0015);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(214));
        assert_eq!(cpu.machine.snes.ram[0x15], 0);
        assert_eq!(cpu.machine.snes.open_bus, 0);
    }

    #[test]
    fn stz_direct_m16_charges_nonzero_dp_before_the_word_store() {
        let rom = synthetic_rom(&[0x64, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.mf = false;
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.ram[0x0200] = 0xa5;
        cpu.machine.snes.ram[0x0201] = 0x5a;
        cpu.machine.snes.open_bus = 0x7c;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 3);
        assert_eq!(receipt.accesses[2].address, 0x00_0200);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(220));
        assert_eq!(receipt.accesses[2].charged_master_cycles, 16);
        assert_eq!(
            receipt.accesses[2].kind,
            SourceCpuBusAccessKind::Write { value: 0, width: 2 }
        );
        assert_eq!(cpu.machine.snes.ram[0x0200], 0);
        assert_eq!(cpu.machine.snes.ram[0x0201], 0);
        assert_eq!(cpu.machine.snes.open_bus, 0);
    }

    #[test]
    fn sty_direct_x8_charges_nonzero_dp_then_publishes_y_after_store() {
        let rom = synthetic_rom(&[0x84, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.cpu.y = 0x0080;
        cpu.machine.snes.ram[0x0200] = 0x5a;
        cpu.machine.snes.open_bus = 0x7c;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 3);
        assert_eq!(receipt.accesses[2].address, 0x00_0200);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(220));
        assert_eq!(
            receipt.accesses[2].kind,
            SourceCpuBusAccessKind::Write {
                value: 0x80,
                width: 1,
            }
        );
        assert_eq!(cpu.machine.snes.ram[0x0200], 0x80);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn sty_direct_x16_uses_bank_wrapped_word_owner_and_publishes_high_byte() {
        let rom = synthetic_rom(&[0x84, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.cpu.y = 0x9234;
        cpu.machine.snes.ram[0x0200] = 0xa5;
        cpu.machine.snes.ram[0x0201] = 0x5a;
        cpu.machine.snes.open_bus = 0x7c;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 3);
        assert_eq!(receipt.accesses[2].address, 0x00_0200);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(220));
        assert_eq!(
            receipt.accesses[2].kind,
            SourceCpuBusAccessKind::Write {
                value: 0x9234,
                width: 2,
            }
        );
        assert_eq!(&cpu.machine.snes.ram[0x0200..0x0202], &[0x34, 0x92]);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn sty_direct_write_failure_commits_semantic_before_event_but_defers_open_bus() {
        let rom = synthetic_rom(&[0x84, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.cpu.y = 0x0080;
        cpu.machine.snes.ram[0x0200] = 0x5a;
        cpu.machine.snes.open_bus = 0x7c;
        install_hmax_smp_failure(&mut cpu, 1_334);

        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::ApuClock(crate::Snes9xApuClockError::ZeroCycleSmpStep)
            ))
        ));
        assert_eq!(cpu.machine.snes.ram[0x0200], 0x80);
        assert_eq!(cpu.machine.snes.open_bus, 0x7c);
        assert_eq!(
            cpu.machine.pending_completion(),
            Some(CpuSynchronousCompletion::Write)
        );
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn ldy_direct_x8_reads_after_dp_operand_and_publishes_loaded_byte() {
        let rom = synthetic_rom(&[0xa4, 0x15]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.ram[0x15] = 0x80;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 3);
        assert_eq!(receipt.accesses[2].address, 0x00_0015);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(214));
        assert_eq!(cpu.machine.snes.cpu.y, 0x0080);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn ldy_direct_x16_charges_nonzero_dp_and_publishes_high_byte() {
        let rom = synthetic_rom(&[0xa4, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.ram[0x0200] = 0x34;
        cpu.machine.snes.ram[0x0201] = 0x92;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 3);
        assert_eq!(receipt.accesses[2].address, 0x00_0200);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(220));
        assert_eq!(cpu.machine.snes.cpu.y, 0x9234);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn ldx_direct_x8_uses_the_shared_index_load_source_order() {
        let rom = synthetic_rom(&[0xa6, 0x15]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.ram[0x15] = 0x80;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(receipt.accesses[2].address, 0x00_0015);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(214));
        assert_eq!(cpu.machine.snes.cpu.x, 0x0080);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x80);
    }

    #[test]
    fn ldx_direct_x16_charges_nonzero_dp_and_publishes_high_byte() {
        let rom = synthetic_rom(&[0xa6, 0xff]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.dp = 0x0101;
        cpu.machine.snes.ram[0x0200] = 0x34;
        cpu.machine.snes.ram[0x0201] = 0x92;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(receipt.accesses[2].address, 0x00_0200);
        assert_eq!(receipt.accesses[2].timestamp, CpuMasterTimestamp::new(220));
        assert_eq!(cpu.machine.snes.cpu.x, 0x9234);
        assert!(cpu.machine.snes.cpu.n);
        assert!(!cpu.machine.snes.cpu.z);
        assert_eq!(cpu.machine.snes.open_bus, 0x92);
    }

    #[test]
    fn stx_absolute_uses_source_width_order_and_dma_register_owner() {
        let rom = synthetic_rom(&[0x8e, 0x00, 0x43]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.e = false;
        cpu.machine.snes.cpu.xf = false;
        cpu.machine.snes.cpu.x = 0x1801;
        cpu.machine.snes.open_bus = 0x5a;

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    6,
                ),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    6,
                ),
            ],
        );
        let writes = receipt
            .accesses
            .iter()
            .filter(|access| matches!(access.kind, SourceCpuBusAccessKind::Write { .. }))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(writes.len(), 2);
        assert_eq!(writes[0].address, 0x00_4300);
        assert_eq!(writes[0].timestamp.master_cycles(), 222);
        assert_eq!(
            writes[0].kind,
            SourceCpuBusAccessKind::Write {
                value: 0x01,
                width: 1
            }
        );
        assert_eq!(writes[1].address, 0x00_4301);
        assert_eq!(writes[1].timestamp.master_cycles(), 228);
        assert_eq!(
            writes[1].kind,
            SourceCpuBusAccessKind::Write {
                value: 0x18,
                width: 1
            }
        );
        assert_eq!(cpu.machine.timestamp().master_cycles(), 234);
        assert_eq!(cpu.machine.snes.cpu.pc, 0x8003);
        assert_eq!(cpu.machine.snes.open_bus, 0x18);
        assert_eq!(cpu.machine.snes.dma.channel[0].mode, 1);
        assert_eq!(cpu.machine.snes.dma.channel[0].b_adr, 0x18);
        assert!(!cpu.machine.snes.dma.channel[0].fixed);
        assert!(!cpu.machine.snes.dma.channel[0].decrement);
        assert!(!cpu.machine.snes.dma.channel[0].indirect);
        assert!(!cpu.machine.snes.dma.channel[0].from_b);
    }

    #[test]
    fn stx_absolute_eight_bit_and_raw_dma_semantics_preserve_source_open_bus() {
        let rom = synthetic_rom(&[0x8e, 0x00, 0x43]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.cpu.x = 0xab;
        cpu.machine.snes.open_bus = 0x5a;

        cpu.source_write_semantic(0x80_4311, 0x24).unwrap();
        assert_eq!(cpu.machine.snes.open_bus, 0x5a);
        assert_eq!(cpu.machine.snes.dma.channel[1].b_adr, 0x24);

        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    6,
                ),
            ],
        );
        assert_eq!(cpu.machine.timestamp().master_cycles(), 228);
        assert_eq!(cpu.machine.snes.open_bus, 0xab);
        assert_eq!(cpu.machine.snes.dma_read_reg(0x4300), 0xab);
        assert_eq!(cpu.machine.snes.dma.channel[0].b_adr, 0xff);
    }

    #[test]
    fn nonzero_mdmaen_runs_nested_dma_before_the_outer_cpu_write_charge() {
        let rom = synthetic_rom(&[0xa9, 0x01, 0x8d, 0x0b, 0x42]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.ram[0] = 0x7c;
        let dma = &mut cpu.machine.snes.dma.channel[0];
        dma.a_bank = 0x7e;
        dma.a_adr = 0;
        dma.size = 1;
        dma.mode = 0;
        dma.b_adr = 0x18;
        dma.fixed = false;
        dma.decrement = false;
        dma.from_b = false;

        cpu.step().unwrap();
        let receipt = cpu.step().unwrap();

        assert_source_transaction_shape(
            &receipt,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    6,
                ),
            ],
        );
        assert_eq!(receipt.accesses.len(), 3);
        let write = receipt.accesses.last().unwrap();
        assert_eq!(write.address, 0x00_420b);
        assert_eq!(write.timestamp, CpuMasterTimestamp::new(238));
        assert_eq!(write.charged_master_cycles, 6);
        assert_eq!(
            receipt.transactions[2].started_at,
            CpuMasterTimestamp::new(272)
        );
        assert_eq!(
            receipt.transactions[2].ended_at,
            CpuMasterTimestamp::new(278)
        );
        assert_eq!(cpu.machine.snes.ppu.vram[0], 0x007c);
        assert_eq!(cpu.machine.snes.open_bus, 0x01);
        assert_eq!(cpu.machine.snes.cpu.pc, 0x8005);
    }

    #[test]
    fn nonzero_hdmaen_remains_fail_closed_before_register_semantics() {
        let rom = synthetic_rom(&[0xa9, 0x01, 0x8d, 0x0c, 0x42]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.step().unwrap();
        let before = cpu.machine.timestamp();

        assert_eq!(cpu.step(), Err(SourceCpuError::UnexpectedInterruptOrDma));

        // Opcode fetch and the absolute operand are source-owned, but neither
        // HDMA enable state nor the enclosing write semantic/charge commits.
        assert_eq!(before, CpuMasterTimestamp::new(214));
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(238));
        assert!(cpu
            .machine
            .snes
            .dma
            .channel
            .iter()
            .all(|channel| !channel.hdma_active));
        assert!(cpu.is_poisoned());
    }

    #[test]
    fn startup_init_opcode_and_map_delta_follows_snes9x_source_order() {
        let rom = synthetic_rom(&[
            0x18, // CLC
            0xfb, // XCE -> native
            0xc2, 0x30, // REP #$30
            0xac, 0xfe, 0x01, // LDY $01fe
            0xa2, 0x02, 0x00, // LDX #$0002
            0xa9, 0x34, 0x12, // LDA #$1234
            0x9d, 0x00, 0x01, // STA $0100,X
            0x8f, 0xe5, 0x03, 0x70, // STA $7003e5
            0xaf, 0xe5, 0x03, 0x70, // LDA $7003e5
            0xc9, 0x34, 0x12, // CMP #$1234
            0x8c, 0xfe, 0x01, // STY $01fe
            0xe2, 0x30, // SEP #$30
            0xa5, 0x12, // LDA $12
            0x9c, 0x2e, 0x21, // STZ $212e
        ]);
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        cpu.machine.snes.ram[0x01fe..0x0200].copy_from_slice(&0xbeefu16.to_le_bytes());
        cpu.machine.snes.ram[0x12] = 0x7a;
        cpu.machine.snes.ppu.screen_windowed[0] = 0xff;

        for _ in 0..3 {
            cpu.step().unwrap();
        }

        let ldy = cpu.step().unwrap();
        assert_source_transaction_shape(
            &ldy,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(cpu.machine.snes.cpu.y, 0xbeef);
        assert_eq!(cpu.machine.snes.open_bus, 0xbe);

        let ldx = cpu.step().unwrap();
        assert_source_transaction_shape(
            &ldx,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
            ],
        );
        cpu.step().unwrap(); // LDA #$1234

        let sta_indexed = cpu.step().unwrap();
        assert_source_transaction_shape(
            &sta_indexed,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 6),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(&cpu.machine.snes.ram[0x102..0x104], &[0x34, 0x12]);

        let sta_long = cpu.step().unwrap();
        assert_source_transaction_shape(
            &sta_long,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 24),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(
            sta_long.accesses[1].kind,
            SourceCpuBusAccessKind::ReadLong { value: 0x70_03e5 }
        );
        assert_eq!(&cpu.machine.snes.cart.ram[0x3e5..0x3e7], &[0x34, 0x12]);

        let lda_long = cpu.step().unwrap();
        assert_source_transaction_shape(
            &lda_long,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 24),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(cpu.machine.snes.cpu.a, 0x1234);

        let cmp = cpu.step().unwrap();
        assert_source_transaction_shape(
            &cmp,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
            ],
        );
        assert!(cpu.machine.snes.cpu.z);

        let sty = cpu.step().unwrap();
        assert_source_transaction_shape(
            &sty,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                    16,
                ),
            ],
        );
        assert_eq!(&cpu.machine.snes.ram[0x1fe..0x200], &[0xef, 0xbe]);

        cpu.step().unwrap(); // SEP #$30
        let lda_direct = cpu.step().unwrap();
        assert_source_transaction_shape(
            &lda_direct,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 8),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    8,
                ),
            ],
        );
        assert_eq!(cpu.machine.snes.cpu.a as u8, 0x7a);

        let stz_ppu = cpu.step().unwrap();
        assert_source_transaction_shape(
            &stz_ppu,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    6,
                ),
            ],
        );
        assert_eq!(cpu.machine.snes.ppu.screen_windowed[0], 0);
        assert_eq!(cpu.machine.snes.open_bus, 0);
    }

    #[test]
    #[ignore = "requires the local Zelda 3 ROM and captured cold-route SRAM"]
    fn local_zelda_rom_matches_post_handoff_cpu_through_first_nmi_apui_read() {
        let rom_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3.sfc");
        let rom = std::fs::read(rom_path).expect("local zelda3.sfc is required for this proof");
        let sram_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../routes/full_run/comparisons/continuous-audio/initial.srm");
        let sram =
            std::fs::read(sram_path).expect("captured route SRAM is required for this proof");
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&sram)).unwrap();

        // First consume the permanent reset-through-IPL transaction stream so
        // this test begins at the exact committed `$88fc STZ $2143` boundary.
        let bootstrap = records()
            .into_iter()
            .find(|record| record["kind"] == "bootstrap-events")
            .unwrap();
        let mut actual = VecDeque::new();
        let mut compared = 0usize;
        let mut terminal_field_index = 0u64;
        visit_cpu_timing_transactions(&bootstrap, |expected| {
            while actual.is_empty() {
                actual.extend(cpu.step().unwrap().transactions);
            }
            assert_timing_transaction(compared, actual.pop_front().unwrap(), expected);
            terminal_field_index = expected.end_field_index;
            compared += 1;
        });
        assert!(actual.is_empty());

        let receipt = post_handoff_first_nmi_record();
        let marker = &receipt["first_hmax_crossing_transaction"]["transaction"];
        let mut expected = Vec::new();
        visit_cpu_timing_transactions(&receipt, |mut transaction| {
            transaction.start_field_index += terminal_field_index;
            transaction.end_field_index += terminal_field_index;
            expected.push(transaction);
        });
        let crossing = expected
            .iter()
            .position(|transaction| {
                transaction.kind == marker["kind"].as_u64().unwrap() as u8
                    && transaction.duration == marker["duration"].as_u64().unwrap() as u8
                    && transaction.origin_pc == marker["origin_pc"].as_u64().unwrap() as u32
                    && transaction.opcode == marker["opcode"].as_u64().unwrap() as u8
                    && transaction.start_v_counter
                        == marker["start_v_counter"].as_u64().unwrap() as u16
                    && transaction.start_cpu_cycle
                        == marker["start_cpu_cycle"].as_u64().unwrap() as u16
            })
            .expect("fixture omitted its declared first HMax crossing");
        let opcode_fetch = expected[crossing - 1];
        assert_eq!(opcode_fetch.kind, 0);
        assert_eq!(opcode_fetch.origin_pc, 0x00_87da);
        assert_eq!(opcode_fetch.opcode, 0x9d);
        assert_eq!(opcode_fetch.duration, 8);
        assert_eq!(opcode_fetch.start_cpu_cycle, 1_360);
        assert_eq!(opcode_fetch.end_cpu_cycle, 1_368);

        // Every transaction before the `$87da` instruction is a complete
        // instruction receipt and can be compared tuple-for-tuple.
        actual.clear();
        for (index, expected) in expected[..crossing - 1].iter().copied().enumerate() {
            while actual.is_empty() {
                actual.extend(cpu.step().unwrap().transactions);
            }
            assert_timing_transaction(index, actual.pop_front().unwrap(), expected);
        }
        assert!(actual.is_empty());
        assert_eq!(cpu.machine.snes.cpu.pc, 0x87da);
        assert_eq!(
            cpu.machine.timestamp().master_cycles(),
            opcode_fetch.absolute_start_master_cycle()
        );
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(120, 1_360)
        );

        // Continue transaction-for-transaction through the committed first-NMI
        // receipt. This intentionally exercises the same source CPU owner
        // across scanline/APU boundaries rather than restarting at a raster
        // checkpoint.
        for (index, expected) in expected[crossing - 1..].iter().copied().enumerate() {
            while actual.is_empty() {
                actual.extend(cpu.step().unwrap().transactions);
            }
            assert_timing_transaction(crossing - 1 + index, actual.pop_front().unwrap(), expected);
        }
        assert!(actual.is_empty());
        assert!(cpu.machine.snes.nmi_enabled);
        assert!(cpu.machine.snes.auto_joy_read);
        assert!(cpu.machine.snes.in_vblank);
        assert!(!cpu.machine.snes.in_nmi);
        assert!(!cpu.machine.snes.cpu.nmi_wanted);
        assert_eq!(cpu.machine.nmi_acceptance_not_before, None);
        assert_eq!(cpu.machine.snes.cpu.k, 0);
        assert_eq!(cpu.machine.snes.cpu.sp, 0x01f2);
        assert!(cpu.machine.snes.cpu.i);
        assert!(!cpu.machine.snes.cpu.d);

        // Continue the same CPU owner through every authoritative source
        // transaction after the completed $8080e1 anchor. The fixture ends
        // after $8a33 and deliberately excludes $8a35's raw opcode fetch.
        let setup = first_nmi_dma_setup_record();
        assert_eq!(
            setup["cpu_timing_transaction_sequence"]["record_count"],
            157
        );
        assert_eq!(setup["stop_before_instruction"]["origin_pc"], 0x00_8a35);
        let setup_start_field_index = expected.last().unwrap().end_field_index;
        let mut setup_expected = Vec::new();
        visit_cpu_timing_transactions(&setup, |mut transaction| {
            transaction.start_field_index += setup_start_field_index;
            transaction.end_field_index += setup_start_field_index;
            setup_expected.push(transaction);
        });
        assert_eq!(setup_expected.first().unwrap().origin_pc, 0x00_80e4);
        assert_eq!(setup_expected.last().unwrap().origin_pc, 0x00_8a33);

        actual.clear();
        for (index, expected) in setup_expected.into_iter().enumerate() {
            while actual.is_empty() {
                actual.extend(cpu.step().unwrap().transactions);
            }
            assert_timing_transaction(index, actual.pop_front().unwrap(), expected);
        }
        assert!(actual.is_empty());

        assert_eq!(cpu.program_address(), 0x00_8a35);
        assert_eq!(cpu.machine.snes.cpu.a as u8, 0x07);
        assert_eq!(cpu.machine.snes.ppu.vram_pointer, 0x4100);
        for (channel, source_address, size) in [
            (0usize, 0x0aceusize, 0x0040u16),
            (1usize, 0x0ad2usize, 0x0040u16),
            (2usize, 0x0ad6usize, 0x0020u16),
        ] {
            let dma = cpu.machine.snes.dma.channel[channel];
            assert_eq!(dma.mode, 1);
            assert_eq!(dma.b_adr, 0x18);
            assert_eq!(dma.a_bank, 0x10);
            assert_eq!(
                dma.a_adr,
                u16::from_le_bytes([
                    cpu.machine.snes.ram[source_address],
                    cpu.machine.snes.ram[source_address + 1],
                ])
            );
            assert_eq!(dma.size, size);
            assert!(!dma.dma_active);
        }
        assert!(!cpu.machine.snes.dma.dma_busy);
        assert!(cpu
            .machine
            .snes
            .dma
            .channel
            .iter()
            .all(|channel| !channel.dma_active));

        // The next source instruction is the bounded nonzero MDMAEN path. At
        // the authoritative H714 stop, its semantic begins at H738; channel 1
        // zero-based byte 9 ends at H1364 and drains HMax, then all 160 bytes
        // and the outer write finish on the following line at H742 (including
        // that line's 40-clock WRAM
        // refresh). No PC/mask special case participates.
        let dma = cpu.step().unwrap();
        assert_eq!(dma.origin_pc, 0x00_8a35);
        assert_eq!(dma.opcode, 0x8d);
        assert_source_transaction_shape(
            &dma,
            &[
                (
                    SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
                    8,
                ),
                (SourceCpuTransactionKind::CpuOpsAddCyclesDraining, 16),
                (
                    SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
                    6,
                ),
            ],
        );
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(227, 742)
        );
        assert_eq!(cpu.program_address(), 0x00_8a38);
        assert!(cpu
            .machine
            .snes
            .dma
            .channel
            .iter()
            .all(|channel| !channel.dma_active));
        assert!(!cpu.machine.snes.dma.dma_busy);

        for _ in 0..82 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_8b50);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_911_998));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(236, 814)
        );
        let ldx = cpu.step().unwrap();
        assert_eq!(ldx.origin_pc, 0x00_8b50);
        assert_eq!(ldx.opcode, 0xae);

        for _ in 0..24 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_8bae);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_925_428));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(246, 604)
        );
        let stz = cpu.step().unwrap();
        assert_eq!(stz.origin_pc, 0x00_8bae);
        assert_eq!(stz.opcode, 0x64);

        for _ in 0..11 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_8bcf);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_278));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(249, 1362)
        );
        let ldy = cpu.step().unwrap();
        assert_eq!(ldy.origin_pc, 0x00_8bcf);
        assert_eq!(ldy.opcode, 0xa4);

        for _ in 0..3 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_8c22);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_370));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 90)
        );
        let ldx_direct = cpu.step().unwrap();
        assert_eq!(ldx_direct.origin_pc, 0x00_8c22);
        assert_eq!(ldx_direct.opcode, 0xa6);

        for _ in 0..2 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_8c77);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_440));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 160)
        );
        let asl = cpu.step().unwrap();
        assert_eq!(asl.origin_pc, 0x00_8c77);
        assert_eq!(asl.opcode, 0x0a);

        for _ in 0..2 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_8c7b);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_492));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 212)
        );
        let jump = cpu.step().unwrap();
        assert_eq!(jump.origin_pc, 0x00_8c7b);
        assert_eq!(jump.opcode, 0x7c);

        for _ in 0..2 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_83d1);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_626));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 346)
        );
        let joy_latch = cpu.step().unwrap();
        assert_eq!(joy_latch.origin_pc, 0x00_83d1);
        assert_eq!(joy_latch.opcode, 0x9c);

        assert_eq!(cpu.program_address(), 0x00_83d4);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_662));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 382)
        );
        let joy_result = cpu.step().unwrap();
        assert_eq!(joy_result.origin_pc, 0x00_83d4);
        assert_eq!(joy_result.opcode, 0xad);

        for _ in 0..5 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_83e2);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_858));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 578)
        );
        let tay = cpu.step().unwrap();
        assert_eq!(tay.origin_pc, 0x00_83e2);
        assert_eq!(tay.opcode, 0xa8);

        assert_eq!(cpu.program_address(), 0x00_83e3);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_872));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 592)
        );
        let eor = cpu.step().unwrap();
        assert_eq!(eor.origin_pc, 0x00_83e3);
        assert_eq!(eor.opcode, 0x45);

        assert_eq!(cpu.program_address(), 0x00_83e5);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_896));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 616)
        );
        let and = cpu.step().unwrap();
        assert_eq!(and.origin_pc, 0x00_83e5);
        assert_eq!(and.opcode, 0x25);

        assert_eq!(cpu.program_address(), 0x00_83e7);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_920));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 640)
        );
        let store = cpu.step().unwrap();
        assert_eq!(store.origin_pc, 0x00_83e7);
        assert_eq!(store.opcode, 0x85);

        assert_eq!(cpu.program_address(), 0x00_83e9);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_930_944));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(250, 664)
        );
        let sty = cpu.step().unwrap();
        assert_eq!(sty.origin_pc, 0x00_83e9);
        assert_eq!(sty.opcode, 0x84);

        for _ in 0..62 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_81d6);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_932_730));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(251, 1_086)
        );
        let and_immediate = cpu.step().unwrap();
        assert_eq!(and_immediate.origin_pc, 0x00_81d6);
        assert_eq!(and_immediate.opcode, 0x29);

        for _ in 0..10 {
            cpu.step().unwrap();
        }
        assert_eq!(cpu.program_address(), 0x00_8228);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_932_996));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(251, 1_352)
        );
        let pld = cpu.step().unwrap();
        assert_eq!(pld.origin_pc, 0x00_8228);
        assert_eq!(pld.opcode, 0x2b);

        assert_eq!(cpu.program_address(), 0x00_8229);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_933_032));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(252, 24)
        );
        let ply = cpu.step().unwrap();
        assert_eq!(ply.origin_pc, 0x00_8229);
        assert_eq!(ply.opcode, 0x7a);

        assert_eq!(cpu.program_address(), 0x00_822a);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_933_068));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(252, 60)
        );
        let plx = cpu.step().unwrap();
        assert_eq!(plx.origin_pc, 0x00_822a);
        assert_eq!(plx.opcode, 0xfa);

        cpu.step().unwrap();
        assert_eq!(cpu.program_address(), 0x00_822c);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_933_140));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(252, 132)
        );
        assert_eq!(
            cpu.step(),
            Err(SourceCpuError::UnsupportedOpcode {
                pc: 0x00_822c,
                opcode: 0x40,
            })
        );
        assert_eq!(cpu.program_address(), 0x00_822d);
        assert_eq!(cpu.machine.timestamp(), CpuMasterTimestamp::new(28_933_148));
        assert_eq!(
            cpu.machine.timeline.raster_position(),
            crate::CpuRasterPosition::new(252, 140)
        );
        assert!(cpu.is_poisoned());
    }

    #[test]
    #[ignore = "requires the local, untracked Zelda 3 ROM"]
    fn local_zelda_rom_reaches_first_cc_with_fixture_apui_values() {
        let rom_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3.sfc");
        let rom = std::fs::read(rom_path).expect("local zelda3.sfc is required for this proof");
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        let bootstrap = records()
            .into_iter()
            .find(|record| record["kind"] == "bootstrap-events")
            .unwrap();
        let fixture_accesses = cpu_apu_accesses(&bootstrap);
        let expected_timing = cpu_timing_transactions_through_first_cc(&bootstrap);
        let (reset, handshake) = split_first_cc_cpu_accesses(&fixture_accesses);
        let expected = reset
            .iter()
            .chain(handshake)
            .map(|event| {
                (
                    event.absolute_master_cycle(),
                    event.port,
                    event.value,
                    event.is_read,
                )
            })
            .collect::<Vec<_>>();

        let mut actual = Vec::new();
        let mut actual_timing = Vec::new();
        for _ in 0..50_000 {
            let receipt = cpu.step().unwrap();
            actual_timing.extend(receipt.transactions.iter().copied());
            for access in receipt.accesses {
                let Some(port) = Snes::synchronous_cpu_apu_port(access.address) else {
                    continue;
                };
                let (value, is_read) = match access.kind {
                    SourceCpuBusAccessKind::Read { value, width: 1 } => (value as u8, true),
                    SourceCpuBusAccessKind::Write { value, width: 1 } => (value as u8, false),
                    _ => continue,
                };
                actual.push((access.timestamp.master_cycles(), port & 3, value, is_read));
                if is_read && port & 3 == 0 && value == 0xcc {
                    let handshake_start = actual
                        .iter()
                        .position(|event| *event == expected[reset.len()])
                        .expect("cold execution must reach the fixture's first handshake access");
                    assert!(actual[reset.len()..handshake_start]
                        .iter()
                        .all(|event| event.2 == 0 && event.3));
                    let comparable = actual[..reset.len()]
                        .iter()
                        .chain(&actual[handshake_start..])
                        .copied()
                        .collect::<Vec<_>>();
                    assert_eq!(comparable, expected);
                    assert_eq!(actual_timing.len(), expected_timing.len());
                    for (index, (actual, expected)) in
                        actual_timing.iter().zip(&expected_timing).enumerate()
                    {
                        assert_timing_transaction(index, *actual, *expected);
                    }
                    return;
                }
            }
        }
        panic!("source CPU subset did not reach the first CC acknowledgement");
    }

    #[test]
    #[ignore = "requires the local, untracked Zelda 3 ROM and compares 3.2M transactions"]
    fn local_zelda_rom_matches_every_timing_transaction_through_final_ipl_handoff() {
        let rom_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3.sfc");
        let rom = std::fs::read(rom_path).expect("local zelda3.sfc is required for this proof");
        let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        let bootstrap = records()
            .into_iter()
            .find(|record| record["kind"] == "bootstrap-events")
            .unwrap();
        let mut actual = VecDeque::new();
        let mut compared = 0usize;
        visit_cpu_timing_transactions(&bootstrap, |expected| {
            while actual.is_empty() {
                let receipt = cpu.step().unwrap();
                actual.extend(receipt.transactions);
            }
            assert_timing_transaction(compared, actual.pop_front().unwrap(), expected);
            compared += 1;
        });
        assert!(
            actual.is_empty(),
            "fixture ended in the middle of a CPU instruction"
        );
        assert_eq!(
            compared,
            bootstrap["cpu_timing_transaction_sequence"]["record_count"]
                .as_u64()
                .unwrap() as usize
        );
        let handoff = &bootstrap["final_ipl_handoff"];
        assert_eq!(
            cpu.machine.snes.apu.cycles,
            handoff["absolute_cycle"].as_u64().unwrap() as u32
        );
        assert_eq!(
            cpu.machine.snes.apu.spc.pc,
            handoff["target_pc"].as_u64().unwrap() as u16
        );
    }
}
