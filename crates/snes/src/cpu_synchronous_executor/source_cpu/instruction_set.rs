//! Source-ordered 65816 instruction semantics shared by CPU hardware owners.
//!
//! This layer owns register changes and the order of operand, data, stack,
//! and internal-cycle transactions. The bus owner supplies their semantics,
//! event draining, publication and failure behavior. It supplies no reset or
//! checkpoint constructor and cannot weaken an owner's seed contract.

use super::{
    bcd_add, bcd_sub, SourceCpuBusAccess, SourceCpuError, WordWrap, WordWriteOrder, ONE_CYCLE,
    TWO_CYCLES,
};
use crate::cpu::CpuState;

pub(super) trait SourceCpuInstructionBus {
    fn cpu(&self) -> &CpuState;
    fn cpu_mut(&mut self) -> &mut CpuState;
    fn set_open_bus(&mut self, value: u8);
    fn immediate8(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError>;

    fn immediate16(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError>;

    fn immediate16_slow(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError>;

    fn absolute_long_address(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError>;

    fn read_byte(
        &mut self,
        address: u32,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError>;

    fn write_byte(
        &mut self,
        address: u32,
        value: u8,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError>;

    fn read_word(
        &mut self,
        address: u32,
        wrap: WordWrap,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError>;

    fn write_word(
        &mut self,
        address: u32,
        value: u16,
        wrap: WordWrap,
        order: WordWriteOrder,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError>;

    fn add_cycles(&mut self, cycles: u32) -> Result<(), SourceCpuError>;
}

pub(super) trait SourceCpuInstructions: SourceCpuInstructionBus {
    /// cpumacro.h:S9xOpcode_NMI/IRQ native entry bus sequence. Interrupt
    /// selection, pending-latch retirement and deadlines belong to the owner.
    fn enter_native_interrupt_bus(
        &mut self,
        vector_address: u32,
        memory_speed: u8,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        self.add_cycles(u32::from(memory_speed) + ONE_CYCLE)?;
        let program_bank = self.cpu().k;
        self.push_byte(program_bank, accesses)?;
        let program_counter = self.cpu().pc;
        self.push_word(program_counter, accesses)?;
        let status = self.cpu().pack_flags();
        self.push_byte(status, accesses)?;
        self.set_open_bus(status);
        self.cpu_mut().d = false;
        self.cpu_mut().i = true;
        let vector = self.read_word(vector_address, WordWrap::Bank, accesses)?;
        self.set_open_bus((vector >> 8) as u8);
        self.cpu_mut().k = 0;
        self.cpu_mut().pc = vector;
        Ok(())
    }

    fn execute(
        &mut self,
        opcode: u8,
        origin_pc: u32,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        match opcode {
            0x05 => {
                let address = self.direct_address(true, accesses)?;
                let operand = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.ora_accumulator(operand);
            }
            0x07 => {
                let address = self.direct_indirect_long_address(accesses)?;
                let operand = self.read_by_m(address, WordWrap::None, accesses)?;
                self.ora_accumulator(operand);
            }
            0x09 => {
                let operand = self.immediate_by_m(accesses)?;
                self.ora_accumulator(operand);
            }
            0x0a => {
                self.add_cycles(ONE_CYCLE)?;
                self.asl_accumulator();
            }
            0x0b => {
                self.add_cycles(ONE_CYCLE)?;
                let direct_page = self.cpu().dp;
                self.push_word(direct_page, accesses)?;
                self.set_open_bus(direct_page as u8);
            }
            0x10 => self.branch(!self.cpu().n, accesses)?,
            0x15 => {
                let address = self.direct_indexed_x_address(true, accesses)?;
                let operand = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.ora_accumulator(operand);
            }
            0x08 => {
                let flags = self.cpu().pack_flags();
                self.add_cycles(ONE_CYCLE)?;
                self.push_byte(flags, accesses)?;
                self.set_open_bus(flags);
            }
            0x18 => {
                self.cpu_mut().c = false;
                self.add_cycles(ONE_CYCLE)?;
            }
            0x1a => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().mf {
                    let value = (self.cpu().a as u8).wrapping_add(1);
                    self.cpu_mut().a = (self.cpu().a & 0xff00) | u16::from(value);
                    u16::from(value)
                } else {
                    self.cpu_mut().a = self.cpu().a.wrapping_add(1);
                    self.cpu().a
                };
                self.set_zn(value, self.cpu().mf);
            }
            0x1b => {
                self.add_cycles(ONE_CYCLE)?;
                self.cpu_mut().sp = self.cpu().a;
                self.fix_emulation_stack();
            }
            0x1d => {
                let address = self.absolute_indexed_read_address(self.cpu().x, accesses)?;
                let operand = self.read_by_m(address, WordWrap::None, accesses)?;
                self.ora_accumulator(operand);
            }
            0x20 => {
                let target = self.immediate16(false, accesses)?;
                self.add_cycles(ONE_CYCLE)?;
                let return_pc = self.cpu().pc.wrapping_sub(1);
                self.push_word(return_pc, accesses)?;
                self.cpu_mut().pc = target;
            }
            0x22 => {
                let target = self.absolute_long_address(false, accesses)?;
                self.add_cycles(ONE_CYCLE)?;
                let program_bank = self.cpu().k;
                self.push_byte_without_emulation_bounds(program_bank, accesses)?;
                let return_pc = self.cpu().pc.wrapping_sub(1);
                self.push_word_without_emulation_bounds(return_pc, accesses)?;
                if self.cpu().e {
                    // JSL is a 65816 instruction and deliberately does not
                    // page-wrap either push in emulation mode. Snes9x repairs
                    // SH only after both full-stack pushes have completed.
                    self.cpu_mut().sp = 0x0100 | (self.cpu().sp & 0x00ff);
                }
                self.cpu_mut().pc = target as u16;
                self.cpu_mut().k = (target >> 16) as u8;
            }
            0x25 => {
                let address = self.direct_address(true, accesses)?;
                let operand = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.and_accumulator(operand);
            }
            0x28 => {
                self.add_cycles(TWO_CYCLES)?;
                let flags = self.pull_byte(accesses)?;
                self.set_open_bus(flags);
                self.cpu_mut().unpack_flags(flags);
                self.fix_status_widths();
            }
            0x29 => {
                let operand = self.immediate_by_m(accesses)?;
                if self.cpu().mf {
                    let result = u16::from((self.cpu().a as u8) & operand as u8);
                    self.cpu_mut().a = (self.cpu().a & 0xff00) | result;
                    self.set_zn(result, true);
                } else {
                    self.cpu_mut().a &= operand;
                    self.set_zn(self.cpu().a, false);
                }
            }
            0x2a => {
                self.add_cycles(ONE_CYCLE)?;
                self.rol_accumulator();
            }
            0x2b => {
                self.add_cycles(TWO_CYCLES)?;
                let direct_page = self.pull_word_bank(accesses)?;
                self.cpu_mut().dp = direct_page;
                self.set_zn(direct_page, false);
                self.set_open_bus((direct_page >> 8) as u8);
                if self.cpu().e {
                    // PLD uses PullW even in emulation mode, then repairs SH.
                    self.cpu_mut().sp = 0x0100 | (self.cpu().sp & 0x00ff);
                }
            }
            0x30 => self.branch(self.cpu().n, accesses)?,
            0x38 => {
                self.cpu_mut().c = true;
                self.add_cycles(ONE_CYCLE)?;
            }
            0x3a => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().mf {
                    let value = (self.cpu().a as u8).wrapping_sub(1);
                    self.cpu_mut().a = (self.cpu().a & 0xff00) | u16::from(value);
                    u16::from(value)
                } else {
                    self.cpu_mut().a = self.cpu().a.wrapping_sub(1);
                    self.cpu().a
                };
                self.set_zn(value, self.cpu().mf);
            }
            0x3b => {
                self.add_cycles(ONE_CYCLE)?;
                self.cpu_mut().a = self.cpu().sp;
                self.set_zn(self.cpu().a, false);
            }
            0x3f => {
                let address = self.absolute_long_address(true, accesses)?;
                let address = address.wrapping_add(u32::from(self.cpu().x)) & 0x00ff_ffff;
                let operand = self.read_by_m(address, WordWrap::None, accesses)?;
                self.and_accumulator(operand);
            }
            0x40 => {
                self.add_cycles(TWO_CYCLES)?;
                let flags = self.pull_byte(accesses)?;
                self.cpu_mut().unpack_flags(flags);
                if self.cpu().e {
                    let target = self.pull_word(accesses)?;
                    self.cpu_mut().pc = target;
                    self.set_open_bus((target >> 8) as u8);
                } else {
                    let target = self.pull_word_bank(accesses)?;
                    self.cpu_mut().pc = target;
                    let program_bank = self.pull_byte(accesses)?;
                    self.cpu_mut().k = program_bank;
                    self.set_open_bus(program_bank);
                }
                self.fix_status_widths();
            }
            0x45 => {
                let address = self.direct_address(true, accesses)?;
                let operand = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.eor_accumulator(operand);
            }
            0x46 => {
                let address = self.direct_address(true, accesses)?;
                let value = if self.cpu().mf {
                    u16::from(self.read_byte(address, accesses)?)
                } else {
                    self.read_word(address, WordWrap::Bank, accesses)?
                };
                self.cpu_mut().c = value & 1 != 0;
                let result = value >> 1;
                self.add_cycles(ONE_CYCLE)?;
                self.write_by_m(address, result, WordWriteOrder::HighLow, accesses)?;
                self.set_open_bus(result as u8);
                self.set_zn(result, self.cpu().mf);
            }
            0x48 => {
                self.add_cycles(ONE_CYCLE)?;
                let a = self.cpu().a;
                if self.cpu().mf {
                    self.push_byte(a as u8, accesses)?;
                } else {
                    self.push_word(a, accesses)?;
                }
                self.set_open_bus(a as u8);
            }
            0x49 => {
                let operand = self.immediate_by_m(accesses)?;
                self.eor_accumulator(operand);
            }
            0x4a => {
                self.add_cycles(ONE_CYCLE)?;
                self.lsr_accumulator();
            }
            0x4b => {
                self.add_cycles(ONE_CYCLE)?;
                let program_bank = self.cpu().k;
                self.push_byte(program_bank, accesses)?;
                self.set_open_bus(program_bank);
            }
            0x4c => {
                let target = self.immediate16(false, accesses)?;
                self.cpu_mut().pc = target;
            }
            0x54 => {
                let destination_bank = self.immediate8(false, accesses)?;
                self.cpu_mut().db = destination_bank;
                let source_bank = self.immediate8(true, accesses)?;
                let source_address = (u32::from(source_bank) << 16) | u32::from(self.cpu().x);
                let destination_address =
                    (u32::from(destination_bank) << 16) | u32::from(self.cpu().y);
                let value = self.read_byte(source_address, accesses)?;
                self.set_open_bus(value);
                self.write_byte(destination_address, value, accesses)?;

                if self.cpu().xf {
                    self.cpu_mut().x =
                        (self.cpu().x & 0xff00) | u16::from((self.cpu().x as u8).wrapping_add(1));
                    self.cpu_mut().y =
                        (self.cpu().y & 0xff00) | u16::from((self.cpu().y as u8).wrapping_add(1));
                } else {
                    self.cpu_mut().x = self.cpu().x.wrapping_add(1);
                    self.cpu_mut().y = self.cpu().y.wrapping_add(1);
                }
                self.cpu_mut().a = self.cpu().a.wrapping_sub(1);
                if self.cpu().a != 0xffff {
                    self.cpu_mut().pc = self.cpu().pc.wrapping_sub(3);
                }
                self.add_cycles(TWO_CYCLES)?;
            }
            0x58 => {
                self.add_cycles(ONE_CYCLE)?;
                self.cpu_mut().i = false;
            }
            0x59 => {
                let address = self.absolute_indexed_read_address(self.cpu().y, accesses)?;
                let operand = self.read_by_m(address, WordWrap::None, accesses)?;
                self.eor_accumulator(operand);
            }
            0x5a => {
                self.add_cycles(ONE_CYCLE)?;
                let y = self.cpu().y;
                if self.cpu().xf {
                    self.push_byte(y as u8, accesses)?;
                } else {
                    self.push_word(y, accesses)?;
                }
                self.set_open_bus(y as u8);
            }
            0x5b => {
                self.add_cycles(ONE_CYCLE)?;
                self.cpu_mut().dp = self.cpu().a;
                self.set_zn(self.cpu().dp, false);
            }
            0x5c => {
                let target = self.absolute_long_address(false, accesses)?;
                self.cpu_mut().pc = target as u16;
                self.cpu_mut().k = (target >> 16) as u8;
            }
            0x60 => {
                self.add_cycles(TWO_CYCLES)?;
                let target = self.pull_word(accesses)?;
                self.add_cycles(ONE_CYCLE)?;
                self.cpu_mut().pc = target.wrapping_add(1);
            }
            0x64 => {
                let address = self.direct_address(false, accesses)?;
                if self.cpu().mf {
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
                self.set_open_bus(0);
            }
            0x65 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.adc(value);
            }
            0x68 => {
                self.add_cycles(TWO_CYCLES)?;
                let value = if self.cpu().mf {
                    let value = self.pull_byte(accesses)?;
                    self.cpu_mut().a = (self.cpu().a & 0xff00) | u16::from(value);
                    u16::from(value)
                } else {
                    let value = self.pull_word(accesses)?;
                    self.cpu_mut().a = value;
                    value
                };
                self.set_open_bus(if self.cpu().mf {
                    value as u8
                } else {
                    (value >> 8) as u8
                });
                self.set_zn(value, self.cpu().mf);
            }
            0x69 => {
                let value = self.immediate_by_m(accesses)?;
                self.adc(value);
            }
            0x6d => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.adc(value);
            }
            0x6b => {
                self.add_cycles(TWO_CYCLES)?;
                let target = self.pull_word_bank(accesses)?;
                self.cpu_mut().pc = target;
                let program_bank = self.pull_byte_without_emulation_bounds(accesses)?;
                self.cpu_mut().k = program_bank;
                if self.cpu().e {
                    // RTL shares JSL's full 65816 stack semantics in
                    // emulation mode, then repairs SH after all pulls.
                    self.cpu_mut().sp = 0x0100 | (self.cpu().sp & 0x00ff);
                }
                self.cpu_mut().pc = self.cpu().pc.wrapping_add(1);
            }
            0x70 => self.branch(self.cpu().v, accesses)?,
            0x78 => {
                self.add_cycles(ONE_CYCLE)?;
                self.cpu_mut().i = true;
            }
            0x7a => {
                let value = self.pull_by_x(accesses)?;
                self.cpu_mut().y = value;
                self.set_zn(value, self.cpu().xf);
                self.set_open_bus(if self.cpu().xf {
                    value as u8
                } else {
                    (value >> 8) as u8
                });
            }
            0x7c => {
                let operand = self.immediate16_slow(true, accesses)?;
                self.add_cycles(ONE_CYCLE)?;
                let pointer = operand.wrapping_add(self.cpu().x);
                let address = (u32::from(self.cpu().k) << 16) | u32::from(pointer);
                let target = self.read_word(address, WordWrap::Bank, accesses)?;
                self.set_open_bus((target >> 8) as u8);
                self.cpu_mut().pc = target;
            }
            0x7d => {
                let address = self.absolute_indexed_read_address(self.cpu().x, accesses)?;
                let operand = self.read_by_m(address, WordWrap::None, accesses)?;
                self.adc(operand);
            }
            0x79 => {
                let address = self.absolute_indexed_read_address(self.cpu().y, accesses)?;
                let operand = self.read_by_m(address, WordWrap::None, accesses)?;
                self.adc(operand);
            }
            0x7f => {
                let address = self.absolute_long_address(true, accesses)?;
                let address = address.wrapping_add(u32::from(self.cpu().x)) & 0x00ff_ffff;
                let operand = self.read_by_m(address, WordWrap::None, accesses)?;
                self.adc(operand);
            }
            0x80 => self.branch(true, accesses)?,
            0x84 => {
                let address = self.direct_address(false, accesses)?;
                let value = self.cpu().y;
                self.store_index_register(address, value, accesses)?;
            }
            0x85 => {
                let address = self.direct_address(false, accesses)?;
                self.store_accumulator(address, accesses)?;
            }
            0x86 => {
                let address = self.direct_address(false, accesses)?;
                let value = self.cpu().x;
                self.store_index_register(address, value, accesses)?;
            }
            0x88 => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().xf {
                    self.cpu_mut().y = u16::from((self.cpu().y as u8).wrapping_sub(1));
                    self.cpu().y
                } else {
                    self.cpu_mut().y = self.cpu().y.wrapping_sub(1);
                    self.cpu().y
                };
                self.set_zn(value, self.cpu().xf);
            }
            0x89 => {
                let operand = self.immediate_by_m(accesses)?;
                let accumulator = if self.cpu().mf {
                    self.cpu().a & 0x00ff
                } else {
                    self.cpu().a
                };
                self.cpu_mut().z = accumulator & operand == 0;
            }
            0x8c => {
                let address = self.absolute_address(false, accesses)?;
                let value = self.cpu().y;
                self.store_index_register(address, value, accesses)?;
            }
            0x8d => {
                let address = self.absolute_address(false, accesses)?;
                self.store_accumulator(address, accesses)?;
            }
            0x8e => {
                let address = self.absolute_address(false, accesses)?;
                let value = self.cpu().x;
                self.store_index_register(address, value, accesses)?;
            }
            0x8f => {
                let address = self.absolute_long_address(false, accesses)?;
                self.store_accumulator(address, accesses)?;
            }
            0x8a => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().mf {
                    let value = self.cpu().x as u8;
                    self.cpu_mut().a = (self.cpu().a & 0xff00) | u16::from(value);
                    u16::from(value)
                } else {
                    self.cpu_mut().a = self.cpu().x;
                    self.cpu().a
                };
                self.set_zn(value, self.cpu().mf);
            }
            0x8b => {
                self.add_cycles(ONE_CYCLE)?;
                let data_bank = self.cpu().db;
                self.push_byte(data_bank, accesses)?;
                self.set_open_bus(data_bank);
            }
            0x90 => self.branch(!self.cpu().c, accesses)?,
            0x95 => {
                let address = self.direct_indexed_x_address(false, accesses)?;
                self.store_accumulator(address, accesses)?;
            }
            0x97 => {
                let address = self
                    .direct_indirect_long_address(accesses)?
                    .wrapping_add(u32::from(self.cpu().y))
                    & 0x00ff_ffff;
                self.store_accumulator(address, accesses)?;
            }
            0x98 => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().mf {
                    let value = self.cpu().y as u8;
                    self.cpu_mut().a = (self.cpu().a & 0xff00) | u16::from(value);
                    u16::from(value)
                } else {
                    self.cpu_mut().a = self.cpu().y;
                    self.cpu().a
                };
                self.set_zn(value, self.cpu().mf);
            }
            0x99 => {
                let address = self.absolute_address(false, accesses)?;
                // cpuaddr.h:AbsoluteIndexedY1 always charges the internal
                // cycle for a write before adding Y to the data-bank address.
                self.add_cycles(ONE_CYCLE)?;
                let address = address.wrapping_add(u32::from(self.cpu().y)) & 0x00ff_ffff;
                self.store_accumulator(address, accesses)?;
            }
            0x9d => {
                let address = self.absolute_address(false, accesses)?;
                // cpuaddr.h:AbsoluteIndexedXX0 always charges this cycle;
                // AbsoluteIndexedXX1 also always charges it for writes.
                self.add_cycles(ONE_CYCLE)?;
                let address = address.wrapping_add(u32::from(self.cpu().x)) & 0x00ff_ffff;
                self.store_accumulator(address, accesses)?;
            }
            0x9e => {
                let address = self.absolute_address(false, accesses)?;
                self.add_cycles(ONE_CYCLE)?;
                let address = address.wrapping_add(u32::from(self.cpu().x)) & 0x00ff_ffff;
                self.write_by_m(address, 0, WordWriteOrder::LowHigh, accesses)?;
                self.set_open_bus(0);
            }
            0x9f => {
                let address = self.absolute_long_address(false, accesses)?;
                let address = address.wrapping_add(u32::from(self.cpu().x)) & 0x00ff_ffff;
                self.store_accumulator(address, accesses)?;
            }
            0x9b => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().xf {
                    self.cpu().x & 0x00ff
                } else {
                    self.cpu().x
                };
                self.cpu_mut().y = value;
                self.set_zn(value, self.cpu().xf);
            }
            0x9c => {
                let address = self.absolute_address(false, accesses)?;
                self.write_by_m(address, 0, WordWriteOrder::LowHigh, accesses)?;
                self.set_open_bus(0);
            }
            0xa0 => {
                let value = self.immediate_by_x(accesses)?;
                if self.cpu().xf {
                    self.cpu_mut().y = value & 0xff;
                } else {
                    self.cpu_mut().y = value;
                }
                self.set_zn(value, self.cpu().xf);
            }
            0xa2 => {
                let value = self.immediate_by_x(accesses)?;
                self.cpu_mut().x = if self.cpu().xf { value & 0xff } else { value };
                self.set_zn(value, self.cpu().xf);
            }
            0xa4 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.cpu_mut().y = value;
                self.set_zn(value, self.cpu().xf);
            }
            0xa5 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.load_accumulator(value);
            }
            0xa6 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.cpu_mut().x = value;
                self.set_zn(value, self.cpu().xf);
            }
            0xa7 => {
                let address = self.direct_indirect_long_address(accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.load_accumulator(value);
            }
            0xa8 => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().xf {
                    self.cpu().a & 0x00ff
                } else {
                    self.cpu().a
                };
                self.cpu_mut().y = value;
                self.set_zn(value, self.cpu().xf);
            }
            0xa9 => {
                let value = self.immediate_by_m(accesses)?;
                if self.cpu().mf {
                    self.cpu_mut().a = (self.cpu().a & 0xff00) | (value & 0xff);
                } else {
                    self.cpu_mut().a = value;
                }
                self.set_zn(value, self.cpu().mf);
            }
            0xaa => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().xf {
                    self.cpu().a & 0xff
                } else {
                    self.cpu().a
                };
                self.cpu_mut().x = value;
                self.set_zn(value, self.cpu().xf);
            }
            0xac => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.cpu_mut().y = value;
                self.set_zn(value, self.cpu().xf);
            }
            0xae => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.cpu_mut().x = value;
                self.set_zn(value, self.cpu().xf);
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
                self.cpu_mut().db = data_bank;
                self.set_zn(u16::from(data_bank), true);
                self.set_open_bus(data_bank);
            }
            0xb0 => self.branch(self.cpu().c, accesses)?,
            0xb1 => {
                let address = self.direct_indirect_indexed_y_address(false, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.load_accumulator(value);
            }
            0xb5 => {
                let address = self.direct_indexed_x_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.load_accumulator(value);
            }
            0xb7 => {
                let address = self
                    .direct_indirect_long_address(accesses)?
                    .wrapping_add(u32::from(self.cpu().y))
                    & 0x00ff_ffff;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.load_accumulator(value);
            }
            0xb9 => {
                let address = self.absolute_indexed_read_address(self.cpu().y, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.load_accumulator(value);
            }
            0xbf => {
                // cpuaddr.h:AbsoluteLongIndexedX is exactly AbsoluteLong(READ)
                // plus X. Unlike the 16-bit absolute indexed modes, it owns no
                // page-cross or mandatory internal cycle.
                let address = self.absolute_long_address(true, accesses)?;
                let address = address.wrapping_add(u32::from(self.cpu().x)) & 0x00ff_ffff;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.load_accumulator(value);
            }
            0xbb => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().xf {
                    self.cpu().y & 0x00ff
                } else {
                    self.cpu().y
                };
                self.cpu_mut().x = value;
                self.set_zn(value, self.cpu().xf);
            }
            0xbc => {
                let address = self.absolute_indexed_read_address(self.cpu().x, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.cpu_mut().y = value;
                self.set_zn(value, self.cpu().xf);
            }
            0xbd => {
                let address = self.absolute_indexed_read_address(self.cpu().x, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.load_accumulator(value);
            }
            0xc0 => {
                let value = self.immediate_by_x(accesses)?;
                self.compare(self.cpu().y, value, self.cpu().xf);
            }
            0xc9 => {
                let value = self.immediate_by_m(accesses)?;
                self.compare(self.cpu().a, value, self.cpu().mf);
            }
            0xc2 => {
                let mask = self.immediate8(true, accesses)?;
                let flags = self.cpu().pack_flags() & !mask;
                self.cpu_mut().unpack_flags(flags);
                self.add_cycles(ONE_CYCLE)?;
                self.fix_status_widths();
            }
            0xc5 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.compare(self.cpu().a, value, self.cpu().mf);
            }
            0xc6 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::Bank, accesses)?;
                let result = if self.cpu().mf {
                    u16::from((value as u8).wrapping_sub(1))
                } else {
                    value.wrapping_sub(1)
                };
                self.add_cycles(ONE_CYCLE)?;
                if self.cpu().mf {
                    self.write_byte(address, result as u8, accesses)?;
                } else {
                    self.write_word(
                        address,
                        result,
                        WordWrap::Bank,
                        WordWriteOrder::HighLow,
                        accesses,
                    )?;
                }
                self.set_open_bus(result as u8);
                self.set_zn(result, self.cpu().mf);
            }
            0xc8 => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().xf {
                    self.cpu_mut().y = u16::from((self.cpu().y as u8).wrapping_add(1));
                    self.cpu().y
                } else {
                    self.cpu_mut().y = self.cpu().y.wrapping_add(1);
                    self.cpu().y
                };
                self.set_zn(value, self.cpu().xf);
            }
            0xca => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().xf {
                    self.cpu_mut().x = u16::from((self.cpu().x as u8).wrapping_sub(1));
                    self.cpu().x
                } else {
                    self.cpu_mut().x = self.cpu().x.wrapping_sub(1);
                    self.cpu().x
                };
                self.set_zn(value, self.cpu().xf);
            }
            0xcd => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.compare(self.cpu().a, value, self.cpu().mf);
            }
            0xce => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                let result = if self.cpu().mf {
                    u16::from((value as u8).wrapping_sub(1))
                } else {
                    value.wrapping_sub(1)
                };
                self.add_cycles(ONE_CYCLE)?;
                self.write_by_m(address, result, WordWriteOrder::HighLow, accesses)?;
                self.set_open_bus(result as u8);
                self.set_zn(result, self.cpu().mf);
            }
            0xd0 => self.branch(!self.cpu().z, accesses)?,
            0xd5 => {
                let address = self.direct_indexed_x_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.compare(self.cpu().a, value, self.cpu().mf);
            }
            0xdd => {
                let address = self.absolute_indexed_read_address(self.cpu().x, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                self.compare(self.cpu().a, value, self.cpu().mf);
            }
            0xda => {
                self.add_cycles(ONE_CYCLE)?;
                let x = self.cpu().x;
                if self.cpu().xf {
                    self.push_byte(x as u8, accesses)?;
                } else {
                    self.push_word(x, accesses)?;
                }
                self.set_open_bus(x as u8);
            }
            0xdc => {
                let pointer = self.immediate16(true, accesses)?;
                let target = self.read_word(u32::from(pointer), WordWrap::None, accesses)?;
                self.set_open_bus((target >> 8) as u8);
                let program_bank = self.read_byte(u32::from(pointer) + 2, accesses)?;
                self.set_open_bus(program_bank);
                self.cpu_mut().pc = target;
                self.cpu_mut().k = program_bank;
            }
            0xe0 => {
                let value = self.immediate_by_x(accesses)?;
                self.compare(self.cpu().x, value, self.cpu().xf);
            }
            0xe2 => {
                let mask = self.immediate8(true, accesses)?;
                let flags = self.cpu().pack_flags() | mask;
                self.cpu_mut().unpack_flags(flags);
                self.add_cycles(ONE_CYCLE)?;
                self.fix_status_widths();
            }
            0xe4 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_x(address, WordWrap::Bank, accesses)?;
                self.compare(self.cpu().x, value, self.cpu().xf);
            }
            0xe5 => {
                let address = self.direct_address(true, accesses)?;
                let operand = self.read_by_m(address, WordWrap::Bank, accesses)?;
                self.sbc(operand);
            }
            0xe6 => {
                let address = self.direct_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::Bank, accesses)?;
                let result = if self.cpu().mf {
                    u16::from((value as u8).wrapping_add(1))
                } else {
                    value.wrapping_add(1)
                };
                self.add_cycles(ONE_CYCLE)?;
                self.write_by_m(address, result, WordWriteOrder::HighLow, accesses)?;
                self.set_open_bus(result as u8);
                self.set_zn(result, self.cpu().mf);
            }
            0xe8 => {
                self.add_cycles(ONE_CYCLE)?;
                let value = if self.cpu().xf {
                    self.cpu_mut().x = u16::from((self.cpu().x as u8).wrapping_add(1));
                    self.cpu().x
                } else {
                    self.cpu_mut().x = self.cpu().x.wrapping_add(1);
                    self.cpu().x
                };
                self.set_zn(value, self.cpu().xf);
            }
            0xe9 => {
                let operand = self.immediate_by_m(accesses)?;
                self.sbc(operand);
            }
            0xea => self.add_cycles(ONE_CYCLE)?,
            0xed => {
                let address = self.absolute_address(true, accesses)?;
                let operand = self.read_by_m(address, WordWrap::None, accesses)?;
                self.sbc(operand);
            }
            0xef => {
                let address = self.absolute_long_address(true, accesses)?;
                let operand = self.read_by_m(address, WordWrap::None, accesses)?;
                self.sbc(operand);
            }
            0xee => {
                let address = self.absolute_address(true, accesses)?;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                let result = if self.cpu().mf {
                    u16::from((value as u8).wrapping_add(1))
                } else {
                    value.wrapping_add(1)
                };
                self.add_cycles(ONE_CYCLE)?;
                self.write_by_m(address, result, WordWriteOrder::HighLow, accesses)?;
                self.set_open_bus(result as u8);
                self.set_zn(result, self.cpu().mf);
            }
            0xeb => {
                let value = self.cpu().a;
                self.cpu_mut().a = value.rotate_left(8);
                self.set_zn(self.cpu().a & 0xff, true);
                self.add_cycles(TWO_CYCLES)?;
            }
            0xf0 => self.branch(self.cpu().z, accesses)?,
            0xfa => {
                let value = self.pull_by_x(accesses)?;
                self.cpu_mut().x = value;
                self.set_zn(value, self.cpu().xf);
                self.set_open_bus(if self.cpu().xf {
                    value as u8
                } else {
                    (value >> 8) as u8
                });
            }
            0xfe => {
                let address = self.absolute_address(true, accesses)?;
                self.add_cycles(ONE_CYCLE)?;
                let address = address.wrapping_add(u32::from(self.cpu().x)) & 0x00ff_ffff;
                let value = self.read_by_m(address, WordWrap::None, accesses)?;
                let result = if self.cpu().mf {
                    u16::from((value as u8).wrapping_add(1))
                } else {
                    value.wrapping_add(1)
                };
                self.add_cycles(ONE_CYCLE)?;
                self.write_by_m(address, result, WordWriteOrder::HighLow, accesses)?;
                self.set_open_bus(result as u8);
                self.set_zn(result, self.cpu().mf);
            }
            0xfb => {
                self.add_cycles(ONE_CYCLE)?;
                let cpu = self.cpu_mut();
                std::mem::swap(&mut cpu.c, &mut cpu.e);
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
        if self.cpu().mf {
            Ok(u16::from(self.immediate8(true, accesses)?))
        } else {
            self.immediate16(true, accesses)
        }
    }

    fn immediate_by_x(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        if self.cpu().xf {
            Ok(u16::from(self.immediate8(true, accesses)?))
        } else {
            self.immediate16(true, accesses)
        }
    }

    fn direct_address(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        let operand = self.immediate8(update_open_bus, accesses)?;
        let dp = self.cpu().dp;
        if dp as u8 != 0 {
            self.add_cycles(ONE_CYCLE)?;
        }
        Ok(u32::from(dp.wrapping_add(u16::from(operand))))
    }

    fn direct_indexed_x_address(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        let address = self.direct_address(update_open_bus, accesses)? as u16;
        let indexed = if self.cpu().e && self.cpu().dp as u8 == 0 {
            (address & 0xff00) | u16::from((address as u8).wrapping_add(self.cpu().x as u8))
        } else {
            address.wrapping_add(self.cpu().x)
        };
        self.add_cycles(ONE_CYCLE)?;
        Ok(u32::from(indexed))
    }

    fn absolute_address(
        &mut self,
        update_open_bus: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        let address = self.immediate16(update_open_bus, accesses)?;
        Ok((u32::from(self.cpu().db) << 16) | u32::from(address))
    }

    fn absolute_indexed_read_address(
        &mut self,
        index: u16,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        let address = self.absolute_address(true, accesses)?;
        let crosses_page = u16::from(address as u8).wrapping_add(u16::from(index as u8)) >= 0x100;
        if !self.cpu().xf || crosses_page {
            self.add_cycles(ONE_CYCLE)?;
        }
        Ok(address.wrapping_add(u32::from(index)) & 0x00ff_ffff)
    }

    fn direct_indirect_long_address(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        let pointer = self.direct_address(true, accesses)?;
        let low = self.read_word(pointer, WordWrap::None, accesses)?;
        self.set_open_bus((low >> 8) as u8);
        let bank = self.read_byte(pointer.wrapping_add(2), accesses)?;
        self.set_open_bus(bank);
        Ok((u32::from(bank) << 16) | u32::from(low))
    }

    fn direct_indirect_indexed_y_address(
        &mut self,
        write: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        let pointer_address = self.direct_address(true, accesses)?;
        let pointer_wrap = if self.cpu().e {
            if self.cpu().dp as u8 == 0 {
                WordWrap::Page
            } else {
                WordWrap::Bank
            }
        } else {
            WordWrap::None
        };
        let pointer = self.read_word(pointer_address, pointer_wrap, accesses)?;
        self.set_open_bus((pointer >> 8) as u8);
        if write
            || !self.cpu().xf
            || u16::from(pointer as u8).wrapping_add(u16::from(self.cpu().y as u8)) >= 0x100
        {
            self.add_cycles(ONE_CYCLE)?;
        }
        Ok(((u32::from(self.cpu().db) << 16) | u32::from(pointer))
            .wrapping_add(u32::from(self.cpu().y))
            & 0x00ff_ffff)
    }

    fn branch(
        &mut self,
        taken: bool,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let offset = self.immediate8(true, accesses)? as i8;
        let old_pc = self.cpu().pc;
        let target = old_pc.wrapping_add_signed(i16::from(offset));
        if taken {
            self.add_cycles(ONE_CYCLE)?;
            if self.cpu().e && old_pc & 0xff00 != target & 0xff00 {
                self.add_cycles(ONE_CYCLE)?;
            }
            self.cpu_mut().pc = target;
        }
        Ok(())
    }

    fn store_accumulator(
        &mut self,
        address: u32,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let value = self.cpu().a;
        self.write_by_m(address, value, WordWriteOrder::LowHigh, accesses)?;
        self.set_open_bus(if self.cpu().mf {
            value as u8
        } else {
            (value >> 8) as u8
        });
        Ok(())
    }

    fn store_index_register(
        &mut self,
        address: u32,
        value: u16,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        if self.cpu().xf {
            self.write_byte(address, value as u8, accesses)?;
            self.set_open_bus(value as u8);
        } else {
            self.write_word(
                address,
                value,
                WordWrap::Bank,
                WordWriteOrder::LowHigh,
                accesses,
            )?;
            self.set_open_bus((value >> 8) as u8);
        }
        Ok(())
    }

    fn load_accumulator(&mut self, value: u16) {
        if self.cpu().mf {
            self.cpu_mut().a = (self.cpu().a & 0xff00) | (value & 0xff);
        } else {
            self.cpu_mut().a = value;
        }
        self.set_zn(value, self.cpu().mf);
    }

    fn ora_accumulator(&mut self, operand: u16) {
        if self.cpu().mf {
            let result = u16::from((self.cpu().a as u8) | operand as u8);
            self.cpu_mut().a = (self.cpu().a & 0xff00) | result;
            self.set_zn(result, true);
        } else {
            self.cpu_mut().a |= operand;
            self.set_zn(self.cpu().a, false);
        }
    }

    fn and_accumulator(&mut self, operand: u16) {
        if self.cpu().mf {
            let result = u16::from((self.cpu().a as u8) & operand as u8);
            self.cpu_mut().a = (self.cpu().a & 0xff00) | result;
            self.set_zn(result, true);
        } else {
            self.cpu_mut().a &= operand;
            self.set_zn(self.cpu().a, false);
        }
    }

    fn eor_accumulator(&mut self, operand: u16) {
        if self.cpu().mf {
            let result = u16::from((self.cpu().a as u8) ^ operand as u8);
            self.cpu_mut().a = (self.cpu().a & 0xff00) | result;
            self.set_zn(result, true);
        } else {
            self.cpu_mut().a ^= operand;
            self.set_zn(self.cpu().a, false);
        }
    }

    fn read_by_m(
        &mut self,
        address: u32,
        wrap: WordWrap,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let value = if self.cpu().mf {
            u16::from(self.read_byte(address, accesses)?)
        } else {
            self.read_word(address, wrap, accesses)?
        };
        self.set_open_bus(if self.cpu().mf {
            value as u8
        } else {
            (value >> 8) as u8
        });
        Ok(value)
    }

    fn read_by_x(
        &mut self,
        address: u32,
        wrap: WordWrap,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let value = if self.cpu().xf {
            u16::from(self.read_byte(address, accesses)?)
        } else {
            self.read_word(address, wrap, accesses)?
        };
        self.set_open_bus(if self.cpu().xf {
            value as u8
        } else {
            (value >> 8) as u8
        });
        Ok(value)
    }

    fn write_by_m(
        &mut self,
        address: u32,
        value: u16,
        order: WordWriteOrder,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        if self.cpu().mf {
            self.write_byte(address, value as u8, accesses)
        } else {
            self.write_word(address, value, WordWrap::None, order, accesses)
        }
    }

    fn push_byte(
        &mut self,
        value: u8,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        if self.cpu().e {
            let address = u32::from(self.cpu().sp);
            self.write_byte(address, value, accesses)?;
            let low = (self.cpu().sp as u8).wrapping_sub(1);
            self.cpu_mut().sp = 0x0100 | u16::from(low);
        } else {
            self.push_byte_without_emulation_bounds(value, accesses)?;
        }
        Ok(())
    }

    fn push_byte_without_emulation_bounds(
        &mut self,
        value: u8,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let address = u32::from(self.cpu().sp);
        self.write_byte(address, value, accesses)?;
        self.cpu_mut().sp = self.cpu().sp.wrapping_sub(1);
        Ok(())
    }

    fn push_word(
        &mut self,
        value: u16,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        if self.cpu().e {
            let address = 0x0100 | u32::from((self.cpu().sp as u8).wrapping_sub(1));
            self.write_word(
                address,
                value,
                WordWrap::Page,
                WordWriteOrder::HighLow,
                accesses,
            )?;
            let low = (self.cpu().sp as u8).wrapping_sub(2);
            self.cpu_mut().sp = 0x0100 | u16::from(low);
        } else {
            self.push_word_without_emulation_bounds(value, accesses)?;
        }
        Ok(())
    }

    fn push_word_without_emulation_bounds(
        &mut self,
        value: u16,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        let address = u32::from(self.cpu().sp.wrapping_sub(1));
        self.write_word(
            address,
            value,
            WordWrap::Bank,
            WordWriteOrder::HighLow,
            accesses,
        )?;
        self.cpu_mut().sp = self.cpu().sp.wrapping_sub(2);
        Ok(())
    }

    fn pull_byte(&mut self, accesses: &mut Vec<SourceCpuBusAccess>) -> Result<u8, SourceCpuError> {
        if self.cpu().e {
            let low = (self.cpu().sp as u8).wrapping_add(1);
            self.cpu_mut().sp = 0x0100 | u16::from(low);
            self.read_byte(u32::from(self.cpu().sp), accesses)
        } else {
            self.pull_byte_without_emulation_bounds(accesses)
        }
    }

    fn pull_byte_without_emulation_bounds(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        self.cpu_mut().sp = self.cpu().sp.wrapping_add(1);
        self.read_byte(u32::from(self.cpu().sp), accesses)
    }

    fn pull_word(&mut self, accesses: &mut Vec<SourceCpuBusAccess>) -> Result<u16, SourceCpuError> {
        if self.cpu().e {
            let low = (self.cpu().sp as u8).wrapping_add(1);
            self.cpu_mut().sp = 0x0100 | u16::from(low);
            let value = self.read_word(u32::from(self.cpu().sp), WordWrap::Page, accesses)?;
            let low = (self.cpu().sp as u8).wrapping_add(1);
            self.cpu_mut().sp = 0x0100 | u16::from(low);
            Ok(value)
        } else {
            let address = self.cpu().sp.wrapping_add(1);
            let value = self.read_word(u32::from(address), WordWrap::Bank, accesses)?;
            self.cpu_mut().sp = self.cpu().sp.wrapping_add(2);
            Ok(value)
        }
    }

    fn pull_word_bank(
        &mut self,
        accesses: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        let address = self.cpu().sp.wrapping_add(1);
        let value = self.read_word(u32::from(address), WordWrap::Bank, accesses)?;
        self.cpu_mut().sp = self.cpu().sp.wrapping_add(2);
        Ok(value)
    }

    fn pull_by_x(&mut self, accesses: &mut Vec<SourceCpuBusAccess>) -> Result<u16, SourceCpuError> {
        self.add_cycles(TWO_CYCLES)?;
        if self.cpu().xf {
            Ok(u16::from(self.pull_byte(accesses)?))
        } else {
            self.pull_word_bank(accesses)
        }
    }

    fn compare(&mut self, register: u16, operand: u16, eight_bit: bool) {
        if eight_bit {
            let result = (register as u8).wrapping_sub(operand as u8);
            self.cpu_mut().c = (register as u8) >= (operand as u8);
            self.set_zn(u16::from(result), true);
        } else {
            let result = register.wrapping_sub(operand);
            self.cpu_mut().c = register >= operand;
            self.set_zn(result, false);
        }
    }

    fn adc(&mut self, operand: u16) {
        let eight_bit = self.cpu().mf;
        let mask = if eight_bit { 0x00ff } else { 0xffff };
        let sign = if eight_bit { 0x0080 } else { 0x8000 };
        let lhs = self.cpu().a & mask;
        let rhs = operand & mask;
        let carry = u32::from(self.cpu().c);
        let binary = u32::from(lhs) + u32::from(rhs) + carry;
        let mut result = binary;
        if self.cpu().d {
            result = bcd_add(lhs, rhs, carry, if eight_bit { 2 } else { 4 });
        }
        self.cpu_mut().c = result > u32::from(mask);
        let narrowed = result as u16 & mask;
        self.cpu_mut().v = (!(lhs ^ rhs) & (rhs ^ binary as u16) & sign) != 0;
        self.cpu_mut().a = (self.cpu().a & !mask) | narrowed;
        self.set_zn(narrowed, eight_bit);
    }

    fn sbc(&mut self, operand: u16) {
        let eight_bit = self.cpu().mf;
        let mask = if eight_bit { 0x00ff } else { 0xffff };
        let sign = if eight_bit { 0x0080 } else { 0x8000 };
        let lhs = self.cpu().a & mask;
        let rhs = operand & mask;
        let carry = u32::from(self.cpu().c);
        let (result, overflow) = if self.cpu().d {
            bcd_sub(lhs, rhs, carry, if eight_bit { 2 } else { 4 })
        } else {
            let result = i32::from(lhs) - i32::from(rhs) + carry as i32 - 1;
            let narrowed = result as u16 & mask;
            (result, ((lhs ^ rhs) & (lhs ^ narrowed) & sign) != 0)
        };
        self.cpu_mut().c = result > i32::from(mask);
        if !self.cpu().d {
            self.cpu_mut().c = result >= 0;
        }
        let narrowed = result as u16 & mask;
        self.cpu_mut().v = overflow;
        self.cpu_mut().a = (self.cpu().a & !mask) | narrowed;
        self.set_zn(narrowed, eight_bit);
    }

    fn rol_accumulator(&mut self) {
        let eight_bit = self.cpu().mf;
        let mask = if eight_bit { 0x00ff } else { 0xffff };
        let carry_bit = if eight_bit { 0x0100 } else { 0x1_0000 };
        let result = (u32::from(self.cpu().a & mask) << 1) | u32::from(self.cpu().c);
        self.cpu_mut().c = result & carry_bit != 0;
        let value = result as u16 & mask;
        self.cpu_mut().a = (self.cpu().a & !mask) | value;
        self.set_zn(value, eight_bit);
    }

    fn asl_accumulator(&mut self) {
        let eight_bit = self.cpu().mf;
        let mask = if eight_bit { 0x00ff } else { 0xffff };
        let sign = if eight_bit { 0x0080 } else { 0x8000 };
        let value = self.cpu().a & mask;
        self.cpu_mut().c = value & sign != 0;
        let result = value.wrapping_shl(1) & mask;
        self.cpu_mut().a = (self.cpu().a & !mask) | result;
        self.set_zn(result, eight_bit);
    }

    fn lsr_accumulator(&mut self) {
        let eight_bit = self.cpu().mf;
        let mask = if eight_bit { 0x00ff } else { 0xffff };
        let value = self.cpu().a & mask;
        self.cpu_mut().c = value & 1 != 0;
        let result = value >> 1;
        self.cpu_mut().a = (self.cpu().a & !mask) | result;
        self.set_zn(result, eight_bit);
    }

    fn set_zn(&mut self, value: u16, eight_bit: bool) {
        let value = if eight_bit { value & 0xff } else { value };
        self.cpu_mut().z = value == 0;
        self.cpu_mut().n = if eight_bit {
            value & 0x80 != 0
        } else {
            value & 0x8000 != 0
        };
    }

    fn fix_status_widths(&mut self) {
        if self.cpu().e {
            self.cpu_mut().mf = true;
            self.cpu_mut().xf = true;
        }
        if self.cpu().xf {
            self.cpu_mut().x &= 0xff;
            self.cpu_mut().y &= 0xff;
        }
    }

    fn fix_emulation_stack(&mut self) {
        if self.cpu().e {
            self.cpu_mut().sp = 0x0100 | (self.cpu().sp & 0xff);
        }
    }
}

impl<T: SourceCpuInstructionBus> SourceCpuInstructions for T {}

#[cfg(test)]
mod tests;
