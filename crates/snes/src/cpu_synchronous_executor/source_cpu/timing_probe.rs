//! Development-only source-ordered CPU timing probe.
//!
//! Unlike the exact cold CPU/APU owner, this accepts an explicit caller-owned
//! machine/PPU-read seed. It never claims that an arbitrary snapshot is an
//! exact cold checkpoint. Unsupported I/O, interrupts and DMA fail closed;
//! a failed instruction poisons the probe instead of silently falling back
//! to the aggregate interpreter or supplying cached semantic results.

use super::instruction_set::{SourceCpuInstructionBus, SourceCpuInstructions};
use super::{
    SourceCpuBusAccess, SourceCpuBusAccessKind, SourceCpuError, SourceCpuInstructionTrace,
    SourceCpuMapClass, SourceCpuStepReceipt, SourceCpuTransaction, SourceCpuTransactionKind,
    WordWrap, WordWriteOrder,
};
use crate::cpu_timeline::{
    CpuMasterTimeline, CpuMasterTimestamp, CpuSynchronousBeamPosition, CpuSynchronousTimelineEvent,
    CpuSynchronousTimelineStartError,
};
use crate::snes::Snes;

mod ppu_read_state;
pub use ppu_read_state::SourcePpuReadState;

#[derive(Debug, thiserror::Error)]
pub enum RomCpuTimingProbeSeedError {
    #[error("timing probe requires an inactive DMA/HDMA/interrupt owner")]
    ActiveHardware,
    #[error("timing probe currently requires the audited LoROM memory map")]
    Cartridge,
    #[error("timing probe counter seed is outside the NTSC hardware range")]
    CounterState,
    #[error(transparent)]
    Timeline(#[from] CpuSynchronousTimelineStartError),
}

/// An accepted NMI has no opcode fetch. Its transactions retain timing and
/// bus ownership without inventing an opcode to fit an instruction receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RomCpuInterruptTransaction {
    pub kind: SourceCpuTransactionKind,
    pub duration_master_cycles: u8,
    pub started_at: CpuMasterTimestamp,
    pub ended_at: CpuMasterTimestamp,
    pub start_wram_refresh_position: u16,
    pub end_wram_refresh_position: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RomCpuNmiReceipt {
    pub interrupted_pc: u32,
    pub memory_speed: u8,
    pub started_at: CpuMasterTimestamp,
    pub ended_at: CpuMasterTimestamp,
    pub accesses: Vec<SourceCpuBusAccess>,
    pub transactions: Vec<RomCpuInterruptTransaction>,
}

pub struct RomCpuTimingProbe {
    snes: Snes,
    timeline: CpuMasterTimeline,
    ppu_reads: SourcePpuReadState,
    active_trace: Option<SourceCpuInstructionTrace>,
    active_interrupt_trace: Option<Vec<RomCpuInterruptTransaction>>,
    poisoned: bool,
}

impl RomCpuTimingProbe {
    pub fn new(
        snes: Snes,
        mut timeline: CpuMasterTimeline,
        ppu_reads: SourcePpuReadState,
    ) -> Result<Self, RomCpuTimingProbeSeedError> {
        if snes.cpu.waiting
            || snes.cpu.stopped
            || snes.cpu.nmi_wanted
            || snes.cpu.irq_wanted
            || snes.h_irq_enabled
            || snes.v_irq_enabled
            || snes.nmi_enabled
            || snes.dma.dma_busy
            || snes.dma.dma_timer != 0
            || snes.dma.hdma_timer != 0
            || snes
                .dma
                .channel
                .iter()
                .any(|channel| channel.dma_active || channel.hdma_active)
            || timeline.bus_workload().dynamic_hdma()
            || timeline.bus_workload().hdma_stall_master_cycles() != 0
        {
            return Err(RomCpuTimingProbeSeedError::ActiveHardware);
        }
        if snes.cart.kind != crate::cart::CartType::LoRom
            || snes.cart.rom.is_empty()
            || !snes.cart.rom.len().is_power_of_two()
            || snes.cart.ram.len() != 0x2000
            || snes.ram.len() != 0x20000
        {
            return Err(RomCpuTimingProbeSeedError::Cartridge);
        }
        if ppu_reads.h_latched > 0x1ff || ppu_reads.v_latched > 261 {
            return Err(RomCpuTimingProbeSeedError::CounterState);
        }
        timeline.begin_synchronous_timeline()?;
        Ok(Self {
            snes,
            timeline,
            ppu_reads,
            active_trace: None,
            active_interrupt_trace: None,
            poisoned: false,
        })
    }

    pub fn snes(&self) -> &Snes {
        &self.snes
    }
    pub fn timeline(&self) -> &CpuMasterTimeline {
        &self.timeline
    }
    pub fn ppu_reads(&self) -> &SourcePpuReadState {
        &self.ppu_reads
    }
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    /// Execute an NMI already accepted by the caller's interrupt owner at
    /// this instruction boundary. This does not invent a VBlank event, clear
    /// RDNMI, enable NMI, or alter the source scheduler's future deadlines.
    pub fn accept_native_nmi(&mut self) -> Result<RomCpuNmiReceipt, SourceCpuError> {
        if self.poisoned {
            return Err(SourceCpuError::Poisoned);
        }
        if self.snes.cpu.e || self.snes.cpu.waiting || self.snes.cpu.stopped {
            return Err(SourceCpuError::UnsupportedNmiEntryState);
        }
        let interrupted_pc = self.program_address();
        let memory_speed = self.snes.hardware_access_time(interrupted_pc);
        if (interrupted_pc as u16) < 0x8000 || !matches!(memory_speed, 6 | 8) {
            return Err(SourceCpuError::UnsupportedBusMap {
                address: interrupted_pc,
            });
        }
        let started_at = self.timeline.timestamp();
        let mut accesses = Vec::new();
        self.active_interrupt_trace = Some(Vec::new());
        if let Err(error) = self.enter_native_interrupt_bus(0x00_ffea, memory_speed, &mut accesses)
        {
            self.poisoned = true;
            self.active_interrupt_trace = None;
            return Err(error);
        }
        Ok(RomCpuNmiReceipt {
            interrupted_pc,
            memory_speed,
            started_at,
            ended_at: self.timeline.timestamp(),
            accesses,
            transactions: self
                .active_interrupt_trace
                .take()
                .expect("accepted NMI owns its trace"),
        })
    }

    pub fn step(&mut self) -> Result<SourceCpuStepReceipt, SourceCpuError> {
        if self.poisoned {
            return Err(SourceCpuError::Poisoned);
        }
        let origin_pc = self.program_address();
        let started_at = self.timeline.timestamp();
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
            .expect("one active source probe instruction");
        Ok(SourceCpuStepReceipt {
            origin_pc,
            opcode,
            started_at,
            ended_at: self.timeline.timestamp(),
            accesses,
            transactions: trace.transactions,
        })
    }

    fn beam(&self) -> CpuSynchronousBeamPosition {
        self.timeline
            .synchronous_beam_position()
            .expect("probe owns a synchronous timeline")
    }

    fn source_read_semantic(&mut self, address: u32) -> Result<u8, SourceCpuError> {
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        if bank & 0x7f < 0x40 {
            if adr == 0x4210 {
                let value = (u8::from(self.snes.in_nmi) << 7) | (self.snes.open_bus & 0x70) | 2;
                self.snes.in_nmi = false;
                return Ok(value);
            }
            if adr == 0x4211 {
                let value = (u8::from(self.snes.cpu.irq_wanted) << 7) | (self.snes.open_bus & 0x7f);
                self.snes.cpu.irq_wanted = false;
                self.snes.in_irq = false;
                return Ok(value);
            }
            if let Some(value) = self.ppu_reads.read(adr, self.beam()) {
                return Ok(value);
            }
            if (0x2134..=0x2136).contains(&adr) {
                let value = self.snes.ppu.read(adr as u8);
                self.ppu_reads.open_bus1 = value;
                return Ok(value);
            }
        }
        match self.source_map_class(address) {
            Some(SourceCpuMapClass::Wram) => {
                let offset = if bank == 0x7e || bank == 0x7f {
                    ((usize::from(bank) - 0x7e) << 16) | usize::from(adr)
                } else {
                    usize::from(adr)
                };
                Ok(self.snes.ram[offset])
            }
            Some(SourceCpuMapClass::LoRom | SourceCpuMapClass::LoRomSram) => {
                Ok(self.snes.cart.read(bank, adr, self.snes.open_bus))
            }
            None => Err(SourceCpuError::UnsupportedBusMap { address }),
        }
    }

    fn source_write_semantic(&mut self, address: u32, value: u8) -> Result<(), SourceCpuError> {
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        if bank & 0x7f < 0x40 && matches!(adr, 0x420b | 0x420c) && value == 0 {
            // With no active DMA/HDMA owner, source mask-zero writes start
            // no transfer. Nonzero masks remain unsupported before mutation.
            return Ok(());
        }
        if bank & 0x7f < 0x40 && adr == 0x4201 {
            self.ppu_reads.write_wrio(value, self.beam());
            return Ok(());
        }
        match self.source_map_class(address) {
            Some(SourceCpuMapClass::Wram) => {
                let offset = if bank == 0x7e || bank == 0x7f {
                    ((usize::from(bank) - 0x7e) << 16) | usize::from(adr)
                } else {
                    usize::from(adr)
                };
                self.snes.ram[offset] = value;
                Ok(())
            }
            Some(SourceCpuMapClass::LoRomSram) => {
                self.snes.cart.write(bank, adr, value);
                Ok(())
            }
            _ => Err(SourceCpuError::UnsupportedBusMap { address }),
        }
    }

    fn read_byte(
        &mut self,
        address: u32,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        let address = address & 0x00ff_ffff;
        let timestamp = self.timeline.timestamp();
        let value = self.source_read_semantic(address)?;
        let duration = self.snes.hardware_access_time(address);
        self.drain_and_record_transaction(
            SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
            u32::from(duration),
        )?;
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
        let timestamp = self.timeline.timestamp();
        self.source_write_semantic(address, value)?;
        let duration = self.snes.hardware_access_time(address);
        self.drain_and_record_transaction(
            SourceCpuTransactionKind::GetSetMemoryAccessAfterSemanticDraining,
            u32::from(duration),
        )?;
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

    fn add_cycles(&mut self, cycles: u32) -> Result<(), SourceCpuError> {
        self.drain_and_record_transaction(SourceCpuTransactionKind::CpuOpsAddCyclesDraining, cycles)
    }

    fn drain_and_record_transaction(
        &mut self,
        kind: SourceCpuTransactionKind,
        cycles: u32,
    ) -> Result<(), SourceCpuError> {
        let started = self.timeline.timestamp();
        let refresh = self.timeline.wram_refresh_cycle() as u16;
        self.timeline
            .advance_synchronous_after_semantics_with(cycles, |event, _| {
                if let CpuSynchronousTimelineEvent::HMax {
                    completed_scanline, ..
                } = event
                {
                    let next = (completed_scanline + 1) % 262;
                    if next == 225 {
                        self.snes.in_vblank = true;
                        self.snes.in_nmi = true;
                    } else if next == 0 {
                        self.snes.in_vblank = false;
                        self.snes.in_nmi = false;
                    }
                }
                Ok::<_, SourceCpuError>(0)
            })?;
        self.record_transaction(
            kind,
            u8::try_from(cycles).expect("source transaction fits in one byte"),
            started,
            self.timeline.timestamp(),
            refresh,
            self.timeline.wram_refresh_cycle() as u16,
        );
        Ok(())
    }

    fn program_address(&self) -> u32 {
        (u32::from(self.snes.cpu.k) << 16) | u32::from(self.snes.cpu.pc)
    }

    fn fetch_opcode(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        let address = self.program_address();
        let timestamp = self.timeline.timestamp();
        let start_wram_refresh_position = self.timeline.wram_refresh_cycle() as u16;
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        let memory_speed = self.snes.hardware_access_time(address);
        if adr < 0x8000 || !matches!(memory_speed, 6 | 8) {
            return Err(SourceCpuError::UnsupportedOpcode {
                pc: address,
                opcode: self.snes.open_bus,
            });
        }
        // cpuexec.cpp reads direct `PCBase` without changing OpenBus or
        // draining a due event, then adds MemSpeed directly.
        let opcode = self.snes.cart.read(bank, adr, self.snes.open_bus);
        self.active_trace
            .as_mut()
            .expect("opcode fetch requires an active instruction trace")
            .opcode = Some(opcode);
        self.active_trace
            .as_mut()
            .expect("opcode fetch requires an active instruction trace")
            .memory_speed = Some(memory_speed);

        self.timeline
            .advance_synchronous_pcbase_opcode_fetch(memory_speed);
        let ended_at = self.timeline.timestamp();
        self.snes.cpu.pc = self.snes.cpu.pc.wrapping_add(1);
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

    fn validate_pcbase_operand(&self, width: u8) -> Result<(), SourceCpuError> {
        let address = self.program_address();
        // cpuexec.cpp switches to S9xOpcodesSlow at the bank-end mapping
        // boundary. Do not silently execute a direct-PCBase operand there.
        if (address as u16) < 0x8000 || u32::from(address as u16) + u32::from(width) >= 0x10000 {
            return Err(SourceCpuError::UnsupportedBusMap { address });
        }
        Ok(())
    }

    fn operand_master_cycles(&self, width: u8) -> u8 {
        // cpuaddr.h:Immediate8/16/AbsoluteLong use the active PCBase
        // MemSpeed, including six-clock FastROM fetches in high banks.
        self.active_trace
            .as_ref()
            .and_then(|trace| trace.memory_speed)
            .expect("source operand follows its opcode fetch")
            * width
    }

    fn immediate8(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        self.validate_pcbase_operand(1)?;
        let address = self.program_address();
        let timestamp = self.timeline.timestamp();
        let value = self
            .snes
            .cart
            .read((address >> 16) as u8, address as u16, self.snes.open_bus);
        let cycles = self.operand_master_cycles(1);
        self.add_cycles(u32::from(cycles))?;
        self.snes.cpu.pc = self.snes.cpu.pc.wrapping_add(1);
        if update_open_bus {
            self.snes.open_bus = value;
        }
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: cycles,
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
        self.validate_pcbase_operand(2)?;
        let address = self.program_address();
        let timestamp = self.timeline.timestamp();
        let bank = (address >> 16) as u8;
        let low = self
            .snes
            .cart
            .read(bank, address as u16, self.snes.open_bus);
        let high = self
            .snes
            .cart
            .read(bank, (address as u16).wrapping_add(1), self.snes.open_bus);
        let value = u16::from_le_bytes([low, high]);
        let cycles = self.operand_master_cycles(2);
        self.add_cycles(u32::from(cycles))?;
        self.snes.cpu.pc = self.snes.cpu.pc.wrapping_add(2);
        if update_open_bus {
            self.snes.open_bus = high;
        }
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: cycles,
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
            self.snes.open_bus = (value >> 8) as u8;
        }
        self.snes.cpu.pc = self.snes.cpu.pc.wrapping_add(2);
        Ok(value)
    }

    fn absolute_long_address(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        self.validate_pcbase_operand(3)?;
        let address = self.program_address();
        let timestamp = self.timeline.timestamp();
        let bank = (address >> 16) as u8;
        let adr = address as u16;
        let low = self.snes.cart.read(bank, adr, self.snes.open_bus);
        let high = self
            .snes
            .cart
            .read(bank, adr.wrapping_add(1), self.snes.open_bus);
        let data_bank = self
            .snes
            .cart
            .read(bank, adr.wrapping_add(2), self.snes.open_bus);
        let value = u32::from(low) | (u32::from(high) << 8) | (u32::from(data_bank) << 16);
        // cpuaddr.h:AbsoluteLong uses one AddCycles transaction for all three
        // already-read direct PCBase bytes.
        let cycles = self.operand_master_cycles(3);
        self.add_cycles(u32::from(cycles))?;
        self.snes.cpu.pc = self.snes.cpu.pc.wrapping_add(3);
        if update_open_bus {
            self.snes.open_bus = data_bank;
        }
        accesses.push(SourceCpuBusAccess {
            address,
            timestamp,
            charged_master_cycles: cycles,
            kind: SourceCpuBusAccessKind::ReadLong { value },
        });
        Ok(value)
    }

    fn read_word(
        &mut self,
        address: u32,
        wrap: WordWrap,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let next = wrap.next(address);
        if self.word_is_direct_transaction(address, next, wrap) {
            let timestamp = self.timeline.timestamp();
            let low = self.source_read_semantic(address)?;
            let high = self.source_read_semantic(next)?;
            let value = u16::from_le_bytes([low, high]);
            let duration = self.snes.hardware_access_time(address) * 2;
            self.drain_and_record_transaction(
                SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                u32::from(duration),
            )?;
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
                self.snes.open_bus = low;
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
            let timestamp = self.timeline.timestamp();
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
            let duration = self.snes.hardware_access_time(address) * 2;
            self.drain_and_record_transaction(
                SourceCpuTransactionKind::GetSetMemoryAccessX2AfterSemanticDraining,
                u32::from(duration),
            )?;
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
            && self.snes.hardware_access_time(address) == self.snes.hardware_access_time(next)
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

    fn record_transaction(
        &mut self,
        kind: SourceCpuTransactionKind,
        duration_master_cycles: u8,
        started_at: CpuMasterTimestamp,
        ended_at: CpuMasterTimestamp,
        start_wram_refresh_position: u16,
        end_wram_refresh_position: u16,
    ) {
        if let Some(trace) = self.active_interrupt_trace.as_mut() {
            trace.push(RomCpuInterruptTransaction {
                kind,
                duration_master_cycles,
                started_at,
                ended_at,
                start_wram_refresh_position,
                end_wram_refresh_position,
            });
            return;
        }
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
}

impl SourceCpuInstructionBus for RomCpuTimingProbe {
    fn cpu(&self) -> &crate::cpu::CpuState {
        &self.snes.cpu
    }
    fn cpu_mut(&mut self) -> &mut crate::cpu::CpuState {
        &mut self.snes.cpu
    }
    fn set_open_bus(&mut self, value: u8) {
        self.snes.open_bus = value;
    }
    fn immediate8(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        RomCpuTimingProbe::immediate8(self, update_open_bus, accesses)
    }

    fn immediate16(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        RomCpuTimingProbe::immediate16(self, update_open_bus, accesses)
    }

    fn immediate16_slow(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        RomCpuTimingProbe::immediate16_slow(self, update_open_bus, accesses)
    }

    fn absolute_long_address(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        RomCpuTimingProbe::absolute_long_address(self, update_open_bus, accesses)
    }

    fn read_byte(
        &mut self,
        address: u32,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        RomCpuTimingProbe::read_byte(self, address, accesses)
    }

    fn write_byte(
        &mut self,
        address: u32,
        value: u8,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        RomCpuTimingProbe::write_byte(self, address, value, accesses)
    }

    fn read_word(
        &mut self,
        address: u32,
        wrap: WordWrap,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        RomCpuTimingProbe::read_word(self, address, wrap, accesses)
    }

    fn write_word(
        &mut self,
        address: u32,
        value: u16,
        wrap: WordWrap,
        order: WordWriteOrder,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        RomCpuTimingProbe::write_word(self, address, value, wrap, order, accesses)
    }

    fn add_cycles(&mut self, cycles: u32) -> Result<(), SourceCpuError> {
        RomCpuTimingProbe::add_cycles(self, cycles)
    }
}

#[cfg(test)]
mod tests;
