//! Native first intro graphics half-slot call. Every pointer update and plane
//! store belongs to this live caller, including reads across LoROM banks.
use super::nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    Read { address: u32, width: u8 },
    Write { address: u32, value: u16, width: u8 },
}

pub(crate) const INTRO_SHEET_ADDRESS: u32 = 0x11_a000;
pub(crate) const INTRO_SHEET: usize = 8;
// Immutable selector/pointer bytes for the intro's request20 and raw sheet8.
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    match address {
        0xe3f9 => Some(0xff),
        0xe3e5 => Some(8),
        0xcffb => Some(0x11),
        0xd0da => Some(0xa0),
        0xd1b9 => Some(0),
        _ => None,
    }
}

pub(crate) fn load_intro(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16,
) -> u64 {
    assert_eq!(
        (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
        (0, 0xe3fa, 0, 0, 0x30)
    );
    let mut w = Work {
        cpu,
        clock,
        bus,
        cycles: 0,
        operand: 0,
    };
    w.absolute_operand(0xaaa, 1);
    w.cpu.x = w.operand;
    w.flags(w.cpu.x, 1); // LDX
    assert_eq!(
        w.cpu.x, 20,
        "initial intro half-slot owner requires request20"
    );
    w.branch(false); // BEQ return
    w.push_bank(w.cpu.db);
    w.push_bank(w.cpu.bank);
    w.pull_bank();
    w.absolute_operand(0xe3e5 + u32::from(w.cpu.x), 1);
    w.load(w.operand, 1);
    assert!(
        w.cpu.a & 0x80 != 0,
        "intro half-slot unexpectedly requires a palette callee"
    );
    w.branch(true); // BMI
    w.absolute_operand(0xaaa, 1);
    w.cpu.x = w.operand;
    w.flags(w.cpu.x, 1);
    w.load_y_immediate(0x44, 1);
    w.store_dp(8, 0, 1);
    w.store_dp(9, 0, 1);
    w.rmw(0xaaa, 1, false, true);
    w.absolute_operand(0xaaa, 1);
    w.load(w.operand, 1);
    w.fetch();
    w.drain(6);
    let value = w.cpu.a as u8;
    w.cpu.p = (w.cpu.p & !CARRY) | (value & 1);
    w.load(u16::from(value >> 1), 1); // LSR
    if w.branch(w.cpu.p & CARRY == 0) {
        // General branch semantics retained although this caller selects20.
    } else {
        w.store_abs(0xaaa, 0, 1);
        w.compare_x(18);
        if !w.branch(w.cpu.p & ZERO != 0) {
            w.load_immediate(3, 1);
            w.store_dp(9, w.cpu.a, 1);
            w.load_y_immediate(0x46, 1);
            w.compare_x(2);
            if !w.branch(w.cpu.p & ZERO == 0) {
                w.store_abs(0x112, 0, 1);
            }
        }
    }
    w.store_abs(0x116, w.cpu.y, 1);
    w.load_immediate(11, 1);
    w.store_dp(0x17, w.cpu.a, 1);
    w.absolute_operand(0xe3d1 + u32::from(w.cpu.x), 1);
    w.cpu.y = w.operand;
    w.flags(w.cpu.y, 1);
    w.fetch();
    w.drain(8);
    w.compare(w.cpu.y, 1, 1); // CPY #1
    if !w.branch(w.cpu.p & ZERO == 0) {
        w.absolute_operand(0xaa4, 1);
        w.cpu.y = w.operand;
        w.flags(w.cpu.y, 1);
    }
    for (table, destinations) in [
        (0xcff3, &[2, 5][..]),
        (0xd0d2, &[1][..]),
        (0xd1b1, &[0][..]),
    ] {
        w.fetch();
        w.drain(16);
        if (table & 255) + u32::from(w.cpu.y) > 255 {
            w.drain(6);
        }
        let value = w.read(table + u32::from(w.cpu.y), 1);
        w.load(value, 1);
        for &destination in destinations {
            w.store_dp(destination, w.cpu.a, 1);
        }
    }
    w.fetch();
    w.drain(8);
    w.drain(6);
    w.cpu.p &= !0x31; // REP #$31
    w.load_y_immediate(32, 2);
    w.store_dp(0xc, w.cpu.y, 2);
    w.fetch();
    w.drain(16);
    w.cpu.x = 0;
    w.flags(0, 2); // LDX #0
    w.load_dp(0);
    let offset = w.dp_operand(8);
    w.add(offset); // ADC $08
    loop {
        w.store_dp(0, w.cpu.a, 2);
        w.clc();
        w.fetch();
        w.drain(16);
        w.add(16);
        if !w.branch(w.cpu.p & ZERO == 0) {
            w.load_immediate(0x8000, 2);
            w.rmw(5, 2, false, false);
        }
        w.store_dp(3, w.cpu.a, 2);
        w.load_y_immediate(7, 2);
        loop {
            let pair = w.indirect(0);
            w.load(pair, 2);
            w.store_long(0x7f1000 + u32::from(w.cpu.x), w.cpu.a);
            w.xba();
            let pair = w.indirect(0);
            w.load(w.cpu.a | pair, 2);
            w.and_byte();
            w.store_dp(8, w.cpu.a, 2);
            w.rmw(0, 2, false, false);
            w.rmw(0, 2, false, false);
            if !w.branch(w.cpu.p & ZERO == 0) {
                w.load_dp(3);
                w.fetch();
                w.drain(6);
                w.load(w.cpu.a.wrapping_add(1), 2);
                w.store_dp(0, w.cpu.a, 2);
                w.rmw(2, 2, false, false);
                w.load_dp(2);
                w.store_dp(5, w.cpu.a, 2);
            }
            let third = w.indirect(3);
            w.load(third, 2);
            w.and_byte();
            w.store_dp(0xa, w.cpu.a, 2);
            let mask = w.dp_operand(8);
            w.load(w.cpu.a | mask, 2);
            w.xba();
            let third = w.dp_operand(0xa);
            w.load(w.cpu.a | third, 2);
            w.store_long(0x7f1010 + u32::from(w.cpu.x), w.cpu.a);
            w.rmw(3, 2, false, false);
            if !w.branch(w.cpu.p & ZERO == 0) {
                w.load_immediate(0x8000, 2);
                w.store_dp(0, w.cpu.a, 2);
                w.load_immediate(0x8010, 2);
                w.store_dp(3, w.cpu.a, 2);
                w.rmw(2, 2, false, false);
                w.rmw(5, 2, false, false);
            }
            for _ in 0..2 {
                w.fetch();
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1);
                w.flags(w.cpu.x, 2);
            }
            w.fetch();
            w.drain(6);
            w.cpu.y = w.cpu.y.wrapping_sub(1);
            w.flags(w.cpu.y, 2);
            if !w.branch(w.cpu.p & NEGATIVE == 0) {
                break;
            }
        }
        w.fetch();
        w.drain(6);
        w.load(w.cpu.x, 2); // TXA
        w.clc();
        w.fetch();
        w.drain(16);
        w.add(16);
        w.fetch();
        w.drain(6);
        w.cpu.x = w.cpu.a; // TAX
        w.load_dp(3);
        w.rmw(0xc, 2, true, false);
        if !w.branch(w.cpu.p & ZERO == 0) {
            break;
        }
    }
    w.fetch();
    w.drain(8);
    w.drain(6);
    w.cpu.p |= 0x30;
    w.cpu.x &= 255;
    w.cpu.y &= 255;
    w.pull_bank();
    w.fetch();
    w.drain(12); // RTL, read the actual caller frame
    w.cpu.pc = w.pop(2).wrapping_add(1);
    w.cpu.bank = w.pop(1) as u8;
    w.cycles
}

struct Work<'a, B> {
    cpu: &'a mut NativeNmiContext,
    clock: &'a mut CpuMasterTimeline,
    bus: B,
    cycles: u64,
    operand: u16,
}
impl<B: FnMut(u64, Access) -> u16> Work<'_, B> {
    fn fetch(&mut self) {
        self.cycles += 8;
        self.clock.advance_synchronous_pcbase_opcode_fetch(8);
    }
    fn drain(&mut self, n: u32) {
        self.cycles += u64::from(n);
        self.clock
            .advance_synchronous_after_semantics_with(n, |_, _| Ok::<u32, String>(0))
            .unwrap();
    }
    fn read(&mut self, address: u32, width: u8) -> u16 {
        let v = (self.bus)(
            self.clock.clock_master_cycles(),
            Access::Read { address, width },
        );
        self.drain(8 * u32::from(width));
        v
    }
    fn write(&mut self, address: u32, value: u16, width: u8) {
        (self.bus)(
            self.clock.clock_master_cycles(),
            Access::Write {
                address,
                value: if width == 1 { value & 255 } else { value },
                width,
            },
        );
        self.drain(8 * u32::from(width));
    }
    fn flags(&mut self, value: u16, width: u8) {
        let v = if width == 1 { value & 255 } else { value };
        self.cpu.p = (self.cpu.p & !(ZERO | NEGATIVE))
            | if v == 0 { ZERO } else { 0 }
            | ((if width == 1 { v } else { v >> 8 }) as u8 & NEGATIVE);
    }
    fn load(&mut self, value: u16, width: u8) {
        if width == 1 {
            self.cpu.a = (self.cpu.a & 0xff00) | (value & 255)
        } else {
            self.cpu.a = value
        }
        self.flags(value, width);
    }
    fn load_immediate(&mut self, value: u16, width: u8) {
        self.fetch();
        self.drain(8 * u32::from(width));
        self.load(value, width);
    }
    fn load_y_immediate(&mut self, value: u16, width: u8) {
        self.fetch();
        self.drain(8 * u32::from(width));
        self.cpu.y = value;
        self.flags(value, width);
    }
    fn branch(&mut self, taken: bool) -> bool {
        self.fetch();
        self.drain(8);
        if taken {
            self.drain(6)
        }
        taken
    }
    fn absolute_operand(&mut self, address: u32, width: u8) {
        self.fetch();
        self.drain(16);
        self.operand = self.read(address, width);
    }
    fn dp_operand(&mut self, address: u32) -> u16 {
        self.fetch();
        self.drain(8);
        self.read(address, 2)
    }
    fn load_dp(&mut self, address: u32) {
        let v = self.dp_operand(address);
        self.load(v, 2);
    }
    fn store_dp(&mut self, address: u32, value: u16, width: u8) {
        self.fetch();
        self.drain(8);
        self.write(address, value, width);
    }
    fn store_abs(&mut self, address: u32, value: u16, width: u8) {
        self.fetch();
        self.drain(16);
        self.write(address, value, width);
    }
    fn store_long(&mut self, address: u32, value: u16) {
        self.fetch();
        self.drain(24);
        self.write(address, value, 2);
    }
    fn indirect(&mut self, dp: u32) -> u16 {
        self.fetch();
        self.drain(8);
        let lo = self.read(dp, 2);
        let bank = self.read(dp + 2, 1);
        self.read(u32::from(lo) | (u32::from(bank) << 16), 2)
    }
    fn rmw(&mut self, address: u32, width: u8, decrement: bool, absolute: bool) {
        self.fetch();
        self.drain(if absolute { 16 } else { 8 });
        let v = self.read(address, width);
        self.drain(6);
        let value = if decrement {
            v.wrapping_sub(1)
        } else {
            v.wrapping_add(1)
        };
        self.flags(value, width);
        self.write(address, value, width);
    }
    fn push_bank(&mut self, value: u8) {
        self.fetch();
        self.drain(6);
        self.write(u32::from(self.cpu.sp), u16::from(value), 1);
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
    }
    fn pop(&mut self, width: u8) -> u16 {
        let v = self.read(u32::from(self.cpu.sp.wrapping_add(1)), width);
        self.cpu.sp = self.cpu.sp.wrapping_add(u16::from(width));
        v
    }
    fn pull_bank(&mut self) {
        self.fetch();
        self.drain(12);
        self.cpu.db = self.pop(1) as u8;
        self.flags(u16::from(self.cpu.db), 1);
    }
    fn compare(&mut self, value: u16, other: u16, width: u8) {
        self.cpu.p = (self.cpu.p & !CARRY) | u8::from(value >= other);
        self.flags(value.wrapping_sub(other), width);
    }
    fn compare_x(&mut self, value: u16) {
        self.fetch();
        self.drain(8);
        self.compare(self.cpu.x, value, 1);
    }
    fn clc(&mut self) {
        self.fetch();
        self.drain(6);
        self.cpu.p &= !CARRY;
    }
    fn add(&mut self, value: u16) {
        let a = self.cpu.a;
        let sum = u32::from(a) + u32::from(value) + u32::from(self.cpu.p & CARRY);
        let v = sum as u16;
        self.cpu.p = (self.cpu.p & !(CARRY | 0x40))
            | u8::from(sum > 65535)
            | if (!(a ^ value) & (a ^ v) & 0x8000) != 0 {
                0x40
            } else {
                0
            };
        self.load(v, 2);
    }
    fn xba(&mut self) {
        self.fetch();
        self.drain(12);
        self.cpu.a = self.cpu.a.swap_bytes();
        self.flags(self.cpu.a, 1);
    }
    fn and_byte(&mut self) {
        self.fetch();
        self.drain(16);
        self.load(self.cpu.a & 255, 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourceCpuBusAccessKind as Kind, SourcePpuReadState,
    };

    fn ram_offset(address: u32) -> Option<usize> {
        if (0x7e0000..0x800000).contains(&address) {
            Some((address - 0x7e0000) as usize)
        } else if address & 0x400000 == 0 && address & 0xffff < 0x2000 {
            Some((address & 0x1fff) as usize)
        } else {
            None
        }
    }
    fn read(address: u32, ram: &[u8], rom: &[u8]) -> u8 {
        if let Some(i) = ram_offset(address) {
            ram[i]
        } else {
            assert!(
                address & 0xffff >= 0x8000,
                "unmapped test read{address:06x}"
            );
            rom[(((address >> 16) & 127) * 0x8000 + (address & 0x7fff)) as usize]
        }
    }

    #[test]
    fn intro_half_slot_matches_rom_bus_reads_plane_stores_banks_and_return() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(original) = std::fs::read(path) else {
            return;
        };
        for address in [0xe3f9, 0xe3e5, 0xcffb, 0xd0da, 0xd1b9] {
            assert_eq!(
                table_byte(address),
                Some(original[(address - 0x8000) as usize])
            );
        }
        // Two data-only pointer fixtures put each plane on a LoROM boundary.
        // The original routine's opcodes are unchanged in all cases.
        for source_low in [0xa000u16, 0xfcf0, 0xfce0] {
            let mut rom = original.clone();
            rom[0x50da] = (source_low >> 8) as u8;
            rom[0x51b9] = source_low as u8;
            for seed in [0u8, 0x7f, 0xff] {
                for (v, h) in [(257, 1266), (225, 510), (261, 1300)] {
                    for odd in [false, true] {
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        for (i, b) in machine.ram.iter_mut().enumerate() {
                            *b = (i ^ (i >> 8)) as u8 ^ seed;
                        }
                        machine.ram[0xaaa] = 20;
                        machine.ram[0x1f7..0x1fa].copy_from_slice(&[0x23, 0x80, 2]);
                        let mut native = NativeNmiContext {
                            a: (u16::from(seed) << 8) | 20,
                            x: 0xe0,
                            y: 0,
                            d: 0,
                            db: 0,
                            bank: 0,
                            pc: 0xe3fa,
                            sp: 0x1f6,
                            p: 0x30 | (seed & 0xc7),
                        };
                        machine.cpu.a = native.a;
                        machine.cpu.x = native.x;
                        machine.cpu.y = native.y;
                        machine.cpu.pc = native.pc;
                        machine.cpu.sp = native.sp;
                        machine.cpu.e = false;
                        machine.cpu.unpack_flags(native.p);
                        let mut ram = machine.ram.clone();
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
                        let mut expected = Vec::new();
                        for step in 0..20000 {
                            if (source.snes().cpu.k, source.snes().cpu.pc) == (2, 0x8024) {
                                break;
                            }
                            assert!(step < 19999, "half-slot did not return");
                            for a in source.step().unwrap().accesses {
                                if a.address < 0x2000
                                    || a.address >= 0x100000
                                    || table_byte(a.address).is_some()
                                {
                                    let (op, read_value) = match a.kind {
                                        Kind::Read { value, width } => (
                                            Access::Read {
                                                address: a.address,
                                                width,
                                            },
                                            value,
                                        ),
                                        Kind::Write { value, width } => (
                                            Access::Write {
                                                address: a.address,
                                                value,
                                                width,
                                            },
                                            0,
                                        ),
                                        _ => panic!("unexpected half-slot bus access {a:?}"),
                                    };
                                    expected.push((a.timestamp.master_cycles(), op, read_value));
                                }
                            }
                        }
                        clock.begin_synchronous_timeline().unwrap();
                        let mut actual = Vec::new();
                        let cycles = load_intro(&mut native, &mut clock, |at, access| {
                            let value = match access {
                                Access::Read { address, width } => {
                                    let low = read(address, &ram, &rom);
                                    if width == 1 {
                                        u16::from(low)
                                    } else {
                                        u16::from_le_bytes([low, read(address + 1, &ram, &rom)])
                                    }
                                }
                                Access::Write {
                                    address,
                                    value,
                                    width,
                                } => {
                                    for byte in 0..width {
                                        ram[ram_offset(address + u32::from(byte)).unwrap()] =
                                            (value >> (8 * byte)) as u8;
                                    }
                                    0
                                }
                            };
                            actual.push((at, access, value));
                            value
                        });
                        let label =
                            format!("pointer={source_low:04x} seed={seed} v={v} h={h} odd={odd}");
                        assert!(cycles > 0);
                        assert_eq!(actual.len(), expected.len(), "{label}");
                        for (i, (a, b)) in actual.iter().zip(&expected).enumerate() {
                            assert_eq!(a, b, "{label} access{i}");
                        }
                        assert_eq!(clock, *source.timeline(), "{label}");
                        assert_eq!(ram, source.snes().ram, "{label}");
                        let c = &source.snes().cpu;
                        assert_eq!(
                            (
                                native.a,
                                native.x,
                                native.y,
                                native.sp,
                                native.pc,
                                native.bank,
                                native.db,
                                native.d,
                                native.p
                            ),
                            (c.a, c.x, c.y, c.sp, c.pc, c.k, c.db, c.dp, c.pack_flags()),
                            "{label}"
                        );
                        assert_eq!(
                            actual
                                .iter()
                                .filter(|(_, a, _)| matches!(
                                    a,
                                    Access::Write {
                                        address: 0x7f1000..=0x7f13ff,
                                        ..
                                    }
                                ))
                                .count(),
                            512
                        );
                    }
                }
            }
        }
    }
}
