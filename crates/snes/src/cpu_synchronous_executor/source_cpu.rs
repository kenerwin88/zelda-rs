//! Opt-in, source-ordered subset of the pinned Snes9x 65816 executor.
//!
//! This is intentionally separate from the legacy C-port interpreter. Each
//! helper below corresponds to a concrete Snes9x `PCBase`, `Immediate*`,
//! `S9xGet*`, `S9xSet*`, or `AddCycles` transaction boundary.

use super::{CpuSynchronousCompletion, CpuSynchronousMachine, CpuSynchronousMachineError};
use crate::cart::CartType;
use crate::cpu_timeline::{CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuMasterTimestamp};
use crate::snes::Snes;
use crate::snes9x_apu_clock::Snes9xApuClockState;

const ONE_CYCLE: u32 = 6;
const TWO_CYCLES: u32 = 12;
const RESET_CPU_MASTER_CYCLE: u64 = 182;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCpuBusAccessKind {
    OpcodeFetch { value: u8 },
    Read { value: u16, width: u8 },
    Write { value: u16, width: u8 },
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
    #[error("unsupported source CPU opcode ${opcode:02x} at ${pc:06x}")]
    UnsupportedOpcode { pc: u32, opcode: u8 },
    #[error("source CPU bus map at ${address:06x} is outside the audited cold subset")]
    UnsupportedBusMap { address: u32 },
    #[error("source CPU executor is poisoned by an earlier partial instruction")]
    Poisoned,
    #[error("source CPU execution encountered an enabled NMI, DMA, or HDMA path")]
    UnexpectedInterruptOrDma,
    #[error(transparent)]
    Machine(#[from] CpuSynchronousMachineError),
}

/// Isolated pinned-Snes9x cold CPU/APU executor for the audited opcode subset.
pub struct Snes9xColdCpuExecutor {
    machine: CpuSynchronousMachine,
    active_trace: Option<SourceCpuInstructionTrace>,
    poisoned: bool,
}

struct SourceCpuInstructionTrace {
    origin_pc: u32,
    opcode: Option<u8>,
    transactions: Vec<SourceCpuTransaction>,
}

impl Snes9xColdCpuExecutor {
    /// Construct the exact cold CPU subset seed, then perform Snes9x's reset
    /// vector `S9xGetWord($00fffc)` transaction at T=182..198.
    pub fn from_lorom_reset(rom: &[u8]) -> Result<Self, SourceCpuError> {
        if rom.is_empty() || !rom.len().is_power_of_two() {
            return Err(SourceCpuError::InvalidLoRom);
        }

        let mut snes = Snes::new();
        snes.cart.load(CartType::LoRom, rom, 0x2000);
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

    pub fn step(&mut self) -> Result<SourceCpuStepReceipt, SourceCpuError> {
        if self.poisoned {
            return Err(SourceCpuError::Poisoned);
        }
        self.assert_cold_interrupt_state()?;
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
            transactions: Vec::new(),
        });
        let mut accesses = Vec::new();
        let result = (|| {
            let opcode = self.fetch_opcode(&mut accesses)?;
            self.execute(opcode, origin_pc, &mut accesses)?;
            self.assert_cold_interrupt_state()?;
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

    fn assert_cold_interrupt_state(&self) -> Result<(), SourceCpuError> {
        let snes = &self.machine.snes;
        if snes.nmi_enabled
            || snes.h_irq_enabled
            || snes.v_irq_enabled
            || snes.cpu.nmi_wanted
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
        if adr < 0x8000 || self.machine.snes.hardware_access_time(address) != 8 {
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
        let ended_at;
        self.machine
            .timeline
            .advance_synchronous_pcbase_opcode_fetch();
        ended_at = self.machine.timestamp();
        self.machine.snes.cpu.pc = self.machine.snes.cpu.pc.wrapping_add(1);
        self.record_transaction(
            SourceCpuTransactionKind::FastPcBaseOpcodeFetchNonDraining,
            8,
            timestamp,
            ended_at,
            start_wram_refresh_position,
            start_wram_refresh_position,
        );
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: 8,
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
            0x28 => {
                self.add_cycles(TWO_CYCLES)?;
                let flags = self.pull_byte(accesses)?;
                self.machine.snes.open_bus = flags;
                self.machine.snes.cpu.unpack_flags(flags);
                self.fix_status_widths();
            }
            0x2a => {
                self.add_cycles(ONE_CYCLE)?;
                self.rol_accumulator();
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
            0x58 => {
                self.add_cycles(ONE_CYCLE)?;
                self.machine.snes.cpu.i = false;
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
            0x80 => self.branch(true, accesses)?,
            0x85 => {
                let address = self.direct_address(false, accesses)?;
                self.store_accumulator(address, accesses)?;
            }
            0x8d => {
                let address = self.absolute_address(false, accesses)?;
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
            0xb7 => {
                let pointer = self.direct_address(true, accesses)?;
                let low = self.read_word(pointer, WordWrap::None, accesses)?;
                self.machine.snes.open_bus = (low >> 8) as u8;
                let bank = self.read_byte(pointer.wrapping_add(2), accesses)?;
                self.machine.snes.open_bus = bank;
                let address = ((u32::from(bank) << 16) | u32::from(low))
                    .wrapping_add(u32::from(self.machine.snes.cpu.y))
                    & 0x00ff_ffff;
                let value = self.read_by_m(address, accesses)?;
                self.load_accumulator(value);
            }
            0xc0 => {
                let value = self.immediate_by_x(accesses)?;
                self.compare(self.machine.snes.cpu.y, value, self.machine.snes.cpu.xf);
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
                let value = self.read_by_m(address, accesses)?;
                self.compare(self.machine.snes.cpu.a, value, self.machine.snes.cpu.mf);
            }
            0xd0 => self.branch(!self.machine.snes.cpu.z, accesses)?,
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
                let value = self.read_by_m(address, accesses)?;
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
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let value = if self.machine.snes.cpu.mf {
            u16::from(self.read_byte(address, accesses)?)
        } else {
            self.read_word(address, WordWrap::None, accesses)?
        };
        self.machine.snes.open_bus = if self.machine.snes.cpu.mf {
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
            CpuSynchronousMachine::synchronize_apu(
                &mut self.machine.snes,
                &mut self.machine.apu_clock,
                timestamp,
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
        if let Some(port) = Snes::synchronous_cpu_apu_port(address) {
            CpuSynchronousMachine::synchronize_apu(
                &mut self.machine.snes,
                &mut self.machine.apu_clock,
                timestamp,
            )?;
            self.machine
                .snes
                .synchronous_cpu_write_apu_port_raw_semantic(address, port, value);
        } else {
            self.source_write_semantic(address, value)?;
        }
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
        } else if adr >= 0x8000 {
            Some(SourceCpuMapClass::LoRom)
        } else {
            None
        }
    }

    fn source_read_semantic(&self, address: u32) -> Result<u8, SourceCpuError> {
        let address = address & 0x00ff_ffff;
        let bank = (address >> 16) as u8;
        let adr = address as u16;
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
            None => Err(SourceCpuError::UnsupportedBusMap { address }),
        }
    }

    fn source_write_semantic(&mut self, address: u32, value: u8) -> Result<(), SourceCpuError> {
        let address = address & 0x00ff_ffff;
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        if (bank & 0x7f) < 0x40 && matches!(adr, 0x4200 | 0x420b | 0x420c) {
            if value == 0 {
                return Ok(());
            }
            return Err(SourceCpuError::UnexpectedInterruptOrDma);
        }
        if (bank & 0x7f) < 0x40 && (0x2100..=0x21ff).contains(&adr) {
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

    fn guard_control_write(&self, address: u32, value: u8) -> Result<(), SourceCpuError> {
        let mirrored = (address >> 16) as u8 & 0x7f;
        if mirrored < 0x40 && matches!(address as u16, 0x4200 | 0x420b | 0x420c) && value != 0 {
            return Err(SourceCpuError::UnexpectedInterruptOrDma);
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

    fn synthetic_rom(program: &[u8]) -> Vec<u8> {
        let mut rom = vec![0xea; 0x8000];
        rom[..program.len()].copy_from_slice(program);
        rom[0x7ffc] = 0x00;
        rom[0x7ffd] = 0x80;
        rom
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
        assert_eq!(actual_kind, expected.kind, "transaction {index}");
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
        cpu.machine.snes.apu.rom_readable = false;
        cpu.machine.snes.apu.spc.pc = 0x0200;
        cpu.machine.snes.apu.ram[0x0200] = 0;
    }

    #[test]
    fn immediate_drain_failure_defers_pc_and_open_bus_then_poison_is_terminal() {
        let mut cpu =
            Snes9xColdCpuExecutor::from_lorom_reset(&synthetic_rom(&[0xc2, 0x20])).unwrap();
        install_hmax_smp_failure(&mut cpu, 1_350);
        assert!(matches!(
            cpu.step(),
            Err(SourceCpuError::Machine(
                CpuSynchronousMachineError::UnsupportedSmpOpcode(_)
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
                CpuSynchronousMachineError::UnsupportedSmpOpcode(_)
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
                CpuSynchronousMachineError::UnsupportedSmpOpcode(_)
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
