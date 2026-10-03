//! First intro setup, graphics-call boundaries, and logo/clear dispatch.
//! Stack and indirect dispatch operands come from the live native bus.

use super::nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    ReadByte(u32),
    ReadWord(u32),
    WriteByte(u16, u8),
    WriteWord(u16, u16),
    WriteLongWord(u32, u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IntroWork {
    Clear,
    Text,
    Items,
    Follower,
    Darkening,
    Poly,
    Animation,
    Title,
}

// Immutable entries used by the native setup and column-clear owners.
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    match address {
        0x0cd890..=0x0cd89b => Some([0x9c,0xd8,12,0x11,0xd9,12,0x28,0xd9,12,0x4d,0xda,12][(address-0x0cd890)as usize]),
        0x0ccd8b..=0x0ccd9c => Some([0x9d,0xcd,12,0xf2,0xcd,12,0x53,0xce,12,
            0xa5,0xce,12,0xb1,0xce,12,0xbd,0xce,12][(address-0x0ccd8b) as usize]),
        0x0c_c139 => Some(0x5d),
        0x0c_c13a => Some(0xc1),
        0x0c_c13b => Some(0x0c),
        0x0c_c13c => Some(0x70),
        0x0c_c13d => Some(0xc1),
        0x0c_c13e => Some(0x0c),
        0x0c_c13f..=0x0c_c141 => Some([0x3c, 0xc3, 0x0c][(address - 0x0c_c13f) as usize]),
        0x0c_c142..=0x0c_c147 => Some([0x04, 0xc4, 0x0c][((address - 0x0c_c142) % 3) as usize]),
        0x0c_c148..=0x0c_c14d => Some([0x5c, 0xc2, 0x0c, 0xae, 0xc2, 0x0c][(address - 0x0c_c148) as usize]),
        0x0c_c14e..=0x0c_c153 => Some([0x84, 0xc2, 0x0c, 0xd4, 0xc2, 0x0c][(address - 0x0c_c14e) as usize]),
        0x0c_c17f..=0x0c_c196 => Some([0xa0, 0xc1, 0x0c][((address - 0x0c_c17f) % 3) as usize]),
        0x0c_c197..=0x0c_c199 => Some([0x16, 0x81, 0x02][(address - 0x0c_c197) as usize]),
        0x0c_c19a..=0x0c_c19c => Some([0x31, 0xd2, 0x00][(address - 0x0c_c19a) as usize]),
        0x0c_c19d..=0x0c_c19f => Some([0x23, 0xd4, 0x00][(address - 0x0c_c19d) as usize]),
        0x0c_ed7a..=0x0c_ed7d => Some([0x60, 0x70, 0x80, 0x88][(address - 0x0c_ed7a) as usize]),
        0x0c_ed7e..=0x0c_ed81 => Some([0x69, 0x6b, 0x6d, 0x6e][(address - 0x0c_ed7e) as usize]),
        _ => None,
    }
}

pub(crate) fn enter_name_file(cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,
    bus:impl FnMut(u64,Access)->u16)->u64 {
    assert_eq!((cpu.bank,cpu.pc,cpu.db,cpu.d,cpu.p&0x38),(12,0xd88a,0,0,0x30));
    let mut w=Work{cpu,clock,bus,cycles:0};
    w.load_dp(0x11);assert!(w.cpu.a&255<=3);
    w.jsl(0xd88f,0,0x879c);w.jump_table_long();
    w.cycles
}

pub(crate) fn enter_file_select(cpu:&mut NativeNmiContext,clock:&mut CpuMasterTimeline,
    bus:impl FnMut(u64,Access)->u16)->u64 {
    assert_eq!((cpu.bank,cpu.pc,cpu.db,cpu.d,cpu.p&0x38),(12,0xcd7d,0,0,0x30));
    let mut w=Work{cpu,clock,bus,cycles:0};
    for address in [0xe4,0xe5,0xea,0xeb] {w.store_dp(address,0,1);}
    w.load_dp(0x11);assert!(w.cpu.a&255<=5);
    w.jsl(0xcd8a,0,0x879c);w.jump_table_long();
    if matches!(w.cpu.pc,0xcdf2|0xce53|0xcea5|0xceb1|0xcebd) {return w.cycles;}
    assert_eq!((w.cpu.bank,w.cpu.pc),(12,0xcd9d));
    w.jsl(0xcda0,0,0x893d);
    w.load_immediate(0x80);w.store_abs(0x2100,0x80);w.store_dp(0x13,0x80,1);
    w.store_abs(0x420c,0);w.store_dp(0x9b,0,1);w.rtl();
    assert_eq!((w.cpu.bank,w.cpu.pc),(12,0xcda1));
    w.cycles
}

pub(crate) fn enter_half_slot(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16,
) -> u64 {
    assert_eq!(
        (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
        (12, 0xc120, 0, 0, 0x30)
    );
    let mut w = Work {
        cpu,
        clock,
        bus,
        cycles: 0,
    };
    w.module_dispatch(0);
    assert_eq!((w.cpu.bank, w.cpu.pc), (12, 0xc15d));
    w.jsl(0xc160, 2, 0x8000);
    w.load_immediate(0x80);
    w.store_abs(0x710, 0x80);
    w.jsl(0x8008, 0, 0x893d);
    // EnableForceBlank writes hardware immediately, then its WRAM mirrors.
    w.load_immediate(0x80);
    w.store_abs(0x2100, 0x80);
    w.store_dp(0x13, 0x80, 1);
    w.store_abs(0x420c, 0);
    w.store_dp(0x9b, 0, 1);
    w.rtl();
    assert_eq!((w.cpu.bank, w.cpu.pc), (2, 0x8009));
    w.load_immediate(0x10);
    w.store_dp(0x1c, 0x10, 1);
    w.store_dp(0x1d, 0, 1);
    w.jsr(0x8011, 0xc500);
    // InitializeBackgroundSettings includes SETINI and both CHR-base writes.
    w.store_abs(0x2133, 0);
    w.load_immediate(9);
    w.store_dp(0x94, 9, 1);
    w.store_dp(0x95, 0, 1);
    for (address, value) in [
        (0x2107, 0x13),
        (0x2108, 3),
        (0x2109, 0x63),
        (0x210b, 0x22),
        (0x210c, 7),
    ] {
        w.load_immediate(value);
        w.store_abs(address, value);
    }
    for (address, value) in [(0x9a, 0x20), (0x9c, 0x20), (0x9d, 0x40), (0x9e, 0x80)] {
        w.load_immediate(value);
        w.store_dp(address, u16::from(value), 1);
    }
    w.rts();
    assert_eq!(w.cpu.pc, 0x8012);
    w.load_immediate(0x20);
    w.store_dp(0x99, 0x20, 1);
    w.load_immediate(2);
    w.store_abs(0x2101, 2);
    w.load_immediate(20);
    w.store_abs(0xaaa, 20);
    w.jsl(0x8023, 0, 0xe3fa);
    w.cycles
}

/// Return from the initial graphics call through the remaining setup work.
/// The cold caller has no pending overworld music upload; that gate is still
/// read from its live WRAM owner before choosing the source branch.
pub(crate) fn finish_setup(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16,
) -> u64 {
    assert_eq!(
        (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
        (2, 0x8024, 0, 0, 0x30)
    );
    let mut w = Work {
        cpu,
        clock,
        bus,
        cycles: 0,
    };
    w.store_abs(0xaaa, 0);
    w.jsr(0x8029, 0x854c);
    w.fetch();
    w.drain(16);
    let music = w.read(0x136, 1);
    w.cpu.load_a_byte(music as u8);
    assert_eq!(
        music, 0,
        "cold intro setup cannot own a pending music upload"
    );
    w.fetch();
    w.drain(8);
    w.drain(6); // BEQ to the music callee's RTS
    w.rts();
    assert_eq!(w.cpu.pc, 0x802a);
    w.fetch();
    w.drain(8);
    w.drain(6);
    w.cpu.p &= !0x20; // REP #$20
    w.load_x_byte(0x80);
    w.store_abs(0x2115, w.cpu.x as u8);
    w.load_a_word(0x27f0);
    w.store_abs_word(0x2116, w.cpu.a);
    w.load_x_byte(0x20);
    w.load_a_word(0x7fff);
    loop {
        w.store_abs_word(0x2118, 0);
        w.fetch();
        w.drain(24);
        (w.bus)(
            w.clock.clock_master_cycles(),
            Access::WriteLongWord(0x7ec620 + u32::from(w.cpu.x), w.cpu.a),
        );
        w.drain(16);
        for _ in 0..2 {
            w.fetch();
            w.drain(6);
            w.cpu.x = u16::from((w.cpu.x as u8).wrapping_sub(1));
            w.flags(w.cpu.x, 1);
        }
        w.fetch();
        w.drain(8);
        if w.cpu.p & NEGATIVE != 0 {
            break;
        }
        w.drain(6);
    }
    w.load_a_word(0x1ffe);
    w.store_dp(0xc8, w.cpu.a, 2);
    w.load_a_word(0x1bfe);
    w.store_dp(0xca, w.cpu.a, 2);
    w.fetch();
    w.drain(8);
    w.drain(6);
    w.cpu.p |= 0x20; // SEP #$20
    w.rtl();
    w.cycles
}

/// First setup caller through the source logo stores and long dispatch to
/// the WRAM clear. The clear body has not executed when this owner returns.
pub(crate) fn enter_first_clear(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16,
) -> u64 {
    enter_continue(cpu, clock, bus, true, (12, 0xc1a0))
}

pub(crate) fn enter_following_clear(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16,
) -> u64 {
    enter_continue(cpu, clock, bus, false, (12, 0xc1a0))
}

pub(crate) fn enter_text_pointers(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16,
) -> u64 {
    enter_continue(cpu, clock, bus, false, (2, 0x8116))
}
pub(crate) fn enter_item_graphics(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16,
) -> u64 {
    enter_continue(cpu, clock, bus, false, (0, 0xd231))
}
pub(crate) fn enter_follower_graphics(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16,
) -> u64 {
    enter_continue(cpu, clock, bus, false, (0, 0xd423))
}
pub(crate) fn enter_poly(cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16) -> u64 {
    assert_eq!((cpu.bank,cpu.pc,cpu.db,cpu.d,cpu.p&0x38),(12,0xc120,0,0,0x30));
    let mut w=Work {cpu,clock,bus,cycles:0};
    w.module_dispatch(2);
    assert_eq!((w.cpu.bank,w.cpu.pc),(12,0xc33c));
    w.cycles
}

pub(crate) fn enter_animation(cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16, submodule: u8) -> u64 {
    assert!(matches!(submodule,3|4));
    assert_eq!((cpu.bank,cpu.pc,cpu.db,cpu.d,cpu.p&0x38),(12,0xc120,0,0,0x30));
    let mut w=Work {cpu,clock,bus,cycles:0};
    w.module_dispatch(submodule);
    assert_eq!((w.cpu.bank,w.cpu.pc),(12,0xc404));
    w.cycles
}

pub(crate) fn enter_title(cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16, submodule: u8) -> u64 {
    assert!(matches!(submodule,5..=8));
    assert_eq!((cpu.bank,cpu.pc,cpu.db,cpu.d,cpu.p&0x38),(12,0xc120,0,0,0x30));
    let mut w=Work {cpu,clock,bus,cycles:0};
    w.module_dispatch(submodule);
    assert_eq!(w.cpu.bank,12);
    assert!(matches!(w.cpu.pc,0xc25c|0xc2ae|0xc284|0xc2d4|0xc2f0));
    w.cycles
}

pub(crate) fn enter_darkening(cpu: &mut NativeNmiContext, clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16) -> u64 {
    enter_continue(cpu, clock, bus, false, (12, 0xc1f5))
}

fn enter_continue(
    cpu: &mut NativeNmiContext,
    clock: &mut CpuMasterTimeline,
    bus: impl FnMut(u64, Access) -> u16,
    first: bool,
    target: (u8, u16),
) -> u64 {
    assert_eq!(
        (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
        (12, if first { 0xc161 } else { 0xc120 }, 0, 0, 0x30)
    );
    let mut w = Work {
        cpu,
        clock,
        bus,
        cycles: 0,
    };
    if first {
        w.load_immediate(15);
        w.store_dp(0x13, w.cpu.a, 1);
        w.store_dp(0xb0, 0, 1);
        w.increment_dp_byte(0x15);
        w.increment_dp_byte(0x11);
        w.load_immediate(10);
        w.store_abs(0x12f, w.cpu.a as u8);
    } else {
        w.module_dispatch(1);
        assert_eq!((w.cpu.bank, w.cpu.pc), (12, 0xc170));
    }
    w.jsr(0xc172, 0xed82);
    // PHB, PHK, PLB: indexed logo operands use the routine's actual bank.
    for bank in [w.cpu.db, w.cpu.bank] {
        w.fetch();
        w.drain(6);
        w.push(u16::from(bank), 1);
    }
    w.pull_bank();
    w.fetch();
    w.drain(8);
    w.cpu.y = 3;
    w.flags(3, 1);
    w.load_x_byte(12);
    loop {
        w.load_immediate(2);
        w.store_indexed_byte(0xa20 + w.cpu.y, w.cpu.a as u8);
        w.load_logo_byte(0xed7a);
        w.store_indexed_byte(0x800 + w.cpu.x, w.cpu.a as u8);
        w.load_immediate(0x68);
        w.store_indexed_byte(0x801 + w.cpu.x, w.cpu.a as u8);
        w.load_logo_byte(0xed7e);
        w.store_indexed_byte(0x802 + w.cpu.x, w.cpu.a as u8);
        w.load_immediate(0x32);
        w.store_indexed_byte(0x803 + w.cpu.x, w.cpu.a as u8);
        for _ in 0..4 {
            w.fetch();
            w.drain(6);
            w.cpu.x = u16::from((w.cpu.x as u8).wrapping_sub(1));
            w.flags(w.cpu.x, 1);
        }
        w.fetch();
        w.drain(6);
        w.cpu.y = u16::from((w.cpu.y as u8).wrapping_sub(1));
        w.flags(w.cpu.y, 1);
        w.fetch();
        w.drain(8);
        if w.cpu.p & NEGATIVE != 0 {
            break;
        }
        w.drain(6);
    }
    w.pull_bank();
    w.rts();
    assert_eq!((w.cpu.bank, w.cpu.pc, w.cpu.db), (12, 0xc173, 0));
    w.load_dp(0xb0);
    assert!(
        if target == (2, 0x8116) {
            w.cpu.a as u8 == 8
        } else if target == (0, 0xd231) {
            w.cpu.a as u8 == 9
        } else if target == (0, 0xd423) {
            w.cpu.a as u8 == 10
        } else if target == (12, 0xc1f5) {
            w.cpu.a as u8 >= 11
        } else {
            w.cpu.a as u8 <= 7
        },
        "intro continuation does not own this selector"
    );
    w.increment_dp_byte(0xb0);
    w.fetch();
    w.drain(8); // CMP #$0B
    let value = w.cpu.a as u8;
    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(value >= 11);
    w.flags(u16::from(value.wrapping_sub(11)), 1);
    w.fetch();
    w.drain(8); // BCS
    if w.cpu.p & CARRY != 0 {
        w.drain(6);
        w.cpu.pc = 0xc1f5;
        assert_eq!((w.cpu.bank,w.cpu.pc),target);
        return w.cycles;
    }
    w.jsl(0xc17e, 0, 0x879c);
    w.jump_table_long();
    assert_eq!((w.cpu.bank, w.cpu.pc), target);
    w.cycles
}

struct Work<'a, B> {
    cpu: &'a mut NativeNmiContext,
    clock: &'a mut CpuMasterTimeline,
    bus: B,
    cycles: u64,
}
impl<B: FnMut(u64, Access) -> u16> Work<'_, B> {
    fn module_dispatch(&mut self, expected: u8) {
        self.load_dp(0x11);
        assert_eq!(self.cpu.a as u8, expected);
        self.fetch();
        self.drain(8);
        let compared = self.cpu.a as u8;
        self.cpu.p = (self.cpu.p & !CARRY) | u8::from(compared >= 8);
        self.flags(u16::from(compared.wrapping_sub(8)), 1);
        self.fetch();
        self.drain(8);
        if compared<8 {
            self.drain(6); // BCC $C133
        } else {
            self.load_dp(0xf6);
            self.fetch();self.drain(8);self.cpu.a&=0xffc0;self.flags(self.cpu.a&255,1);
            self.fetch();self.drain(8);let value=self.read(0xf4,1);self.cpu.a|=value;self.flags(self.cpu.a&255,1);
            self.fetch();self.drain(8);self.cpu.a&=0xffd0;self.flags(self.cpu.a&255,1);
            self.fetch();self.drain(8);
            if self.cpu.p&ZERO!=0 {self.drain(6);} else {
                self.fetch();self.drain(16);self.cpu.pc=0xc2f0;
                return;
            }
        }
        self.load_dp(0x11);
        self.jsl(0xc138, 0, 0x879c);
        self.jump_table_long();
    }
    fn jump_table_long(&mut self) {
        // JumpTableLong consumes its own real JSL frame. It preserves byte Y in
        // $05 and constructs the long table base from the bytes it actually pulls.
        self.store_dp(5, self.cpu.y, 1);
        self.fetch();
        self.drain(12);
        self.cpu.y = self.pop(1);
        self.flags(self.cpu.y, 1); // PLY
        self.store_dp(2, self.cpu.y, 1);
        self.fetch();
        self.drain(8);
        self.drain(6);
        self.cpu.p &= !0x30; // REP #$30
        self.fetch();
        self.drain(16);
        self.cpu.a &= 0xff;
        self.flags(self.cpu.a, 2);
        self.store_dp(3, self.cpu.a, 2);
        self.fetch();
        self.drain(6); // ASL A
        self.cpu.p = (self.cpu.p & !CARRY) | ((self.cpu.a >> 15) as u8);
        self.cpu.a <<= 1;
        self.flags(self.cpu.a, 2);
        self.fetch();
        self.drain(8);
        let operand = self.read(3, 2); // ADC $03
        let old = self.cpu.a;
        let sum = u32::from(old) + u32::from(operand) + u32::from(self.cpu.p & CARRY);
        self.cpu.a = sum as u16;
        self.cpu.p = (self.cpu.p & !(CARRY | 0x40))
            | u8::from(sum > 0xffff)
            | if (!(old ^ operand) & (old ^ self.cpu.a) & 0x8000) != 0 {
                0x40
            } else {
                0
            };
        self.flags(self.cpu.a, 2);
        self.fetch();
        self.drain(6);
        self.cpu.y = self.cpu.a; // TAY
        self.fetch();
        self.drain(12);
        self.cpu.a = self.pop(2);
        self.flags(self.cpu.a, 2); // PLA
        self.store_dp(3, self.cpu.a, 2);
        for destination in [0, 1] {
            self.fetch();
            self.drain(6);
            self.cpu.y = self.cpu.y.wrapping_add(1);
            self.flags(self.cpu.y, 2);
            self.fetch();
            self.drain(8); // LDA [$02],Y
            let low = self.read(2, 2);
            let bank = self.read(4, 1);
            let address =
                ((u32::from(bank) << 16) | u32::from(low)).wrapping_add(u32::from(self.cpu.y));
            self.cpu.a = self.read(address, 2);
            self.flags(self.cpu.a, 2);
            self.store_dp(destination, self.cpu.a, 2);
        }
        self.fetch();
        self.drain(8);
        self.drain(6);
        self.cpu.p |= 0x30;
        self.cpu.x &= 255;
        self.cpu.y &= 255;
        self.fetch();
        self.drain(8);
        self.cpu.y = self.read(5, 1);
        self.flags(self.cpu.y, 1);
        self.fetch();
        self.drain(16);
        self.cpu.pc = self.read(0, 2);
        self.cpu.bank = self.read(2, 1) as u8;
    }
    fn increment_dp_byte(&mut self, address: u16) {
        self.fetch();
        self.drain(8);
        let value = (self.read(u32::from(address), 1) as u8).wrapping_add(1);
        self.drain(6);
        self.flags(u16::from(value), 1);
        self.write(address, u16::from(value), 1);
    }
    fn store_indexed_byte(&mut self, address: u16, value: u8) {
        self.fetch();
        self.drain(16);
        self.drain(6);
        self.write(address, u16::from(value), 1);
    }
    fn load_logo_byte(&mut self, base: u16) {
        self.fetch();
        self.drain(16);
        if (base & 255) + self.cpu.y > 255 {
            self.drain(6);
        }
        let address = (u32::from(self.cpu.db) << 16) | u32::from(base.wrapping_add(self.cpu.y));
        let value = self.read(address, 1);
        self.cpu.load_a_byte(value as u8);
    }
    fn pull_bank(&mut self) {
        self.fetch();
        self.drain(12);
        self.cpu.db = self.pop(1) as u8;
        self.flags(u16::from(self.cpu.db), 1);
    }
    fn load_x_byte(&mut self, value: u8) {
        self.fetch();
        self.drain(8);
        self.cpu.x = u16::from(value);
        self.flags(self.cpu.x, 1);
    }
    fn load_a_word(&mut self, value: u16) {
        self.fetch();
        self.drain(16);
        self.cpu.a = value;
        self.flags(value, 2);
    }
    fn store_abs_word(&mut self, address: u16, value: u16) {
        self.fetch();
        self.drain(16);
        if (0x2100..=0x213e).contains(&address) {
            // PPU words are two physical byte transactions, with a bus drain
            // between them; WRAM words remain one direct transaction.
            self.write(address, value & 255, 1);
            self.write(address + 1, value >> 8, 1);
        } else {
            self.write(address, value, 2);
        }
    }
    fn fetch(&mut self) {
        self.cycles += 8;
        self.clock.advance_synchronous_pcbase_opcode_fetch(8);
    }
    fn drain(&mut self, cycles: u32) {
        self.cycles += u64::from(cycles);
        self.clock
            .advance_synchronous_after_semantics_with(cycles, |_, _| Ok::<u32, String>(0))
            .unwrap();
    }
    fn flags(&mut self, value: u16, width: u8) {
        self.cpu.p = (self.cpu.p & !(NEGATIVE | ZERO))
            | ((if width == 1 { value } else { value >> 8 }) as u8 & NEGATIVE)
            | if value == 0 { ZERO } else { 0 };
    }
    fn read(&mut self, address: u32, width: u8) -> u16 {
        let op = if width == 1 {
            Access::ReadByte(address)
        } else {
            Access::ReadWord(address)
        };
        let value = (self.bus)(self.clock.clock_master_cycles(), op);
        self.drain(8 * u32::from(width));
        value
    }
    fn write(&mut self, address: u16, value: u16, width: u8) {
        let op = if width == 1 {
            Access::WriteByte(address, value as u8)
        } else {
            Access::WriteWord(address, value)
        };
        (self.bus)(self.clock.clock_master_cycles(), op);
        self.drain(if matches!(address, 0x2100..=0x213f | 0x420c) {
            6
        } else {
            8 * u32::from(width)
        });
    }
    fn load_dp(&mut self, address: u16) {
        self.fetch();
        self.drain(8);
        let value = self.read(u32::from(address), 1);
        self.cpu.load_a_byte(value as u8);
    }
    fn store_dp(&mut self, address: u16, value: u16, width: u8) {
        self.fetch();
        self.drain(8);
        self.write(address, value, width);
    }
    fn store_abs(&mut self, address: u16, value: u8) {
        self.fetch();
        self.drain(16);
        self.write(address, u16::from(value), 1);
    }
    fn load_immediate(&mut self, value: u8) {
        self.fetch();
        self.drain(8);
        self.cpu.load_a_byte(value);
    }
    fn push(&mut self, value: u16, width: u8) {
        assert!((2..0x2000).contains(&self.cpu.sp));
        self.write(self.cpu.sp + 1 - u16::from(width), value, width);
        self.cpu.sp -= u16::from(width);
    }
    fn pop(&mut self, width: u8) -> u16 {
        assert!(self.cpu.sp + u16::from(width) < 0x2000);
        let value = self.read(u32::from(self.cpu.sp + 1), width);
        self.cpu.sp += u16::from(width);
        value
    }
    fn jsl(&mut self, returned: u16, bank: u8, pc: u16) {
        self.fetch();
        self.drain(24);
        self.drain(6);
        self.push(u16::from(self.cpu.bank), 1);
        self.push(returned, 2);
        self.cpu.bank = bank;
        self.cpu.pc = pc;
    }
    fn jsr(&mut self, returned: u16, pc: u16) {
        self.fetch();
        self.drain(16);
        self.drain(6);
        self.push(returned, 2);
        self.cpu.pc = pc;
    }
    fn rtl(&mut self) {
        self.fetch();
        self.drain(12);
        self.cpu.pc = self.pop(2).wrapping_add(1);
        self.cpu.bank = self.pop(1) as u8;
    }
    fn rts(&mut self) {
        self.fetch();
        self.drain(12);
        self.cpu.pc = self.pop(2).wrapping_add(1);
        self.drain(6);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, SourceCpuBusAccessKind as Kind};

    #[test]
    fn first_intro_setup_matches_rom_stack_bus_ppu_and_graphics_callee_entry() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(mut rom) = std::fs::read(path) else {
            return;
        };
        for address in 0x0c_c139..=0x0c_c13b {
            assert_eq!(
                table_byte(address),
                Some(rom[0x60000 + (address as usize & 0x7fff)])
            );
        }
        // Only the test reset prelude is replaced. The intro, dispatch and
        // both setup subroutines execute the original ROM instructions.
        rom[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
        rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        for seed in [0u8, 0x7f, 0xff] {
            for (v, h) in [(256, 690), (256, 510), (261, 1300)] {
                for odd in [false, true] {
                    let mut source = snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
                    let target = CpuMasterTimeline::at_raster(
                        u64::from(odd),
                        CpuRasterPosition::new(v, h),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::NON_INTERLACE_EVEN,
                    )
                    .clock_master_cycles();
                    while source.machine().timestamp().master_cycles() < target {
                        source.step().unwrap();
                    }
                    let mut ram = source.machine().snes().ram.clone();
                    for (i, b) in ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 8)) as u8 ^ seed;
                    }
                    ram[0x11] = 0;
                    let mut native = NativeNmiContext {
                        a: 0xab0c,
                        x: u16::from(seed),
                        y: u16::from(seed),
                        d: 0,
                        db: 0,
                        bank: 12,
                        pc: 0xc120,
                        sp: 0x1fc,
                        p: 0x30 | (seed & 0xc7),
                    };
                    let mut registers = source.machine().snes().cpu.clone();
                    registers.a = native.a;
                    registers.x = native.x;
                    registers.y = native.y;
                    registers.pc = native.pc;
                    registers.sp = native.sp;
                    registers.k = native.bank;
                    registers.db = 0;
                    registers.dp = 0;
                    registers.e = false;
                    registers.unpack_flags(native.p);
                    let sram = source.machine().snes().cart.ram.clone();
                    source
                        .rebase_private_timing_state_at(
                            source.machine().timestamp(),
                            &registers,
                            &ram,
                            &sram,
                        )
                        .unwrap();
                    let mut clock = source.machine().timeline().clone();
                    let mut ppu = source.machine().snes().ppu.clone();
                    let mut expected = Vec::new();
                    for step in 0..256 {
                        if (
                            source.machine().snes().cpu.k,
                            source.machine().snes().cpu.pc,
                        ) == (0, 0xe3fa)
                        {
                            break;
                        }
                        assert!(step < 255, "intro setup failed to reach graphics call");
                        for access in source.step().unwrap().accesses {
                            let address = access.address;
                            if address < 0x2000
                                || matches!(address, 0x2100..=0x213f | 0x420c | 0x0c_c139..=0x0c_c13b)
                            {
                                let op = match access.kind {
                                    Kind::Read { width: 1, .. } => Access::ReadByte(address),
                                    Kind::Read { width: 2, .. } => Access::ReadWord(address),
                                    Kind::Write { value, width: 1 } => {
                                        Access::WriteByte(address as u16, value as u8)
                                    }
                                    Kind::Write { value, width: 2 } => {
                                        Access::WriteWord(address as u16, value)
                                    }
                                    _ => panic!("unexpected intro transaction {access:?}"),
                                };
                                expected.push((access.timestamp.master_cycles(), op));
                            }
                        }
                    }
                    let mut actual = Vec::new();
                    let cycles = enter_half_slot(&mut native, &mut clock, |at, access| {
                        actual.push((at, access));
                        let read = |address: u32| {
                            table_byte(address).unwrap_or_else(|| ram[address as usize])
                        };
                        match access {
                            Access::WriteLongWord(..) => unreachable!(),
                            Access::ReadByte(address) => u16::from(read(address)),
                            Access::ReadWord(address) => {
                                u16::from_le_bytes([read(address), read(address + 1)])
                            }
                            Access::WriteByte(0x420c, value) => {
                                assert_eq!(value, 0);
                                0
                            }
                            Access::WriteByte(address @ 0x2100..=0x213f, value) => {
                                ppu.write(address as u8, value);
                                0
                            }
                            Access::WriteByte(address, value) => {
                                ram[usize::from(address)] = value;
                                0
                            }
                            Access::WriteWord(address, value) => {
                                ram[usize::from(address)..usize::from(address) + 2]
                                    .copy_from_slice(&value.to_le_bytes());
                                0
                            }
                        }
                    });
                    let label = format!("seed={seed} v={v} h={h} odd={odd}");
                    assert!(cycles > 0);
                    assert_eq!(actual, expected, "{label}");
                    assert_eq!(clock, *source.machine().timeline(), "{label}");
                    assert_eq!(ram, source.machine().snes().ram, "{label}");
                    assert_eq!(
                        bincode::serialize(&ppu).unwrap(),
                        bincode::serialize(&source.machine().snes().ppu).unwrap(),
                        "{label}"
                    );
                    let cpu = &source.machine().snes().cpu;
                    assert_eq!(
                        (
                            native.a,
                            native.x,
                            native.y,
                            native.sp,
                            native.pc,
                            native.bank,
                            native.p
                        ),
                        (cpu.a, cpu.x, cpu.y, cpu.sp, cpu.pc, cpu.k, cpu.pack_flags()),
                        "{label}"
                    );
                }
            }
        }
    }
    #[test]
    fn intro_setup_suffix_matches_rom_bus_ppu_palette_and_return() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(mut rom) = std::fs::read(path) else {
            return;
        };
        // Only the test reset prelude is replaced. The intro, dispatch and
        // both setup subroutines execute the original ROM instructions.
        rom[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
        rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        for seed in [0u8, 0x7f, 0xff] {
            for (v, h) in [(149, 606), (149, 510), (261, 1300)] {
                for odd in [false, true] {
                    let mut source = snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
                    let target = CpuMasterTimeline::at_raster(
                        u64::from(odd),
                        CpuRasterPosition::new(v, h),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::NON_INTERLACE_EVEN,
                    )
                    .clock_master_cycles();
                    while source.machine().timestamp().master_cycles() < target {
                        source.step().unwrap();
                    }
                    let mut ram = source.machine().snes().ram.clone();
                    for (i, b) in ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 8)) as u8 ^ seed;
                    }
                    ram[0x136] = 0;
                    ram[0x1fa..0x1fd].copy_from_slice(&[0x60, 0xc1, 12]);
                    let mut native = NativeNmiContext {
                        a: 0xab0c,
                        x: u16::from(seed),
                        y: u16::from(seed),
                        d: 0,
                        db: 0,
                        bank: 2,
                        pc: 0x8024,
                        sp: 0x1f9,
                        p: 0x30 | (seed & 0xc7),
                    };
                    let mut registers = source.machine().snes().cpu.clone();
                    registers.a = native.a;
                    registers.x = native.x;
                    registers.y = native.y;
                    registers.pc = native.pc;
                    registers.sp = native.sp;
                    registers.k = native.bank;
                    registers.db = 0;
                    registers.dp = 0;
                    registers.e = false;
                    registers.unpack_flags(native.p);
                    let sram = source.machine().snes().cart.ram.clone();
                    source
                        .rebase_private_timing_state_at(
                            source.machine().timestamp(),
                            &registers,
                            &ram,
                            &sram,
                        )
                        .unwrap();
                    let mut clock = source.machine().timeline().clone();
                    let mut ppu = source.machine().snes().ppu.clone();
                    let mut expected = Vec::new();
                    for step in 0..512 {
                        if (
                            source.machine().snes().cpu.k,
                            source.machine().snes().cpu.pc,
                        ) == (12, 0xc161)
                        {
                            break;
                        }
                        assert!(step < 511, "intro setup failed to return");
                        for access in source.step().unwrap().accesses {
                            let address = access.address;
                            if address < 0x2000
                                || matches!(address, 0x2100..=0x213f | 0x7ec620..=0x7ec641)
                            {
                                let op = match access.kind {
                                    Kind::Read { width: 1, .. } => Access::ReadByte(address),
                                    Kind::Read { width: 2, .. } => Access::ReadWord(address),
                                    Kind::Write { value, width: 1 } => {
                                        Access::WriteByte(address as u16, value as u8)
                                    }
                                    Kind::Write { value, width: 2 } if address >= 0x7e0000 => {
                                        Access::WriteLongWord(address, value)
                                    }
                                    Kind::Write { value, width: 2 } => {
                                        Access::WriteWord(address as u16, value)
                                    }
                                    _ => panic!("unexpected intro transaction {access:?}"),
                                };
                                expected.push((access.timestamp.master_cycles(), op));
                            }
                        }
                    }
                    let mut actual = Vec::new();
                    let cycles = finish_setup(&mut native, &mut clock, |at, access| {
                        actual.push((at, access));
                        let read = |address: u32| ram[address as usize];
                        match access {
                            Access::WriteLongWord(address, value) => {
                                let offset = address as usize - 0x7e0000;
                                ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
                                0
                            }
                            Access::ReadByte(address) => u16::from(read(address)),
                            Access::ReadWord(address) => {
                                u16::from_le_bytes([read(address), read(address + 1)])
                            }
                            Access::WriteByte(0x420c, value) => {
                                assert_eq!(value, 0);
                                0
                            }
                            Access::WriteByte(address @ 0x2100..=0x213f, value) => {
                                ppu.write(address as u8, value);
                                0
                            }
                            Access::WriteByte(address, value) => {
                                ram[usize::from(address)] = value;
                                0
                            }
                            Access::WriteWord(address, value) => {
                                ram[usize::from(address)..usize::from(address) + 2]
                                    .copy_from_slice(&value.to_le_bytes());
                                0
                            }
                        }
                    });
                    let label = format!("seed={seed} v={v} h={h} odd={odd}");
                    assert!(cycles > 0);
                    assert_eq!(actual, expected, "{label}");
                    assert_eq!(clock, *source.machine().timeline(), "{label}");
                    assert_eq!(ram, source.machine().snes().ram, "{label}");
                    assert_eq!(
                        bincode::serialize(&ppu).unwrap(),
                        bincode::serialize(&source.machine().snes().ppu).unwrap(),
                        "{label}"
                    );
                    let cpu = &source.machine().snes().cpu;
                    assert_eq!(
                        (
                            native.a,
                            native.x,
                            native.y,
                            native.sp,
                            native.pc,
                            native.bank,
                            native.p
                        ),
                        (cpu.a, cpu.x, cpu.y, cpu.sp, cpu.pc, cpu.k, cpu.pack_flags()),
                        "{label}"
                    );
                }
            }
        }
    }
    #[test]
    fn first_clear_entry_matches_rom_logo_bus_dispatch_and_live_registers() {
        use snes::{CartType, RomCpuTimingProbe, Snes, SourcePpuReadState};
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(rom) = std::fs::read(path) else { return };
        for address in (0x0cc139..=0x0cc13e)
            .chain(0x0cc17f..=0x0cc19f)
            .chain(0x0ced7a..=0x0ced81)
        {
            assert_eq!(
                table_byte(address),
                Some(rom[0x60000 + (address as usize & 0x7fff)])
            );
        }
        for (first, selector) in std::iter::once((true, 0)).chain((0..12).chain([25,255]).map(|s| (false, s))) {
            for seed in [0u8, 0x7f, 0xff] {
                for (v, h) in [(151, 750), (151, 510), (261, 1300)] {
                    for odd in [false, true] {
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        for (i, b) in machine.ram.iter_mut().enumerate() {
                            *b = (i ^ (i >> 8)) as u8 ^ seed;
                        }
                        machine.ram[0x11] = 1;
                        machine.ram[0xb0] = selector;
                        let mut native = NativeNmiContext {
                            a: 0xab00 | u16::from(seed),
                            x: u16::from(seed),
                            y: u16::from(seed),
                            d: 0,
                            db: 0,
                            bank: 12,
                            pc: if first { 0xc161 } else { 0xc120 },
                            sp: 0x1fc,
                            p: 0x30 | (seed & 0xc7),
                        };
                        machine.cpu.a = native.a;
                        machine.cpu.x = native.x;
                        machine.cpu.y = native.y;
                        machine.cpu.sp = native.sp;
                        machine.cpu.pc = native.pc;
                        machine.cpu.k = native.bank;
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
                        clock.begin_synchronous_timeline().unwrap();
                        let mut expected = Vec::new();
                        for step in 0..512 {
                            if (source.snes().cpu.k, source.snes().cpu.pc)
                                == if selector == 8 {
                                    (2, 0x8116)
                                } else if selector == 9 {
                                    (0, 0xd231)
                                } else if selector == 10 {
                                    (0, 0xd423)
                                } else if selector >= 11 {
                                    (12, 0xc1f5)
                                } else {
                                    (12, 0xc1a0)
                                }
                            {
                                break;
                            }
                            assert!(step < 511);
                            for access in source.step().unwrap().accesses {
                                let address = access.address;
                                let normalized =
                                    if address >> 16 < 0x40 && address & 0xffff < 0x2000 {
                                        address & 0xffff
                                    } else if table_byte(address).is_some() {
                                        address
                                    } else {
                                        continue;
                                    };
                                let (op, value) = match access.kind {
                                    Kind::Read { value, width: 1 } => {
                                        (Access::ReadByte(normalized), value)
                                    }
                                    Kind::Read { value, width: 2 } => {
                                        (Access::ReadWord(normalized), value)
                                    }
                                    Kind::Write { value, width: 1 } => {
                                        (Access::WriteByte(normalized as u16, value as u8), 0)
                                    }
                                    Kind::Write { value, width: 2 } => {
                                        (Access::WriteWord(normalized as u16, value), 0)
                                    }
                                    _ => panic!("unexpected intro caller access {access:?}"),
                                };
                                expected.push((access.timestamp.master_cycles(), op, value));
                            }
                        }
                        let mut actual = Vec::new();
                        let cycles = enter_continue(
                            &mut native,
                            &mut clock,
                            |at, op| {
                                let read =
                                    |a: u32| table_byte(a).unwrap_or_else(|| ram[a as usize]);
                                let value = match op {
                                    Access::ReadByte(a) => u16::from(read(a)),
                                    Access::ReadWord(a) => {
                                        u16::from_le_bytes([read(a), read(a + 1)])
                                    }
                                    Access::WriteByte(a, v) => {
                                        ram[usize::from(a)] = v;
                                        0
                                    }
                                    Access::WriteWord(a, v) => {
                                        ram[usize::from(a)..usize::from(a) + 2]
                                            .copy_from_slice(&v.to_le_bytes());
                                        0
                                    }
                                    Access::WriteLongWord(..) => unreachable!(),
                                };
                                actual.push((at, op, value));
                                value
                            },
                            first,
                            if selector == 8 {
                                (2, 0x8116)
                            } else if selector == 9 {
                                (0, 0xd231)
                            } else if selector == 10 {
                                (0, 0xd423)
                            } else if selector >= 11 {
                                (12, 0xc1f5)
                            } else {
                                (12, 0xc1a0)
                            },
                        );
                        let label = format!(
                            "first={first} selector={selector} seed={seed} v={v} h={h} odd={odd}"
                        );
                        assert!(cycles > 0);
                        assert_eq!(actual, expected, "{label}");
                        assert_eq!(ram, source.snes().ram, "{label}");
                        assert_eq!(clock, *source.timeline(), "{label}");
                        let cpu = &source.snes().cpu;
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
                            (
                                cpu.a,
                                cpu.x,
                                cpu.y,
                                cpu.sp,
                                cpu.pc,
                                cpu.k,
                                cpu.db,
                                cpu.dp,
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
