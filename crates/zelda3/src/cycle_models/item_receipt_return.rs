//! Bank-09 receipt return after animated graphics. Every operation retires on
//! the caller's live bus and stack, including sound panning and XY placement.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ItemReceiptReturnWork {
    next: Option<(u8, u16)>,
}

impl Default for ItemReceiptReturnWork {
    fn default() -> Self {
        Self {
            next: Some((9, 0x87a4)),
        }
    }
}

impl ItemReceiptReturnWork {
    pub(crate) fn complete(&self) -> bool {
        self.next.is_none()
    }

    pub(crate) fn routine(&self) -> u32 {
        match self.next {
            Some((0x0d, _)) => 0x0dbb67,
            Some((9, 0xad06..=0xad1a)) => 0x09ad06,
            _ => 0x0987a4,
        }
    }

    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        let (bank, pc) = self.next?;
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (bank, pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut next_bank = bank;
        let mut next;
        if bank == 0x0d {
            next = match pc {
                0xbb67 | 0xbb6a => {
                    let value = w.dp_read(if pc == 0xbb67 { 0x23 } else { 0x22 }, 1);
                    w.load(value, 1);
                    pc + 2
                }
                0xbb69 => {
                    w.swap_bytes();
                    0xbb6a
                }
                0xbb6c => {
                    w.branch(true);
                    0xbba8
                }
                0xbba8 => {
                    w.width(0x20, true);
                    0xbbaa
                }
                0xbbaa => {
                    w.drain(6);
                    w.push(w.cpu.x, 1);
                    0xbbab
                }
                0xbbab => {
                    w.item_x(0, 1);
                    0xbbad
                }
                0xbbad | 0xbbb0 => {
                    w.drain(6);
                    w.cpu.p |= CARRY;
                    pc + 1
                }
                0xbbae => {
                    let rhs = w.dp_read(0xe2, 2);
                    w.sbc(rhs, 2);
                    0xbbb0
                }
                0xbbb1 => {
                    w.drain(16);
                    w.sbc(0x50, 2);
                    0xbbb4
                }
                0xbbb4 | 0xbbba => {
                    w.compare_immediate(if pc == 0xbbb4 { 0x50 } else { 0 }, 2);
                    pc + 3
                }
                0xbbb7 | 0xbbbd => {
                    let taken = if pc == 0xbbb7 {
                        w.cpu.p & CARRY == 0
                    } else {
                        w.cpu.p & NEGATIVE != 0
                    };
                    if w.branch(taken) {
                        0xbbc0
                    } else {
                        pc + 2
                    }
                }
                0xbbb9 | 0xbbbf => {
                    w.drain(6);
                    w.cpu.x = (w.cpu.x + 1) & 255;
                    w.flags(w.cpu.x, 1);
                    pc + 1
                }
                0xbbc0 => {
                    w.width(0x20, false);
                    0xbbc2
                }
                0xbbc2 => {
                    w.drain(24);
                    let value = w.read(0x0dbb5b + u32::from(w.cpu.x), 1);
                    w.load(value, 1);
                    0xbbc6
                }
                0xbbc6 => {
                    w.drain(12);
                    w.cpu.x = w.pull(1);
                    w.flags(w.cpu.x, 1);
                    0xbbc7
                }
                0xbbc7 => {
                    w.rtl();
                    next_bank = w.cpu.bank;
                    w.cpu.pc
                }
                _ => unreachable!("native receipt pan phase {pc:04x}"),
            };
        } else {
            next = match pc {
                0x87a4 => {
                    let value = w.dp_read(0x72, 1);
                    w.load(value, 1);
                    0x87a6
                }
                0x87a6 | 0x87aa | 0x880b | 0x880f | 0x8813 | 0x8817 | 0x881b | 0x882d | 0x8884
                | 0x8896 | 0x889a | 0x889e | 0x88a2 | 0x88a6 | 0x88aa => {
                    let value = match pc {
                        0x87a6 => 6,
                        0x87aa => 0x18,
                        0x880b | 0x889e => 0x20,
                        0x880f | 0x88a2 => 0x37,
                        0x8813 | 0x88a6 => 0x38,
                        0x8817 | 0x88aa => 0x39,
                        0x881b => 0x26,
                        0x8896 => 0x3e,
                        0x889a => 0x17,
                        _ => 1,
                    };
                    w.compare_immediate(value, 1);
                    pc + 2
                }
                0x87a8 | 0x87ac => {
                    let taken = if pc == 0x87a8 {
                        w.cpu.p & ZERO != 0
                    } else {
                        w.cpu.p & ZERO == 0
                    };
                    next = if w.branch(taken) {
                        if pc == 0x87a8 {
                            0x87ae
                        } else {
                            0x87bb
                        }
                    } else {
                        pc + 2
                    };
                    assert_ne!(next, 0x87ae, "sword graphics retain their separate caller");
                    next
                }
                0x87bb => {
                    w.drain(12);
                    w.cpu.x = w.pull(1);
                    w.flags(w.cpu.x, 1);
                    0x87bc
                }
                0x87bc | 0x87cf | 0x87fe | 0x882a => {
                    w.drain(16);
                    let value = w.read(w.absolute(if pc == 0x87bc { 0x02d8 } else { 0x02e9 }), 1);
                    w.load(value, 1);
                    pc + 3
                }
                0x87bf | 0x87f3 | 0x87f8 | 0x8801 | 0x8826 | 0xad08 | 0xad0d | 0xad12 | 0xad17 => {
                    let address = match pc {
                        0x87bf => 0x0c5e,
                        0x87f3 => 0x039f,
                        0x87f8 => 0x0bf0,
                        0x8801 => 0x0c54,
                        0x8826 => 0x03b1,
                        0xad08 => 0x0bfa,
                        0xad0d => 0x0c0e,
                        0xad12 => 0x0c04,
                        _ => 0x0c18,
                    };
                    w.operand_then_internal(2);
                    w.write(w.absolute(address) + u32::from(w.cpu.x), w.cpu.a & 255, 1);
                    pc + 3
                }
                0x87c2 => {
                    w.drain(6);
                    w.cpu.y = w.cpu.a & 255;
                    w.flags(w.cpu.y, 1);
                    0x87c3
                }
                0x87c3 => {
                    w.operand_then_internal(2);
                    w.write(w.absolute(0x03a4) + u32::from(w.cpu.x), 0, 1);
                    0x87c6
                }
                0x87c6 | 0x87f6 | 0x88c2 | 0x88d8 | 0x88ed => {
                    w.immediate(
                        match pc {
                            0x87c6 => 9,
                            0x87f6 => 5,
                            0x88c2 => 10,
                            0x88d8 => 6,
                            _ => 0xf8,
                        },
                        1,
                    );
                    pc + 2
                }
                0x87c8 | 0x88d0 | 0x88df | 0x88e9 => {
                    w.drain(8);
                    let rhs = match pc {
                        0x87c8 => 1,
                        0x88d0 => 0x20,
                        0x88df => 3,
                        _ => 2,
                    };
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= rhs);
                    w.flags(w.cpu.y.wrapping_sub(rhs) & 255, 1);
                    pc + 2
                }
                0x87ca => {
                    assert!(
                        w.cpu.p & ZERO == 0,
                        "master sword cutscene is outside animated-item admission"
                    );
                    w.branch(true);
                    0x87f3
                }
                0x87fb | 0x887b => {
                    w.drain(6);
                    w.push(w.cpu.y, 1);
                    pc + 1
                }
                0x87fc | 0x8806 | 0x881f | 0x8823 | 0x88e3 => {
                    w.item_y(
                        match pc {
                            0x87fc => 0x60,
                            0x8806 => 0x38,
                            0x881f => 2,
                            0x8823 => 0x68,
                            _ => 0,
                        },
                        1,
                    );
                    pc + 2
                }
                0x8804 | 0x880d | 0x8811 | 0x8815 | 0x8819 | 0x881d | 0x882f | 0x887f | 0x8886
                | 0x8898 | 0x889c | 0x88a0 | 0x88a4 | 0x88a8 | 0x88ac | 0x88ce | 0x88d2
                | 0x88e1 | 0x88eb | 0x8901 | 0x891a => {
                    let unequal = matches!(
                        pc,
                        0x881d | 0x882f | 0x887f | 0x8886 | 0x88ac | 0x88d2 | 0x88e1 | 0x88eb
                    );
                    let taken = (w.cpu.p & ZERO == 0) == unequal;
                    let target = match pc {
                        0x8804 => 0x8808,
                        0x880d | 0x8811 | 0x8815 | 0x8819 => 0x8823,
                        0x881d => 0x8825,
                        0x882f => 0x887b,
                        0x887f | 0x8886 => 0x8893,
                        0x8898 | 0x889c => 0x88c2,
                        0x88a0 | 0x88a4 | 0x88a8 => 0x88ae,
                        0x88ac => 0x88b9,
                        0x88ce => 0x88dc,
                        0x88d2 => 0x88d8,
                        0x88e1 => 0x88e5,
                        0x88eb => 0x88f3,
                        0x8901 => 0x890e,
                        _ => 0x8924,
                    };
                    if w.branch(taken) {
                        target
                    } else {
                        pc + 2
                    }
                }
                0x8821 | 0x8891 | 0x88b7 | 0x88d6 => {
                    w.branch(true);
                    if pc == 0x8821 {
                        0x8825
                    } else if pc == 0x88d6 {
                        0x88dc
                    } else {
                        0x88c2
                    }
                }
                0x8825 => {
                    w.drain(6);
                    w.load(w.cpu.y, 1);
                    0x8826
                }
                0x8829 | 0x88f7 => {
                    w.drain(12);
                    w.cpu.y = w.pull(1);
                    w.flags(w.cpu.y, 1);
                    pc + 1
                }
                0x8831 | 0x8834 => {
                    w.drain(12);
                    let value = w.pull(1);
                    w.load(value, 1);
                    pc + 1
                }
                0x8832 | 0x8835 | 0x8842 | 0x884b | 0x8855 | 0x885d | 0x886d | 0x88c4 | 0x88da
                | 0x88ef | 0x88fd | 0x890c | 0x8916 | 0x8922 | 0x8929 => {
                    let (address, width) = match pc {
                        0x8832 => (0x73, 1),
                        0x8835 => (0x72, 1),
                        0x8842 | 0x886d | 0x88fd | 0x890c | 0x8916 => (0, 2),
                        0x884b | 0x8922 | 0x8929 => (2, 2),
                        0x8855 => (1, 1),
                        0x885d => (3, 1),
                        0x88ef => (8, 1),
                        _ => (2, 1),
                    };
                    let value = if width == 1 { w.cpu.a & 255 } else { w.cpu.a };
                    w.dp_store(address, value, width);
                    pc + 2
                }
                0x8837 | 0x885f | 0x88f8 => {
                    w.width(0x20, true);
                    pc + 2
                }
                0x884d | 0x892b => {
                    w.width(0x20, false);
                    pc + 2
                }
                0x8839 | 0x8844 | 0x884f | 0x8857 | 0x88ff | 0x890e | 0x8918 | 0x8924 | 0xad06
                | 0xad0b | 0xad10 | 0xad15 => {
                    let (address, width) = match pc {
                        0x8839 | 0x8844 => (0x72, 2),
                        0x884f | 0xad0b => (1, 1),
                        0x8857 | 0xad15 => (3, 1),
                        0x88ff | 0x8918 => (4, 2),
                        0x890e => (0, 2),
                        0x8924 => (2, 2),
                        0xad10 => (2, 1),
                        _ => (0, 1),
                    };
                    let value = w.dp_read(address, width);
                    w.load(value, width);
                    pc + 2
                }
                0x883b | 0x8846 | 0x8864 | 0x8872 | 0x8906 | 0x891f => {
                    w.mask(
                        if pc == 0x883b {
                            0x1f80
                        } else if pc == 0x8846 {
                            0x7e
                        } else {
                            255
                        },
                        2,
                    );
                    pc + 3
                }
                0x883e..=0x8841 => {
                    w.drain(6);
                    let old = w.cpu.a;
                    w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                    w.load(old >> 1, 2);
                    pc + 1
                }
                0x8849 | 0x884a => {
                    w.shift_left(2);
                    pc + 1
                }
                0x8851 | 0x8859 | 0x886a | 0x8875 | 0x8910 | 0x8913 | 0x8926 => {
                    w.clear_carry();
                    pc + 1
                }
                0x8852 | 0x885a => {
                    w.drain(16);
                    let rhs = w.read(w.absolute(if pc == 0x8852 { 0x062f } else { 0x062d }), 1);
                    w.adc_byte(rhs);
                    pc + 3
                }
                0x8861 | 0x886f | 0x88cb | 0x8903 | 0x891c => {
                    let (base, width) = match pc {
                        0x8861 | 0x8903 => (0x836c, 2),
                        0x88cb => (0x8450, 1),
                        _ => (0x83b8, 2),
                    };
                    w.drain(16);
                    if (base & 255) + w.cpu.y > 255 {
                        w.drain(6);
                    }
                    let value = w.read(w.absolute(base) + u32::from(w.cpu.y), width);
                    w.load(value, width);
                    pc + 3
                }
                0x8867 | 0x8909 | 0x888c | 0x88b2 | 0x88bd => {
                    let width = if pc == 0x8867 || pc == 0x8909 { 2 } else { 1 };
                    w.drain(8 * u32::from(width));
                    let rhs = match pc {
                        0x888c => 0x2c,
                        0x88b2 => 0x13,
                        0x88bd => 0x0f,
                        _ => 0xff00,
                    };
                    w.load(
                        if width == 1 {
                            (w.cpu.a | rhs) & 255
                        } else {
                            w.cpu.a | rhs
                        },
                        width,
                    );
                    pc + 1 + u16::from(width)
                }
                0x886b | 0x8876 | 0x8911 | 0x8914 | 0x8927 => {
                    let rhs = w.dp_read(
                        match pc {
                            0x886b => 0,
                            0x8876 => 2,
                            0x8911 => 0x20,
                            0x8914 => 8,
                            _ => 0x22,
                        },
                        2,
                    );
                    w.adc(rhs);
                    pc + 2
                }
                0x8878 | 0x892d => {
                    w.operand_then_internal(2);
                    if pc == 0x8878 {
                        0x8929
                    } else {
                        0x80c3
                    }
                }
                0x887c | 0x8881 | 0x8893 | 0x8808 | 0x88c8 => {
                    let address = if pc == 0x887c { 0x0c54 } else { 0x0c5e };
                    w.drain(16);
                    if (address & 255) + w.cpu.x > 255 {
                        w.drain(6);
                    }
                    let value = w.read(w.absolute(address) + u32::from(w.cpu.x), 1);
                    if pc == 0x88c8 {
                        w.cpu.y = value;
                        w.flags(value, 1);
                    } else {
                        w.load(value, 1);
                    }
                    pc + 3
                }
                0x8888 | 0x88ae | 0x88b9 => {
                    w.operand_then_internal(3);
                    w.push(u16::from(w.cpu.bank), 1);
                    w.push(pc + 3, 2);
                    next_bank = 0x0d;
                    0xbb67
                }
                0x888e | 0x88b4 | 0x88bf => {
                    w.drain(16);
                    w.write(
                        w.absolute(match pc {
                            0x888e => 0x012e,
                            0x88b4 => 0x012c,
                            _ => 0x012f,
                        }),
                        w.cpu.a & 255,
                        1,
                    );
                    pc + 3
                }
                0x88c6 | 0x88d4 | 0x88e5 | 0x88e7 | 0x88f5 => {
                    w.dp_store(
                        match pc {
                            0x88c6 => 3,
                            0x88d4 => 2,
                            0x88e5 => 8,
                            0x88e7 => 9,
                            _ => 5,
                        },
                        0,
                        1,
                    );
                    pc + 2
                }
                0x88dc => {
                    w.drain(16);
                    w.cpu.y = w.read(w.absolute(0x02e9), 1);
                    w.flags(w.cpu.y, 1);
                    0x88df
                }
                0x88f1 => {
                    w.dp_increment(9, 1, false);
                    0x88f3
                }
                0x88f3 => {
                    w.dp_store(4, w.cpu.y, 1);
                    0x88f5
                }
                0x88fa => {
                    w.immediate(0xfff2, 2);
                    0x88fd
                }
                0x80c3 => {
                    w.item_call();
                    0xad06
                }
                0xad1a => {
                    w.rts();
                    w.cpu.pc
                }
                0x80c6 => {
                    w.drain(12);
                    w.cpu.db = w.pull(1) as u8;
                    w.flags(u16::from(w.cpu.db), 1);
                    0x80c7
                }
                0x80c7 => {
                    w.rtl();
                    self.next = None;
                    return Some(w.cycles);
                }
                _ => unreachable!("native receipt return phase {pc:04x}"),
            };
        }
        w.cpu.bank = next_bank;
        w.cpu.pc = next;
        self.next = Some((next_bank, next));
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };

    #[test]
    fn receipt_return_matches_source_instruction_bus_and_live_stack() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for address in (0x09836c..0x09849c).chain(0x0dbb5b..0x0dbb5e) {
            assert_eq!(
                crate::zelda_rtl::native_item_receipt_table_byte(address),
                Some(rom[(address as usize >> 16) * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        let mut cases = 0;
        let mut instructions = 0;
        for item in 0..76u8 {
            let gfx = crate::zelda_rtl::native_item_receipt_table_byte(0x098404 + u32::from(item))
                .unwrap();
            if matches!(gfx, 6 | 0x18) {
                continue;
            }
            for method in 0..4u8 {
                for variant in 0..6 {
                    let odd = variant & 1 != 0;
                    let slot = (item + method) % 5;
                    let sp = 0x1e4;
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, byte) in machine.ram.iter_mut().enumerate() {
                        *byte = (i ^ (i >> 7) ^ usize::from(item)) as u8;
                    }
                    machine.ram[0x72] = gfx;
                    machine.ram[0x2d8] = item;
                    machine.ram[0x2e9] = method;
                    let link_x = [0u16, 0x60, 0xff][variant / 2];
                    machine.ram[0x22..0x24].copy_from_slice(&link_x.to_le_bytes());
                    machine.ram[0xe2..0xe4].fill(0);
                    let mut stack = vec![slot];
                    if method == 1 {
                        stack.extend([0x1b, 0x70]);
                    }
                    stack.extend([7, 0x0a, 0x9a, 7]);
                    machine.ram[sp + 1..sp + 1 + stack.len()].copy_from_slice(&stack);
                    let mut cpu = NativeNmiContext {
                        a: 0x4840,
                        x: 0xc0,
                        y: 255,
                        d: 0,
                        db: 9,
                        bank: 9,
                        pc: 0x87a4,
                        sp: sp as u16,
                        p: 0x30,
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.db = cpu.db;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let mut clock = CpuMasterTimeline::at_raster(
                        4585,
                        CpuRasterPosition::new(
                            if odd { 261 } else { 245 },
                            if odd { 1350 } else { 394 },
                        ),
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
                    let mut work = ItemReceiptReturnWork::default();
                    let mut count = 0;
                    while !work.complete() {
                        let label = format!(
                            "item={item:02x} method={method} odd={odd} pc={:06x}",
                            (u32::from(cpu.bank) << 16) | u32::from(cpu.pc)
                        );
                        super::super::native_work::compare_instruction(
                            &label,
                            &mut cpu,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        work = work.clone();
                        count += 1;
                        assert!(count < 200);
                    }
                    assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.db),
                        (7, 0x9a0b, (sp + stack.len()) as u16, 7)
                    );
                    assert_eq!(ram, source.snes().ram);
                    cases += 1;
                    instructions += count;
                }
            }
        }
        println!("receipt return source comparison: {cases} cases, {instructions} instructions");
    }
}
