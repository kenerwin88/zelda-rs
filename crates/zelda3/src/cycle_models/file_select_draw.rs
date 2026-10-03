//! Save-slot drawing retains its live SRAM reads, scratch pointers, stack and
//! individual OAM/stripe stores through interrupts. No completed row is replayed.
use super::{
    intro_clear::Access,
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileSelectDraw {
    entry: u16,
    pc: u16,
    complete: bool,
}
impl FileSelectDraw {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(entry, 0xd63c | 0xd6af | 0xd7a5 | 0xd7db));
        Self {
            entry,
            pc: entry,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x0c0000 | u32::from(self.entry)
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (12, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let x = if w.cpu.p & 0x10 != 0 { 1 } else { 2 };
        let mask = if x == 1 { 255 } else { 65535 };
        w.fetch();
        self.pc = match self.pc {
            0xd6af | 0xd6c4 | 0xd6f2 | 0xd6fa | 0xd741 | 0xd749 | 0xd7a0 | 0xd7db | 0xd839
            | 0xd881 => {
                w.width(
                    if matches!(self.pc, 0xd6f2 | 0xd6fa | 0xd741 | 0xd749) {
                        0x10
                    } else {
                        0x30
                    },
                    matches!(self.pc, 0xd6af | 0xd6f2 | 0xd741 | 0xd7a0 | 0xd7db | 0xd881),
                );
                self.pc + 2
            }
            0xd643 | 0xd679 | 0xd6b1 | 0xd701 | 0xd71c | 0xd750 | 0xd762 | 0xd774 | 0xd779
            | 0xd798 | 0xd7c0 | 0xd7c5 | 0xd7fa | 0xd868 | 0xd872 => {
                w.immediate(
                    match self.pc {
                        0xd643 => 6,
                        0xd679 => 0x520,
                        0xd6b1 => 0x116,
                        0xd701 | 0xd750 => 0xf0,
                        0xd762 | 0xd779 | 0xd798 | 0xd7c5 => 2,
                        0xd7c0 => 0x7e,
                        0xd7fa => 9,
                        0xd868 => 0x3c,
                        _ => 0,
                    },
                    m,
                );
                self.pc + 1 + u16::from(m)
            }
            0xd663 | 0xd67c | 0xd7b0 | 0xd805 | 0xd817 | 0xd82a => {
                w.drain(8 * u32::from(x));
                let v = match self.pc {
                    0xd663 => 1,
                    0xd67c => 10,
                    0xd82a => 4,
                    _ => 0,
                };
                if matches!(self.pc, 0xd663 | 0xd805 | 0xd817) {
                    w.cpu.y = v;
                } else {
                    w.cpu.x = v;
                }
                w.flags(v, x);
                self.pc + 1 + u16::from(x)
            }
            0xd63c | 0xd6f0 | 0xd6f1 | 0xd717 | 0xd73f | 0xd740 | 0xd75d | 0xd7af | 0xd843
            | 0xd84d | 0xd86d => {
                w.drain(6);
                w.push(
                    if matches!(self.pc, 0xd6f0 | 0xd73f | 0xd84d | 0xd86d) {
                        w.cpu.y
                    } else {
                        w.cpu.x
                    },
                    x,
                );
                self.pc + 1
            }
            0xd687 | 0xd7df | 0xd7e4 | 0xd85c => {
                w.drain(6);
                w.push(w.cpu.a & if m == 1 { 255 } else { 65535 }, m);
                self.pc + 1
            }
            0xd662 | 0xd6fc | 0xd716 | 0xd74b | 0xd75c | 0xd7bf | 0xd857 | 0xd877 | 0xd87c => {
                w.drain(12);
                let v = w.pull(x);
                if matches!(self.pc, 0xd716 | 0xd75c | 0xd857 | 0xd877) {
                    w.cpu.y = v;
                } else {
                    w.cpu.x = v;
                }
                w.flags(v, x);
                self.pc + 1
            }
            0xd68f | 0xd724 | 0xd767 | 0xd85b | 0xd883 | 0xd886 => {
                w.drain(12);
                let v = w.pull(m);
                w.load(v, m);
                self.pc + 1
            }
            0xd63d | 0xd672 | 0xd6f4 | 0xd743 | 0xd7e7 => {
                let v = w.dp_read(
                    if self.pc == 0xd63d || self.pc == 0xd672 {
                        0
                    } else {
                        14
                    },
                    x,
                );
                if self.pc == 0xd63d {
                    w.cpu.y = v;
                } else {
                    w.cpu.x = v;
                }
                w.flags(v, x);
                self.pc + 2
            }
            0xd688 | 0xd6b8 | 0xd6c6 | 0xd7a5 | 0xd7aa | 0xd7b2 | 0xd7dd | 0xd7e2 | 0xd82d
            | 0xd833 | 0xd83b | 0xd84e => {
                let v = w.dp_read(
                    match self.pc {
                        0xd688 | 0xd7e2 | 0xd833 => 4,
                        0xd7aa => 1,
                        0xd7b2 => 0x1a,
                        0xd7dd => 2,
                        0xd82d => 6,
                        _ => 0,
                    },
                    m,
                );
                w.load(v, m);
                self.pc + 2
            }
            0xd844 => {
                w.drain(14);
                let v = w.read(u32::from(2 + w.cpu.x), 1);
                w.load(v, 1);
                self.pc + 2
            }
            0xd646 | 0xd670 | 0xd677 | 0xd6c2 | 0xd7e0 | 0xd7e5 | 0xd7fd | 0xd7ff | 0xd801
            | 0xd814 | 0xd826 | 0xd828 | 0xd884 | 0xd887 => {
                let address = match self.pc {
                    0xd677 | 0xd7ff | 0xd826 | 0xd884 => 4,
                    0xd6c2 => 14,
                    0xd7e0 => 8,
                    0xd7e5 => 10,
                    0xd801 | 0xd828 => 6,
                    _ => 2,
                };
                w.dp_store(
                    address,
                    if matches!(self.pc, 0xd677 | 0xd828) {
                        w.cpu.y
                    } else {
                        w.cpu.a
                    },
                    2,
                );
                self.pc + 2
            }
            0xd65e | 0xd690 => {
                w.dp_increment(2, 2, false);
                self.pc + 2
            }
            0xd648 | 0xd666 | 0xd6be | 0xd6f6 | 0xd745 | 0xd7e9 => {
                let a = match self.pc {
                    0xd648 => 0x7003d9,
                    0xd666 => 0x70036c,
                    0xd6be => 0x848c,
                    0xd6f6 => 0x700359,
                    0xd745 => 0x70035a,
                    _ => 0x700405,
                };
                w.drain(24);
                let v = w.read(a + u32::from(w.cpu.x), m);
                w.load(v, m);
                self.pc + 4
            }
            0xd63f | 0xd674 | 0xd6ca | 0xd6e7 | 0xd70a | 0xd739 | 0xd756 | 0xd77e | 0xd7b9
            | 0xd83f | 0xd847 | 0xd862 => {
                let base = match self.pc {
                    0xd63f => 0xd630,
                    0xd674 => 0xd636,
                    0xd6ca => 0xd69c,
                    0xd6e7 => 0xd6a6,
                    0xd70a => 0xd69f,
                    0xd739 => 0xd6a9,
                    0xd756 => 0xd6a3,
                    0xd77e => 0xd6ac,
                    0xd7b9 => 0xd7a3,
                    0xd83f => 0xd7d5,
                    0xd847 => 0xd7cb,
                    _ => 0xd7d8,
                };
                let index = if matches!(self.pc, 0xd674 | 0xd7b9 | 0xd847 | 0xd862) {
                    w.cpu.x
                } else {
                    w.cpu.y
                };
                w.drain(16);
                if x == 2 || (base & 255) + index > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(base) + u32::from(index), m);
                if self.pc == 0xd674 {
                    w.cpu.y = v;
                    w.flags(v, 2);
                } else if self.pc == 0xd862 {
                    w.adc_byte(v);
                } else {
                    w.load(v, m);
                }
                self.pc + 3
            }
            0xd6ce | 0xd6d9 | 0xd729 | 0xd731 | 0xd76c | 0xd789 | 0xd852 | 0xd85f => {
                let indexed = matches!(self.pc, 0xd6d9 | 0xd731 | 0xd789 | 0xd852);
                w.drain(8);
                let base = w.read(
                    if self.pc == 0xd852 {
                        8
                    } else if self.pc == 0xd85f {
                        10
                    } else if indexed {
                        2
                    } else {
                        4
                    },
                    2,
                );
                let index = if indexed { w.cpu.y } else { 0 };
                if indexed && (x == 2 || (base & 255) + index > 255) {
                    w.drain(6);
                }
                let v = w.read(w.absolute(base) + u32::from(index), 1);
                w.load(v, 1);
                self.pc + 2
            }
            0xd650 | 0xd657 | 0xd67f => {
                w.drain(22);
                w.write(
                    w.absolute(if self.pc == 0xd657 { 0x102c } else { 0x1002 })
                        + u32::from(w.cpu.y),
                    w.cpu.a,
                    2,
                );
                self.pc + 3
            }
            0xd6b5 | 0xd7a7 | 0xd7ac | 0xd7bc | 0xd7c2 | 0xd7c7 => {
                w.drain(16);
                let a = match self.pc {
                    0xd6b5 => 0x100,
                    0xd7a7 => 0x800,
                    0xd7ac => 0x801,
                    0xd7bc => 0x802,
                    0xd7c2 => 0x803,
                    _ => 0xa20,
                };
                w.write(w.absolute(a), w.cpu.a & if m == 1 { 255 } else { 65535 }, m);
                self.pc + 3
            }
            0xd6d3 | 0xd6d6 | 0xd6de | 0xd6e4 | 0xd6ea | 0xd6ed | 0xd703 | 0xd706 | 0xd70d
            | 0xd713 | 0xd71e | 0xd721 | 0xd72e | 0xd736 | 0xd73c | 0xd752 | 0xd759 | 0xd764
            | 0xd76e | 0xd771 | 0xd776 | 0xd77b | 0xd781 | 0xd786 | 0xd78b | 0xd791 | 0xd79a
            | 0xd79d | 0xd84a | 0xd858 | 0xd865 | 0xd86a | 0xd874 => {
                let a = match self.pc {
                    0xd6d3 | 0xd72e | 0xd76e | 0xd865 => 0x800,
                    0xd6d6 | 0xd771 => 0x804,
                    0xd6de | 0xd703 | 0xd736 | 0xd752 | 0xd78b | 0xd858 => 0x801,
                    0xd6e4 | 0xd706 | 0xd791 => 0x805,
                    0xd6ea | 0xd73c | 0xd781 | 0xd86a => 0x803,
                    0xd6ed | 0xd786 => 0x807,
                    0xd70d | 0xd759 | 0xd776 | 0xd84a => 0x802,
                    0xd713 | 0xd77b => 0x806,
                    0xd721 | 0xd79d => 0xa21,
                    _ => 0xa20,
                };
                let index = if matches!(self.pc, 0xd84a | 0xd858 | 0xd865 | 0xd86a | 0xd874) {
                    w.cpu.y
                } else {
                    w.cpu.x
                };
                w.drain(22);
                w.write(w.absolute(a) + u32::from(index), w.cpu.a & 255, 1);
                self.pc + 3
            }
            0xd642 | 0xd68e | 0xd6c9 | 0xd6fd | 0xd74c | 0xd83e | 0xd842 | 0xd851 | 0xd871 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xd6bd | 0xd6cd | 0xd71b | 0xd728 | 0xd761 | 0xd76b | 0xd797 | 0xd846 | 0xd85e => {
                w.drain(6);
                w.cpu.x = w.cpu.a & mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xd718 | 0xd75e | 0xd794 | 0xd816 | 0xd86e => {
                w.drain(6);
                let v = if matches!(self.pc, 0xd816 | 0xd86e) {
                    w.cpu.y
                } else {
                    w.cpu.x
                };
                w.load(v, m);
                self.pc + 1
            }
            0xd65a | 0xd65b | 0xd7b8 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1) & mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xd684 | 0xd831 | 0xd832 | 0xd837 | 0xd838 | 0xd87d | 0xd87e => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1) & mask;
                w.flags(w.cpu.x, x);
                self.pc + 1
            }
            0xd65c
            | 0xd65d
            | 0xd682
            | 0xd683
            | 0xd709
            | 0xd755
            | 0xd811
            | 0xd823
            | 0xd878..=0xd87b => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_add(1) & mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xd6fe | 0xd74d => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & mask;
                w.flags(w.cpu.y, x);
                self.pc + 1
            }
            0xd66d..=0xd66f
            | 0xd6c8
            | 0xd719
            | 0xd71a
            | 0xd75f
            | 0xd760
            | 0xd795
            | 0xd796
            | 0xd83d
            | 0xd850
            | 0xd85d
            | 0xd86f
            | 0xd870 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a & 1) as u8;
                let a = w.cpu.a & if m == 1 { 255 } else { 65535 };
                w.load(a >> 1, m);
                self.pc + 1
            }
            0xd6b4 => {
                w.shift_left(2);
                self.pc + 1
            }
            0xd66a | 0xd6ba | 0xd7b4 => {
                w.mask(if self.pc == 0xd7b4 { 8 } else { 255 }, m);
                self.pc + 1 + u16::from(m)
            }
            0xd784 => {
                w.drain(8);
                w.load((w.cpu.a & 255) | 0x40, 1);
                self.pc + 2
            }
            0xd64c | 0xd653 | 0xd68a | 0xd6d0 | 0xd6db | 0xd6e1 | 0xd710 | 0xd725 | 0xd72b
            | 0xd733 | 0xd768 | 0xd78e | 0xd854 | 0xd861 => {
                w.clear_carry();
                self.pc + 1
            }
            0xd64d | 0xd654 | 0xd68b | 0xd6d1 | 0xd6dc | 0xd6e2 | 0xd711 | 0xd726 | 0xd72c
            | 0xd734 | 0xd769 | 0xd78f | 0xd855 => {
                let v = match self.pc {
                    0xd64d => 0x1800,
                    0xd654 | 0xd711 | 0xd855 => 0x10,
                    0xd68b => 0x2a,
                    0xd6d1 => 12,
                    0xd6dc | 0xd72c => 251,
                    0xd734 => 10,
                    0xd769 => 4,
                    _ => 8,
                };
                w.drain(8 * u32::from(m));
                if m == 1 {
                    w.adc_byte(v);
                } else {
                    w.adc(v);
                }
                self.pc + 1 + u16::from(m)
            }
            0xd7ed | 0xd7f5 | 0xd808 | 0xd81a => {
                w.compare_immediate(
                    match self.pc {
                        0xd7ed => 65535,
                        0xd7f5 => 1000,
                        _ => 10,
                    },
                    2,
                );
                self.pc + 3
            }
            0xd80d | 0xd81f => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0xd80e | 0xd820 => {
                w.drain(16);
                w.sbc(10, 2);
                self.pc + 3
            }
            0xd660 | 0xd685 | 0xd692 | 0xd6ff | 0xd74e | 0xd7b6 | 0xd7f0 | 0xd7f8 | 0xd80b
            | 0xd81d | 0xd82f | 0xd835 | 0xd87f => {
                let (taken, target) = match self.pc {
                    0xd660 => (w.cpu.p & ZERO == 0, 0xd648),
                    0xd685 => (w.cpu.p & ZERO == 0, 0xd690),
                    0xd692 => (w.cpu.p & ZERO == 0, 0xd67f),
                    0xd6ff => (w.cpu.p & NEGATIVE == 0, 0xd70a),
                    0xd74e => (w.cpu.p & NEGATIVE == 0, 0xd756),
                    0xd7b6 => (w.cpu.p & ZERO != 0, 0xd7b9),
                    0xd7f0 => (w.cpu.p & ZERO == 0, 0xd7f5),
                    0xd7f8 => (w.cpu.p & CARRY == 0, 0xd805),
                    0xd80b => (w.cpu.p & CARRY == 0, 0xd814),
                    0xd81d => (w.cpu.p & CARRY == 0, 0xd826),
                    0xd82f | 0xd835 => (w.cpu.p & ZERO == 0, 0xd839),
                    _ => (w.cpu.p & NEGATIVE == 0, 0xd843),
                };
                if w.branch(taken) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0xd803 | 0xd812 | 0xd824 => {
                w.branch(true);
                match self.pc {
                    0xd803 => 0xd82a,
                    0xd812 => 0xd808,
                    _ => 0xd81a,
                }
            }
            0xd7f2 => {
                w.drain(16);
                0xd883
            }
            0xd694 | 0xd7a2 | 0xd7ca | 0xd889 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("file drawing continuation {:04x}", self.pc),
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
    fn file_drawers_match_source_for_every_slot_equipment_and_digit_branch() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for entry in [0xd63c, 0xd6af, 0xd7a5, 0xd7db] {
            for variant in 0..60 {
                let slot = variant % 3;
                let level = variant / 3;
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8)) as u8;
                }
                machine.cart.ram.fill(0xa7);
                machine.ram[0..2].copy_from_slice(&(slot as u16 * 2).to_le_bytes());
                machine.ram[2..4].copy_from_slice(&0xd699u16.to_le_bytes());
                machine.ram[4..6].copy_from_slice(&0xd698u16.to_le_bytes());
                machine.ram[14..16].copy_from_slice(&(slot as u16 * 0x500).to_le_bytes());
                machine.ram[0x1a] = level as u8;
                machine.ram[0x1f8..0x1fa].copy_from_slice(&0xcf10u16.to_le_bytes());
                let s = slot * 0x500;
                machine.cart.ram[s + 0x359] = (level % 5) as u8;
                machine.cart.ram[s + 0x35a] = (level % 4) as u8;
                machine.cart.ram[s + 0x36c] = (level + 1) as u8 * 8;
                let deaths = [
                    0u16, 1, 9, 10, 11, 99, 100, 101, 109, 110, 199, 909, 990, 999, 1000, 1001,
                    12345, 65534, 65535, 42,
                ][level];
                machine.cart.ram[s + 0x405..s + 0x407].copy_from_slice(&deaths.to_le_bytes());
                for i in 0..6 {
                    machine.cart.ram[s + 0x3d9 + i * 2..s + 0x3db + i * 2]
                        .copy_from_slice(&((i * 17 + level) as u16).to_le_bytes());
                }
                let mut native = NativeNmiContext {
                    a: 0xab83,
                    x: s as u16,
                    y: 0x83,
                    d: 0,
                    db: 12,
                    bank: 12,
                    pc: entry,
                    sp: 0x1f7,
                    p: if entry == 0xd7a5 { 0x31 } else { 1 },
                };
                machine.cpu.a = native.a;
                machine.cpu.x = native.x;
                machine.cpu.y = native.y;
                machine.cpu.db = 12;
                machine.cpu.k = 12;
                machine.cpu.pc = entry;
                machine.cpu.sp = native.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(native.p);
                // SEP X clears the real index high bytes before the fairy call.
                if entry == 0xd7a5 {
                    native.x &= 255;
                    machine.cpu.x = native.x;
                }
                let mut ram = machine.ram.clone();
                let sram = machine.cart.ram.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(if variant & 1 == 0 { 224 } else { 261 }, 1350),
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
                let mut work = FileSelectDraw::new(entry);
                let mut count = 0;
                while !work.complete() {
                    let label = format!(
                        "file drawing entry{entry:04x} slot{slot} level{level} pc{:04x}",
                        native.pc
                    );
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut native,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, a| {
                            if let Access::Read(a, width) = a {
                                if (0x700000..0x702000).contains(&a) {
                                    let i = (a - 0x700000) as usize;
                                    return Some(if width == 1 {
                                        u16::from(sram[i])
                                    } else {
                                        u16::from_le_bytes([sram[i], sram[i + 1]])
                                    });
                                }
                            }
                            None
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    work = work.clone();
                    count += 1;
                    assert!(count < 3000);
                }
                assert_eq!((native.bank, native.pc, native.sp), (12, 0xcf11, 0x1f9));
            }
        }
    }
}
