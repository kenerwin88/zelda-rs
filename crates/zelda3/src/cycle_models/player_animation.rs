//! Retained moving, pose, swimming and dash animation, including all three
//! far entries. Each state publication and table read uses the inherited CPU
//! and clock; completion follows the source PLB/RTL, never a frame shortcut.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

// Adjacent immutable bytes addressed by the byte-sized animation index past
// the nominal delay table. They are lookup data only, never executed/decoded.
// Keeping the complete indexed window preserves raw byte wrap and page-cross
// behavior without clamping the live animation step to a guessed valid range.
const DELAY_ADJACENT_DATA: [u8; 232] = [
    0x8b, 0x4b, 0xab, 0xa9, 0x04, 0x85, 0x26, 0x80, 0x03, 0x8b, 0x4b, 0xab, 0xa5, 0x5d, 0xc9, 0x04,
    0xd0, 0x03, 0x82, 0x48, 0x01, 0xa5, 0x26, 0xd0, 0x03, 0x82, 0xd6, 0x00, 0x85, 0x00, 0xad, 0x4a,
    0x03, 0xf0, 0x05, 0xad, 0x40, 0x03, 0x85, 0x00, 0xa5, 0x50, 0xd0, 0x3e, 0xa5, 0x6a, 0xf0, 0x15,
    0xa5, 0x6c, 0xf0, 0x06, 0x0a, 0x29, 0xfc, 0xa8, 0x80, 0x15, 0xa5, 0x2f, 0x4a, 0xaa, 0xa5, 0x00,
    0x3d, 0x71, 0xe6, 0xd0, 0x25, 0xa0, 0x04, 0xa5, 0x00, 0x29, 0x0c, 0xf0, 0x02, 0xa0, 0x00, 0xc0,
    0x04, 0xf0, 0x08, 0xa5, 0x00, 0x29, 0x04, 0xd0, 0x08, 0x80, 0x08, 0xa5, 0x00, 0x29, 0x01, 0xf0,
    0x02, 0xc8, 0xc8, 0x84, 0x2f, 0x80, 0x03, 0x8b, 0x4b, 0xab, 0xad, 0x72, 0x03, 0xf0, 0x03, 0x82,
    0x80, 0x01, 0xa5, 0x2f, 0x4a, 0xaa, 0xa5, 0x5e, 0xc9, 0x06, 0xd0, 0x07, 0x8a, 0x18, 0x69, 0x04,
    0xaa, 0x80, 0x10, 0xad, 0x4a, 0x03, 0xf0, 0x0b, 0xa5, 0xf0, 0x29, 0x0f, 0xf0, 0x60, 0x8a, 0x18,
    0x69, 0x04, 0xaa, 0xa5, 0x5d, 0xc9, 0x17, 0xd0, 0x03, 0x82, 0x95, 0x00, 0xa5, 0x11, 0xc9, 0x0e,
    0xf0, 0x26, 0xc9, 0x12, 0xf0, 0x04, 0xc9, 0x13, 0xd0, 0x04, 0xa2, 0x0c, 0x80, 0x1a, 0xad, 0x08,
    0x03, 0x29, 0x80, 0xd0, 0x13, 0xa5, 0x48, 0x29, 0x8d, 0xf0, 0x04, 0xa2, 0x0c, 0x80, 0x09, 0xad,
    0x51, 0x03, 0xd0, 0x04, 0xa5, 0x3c, 0xf0, 0x2c, 0xa5, 0x2e, 0xc9, 0x06, 0xb0, 0x20, 0xad, 0xf5,
    0x02, 0xc9, 0x02, 0xf0, 0x19, 0xbd, 0x75, 0xe6, 0x85, 0x00, 0xa5, 0x2d, 0x18, 0x69, 0x01, 0x85,
    0x2d, 0xc5, 0x00, 0x90, 0x0d, 0x64, 0x2d, 0xa5,
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    if (0x07e69d..0x07e785).contains(&address) {
        return Some(DELAY_ADJACENT_DATA[(address - 0x07e69d) as usize]);
    }
    crate::ZeldaState::native_player_animation_table_byte(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moving_animation_matches_source_all_entries_modes_and_byte_steps() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in (0x07e671..0x07e785).chain(0x07e842..0x07e888) {
            assert_eq!(
                table_byte(a),
                Some(rom[(a >> 16) as usize * 0x8000 + (a as usize & 0x7fff)]),
                "animation table {a:06x}"
            );
        }
        let mut instructions = 0;
        let mut visited = std::collections::BTreeSet::new();
        for entry in [0xe69d, 0xe6a6, 0xe704] {
            for mode in 0..16 {
                for case in 0..256 {
                    let mut machine = snes::Snes::new();
                    machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                    for (i, byte) in machine.ram.iter_mut().enumerate() {
                        *byte = (i as u8).wrapping_mul(19).wrapping_add(case as u8);
                    }
                    let ram = &mut machine.ram;
                    ram[0x26] = [0, 1, 2, 4, 8, 12, 15, 255][case % 8];
                    ram[0x5d] = match mode {
                        2 => 4,
                        3 => 23,
                        _ => 0,
                    };
                    ram[0x50] = if mode == 14 { 0x80 } else { 0 };
                    ram[0x6a] = (case & 1) as u8;
                    ram[0x6c] = [0, 1, 2, 3, 4, 255][case / 4 % 6];
                    ram[0x2f] = if mode == 15 {
                        case as u8
                    } else {
                        (case % 4 * 2) as u8
                    };
                    ram[0x2d] = [0, 1, 2, 3, 7, 15, 254, 255][case / 4 % 8];
                    ram[0x2e] = case as u8;
                    ram[0x5e] = if matches!(mode, 12 | 13) { 6 } else { 0 };
                    ram[0x34a] = u8::from(matches!(mode, 11 | 13));
                    ram[0x340] = [0, 1, 2, 4, 8, 12, 15, 255][case / 8 % 8];
                    ram[0x372] = u8::from(mode == 1 || mode == 15);
                    ram[0x374] = case as u8;
                    ram[0xf0] = [0, 1, 15, 0x80][case / 4 % 4];
                    ram[0x11] = match mode {
                        4 => 14,
                        5 => 18,
                        6 => 19,
                        _ => 15,
                    };
                    ram[0x308] = if mode == 7 { 0x80 } else { 0 };
                    ram[0x48] = if mode == 8 { case as u8 } else { 0 };
                    ram[0x351] = u8::from(mode == 9 || (mode == 1 && case & 32 != 0));
                    ram[0x3c] = if mode == 10 || mode == 1 {
                        (case / 8) as u8
                    } else {
                        0
                    };
                    ram[0x2f5] = [0, 1, 2, 255][case / 16 % 4];
                    ram[0x1fd..0x200].copy_from_slice(&[0x27, 0x93, 2]);
                    let mut cpu = NativeNmiContext {
                        a: 0xab80,
                        x: case as u16,
                        y: 0x47,
                        d: 0,
                        db: (mode & 3) as u8,
                        bank: 7,
                        pc: entry,
                        sp: 0x1fc,
                        p: 0x30 | (case as u8 & 0xc3),
                    };
                    let db = cpu.db;
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.k = cpu.bank;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.db = cpu.db;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let clock = CpuMasterTimeline::at_raster(
                        [0, 1, 30002, 30003][case % 4],
                        snes::CpuRasterPosition::new(224, 1340),
                        snes::CpuBusWorkload::default(),
                        snes::CpuFieldTiming::NON_INTERLACE_EVEN,
                    );
                    let mut source = snes::RomCpuTimingProbe::new(
                        machine,
                        clock,
                        snes::SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    let mut clock = source.timeline().clone();
                    let mut work = PlayerAnimation::new(entry);
                    for _ in 0..256 {
                        if work.complete() {
                            break;
                        }
                        visited.insert(work.pc);
                        let label = format!(
                            "entry={entry:04x} mode={mode} case={case} pc={:04x}",
                            work.pc
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
                        instructions += 1;
                    }
                    assert!(work.complete());
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (2, 0x9328, 0x1ff, db));
                    assert_eq!(ram, source.snes().ram);
                }
            }
        }
        for pc in [
            0xe6a2, 0xe6da, 0xe700, 0xe78b, 0xe791, 0xe7c8, 0xe7f4, 0xe840, 0xe8a1, 0xe8d1, 0xe8ea,
        ] {
            assert!(visited.contains(&pc), "uncovered animation path {pc:04x}");
        }
        eprintln!("12288 animation fixtures, {instructions} instruction boundaries, {} distinct instruction PCs",visited.len());
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlayerAnimation {
    pc: u16,
    complete: bool,
}
impl Default for PlayerAnimation {
    fn default() -> Self {
        Self::new(0xe6a6)
    }
}
impl PlayerAnimation {
    fn new(pc: u16) -> Self {
        assert!(matches!(pc, 0xe69d | 0xe6a6 | 0xe704));
        Self {
            pc,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (7, self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xe69d | 0xe69e | 0xe6a6 | 0xe6a7 | 0xe704 | 0xe705 => {
                w.drain(6);
                w.push(
                    u16::from(if matches!(pc, 0xe69d | 0xe6a6 | 0xe704) {
                        w.cpu.db
                    } else {
                        w.cpu.bank
                    }),
                    1,
                );
                pc + 1
            }
            0xe69f | 0xe6a8 | 0xe706 | 0xe78f | 0xe7cc | 0xe7f8 | 0xe840 | 0xe8ee => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xe790 | 0xe7cd | 0xe7f9 | 0xe841 | 0xe8ef => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0xe6a0 | 0xe78b | 0xe7c8 | 0xe7f4 | 0xe8cb | 0xe8ea => {
                w.immediate(
                    match pc {
                        0xe6a0 => 4,
                        0xe7c8 | 0xe8cb => 1,
                        _ => 0,
                    },
                    1,
                );
                pc + 2
            }
            0xe6a2 | 0xe6b9 | 0xe6c3 | 0xe775 | 0xe77c | 0xe782 | 0xe78d | 0xe7b2 | 0xe7b9
            | 0xe7bf | 0xe7ca | 0xe7de | 0xe7e5 | 0xe7eb | 0xe7f6 | 0xe8b5 | 0xe8bc | 0xe8c2
            | 0xe8d4 | 0xe8db | 0xe8e1 | 0xe8ec => {
                let a = match pc {
                    0xe6a2 => 0x26,
                    0xe77c | 0xe782 | 0xe7b9 | 0xe7bf | 0xe7e5 | 0xe7eb | 0xe8bc | 0xe8c2
                    | 0xe8db | 0xe8e1 => 0x2d,
                    0xe78d | 0xe7ca | 0xe7f6 | 0xe8ec => 0x2e,
                    _ => 0,
                };
                w.dp_store(
                    a,
                    if matches!(pc, 0xe782 | 0xe7bf | 0xe7eb | 0xe8c2 | 0xe8e1) {
                        0
                    } else {
                        w.cpu.a & 255
                    },
                    1,
                );
                pc + 2
            }
            0xe6a9 | 0xe6b2 | 0xe6c5 | 0xe6c9 | 0xe6cd | 0xe6d7 | 0xe6db | 0xe6e4 | 0xe6f0
            | 0xe6f8 | 0xe70f | 0xe713 | 0xe725 | 0xe730 | 0xe739 | 0xe752 | 0xe761 | 0xe765
            | 0xe777 | 0xe784 | 0xe791 | 0xe793 | 0xe7b4 | 0xe7c1 | 0xe7ce | 0xe7e0 | 0xe7ed
            | 0xe7ff | 0xe803 | 0xe807 | 0xe811 | 0xe8a2 | 0xe8b7 | 0xe8c4 | 0xe8d6 | 0xe8e3 => {
                let a = match pc {
                    0xe6a9 | 0xe730 => 0x5d,
                    0xe6b2 => 0x26,
                    0xe6c5 | 0xe7ff => 0x50,
                    0xe6c9 | 0xe803 => 0x6a,
                    0xe6cd | 0xe807 => 0x6c,
                    0xe6d7 | 0xe70f | 0xe811 => 0x2f,
                    0xe713 | 0xe793 => 0x5e,
                    0xe725 => 0xf0,
                    0xe739 => 0x11,
                    0xe752 => 0x48,
                    0xe761 | 0xe8a2 => 0x3c,
                    0xe777 | 0xe7b4 | 0xe7e0 | 0xe8b7 | 0xe8d6 => 0x2d,
                    0xe765 | 0xe784 | 0xe791 | 0xe7c1 | 0xe7ce | 0xe7ed | 0xe8c4 | 0xe8e3 => 0x2e,
                    _ => 0,
                };
                let v = w.dp_read(a, 1);
                if pc == 0xe791 {
                    w.cpu.x = v;
                    w.flags(v, 1);
                } else {
                    w.load(v, 1);
                }
                pc + 2
            }
            0xe6ab | 0xe715 | 0xe732 | 0xe73b | 0xe73f | 0xe743 | 0xe767 | 0xe76e | 0xe787
            | 0xe795 | 0xe7ab | 0xe7c4 | 0xe7d0 | 0xe7d7 | 0xe7f0 | 0xe8a4 | 0xe8c7 | 0xe8e6 => {
                let v = match pc {
                    0xe6ab | 0xe7d0 | 0xe7f0 => 4,
                    0xe715 | 0xe767 | 0xe787 | 0xe795 | 0xe8e6 => 6,
                    0xe732 => 23,
                    0xe73b => 14,
                    0xe73f => 18,
                    0xe743 => 19,
                    0xe7c4 | 0xe8a4 | 0xe8c7 => 9,
                    _ => 2,
                };
                w.compare_immediate(v, 1);
                pc + 2
            }
            0xe6af | 0xe6b6 | 0xe70c | 0xe736 => {
                w.operand_then_internal(2);
                match pc {
                    0xe6af => 0xe7fa,
                    0xe6b6 => 0xe78f,
                    0xe70c => 0xe88f,
                    _ => 0xe7ce,
                }
            }
            0xe6a4 | 0xe6ad | 0xe6b4 | 0xe6be | 0xe6c7 | 0xe6cb | 0xe6cf | 0xe6d5 | 0xe6e0
            | 0xe6e8 | 0xe6ee | 0xe6f4 | 0xe6f6 | 0xe6fc | 0xe702 | 0xe70a | 0xe717 | 0xe71e
            | 0xe723 | 0xe729 | 0xe734 | 0xe73d | 0xe741 | 0xe745 | 0xe749 | 0xe750 | 0xe756
            | 0xe75a | 0xe75f | 0xe763 | 0xe769 | 0xe770 | 0xe780 | 0xe789 | 0xe797 | 0xe7a1
            | 0xe7ad | 0xe7bd | 0xe7c6 | 0xe7d2 | 0xe7d9 | 0xe7e9 | 0xe7f2 | 0xe7fd | 0xe801
            | 0xe805 | 0xe809 | 0xe80f | 0xe81b | 0xe824 | 0xe82a | 0xe831 | 0xe833 | 0xe83a
            | 0xe894 | 0xe89c | 0xe89f | 0xe8a6 | 0xe8ab | 0xe8c0 | 0xe8c9 | 0xe8cd | 0xe8cf
            | 0xe8df | 0xe8e8 => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let (take, target) = match pc {
                    0xe6a4 => (true, 0xe6a9),
                    0xe6ad => (!z, 0xe6b2),
                    0xe6b4 => (!z, 0xe6b9),
                    0xe6be => (z, 0xe6c5),
                    0xe6c7 | 0xe6e0 => (!z, 0xe707),
                    0xe6cb => (z, 0xe6e2),
                    0xe6cf => (z, 0xe6d7),
                    0xe6d5 => (true, 0xe6ec),
                    0xe6e8 => (z, 0xe6ec),
                    0xe6ee => (z, 0xe6f8),
                    0xe6f4 => (!z, 0xe6fe),
                    0xe6f6 => (true, 0xe700),
                    0xe6fc => (z, 0xe700),
                    0xe702 => (true, 0xe707),
                    0xe70a => (z, 0xe70f),
                    0xe717 => (!z, 0xe720),
                    0xe71e => (true, 0xe730),
                    0xe723 => (z, 0xe730),
                    0xe729 => (z, 0xe78b),
                    0xe734 => (!z, 0xe739),
                    0xe73d => (z, 0xe765),
                    0xe741 => (z, 0xe747),
                    0xe745 => (!z, 0xe74b),
                    0xe749 | 0xe75a => (true, 0xe765),
                    0xe750 | 0xe75f => (!z, 0xe765),
                    0xe756 => (z, 0xe75c),
                    0xe763 => (z, 0xe791),
                    0xe769 => (c, 0xe78b),
                    0xe770 => (z, 0xe78b),
                    0xe780 => (!c, 0xe78f),
                    0xe789 => (!z, 0xe78d),
                    0xe797 => (!z, 0xe79e),
                    0xe7a1 => (z, 0xe7a8),
                    0xe7ad => (z, 0xe7f8),
                    0xe7bd => (!c, 0xe7cc),
                    0xe7c6 => (!z, 0xe7ca),
                    0xe7d2 => (c, 0xe7f4),
                    0xe7d9 => (z, 0xe7f4),
                    0xe7e9 => (!c, 0xe7f8),
                    0xe7f2 => (!z, 0xe7f6),
                    0xe7fd => (z, 0xe840),
                    0xe801 | 0xe81b => (!z, 0xe840),
                    0xe805 => (z, 0xe81d),
                    0xe809 => (z, 0xe811),
                    0xe80f => (true, 0xe828),
                    0xe824 => (z, 0xe828),
                    0xe82a => (z, 0xe835),
                    0xe831 => (!z, 0xe83c),
                    0xe833 => (true, 0xe83e),
                    0xe83a => (z, 0xe83e),
                    0xe894 => (z, 0xe8a2),
                    0xe89c => (!c, 0xe8a2),
                    0xe89f => (w.cpu.p & NEGATIVE == 0, 0xe896),
                    0xe8a6 => (c, 0xe8d1),
                    0xe8ab => (!z, 0xe8d1),
                    0xe8c0 => (!c, 0xe8cf),
                    0xe8c9 => (!z, 0xe8cd),
                    0xe8cd => (true, 0xe8ec),
                    0xe8cf => (true, 0xe8ee),
                    0xe8df => (!c, 0xe8ee),
                    _ => (!c, 0xe8ec),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0xe6bb | 0xe6c0 | 0xe707 | 0xe720 | 0xe74b | 0xe75c | 0xe76b | 0xe79e | 0xe7a8
            | 0xe7d4 | 0xe7fa | 0xe815 | 0xe81f | 0xe82c | 0xe835 | 0xe891 | 0xe896 | 0xe8a8 => {
                let a = match pc {
                    0xe6bb | 0xe720 | 0xe79e => 0x34a,
                    0xe6c0 | 0xe7fa | 0xe815 | 0xe81f | 0xe82c | 0xe835 => 0x340,
                    0xe707 => 0x372,
                    0xe74b => 0x308,
                    0xe75c | 0xe8a8 => 0x351,
                    0xe891 | 0xe896 => 0x374,
                    _ => 0x2f5,
                };
                w.drain(16);
                let v = w.read(w.absolute(a), 1);
                w.load(v, 1);
                pc + 3
            }
            0xe6d1 | 0xe80b | 0xe8ae | 0xe8af | 0xe8b0 => {
                w.shift_left(1);
                pc + 1
            }
            0xe6d2 | 0xe6e6 | 0xe6f2 | 0xe6fa | 0xe727 | 0xe74e | 0xe754 | 0xe80c | 0xe822
            | 0xe82f | 0xe838 => {
                w.mask(
                    match pc {
                        0xe6d2 | 0xe80c => 0xfc,
                        0xe6e6 | 0xe822 => 12,
                        0xe6f2 | 0xe82f => 4,
                        0xe727 => 15,
                        0xe74e => 0x80,
                        0xe754 => 0x8d,
                        _ => 1,
                    },
                    1,
                );
                pc + 2
            }
            0xe6d4 | 0xe80e | 0xe6da | 0xe712 | 0xe71d | 0xe72f | 0xe79d | 0xe7a7 | 0xe814
            | 0xe8b1 => {
                w.drain(6);
                let v = w.cpu.a & 255;
                if matches!(pc, 0xe6d4 | 0xe80e) {
                    w.cpu.y = v;
                } else {
                    w.cpu.x = v;
                }
                w.flags(v, 1);
                pc + 1
            }
            0xe6d9 | 0xe711 | 0xe813 => {
                w.drain(6);
                let v = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, 1);
                pc + 1
            }
            0xe6dd | 0xe772 | 0xe7af | 0xe7db | 0xe818 | 0xe899 | 0xe8b2 | 0xe8d1 => {
                let a = match pc {
                    0xe6dd | 0xe818 => 0xe671,
                    0xe772 | 0xe7db => 0xe675,
                    0xe7af => 0xe685,
                    0xe899 => 0xe881,
                    0xe8b2 => 0xe842,
                    _ => 0xe87a,
                };
                w.drain(16);
                if (a & 255) + w.cpu.x > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a) + u32::from(w.cpu.x), 1);
                if pc == 0xe899 {
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a & 255 >= v);
                    w.flags((w.cpu.a & 255).wrapping_sub(v) & 255, 1);
                } else {
                    w.load(
                        if matches!(pc, 0xe6dd | 0xe818) {
                            w.cpu.a & v
                        } else {
                            v
                        },
                        1,
                    );
                }
                pc + 3
            }
            0xe6e2 | 0xe6ea | 0xe81d | 0xe826 | 0xe747 | 0xe758 | 0xe88f => {
                w.drain(8);
                let v = match pc {
                    0xe6e2 | 0xe81d => 4,
                    0xe747 | 0xe758 => 12,
                    0xe88f => 6,
                    _ => 0,
                };
                if matches!(pc, 0xe747 | 0xe758 | 0xe88f) {
                    w.cpu.x = v;
                } else {
                    w.cpu.y = v;
                }
                w.flags(v, 1);
                pc + 2
            }
            0xe6ec | 0xe828 => {
                w.drain(8);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= 4);
                w.flags(w.cpu.y.wrapping_sub(4) & 255, 1);
                pc + 2
            }
            0xe6fe | 0xe6ff | 0xe83c | 0xe83d | 0xe89e | 0xe8a1 => {
                w.drain(6);
                let v = if matches!(pc, 0xe89e | 0xe8a1) {
                    &mut w.cpu.x
                } else {
                    &mut w.cpu.y
                };
                *v = if pc == 0xe89e {
                    v.wrapping_sub(1)
                } else {
                    v.wrapping_add(1)
                } & 255;
                let value = *v;
                w.flags(value, 1);
                pc + 1
            }
            0xe700 | 0xe83e => {
                w.dp_store(0x2f, w.cpu.y, 1);
                pc + 2
            }
            0xe719 | 0xe72b | 0xe799 | 0xe7a3 | 0xe8ad => {
                w.drain(6);
                w.load(w.cpu.x, 1);
                pc + 1
            }
            0xe71a | 0xe72c | 0xe779 | 0xe79a | 0xe7a4 | 0xe7b6 | 0xe7e2 | 0xe8b9 | 0xe8d8 => {
                w.clear_carry();
                pc + 1
            }
            0xe71b | 0xe72d | 0xe77a | 0xe79b | 0xe7a5 | 0xe7b7 | 0xe7e3 | 0xe8ba | 0xe8d9 => {
                w.drain(8);
                w.adc_byte(match pc {
                    0xe71b | 0xe72d => 4,
                    0xe79b | 0xe7a5 => 8,
                    _ => 1,
                });
                pc + 2
            }
            0xe77e | 0xe7bb | 0xe7e7 | 0xe8be | 0xe8dd => {
                let v = w.dp_read(0, 1);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a & 255 >= v);
                w.flags((w.cpu.a & 255).wrapping_sub(v) & 255, 1);
                pc + 2
            }
            0xe786 | 0xe7c3 | 0xe7ef | 0xe8c6 | 0xe8e5 => {
                w.increment_a(1);
                pc + 1
            }
            _ => unreachable!("player animation {pc:04x}"),
        };
        if !self.complete {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}
