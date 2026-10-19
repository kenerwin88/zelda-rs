//! Diagonal collision room relocation and direction setup on the caller's live clock.
//! Tile probes remain explicit child calls; zero-direction paths return natively.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiagonalCollisionTail {
    VerticalProbe { primary_layer: bool, direction: u8 },
    HorizontalProbe { primary_layer: bool, direction: u8 },
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiagonalCollisionPrefix {
    pc: u16,
    primary_layer: bool,
    tail: Option<DiagonalCollisionTail>,
}
impl Default for DiagonalCollisionPrefix {
    fn default() -> Self {
        Self {
            pc: 0xb64f,
            primary_layer: false,
            tail: None,
        }
    }
}
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    [7, 11, 13, 14]
        .get(address.checked_sub(0x07b64b)? as usize)
        .copied()
}
impl DiagonalCollisionPrefix {
    pub(crate) fn after_vertical_probe(primary_layer: bool) -> Self {
        Self {
            pc: 0xb682,
            primary_layer,
            tail: None,
        }
    }
    pub(crate) fn resume_after_vertical_probe(&mut self) {
        assert!(matches!(
            self.tail,
            Some(DiagonalCollisionTail::VerticalProbe { .. })
        ));
        *self = Self::after_vertical_probe(self.primary_layer);
    }

    pub(crate) fn tail(&self) -> Option<DiagonalCollisionTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() {
            return None;
        }
        let wide = (0xb98f..=0xb9a9).contains(&self.pc) || (0xb9bc..=0xb9e0).contains(&self.pc);
        let width = if wide { 2 } else { 1 };
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (7, self.pc, 0, if wide { 0x10 } else { 0x30 })
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xb64f | 0xb654 | 0xb657 | 0xb67f => {
                w.item_call();
                if self.pc == 0xb654 {
                    self.primary_layer = true;
                }
                if self.pc == 0xb67f {
                    self.tail = Some(DiagonalCollisionTail::VerticalProbe {
                        primary_layer: self.primary_layer,
                        direction: (w.cpu.a & 255) as u8,
                    });
                }
                match self.pc {
                    0xb64f => 0xb97c,
                    0xb654 => 0xb660,
                    0xb657 => 0xb9b3,
                    0xb67f => 0xce85,
                    _ => unreachable!(),
                }
            }
            0xb65a | 0xb668 | 0xb7a0 | 0xb7a6 | 0xb7ae | 0xb9e2 | 0xb9e6 | 0xb9ed | 0xb98f
            | 0xb99c | 0xb9bc | 0xb9c4 | 0xb9cc | 0xb9d6 => {
                if self.pc == 0xb65a {
                    self.primary_layer = false;
                }
                let value = w.dp_read(
                    match self.pc {
                        0xb65a => 0x67,
                        0xb668 => 0x67,
                        0xb7a0 => 0x67,
                        0xb7a6 => 0x6b,
                        0xb7ae => 0x6a,
                        0xb9e2 => 0x67,
                        0xb9e6 => 0x30,
                        0xb9ed => 0x31,
                        0xb98f => 0xe6,
                        0xb99c => 0xe0,
                        0xb9bc => 0x20,
                        0xb9c4 => 0x22,
                        0xb9cc => 0xe8,
                        0xb9d6 => 0xe2,
                        _ => unreachable!(),
                    },
                    width,
                );
                w.load(value, width);
                self.pc + 2
            }
            0xb97c | 0xb9b3 => {
                w.drain(16);
                let value = w.read(w.absolute(0x46c), 1);
                w.load(value, 1);
                self.pc + 3
            }
            0xb65e | 0xb662 | 0xb664 | 0xb67d | 0xb7ac | 0xb997 | 0xb9a4 | 0xb9ad | 0xb9c2
            | 0xb9ca | 0xb9d4 | 0xb9de | 0xb9eb | 0xb9f2 => {
                w.dp_store(
                    match self.pc {
                        0xb65e => 0x67,
                        0xb662 => 0x42,
                        0xb664 => 0x43,
                        0xb67d => 0x66,
                        0xb7ac => 0x67,
                        0xb997 => 0x20,
                        0xb9a4 => 0x22,
                        0xb9ad => 0xee,
                        0xb9c2 => 0x0,
                        0xb9ca => 0x2,
                        0xb9d4 => 0x20,
                        0xb9de => 0x22,
                        0xb9eb => 0x30,
                        0xb9f2 => 0x31,
                        _ => unreachable!(),
                    },
                    w.cpu.a & if width == 1 { 255 } else { 65535 },
                    width,
                );
                self.pc + 2
            }
            0xb999 | 0xb9a6 => {
                w.drain(16);
                w.write(
                    w.absolute(if self.pc == 0xb999 { 0x318 } else { 0x31a }),
                    w.cpu.a,
                    2,
                );
                self.pc + 3
            }
            0xb666 | 0xb7b0 | 0xb9f4 => {
                w.dp_store(
                    match self.pc {
                        0xb666 => 0x6a,
                        0xb7b0 => 0x6a,
                        0xb9f4 => 0xee,
                        _ => unreachable!(),
                    },
                    0,
                    1,
                );
                self.pc + 2
            }
            0xb65c | 0xb66a | 0xb675 | 0xb7a2 | 0xb7a8 => {
                w.mask(
                    match self.pc {
                        0xb65c => 0xf,
                        0xb66a => 0xc,
                        0xb675 => 0x8,
                        0xb7a2 => 0xf,
                        0xb7a8 => 0xf,
                        _ => unreachable!(),
                    },
                    1,
                );
                self.pc + 2
            }
            0xb660 | 0xb9ab => {
                w.drain(8);
                w.load(if self.pc == 0xb660 { 15 } else { 1 }, 1);
                self.pc + 2
            }
            0xb673 | 0xb679 => {
                w.item_y(if self.pc == 0xb673 { 0 } else { 2 }, 1);
                self.pc + 2
            }
            0xb67b => {
                w.drain(6);
                w.load(w.cpu.y, 1);
                0xb67c
            }
            0xb67c => {
                w.drain(6);
                let old = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, 1);
                0xb67d
            }
            0xb671 => {
                w.dp_increment(0x6a, 1, true);
                0xb673
            }
            0xb981 | 0xb985 | 0xb989 | 0xb9b6 | 0xb7b2 => {
                w.compare_immediate(
                    match self.pc {
                        0xb981 => 0x4,
                        0xb985 => 0x2,
                        0xb989 => 0x3,
                        0xb9b6 => 0x1,
                        0xb7b2 => 0x2,
                        _ => unreachable!(),
                    },
                    1,
                );
                self.pc + 2
            }
            0xb652 | 0xb66c | 0xb677 | 0xb7a4 | 0xb7aa | 0xb7b4 | 0xb97f | 0xb983 | 0xb987
            | 0xb98b | 0xb9b8 | 0xb9e4 => {
                let (taken, target) = match self.pc {
                    0xb652 => (w.cpu.p & CARRY == 0, 0xb65a),
                    0xb66c => (w.cpu.p & ZERO == 0, 0xb671),
                    0xb677 => (w.cpu.p & ZERO == 0, 0xb67b),
                    0xb7a4 => (w.cpu.p & ZERO != 0, 0xb7ae),
                    0xb7aa => (w.cpu.p & ZERO != 0, 0xb7ae),
                    0xb7b4 => (w.cpu.p & ZERO == 0, 0xb7c2),
                    0xb97f => (w.cpu.p & ZERO != 0, 0xb9b1),
                    0xb983 => (w.cpu.p & ZERO != 0, 0xb9b1),
                    0xb987 => (w.cpu.p & CARRY == 0, 0xb9ab),
                    0xb98b => (w.cpu.p & ZERO == 0, 0xb98d),
                    0xb9b8 => (w.cpu.p & ZERO != 0, 0xb9f4),
                    0xb9e4 => (w.cpu.p & ZERO != 0, 0xb9f4),
                    _ => unreachable!(),
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xb66e => {
                w.operand_then_internal(2);
                0xb7a0
            }
            0xb98d | 0xb9a9 | 0xb9ba | 0xb9e0 => {
                w.width(0x20, matches!(self.pc, 0xb98d | 0xb9ba));
                self.pc + 2
            }
            0xb991 | 0xb99e | 0xb9af | 0xb9be | 0xb9c6 | 0xb9ce | 0xb9d8 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xb994 | 0xb9a1 | 0xb9b1 | 0xb9d1 | 0xb9db | 0xb9e8 | 0xb9ef => {
                w.clear_carry();
                self.pc + 1
            }
            0xb992 | 0xb99f | 0xb9cf | 0xb9d9 => {
                let rhs = w.dp_read(
                    match self.pc {
                        0xb992 => 0xe8,
                        0xb99f => 0xe2,
                        0xb9cf => 0xe6,
                        0xb9d9 => 0xe0,
                        _ => unreachable!(),
                    },
                    2,
                );
                w.sbc(rhs, 2);
                self.pc + 2
            }
            0xb9bf | 0xb9c7 => {
                w.drain(16);
                let rhs = w.read(w.absolute(if self.pc == 0xb9bf { 0x318 } else { 0x31a }), 2);
                w.sbc(rhs, 2);
                self.pc + 3
            }
            0xb995 | 0xb9a2 | 0xb9d2 | 0xb9dc | 0xb9e9 | 0xb9f0 => {
                let rhs = w.dp_read(
                    match self.pc {
                        0xb995 => 0x20,
                        0xb9a2 => 0x22,
                        0xb9d2 => 0x20,
                        0xb9dc => 0x22,
                        0xb9e9 => 0x0,
                        0xb9f0 => 0x2,
                        _ => unreachable!(),
                    },
                    width,
                );
                if width == 1 {
                    w.adc_byte(rhs);
                } else {
                    w.adc(rhs);
                }
                self.pc + 2
            }
            0xb9b0 | 0xb9b2 | 0xb9f6 | 0xb7c2 => {
                w.rts();
                if self.pc == 0xb7c2 && !self.primary_layer {
                    assert_eq!(w.cpu.pc, 0x82be);
                    self.tail = Some(DiagonalCollisionTail::Complete);
                }
                w.cpu.pc
            }
            0xb682 | 0xb688 | 0xb68e | 0xb69c | 0xb6af | 0xb6b7 | 0xb6bb | 0xb6c3 | 0xb6d7
            | 0xb701 | 0xb7b8 => {
                let v = w.dp_read(
                    match self.pc {
                        0xb682 => 0xe,
                        0xb688 => 0x62,
                        0xb68e => 0xe,
                        0xb69c => 0x67,
                        0xb6af => 0xe,
                        0xb6b7 => 0x4d,
                        0xb6bb => 0xc,
                        0xb6c3 => 0xe,
                        0xb6d7 => 0x67,
                        0xb701 => 0x67,
                        0xb7b8 => 0x2f,
                        _ => unreachable!(),
                    },
                    1,
                );
                w.load(v, 1);
                self.pc + 2
            }
            0xb684 | 0xb68a | 0xb690 | 0xb69e | 0xb6a2 | 0xb6b1 | 0xb6bd | 0xb6c5 | 0xb6d3
            | 0xb6d9 | 0xb703 | 0xb70e | 0xb7ba => {
                w.mask(
                    match self.pc {
                        0xb684 => 0x30,
                        0xb68a => 0x2,
                        0xb690 => 0x30,
                        0xb69e => 0x3,
                        0xb6a2 => 0x2,
                        0xb6b1 => 0x3,
                        0xb6bd => 0x3,
                        0xb6c5 => 0x3,
                        0xb6d3 => 0x3,
                        0xb6d9 => 0x3,
                        0xb703 => 0x3,
                        0xb70e => 0x2,
                        0xb7ba => 0x4,
                        _ => unreachable!(),
                    },
                    1,
                );
                self.pc + 2
            }
            0xb696 => {
                let v = w.dp_read(0x67, 1);
                w.load(w.cpu.a & v, 1);
                self.pc + 2
            }
            0xb692 | 0xb693 | 0xb694 | 0xb695 | 0xb715 => {
                w.drain(6);
                let old = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (old & 1) as u8;
                w.load(old >> 1, 1);
                self.pc + 1
            }
            0xb69a | 0xb6a6 | 0xb70c | 0xb712 | 0xb7b6 | 0xb7be => {
                w.item_y(
                    match self.pc {
                        0xb69a => 0x2,
                        0xb6a6 => 0x3,
                        0xb70c => 0x4,
                        0xb712 => 0x6,
                        0xb7b6 => 0x1,
                        0xb7be => 0x2,
                        _ => unreachable!(),
                    },
                    1,
                );
                self.pc + 2
            }
            0xb714 => {
                w.drain(6);
                w.load(w.cpu.y, 1);
                self.pc + 1
            }
            0xb6aa | 0xb6cb | 0xb6d0 => {
                w.drain(16);
                let v = w.read(
                    w.absolute(match self.pc {
                        0xb6aa => 0x46c,
                        0xb6cb => 0x34a,
                        0xb6d0 => 0x2e8,
                        _ => unreachable!(),
                    }),
                    1,
                );
                w.load(v, 1);
                self.pc + 3
            }
            0xb6c9 => {
                w.dp_store(0x6b, 0, 1);
                self.pc + 2
            }
            0xb6dd | 0xb6e0 | 0xb6e3 | 0xb6e6 | 0xb6e9 | 0xb6ec | 0xb6ef | 0xb6f2 => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0xb6dd => 0x33c,
                        0xb6e0 => 0x33d,
                        0xb6e3 => 0x32f,
                        0xb6e6 => 0x330,
                        0xb6e9 => 0x32b,
                        0xb6ec => 0x32c,
                        0xb6ef => 0x334,
                        0xb6f2 => 0x335,
                        _ => unreachable!(),
                    }),
                    0,
                    1,
                );
                self.pc + 3
            }
            0xb6f5 => {
                w.drain(8);
                w.load(1, 1);
                self.pc + 2
            }
            0xb6f7 => {
                w.drain(16);
                w.write(w.absolute(0x302), w.cpu.a & 255, 1);
                self.pc + 3
            }
            0xb6fa => {
                let v = w.dp_read(0x66, 1);
                w.cpu.y = v;
                w.flags(v, 1);
                self.pc + 2
            }
            0xb6fc => {
                w.drain(16);
                let address = 0xb64bu16 + w.cpu.y;
                if address & 0xff00 != 0xb600 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(address), 1);
                w.load(v, 1);
                self.pc + 3
            }
            0xb6ff | 0xb716 => {
                w.dp_store(
                    match self.pc {
                        0xb6ff => 0x42,
                        0xb716 => 0x66,
                        _ => unreachable!(),
                    },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0xb7c0 => {
                w.dp_store(0x6a, w.cpu.y, 1);
                self.pc + 2
            }
            0xb70a => {
                w.dp_increment(0x6a, 1, true);
                self.pc + 2
            }
            0xb718 => {
                w.item_call();
                self.tail = Some(DiagonalCollisionTail::HorizontalProbe {
                    primary_layer: self.primary_layer,
                    direction: (w.cpu.a & 255) as u8,
                });
                0xcec9
            }
            0xb686 | 0xb68c | 0xb698 | 0xb6a0 | 0xb6a4 | 0xb6ad | 0xb6b3 | 0xb6b9 | 0xb6bf
            | 0xb6c7 | 0xb6ce | 0xb6d5 | 0xb6db | 0xb705 | 0xb710 | 0xb7bc => {
                let (t, target) = match self.pc {
                    0xb686 => (w.cpu.p & ZERO != 0, 0xb6aa),
                    0xb68c => (w.cpu.p & ZERO == 0, 0xb6aa),
                    0xb698 => (w.cpu.p & ZERO == 0, 0xb6aa),
                    0xb6a0 => (w.cpu.p & ZERO != 0, 0xb6aa),
                    0xb6a4 => (w.cpu.p & ZERO == 0, 0xb6fc),
                    0xb6ad => (w.cpu.p & ZERO != 0, 0xb6b7),
                    0xb6b3 => (w.cpu.p & ZERO == 0, 0xb6c3),
                    0xb6b9 => (w.cpu.p & ZERO != 0, 0xb6c3),
                    0xb6bf => (w.cpu.p & ZERO != 0, 0xb6c3),
                    0xb6c7 => (w.cpu.p & ZERO != 0, 0xb701),
                    0xb6ce => (w.cpu.p & ZERO != 0, 0xb6f5),
                    0xb6d5 => (w.cpu.p & ZERO == 0, 0xb6f5),
                    0xb6db => (w.cpu.p & ZERO != 0, 0xb6f5),
                    0xb705 => (w.cpu.p & ZERO == 0, 0xb70a),
                    0xb710 => (w.cpu.p & ZERO == 0, 0xb714),
                    0xb7bc => (w.cpu.p & ZERO != 0, 0xb7c0),
                    _ => unreachable!(),
                };
                if w.branch(t) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xb6a8 | 0xb6b5 | 0xb6c1 => {
                w.branch(true);
                match self.pc {
                    0xb6a8 => 0xb6fc,
                    0xb6b5 => 0xb701,
                    0xb6c1 => 0xb6f5,
                    _ => unreachable!(),
                }
            }
            0xb707 => {
                w.operand_then_internal(2);
                0xb7a0
            }
            _ => panic!("diagonal collision prefix PC {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
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
    fn diagonal_collision_prefix_matches_source_room_modes_and_direction_setup() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut tails = [false; 3];
        for timer in 0..=255u8 {
            for variant in 0..32u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7)) as u8;
                }
                r[0x1f3..0x1f5].copy_from_slice(&[0xbd, 0x82]);
                r[0x46c] = timer;
                r[0x67] = (variant & 15) | if variant >= 16 { 0xf0 } else { 0 };
                r[0x6b] = timer.rotate_left(variant as u32);
                let values: [u16; 8] = [0, 1, 0xffff, 0x7fff, 0x8000, 0xff00, 0xff, 0x5555];
                for (i, a) in [0x20, 0x22, 0xe0, 0xe2, 0xe6, 0xe8].into_iter().enumerate() {
                    r[a..a + 2].copy_from_slice(&values[(i + variant as usize) % 8].to_le_bytes());
                }
                let pc = 0xb64f;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: 8,
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: 0x1f2,
                    p: 0x30 | (timer & 0xc3),
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
                    4586,
                    CpuRasterPosition::new(
                        if variant & 1 == 0 { 22 } else { 261 },
                        if variant & 1 == 0 { 26 } else { 1350 },
                    ),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(variant & 1 != 0),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = DiagonalCollisionPrefix::default();
                while work.tail().is_none() {
                    let label =
                        format!("diagonal timer={timer} variant={variant} pc={:04x}", cpu.pc);
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, _| None,
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                let (index, end, sp) = match work.tail().unwrap() {
                    DiagonalCollisionTail::HorizontalProbe { .. } => {
                        panic!("prefix must stop at the earlier vertical child")
                    }
                    DiagonalCollisionTail::Complete => (0, 0x82be, 0x1f4),
                    DiagonalCollisionTail::VerticalProbe {
                        primary_layer: false,
                        ..
                    } => (1, 0xce85, 0x1f0),
                    DiagonalCollisionTail::VerticalProbe {
                        primary_layer: true,
                        ..
                    } => (2, 0xce85, 0x1ee),
                };
                tails[index] = true;
                assert_eq!((cpu.pc, cpu.sp), (end, sp));
            }
        }
        assert!(tails.iter().all(|seen| *seen));
        println!("diagonal collision prefix: 8192 cases, {instructions} source instructions, all three tail boundaries");
    }
    #[test]
    fn vertical_direction_return_matches_source_masks_resets_and_children() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        for address in 0x07b64b..0x07b64f {
            assert_eq!(
                table_byte(address),
                Some(rom[7 * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        let mut tails = [false; 4];
        for timer in 0..=255u8 {
            for variant in 0..32u8 {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                let r = &mut machine.ram;
                for (i, byte) in r.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7)) as u8;
                }
                let primary_layer = variant & 16 != 0;
                r[0x1f1..0x1f5].copy_from_slice(&[0x56, 0xb6, 0xbd, 0x82]);
                r[0xe] = timer;
                r[0xc] = timer.rotate_left(2);
                r[0x62] = timer >> 6;
                r[0x67] = variant & 15;
                r[0x66] = (variant >> 3) & 1;
                r[0x42] = 15;
                r[0x43] = 15;
                r[0x6a] = variant & 3;
                r[0x6b] = timer.rotate_left(variant as u32);
                r[0x46c] = [0, 1, 2, 4][(variant & 3) as usize];
                r[0x4d] = u8::from(variant & 4 != 0);
                r[0x34a] = u8::from(variant & 8 != 0);
                r[0x2e8] = (timer >> 5) & 3;
                let pc = 0xb682;
                let mut cpu = NativeNmiContext {
                    a: 0x2400,
                    x: 16,
                    y: u16::from(variant),
                    d: 0,
                    db: 7,
                    bank: 7,
                    pc,
                    sp: if primary_layer { 0x1f0 } else { 0x1f2 },
                    p: 0x30 | (timer & 0xc3),
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
                    4586,
                    CpuRasterPosition::new(
                        if variant & 1 == 0 { 22 } else { 261 },
                        if variant & 1 == 0 { 26 } else { 1350 },
                    ),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(variant & 1 != 0),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = DiagonalCollisionPrefix::after_vertical_probe(primary_layer);
                while work.tail().is_none() {
                    let label = format!(
                        "vertical return collision={timer} variant={variant} pc={:04x}",
                        cpu.pc
                    );
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| {
                            if let super::super::intro_clear::Access::Read(a, 1) = access {
                                table_byte(a).map(u16::from)
                            } else {
                                None
                            }
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    work = work.clone();
                    instructions += 1;
                }
                assert_eq!(ram, source.snes().ram);
                let (index, end, sp) = match work.tail().unwrap() {
                    DiagonalCollisionTail::Complete => (0, 0x82be, 0x1f4),
                    DiagonalCollisionTail::HorizontalProbe {
                        primary_layer,
                        direction,
                    } => {
                        assert!(matches!(direction, 2 | 3));
                        (
                            if primary_layer { 2 } else { 1 },
                            0xcec9,
                            if primary_layer { 0x1ee } else { 0x1f0 },
                        )
                    }
                    DiagonalCollisionTail::VerticalProbe { primary_layer, .. } => {
                        assert!(!primary_layer);
                        (3, 0xce85, 0x1f0)
                    }
                };
                tails[index] = true;
                assert_eq!((cpu.pc, cpu.sp), (end, sp));
            }
        }
        assert!(tails.iter().all(|seen| *seen));
        println!("vertical direction return: 8192 cases, {instructions} source instructions, vertical masks, byte resets, four parent/child boundaries");
    }
}
