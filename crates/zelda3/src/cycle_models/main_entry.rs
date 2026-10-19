//! Native main-loop entry, ClearOamBuffer and module dispatch. The caller's
//! existing CPU context and clock own every transaction; no opcodes are decoded.

use super::nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO};
use snes::CpuMasterTimeline;
use super::{intro_clear::Access, native_work::WorkBus};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MainEntryAccess {
    ReadByte(u16),
    ReadWord(u16),
    WriteByte(u16, u8),
    WriteWord(u16, u16),
}

// Immutable module dispatch data, in the same order as module_main_routing.
// These are native routine operands, never observed source execution values.
const MODULE_ENTRIES: [u32; 28] = [
    0x0c_c120, 0x0c_cd7d, 0x0c_d053, 0x0c_d485, 0x0c_d88a, 0x02_8136, 0x02_821e, 0x02_87a2,
    0x02_83bf, 0x02_a475, 0x02_83bf, 0x02_a475, 0x02_991b, 0x02_9938, 0x00_f800, 0x02_9982,
    0x02_9ad7, 0x02_9af9, 0x09_f290, 0x02_9c4a, 0x0c_edad, 0x02_9cfc, 0x02_9e8a, 0x09_f79f,
    0x02_9edc, 0x02_9fec, 0x0e_986e, 0x02_8586,
];

pub(crate) fn module_table_byte(address: u16) -> Option<u8> {
    let index = address.checked_sub(0x8061)?;
    if index >= 84 {
        return None;
    }
    Some((MODULE_ENTRIES[usize::from(index % 28)] >> (8 * (index / 28))) as u8)
}

pub(crate) fn enter_module(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, MainEntryAccess) -> u16,
) -> u8 {
    enter_module_with_bus(cpu, clock, EntryBus(bus))
}

struct EntryBus<F>(F);
impl<F: FnMut(u64, MainEntryAccess) -> u16> WorkBus for EntryBus<F> {
    fn access(&mut self, at: u64, access: Access) -> u16 {
        let access = match access {
            Access::Read(address, 1) => MainEntryAccess::ReadByte(address as u16),
            Access::Read(address, 2) => MainEntryAccess::ReadWord(address as u16),
            Access::Write(address, value, 1) => MainEntryAccess::WriteByte(address as u16, value as u8),
            Access::Write(address, value, 2) => MainEntryAccess::WriteWord(address as u16, value),
            _ => unreachable!(),
        };
        (self.0)(at, access)
    }
}

pub(crate) fn enter_module_with_bus(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl WorkBus,
) -> u8 {
    assert_eq!(
        (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
        (0, 0x8038, 0, 0, 0x30)
    );
    let mut w = Entry { cpu, clock, bus };
    w.fetch();
    w.drain(6);
    w.cpu.p &= !4; // CLI
    w.fetch();
    w.drain(8);
    w.drain(6); // BRA $8051
    w.bus.begin_main_iteration(w.clock);
    w.fetch();
    w.drain(8); // INC $1A
    let counter = w.read_byte(0x1a);
    w.drain(6);
    let counter = counter.wrapping_add(1);
    w.byte_flags(counter);
    w.write_byte(0x1a, counter);
    w.fetch();
    w.drain(16);
    w.drain(6); // JSR ClearOamBuffer
    w.push_word(0x8055);
    w.fetch();
    w.drain(8);
    w.cpu.x = 0x60;
    w.byte_flags(0x60); // LDX #$60
    loop {
        w.fetch();
        w.drain(8);
        w.cpu.load_a_byte(0xf0);
        for group in [0x0801, 0x0881, 0x0901, 0x0981] {
            for offset in (0..0x20).step_by(4) {
                w.fetch();
                w.drain(16);
                w.drain(6); // STA abs,X
                w.write_byte(group + offset + w.cpu.x, w.cpu.a as u8);
            }
        }
        w.fetch();
        w.drain(6);
        w.cpu.load_a_byte(w.cpu.x as u8); // TXA
        w.fetch();
        w.drain(6);
        w.cpu.p |= CARRY; // SEC
        w.fetch();
        w.drain(8); // SBC #$20, native caller has decimal clear
        let old = w.cpu.a as u8;
        let value = old.wrapping_sub(0x20);
        w.cpu.p = (w.cpu.p & !(CARRY | 0x40))
            | u8::from(old >= 0x20)
            | if (old ^ value) & (old ^ 0x20) & 0x80 != 0 {
                0x40
            } else {
                0
            };
        w.cpu.load_a_byte(value);
        w.fetch();
        w.drain(6);
        w.cpu.x = u16::from(value); // TAX
        w.fetch();
        w.drain(8); // BPL
        if value & NEGATIVE != 0 {
            break;
        }
        w.drain(6);
    }
    w.fetch();
    w.drain(12); // RTS
    let returned = w.read_word(w.cpu.sp.wrapping_add(1));
    w.cpu.sp = w.cpu.sp.wrapping_add(2);
    w.drain(6);
    w.cpu.pc = returned.wrapping_add(1);
    assert_eq!(
        w.cpu.pc, 0x8056,
        "native clear returned to a different caller"
    );
    w.fetch();
    w.drain(24);
    w.drain(6); // JSL Module_MainRouting
    w.write_byte(w.cpu.sp, w.cpu.bank);
    w.cpu.sp = w.cpu.sp.wrapping_sub(1);
    w.push_word(0x8059);
    // LDY $10, then the three LDA abs,Y : STA dp pointer bytes.
    w.fetch();
    w.drain(8);
    let module = w.read_byte(0x10);
    assert!(module < 28, "native module selector out of range");
    w.cpu.y = u16::from(module);
    w.byte_flags(module);
    for (base, scratch) in [(0x8061, 3), (0x807d, 4), (0x8099, 5)] {
        w.fetch();
        w.drain(16);
        let value = w.read_byte(base + w.cpu.y);
        w.cpu.load_a_byte(value);
        w.fetch();
        w.drain(8);
        w.write_byte(scratch, value);
    }
    w.fetch();
    w.drain(16); // JML [$0003] consumes actual WRAM pointer
    w.cpu.pc = w.read_word(3);
    w.cpu.bank = w.read_byte(5);
    module
}

struct Entry<'a, B> {
    cpu: &'a mut NativeNmiContext,
    clock: &'a mut CpuMasterTimeline,
    bus: B,
}

impl<B: WorkBus> Entry<'_, B> {
    fn fetch(&mut self) {
        self.clock.advance_synchronous_pcbase_opcode_fetch(8);
    }
    fn drain(&mut self, cycles: u32) {
        let bus = &mut self.bus;
        self.clock
            .advance_synchronous_after_semantics_with_clock(cycles, |event, clock| Ok::<u32, String>(bus.retire_event(event, clock)))
            .expect("native main entry clock");
    }
    fn byte_flags(&mut self, value: u8) {
        self.cpu.p = (self.cpu.p & !(ZERO | NEGATIVE))
            | (value & NEGATIVE)
            | if value == 0 { ZERO } else { 0 };
    }
    fn read_byte(&mut self, address: u16) -> u8 {
        self.bus.observe_clock(self.clock);
        let value = self.bus.access(self.clock.clock_master_cycles(), Access::Read(u32::from(address), 1)) as u8;
        self.drain(8);
        value
    }
    fn read_word(&mut self, address: u16) -> u16 {
        self.bus.observe_clock(self.clock);
        let value = self.bus.access(self.clock.clock_master_cycles(), Access::Read(u32::from(address), 2));
        self.drain(16);
        value
    }
    fn write_byte(&mut self, address: u16, value: u8) {
        self.bus.observe_clock(self.clock);
        self.bus.access(self.clock.clock_master_cycles(), Access::Write(u32::from(address), u16::from(value), 1));
        self.drain(8);
    }
    fn push_word(&mut self, value: u16) {
        assert!((1..0x2000).contains(&self.cpu.sp));
        self.bus.observe_clock(self.clock);
        self.bus.access(self.clock.clock_master_cycles(), Access::Write(u32::from(self.cpu.sp - 1), value, 2));
        self.drain(16);
        self.cpu.sp = self.cpu.sp.wrapping_sub(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourceCpuBusAccessKind as Kind, SourcePpuReadState,
    };

    #[test]
    fn main_entry_matches_rom_oam_store_order_calls_and_every_module_pointer() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(rom) = std::fs::read(path) else { return };
        for address in 0x8061..0x80b5 {
            assert_eq!(
                module_table_byte(address),
                Some(rom[usize::from(address - 0x8000)])
            );
        }
        for module in 0..28 {
            for counter in [0u8, 0x7f, 0xff] {
                for (v, h) in [(252, 224), (225, 510), (261, 1300)] {
                    for odd in [false, true] {
                        let mut native = NativeNmiContext {
                            a: 0xab01,
                            x: 0x53,
                            y: 0x67,
                            d: 0,
                            db: 0,
                            bank: 0,
                            pc: 0x8038,
                            sp: 0x1ff,
                            p: if odd { 0x77 } else { 0x30 },
                        };
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        for (i, b) in machine.ram.iter_mut().enumerate() {
                            *b = (i ^ (i >> 8)) as u8;
                        }
                        machine.ram[0x10] = module;
                        machine.ram[0x1a] = counter;
                        let mut ram = machine.ram.clone();
                        machine.cpu.a = native.a;
                        machine.cpu.x = native.x;
                        machine.cpu.y = native.y;
                        machine.cpu.pc = native.pc;
                        machine.cpu.sp = native.sp;
                        machine.cpu.e = false;
                        machine.cpu.unpack_flags(native.p);
                        let mut clock = CpuMasterTimeline::at_raster(
                            0,
                            CpuRasterPosition::new(v, h),
                            CpuBusWorkload::default(),
                            CpuFieldTiming::non_interlace(odd),
                        );
                        let mut source = RomCpuTimingProbe::new(
                            machine,
                            clock.clone(),
                            SourcePpuReadState::snes9x_reset(),
                        )
                        .unwrap();
                        let target = MODULE_ENTRIES[usize::from(module)];
                        let mut expected = Vec::new();
                        while (u32::from(source.snes().cpu.k) << 16)
                            | u32::from(source.snes().cpu.pc)
                            != target
                        {
                            for access in source.step().unwrap().accesses {
                                if access.address < 0x2000
                                    || (0x8061..0x80b5).contains(&access.address)
                                {
                                    let address = access.address as u16;
                                    let operation = match access.kind {
                                        Kind::Read { width: 1, .. } => {
                                            MainEntryAccess::ReadByte(address)
                                        }
                                        Kind::Read { width: 2, .. } => {
                                            MainEntryAccess::ReadWord(address)
                                        }
                                        Kind::Write { value, width: 1 } => {
                                            MainEntryAccess::WriteByte(address, value as u8)
                                        }
                                        Kind::Write { value, width: 2 } => {
                                            MainEntryAccess::WriteWord(address, value)
                                        }
                                        _ => panic!("unexpected caller transaction {access:?}"),
                                    };
                                    expected.push((access.timestamp.master_cycles(), operation));
                                }
                            }
                        }
                        clock.begin_synchronous_timeline().unwrap();
                        let mut actual = Vec::new();
                        let selected = enter_module(&mut native, &mut clock, |at, access| {
                            actual.push((at, access));
                            match access {
                                MainEntryAccess::ReadByte(address) => u16::from(
                                    module_table_byte(address)
                                        .unwrap_or_else(|| ram[usize::from(address)]),
                                ),
                                MainEntryAccess::ReadWord(address) => u16::from_le_bytes(
                                    ram[usize::from(address)..usize::from(address) + 2]
                                        .try_into()
                                        .unwrap(),
                                ),
                                MainEntryAccess::WriteByte(address, value) => {
                                    ram[usize::from(address)] = value;
                                    0
                                }
                                MainEntryAccess::WriteWord(address, value) => {
                                    ram[usize::from(address)..usize::from(address) + 2]
                                        .copy_from_slice(&value.to_le_bytes());
                                    0
                                }
                            }
                        });
                        let label =
                            format!("module={module} counter={counter} v={v} h={h} odd={odd}");
                        assert_eq!(selected, module);
                        assert_eq!(actual, expected, "{label}");
                        assert_eq!(ram, source.snes().ram, "{label}");
                        assert_eq!(clock, *source.timeline(), "{label}");
                        let cpu = &source.snes().cpu;
                        assert_eq!(
                            (
                                native.a,
                                native.x,
                                native.y,
                                native.d,
                                native.db,
                                native.bank,
                                native.pc,
                                native.sp,
                                native.p
                            ),
                            (
                                cpu.a,
                                cpu.x,
                                cpu.y,
                                cpu.dp,
                                cpu.db,
                                cpu.k,
                                cpu.pc,
                                cpu.sp,
                                cpu.pack_flags()
                            ),
                            "{label}"
                        );
                    }
                }
            }
        }
    }
}
