//! Retained palette loader and its caller return. The phase owns control flow;
//! source addresses only assert the boundary and label cycle-ledger receipts.
//! Copy counts, source pointers and flags stay in the live CPU/scratch state.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

// Fixed offset tables. These are read as data, including the high byte of
// byte-indexed word loads that the caller subsequently masks away.
const OFFSETS: [u8; 177] = [
    0x00, 0x0e, 0x1c, 0x2a, 0x38, 0x46, 0x54, 0x62, 0x70, 0x7e, 0x8c, 0x9a, 0xa8, 0xb6, 0xc4, 0xd2,
    0x00, 0x00, 0x0e, 0x00, 0x1c, 0x00, 0x2a, 0x00, 0x38, 0x00, 0x46, 0x00, 0x54, 0x00, 0x62, 0x00,
    0x70, 0x00, 0x7e, 0x00, 0x8c, 0x00, 0x9a, 0x00, 0xa8, 0x00, 0xb6, 0x00, 0xc4, 0x00, 0xd2, 0x00,
    0xe0, 0x00, 0xee, 0x00, 0xfc, 0x00, 0x0a, 0x01, 0x18, 0x01, 0x26, 0x01, 0x34, 0x01, 0x42, 0x01,
    0x00, 0x0f, 0x1e, 0x2d, 0x3c, 0x00, 0x1c, 0x38, 0x54, 0x70, 0x8c, 0xa8, 0xc4, 0x00, 0x00, 0x2a,
    0x00, 0x54, 0x00, 0x7e, 0x00, 0xa8, 0x00, 0xd2, 0x00, 0xfc, 0x00, 0x26, 0x01, 0x50, 0x01, 0x7a,
    0x01, 0xa4, 0x01, 0xce, 0x01, 0xf8, 0x01, 0x22, 0x02, 0x4c, 0x02, 0x76, 0x02, 0xa0, 0x02, 0xca,
    0x02, 0xf4, 0x02, 0x1e, 0x03, 0x00, 0x00, 0x46, 0x00, 0x8c, 0x00, 0xd2, 0x00, 0x18, 0x01, 0x5e,
    0x01, 0x00, 0x40, 0x00, 0x30, 0x00, 0x00, 0xb4, 0x00, 0x68, 0x01, 0x1c, 0x02, 0xd0, 0x02, 0x84,
    0x03, 0x38, 0x04, 0xec, 0x04, 0xa0, 0x05, 0x54, 0x06, 0x08, 0x07, 0xbc, 0x07, 0x70, 0x08, 0x24,
    0x09, 0xd8, 0x09, 0x8c, 0x0a, 0x40, 0x0b, 0xf4, 0x0b, 0xa8, 0x0c, 0x5c, 0x0d, 0x00, 0x00, 0x78,
    0x00,
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    address
        .checked_sub(0x1bebc6)
        .and_then(|i| OFFSETS.get(i as usize))
        .copied()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Loader {
    SpriteLeft,
    SpriteMain,
    BgMain,
    Bg1,
    Bg2,
    Bg3,
    Environment,
    Hud,
    Dungeon,
}
impl Loader {
    fn from_call(i: u8) -> Self {
        [
            Self::SpriteLeft,
            Self::SpriteMain,
            Self::BgMain,
            Self::Bg1,
            Self::Bg2,
            Self::Bg3,
            Self::Environment,
            Self::Hud,
        ][usize::from(i - 1)]
    }
    fn entry(self) -> u16 {
        match self {
            Self::SpriteLeft => 0xec77,
            Self::SpriteMain => 0xec9e,
            Self::BgMain => 0xeec7,
            Self::Bg1 => 0xeee8,
            Self::Bg2 => 0xef0c,
            Self::Bg3 => 0xeea8,
            Self::Environment => 0xed72,
            Self::Hud => 0xee52,
            Self::Dungeon => 0xee74,
        }
    }
    fn single(self) -> bool {
        matches!(self, Self::SpriteLeft | Self::Bg3 | Self::Environment)
    }
    fn base(self) -> u16 {
        match self {
            Self::SpriteLeft => 0xd39e,
            Self::SpriteMain => 0xd218,
            Self::BgMain => 0xe6c8,
            Self::Bg1 | Self::Bg2 => 0xe86c,
            Self::Bg3 => 0xe604,
            Self::Environment => 0xd446,
            Self::Hud => 0xd660,
            Self::Dungeon => 0xd734,
        }
    }
    fn destination(self) -> u16 {
        match self {
            Self::SpriteLeft => 0x102,
            Self::SpriteMain => 0x122,
            Self::BgMain => 0x42,
            Self::Dungeon => 0x42,
            Self::Bg1 => 0x52,
            Self::Bg2 => 0xb2,
            Self::Bg3 => 0xe2,
            Self::Environment => 0x1d2,
            Self::Hud => 0,
        }
    }
    fn prefix_pc(self, step: u8) -> u16 {
        let offsets: &[u16] = match self {
            Self::SpriteLeft => &[0, 2, 5, 9, 12, 15, 17, 19, 22, 25, 27, 30, 33, 36, 38],
            Self::SpriteMain => &[
                0, 2, 4, 6, 9, 11, 12, 13, 17, 20, 22, 24, 27, 30, 33, 36, 38,
            ],
            Self::BgMain => &[0, 2, 5, 6, 7, 11, 14, 16, 18, 21, 24, 27, 30, 32],
            Self::Bg1 | Self::Bg2 => &[0, 2, 5, 8, 9, 10, 14, 17, 19, 21, 24, 27, 30, 33, 35],
            Self::Bg3 | Self::Environment => &[0, 2, 5, 9, 12, 15, 17, 19, 22, 25, 28, 30],
            Self::Hud => &[0, 2, 5, 9, 12, 15, 17, 19, 22, 25, 28, 31, 33],
            Self::Dungeon => &[
                0, 2, 5, 9, 12, 14, 15, 17, 20, 23, 26, 29, 30, 32, 35, 38, 40, 43, 46, 49, 51,
            ],
        };
        self.entry() + offsets[usize::from(step)]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Select(u8),
    Call(u8),
    Backdrop(u8),
    Prefix(Loader, u8),
    Single(u8),
    Multiple(u8),
    Copy(u8),
    ReturnPalette,
    ReturnCaller,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntroPalettes {
    phase: Phase,
    call: u8,
    copy_return: Option<(Loader, u8)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaletteSource {
    Constant,
    Asset,
    AuxCopy(u16),
    RestoreAdditive,
}
impl Default for IntroPalettes {
    fn default() -> Self {
        Self {
            phase: Phase::Select(0),
            call: 0,
            copy_return: None,
        }
    }
}
impl IntroPalettes {
    /// A real JSL caller can own one loader independently of the intro list.
    pub(crate) fn from_entry(entry: u32) -> Self {
        if entry == 0x0ed5f4 {
            return Self {
                phase: Phase::Backdrop(0),
                call: u8::MAX,
                copy_return: None,
            };
        }
        let loader = match entry {
            0x1bec77 => Loader::SpriteLeft,
            0x1bec9e => Loader::SpriteMain,
            0x1bed72 => Loader::Environment,
            0x1bee74 => Loader::Dungeon,
            0x1beea8 => Loader::Bg3,
            0x1bee52 => Loader::Hud,
            _ => panic!("unsupported palette entry {entry:06x}"),
        };
        Self {
            phase: Phase::Prefix(loader, 0),
            call: u8::MAX,
            copy_return: None,
        }
    }
    /// The caller retains the real JSR return while this owner performs the copy.
    pub(crate) fn from_copy(entry: u32) -> Self {
        Self {
            phase: match entry {
                0x1bef30 => Phase::Single(0),
                0x1bef4b => Phase::Multiple(0),
                _ => panic!("unsupported palette copy {entry:06x}"),
            },
            call: u8::MAX,
            copy_return: None,
        }
    }
    pub(crate) fn palette_source(&self, cpu: &NativeNmiContext) -> PaletteSource {
        match self.phase {
            Phase::Single(7) | Phase::Multiple(11) => PaletteSource::Asset,
            Phase::Copy(3) => PaletteSource::AuxCopy(0x1d0 + cpu.x),
            _ => PaletteSource::Constant,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::Backdrop(_) => 0x0ed5f4,
            Phase::Prefix(loader, _) => 0x1b0000 | u32::from(loader.entry()),
            Phase::Single(_) => 0x1bef30,
            Phase::Multiple(_) => 0x1bef4b,
            Phase::ReturnCaller => 0x028116,
            _ => 0x02c5b2,
        }
    }
    fn pc(&self) -> u16 {
        match self.phase {
            Phase::Select(i) => [
                0xc5d3, 0xc5d5, 0xc5d7, 0xc5da, 0xc5dc, 0xc5df, 0xc5e2, 0xc5e4, 0xc5e7, 0xc5e9,
                0xc5ec, 0xc5ee, 0xc5f1, 0xc5f4,
            ][usize::from(i)],
            Phase::Call(i) => 0xc5f7 + u16::from(i) * 4,
            Phase::Backdrop(i) => [
                0xd5f4, 0xd5f6, 0xd5f9, 0xd5fd, 0xd601, 0xd605, 0xd609, 0xd60b, 0xd60d, 0xd60f,
                0xd611, 0xd613, 0xd615, 0xd617,
            ][usize::from(i)],
            Phase::Prefix(l, i) => l.prefix_pc(i),
            Phase::Single(i) => [
                0xef30, 0xef31, 0xef32, 0xef35, 0xef36, 0xef39, 0xef3b, 0xef3d, 0xef41, 0xef43,
                0xef45, 0xef46, 0xef47, 0xef48, 0xef4a,
            ][usize::from(i)],
            Phase::Multiple(i) => [
                0xef4b, 0xef4d, 0xef4f, 0xef51, 0xef54, 0xef56, 0xef59, 0xef5a, 0xef5c, 0xef5d,
                0xef5f, 0xef61, 0xef65, 0xef67, 0xef69, 0xef6a, 0xef6b, 0xef6c, 0xef6e, 0xef70,
                0xef71, 0xef74, 0xef76, 0xef78, 0xef7a,
            ][usize::from(i)],
            Phase::Copy(i) => [
                0xc61b, 0xc61d, 0xc61f, 0xc623, 0xc627, 0xc628, 0xc629, 0xc62b, 0xc62d,
            ][usize::from(i)],
            Phase::ReturnPalette => 0xc62f,
            Phase::ReturnCaller => 0x811d,
            Phase::Complete => 0x805a,
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d),
            ((self.routine() >> 16) as u8, self.pc(), 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        use Phase::*;
        let returns = matches!(
            self.phase,
            Backdrop(13) | Single(14) | Multiple(24) | ReturnPalette | ReturnCaller
        ) || matches!(self.phase, Prefix(loader,i) if matches!(prefix_action(loader,i),Prep::Return));
        self.phase = match self.phase {
            Select(i) => {
                match i {
                    0 => w.width(0x20, false),
                    1 | 3 | 6 | 8 | 10 => w.immediate(
                        [5, 3, 0, 5, 11][match i {
                            1 => 0,
                            3 => 1,
                            6 => 2,
                            8 => 3,
                            _ => 4,
                        }],
                        1,
                    ),
                    2 | 4 | 5 | 7 | 9 | 11 | 12 | 13 => {
                        let a = match i {
                            2 => 0xab3,
                            4 => 0xab4,
                            5 => 0xab5,
                            7 => 0xab8,
                            9 => 0xab1,
                            11 => 0xaac,
                            12 => 0xabd,
                            _ => 0xaa9,
                        };
                        w.drain(16);
                        let address = w.absolute(a);
                        w.write(address, if i >= 12 { 0 } else { w.cpu.a & 255 }, 1);
                    }
                    _ => unreachable!(),
                }
                if i == 13 {
                    Call(0)
                } else {
                    Select(i + 1)
                }
            }
            Call(i) => {
                self.call = i;
                w.drain(24);
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(self.pc() + 3, 2);
                w.cpu.bank = if i == 0 { 14 } else { 27 };
                if i == 0 {
                    Backdrop(0)
                } else {
                    Prefix(Loader::from_call(i), 0)
                }
            }
            Backdrop(i) => {
                match i {
                    0 => w.width(0x20, true),
                    1 => w.immediate(0, 2),
                    2..=5 => {
                        w.drain(24);
                        w.write(
                            [0x7ec500, 0x7ec540, 0x7ec300, 0x7ec340][usize::from(i - 2)],
                            w.cpu.a,
                            2,
                        );
                    }
                    6 => w.width(0x30, false),
                    7 | 9 | 11 => w.immediate(0x20 << ((i - 7) / 2), 1),
                    8 | 10 | 12 => {
                        w.drain(8);
                        w.write(0x9c + u32::from((i - 8) / 2), w.cpu.a & 255, 1);
                    }
                    13 => w.rtl(),
                    _ => unreachable!(),
                }
                if i == 13 {
                    if self.call == u8::MAX {
                        Complete
                    } else {
                        Call(1)
                    }
                } else {
                    Backdrop(i + 1)
                }
            }
            Prefix(loader, i) => {
                // Shared preparation is selected by the owning native loader,
                // never by reading or dispatching from the runtime source PC.
                let action = prefix_action(loader, i);
                let mut next = Prefix(loader, i + 1);
                match action {
                    Prep::Width(mask) => w.width(mask, true),
                    Prep::ByteWidth => w.width(0x30, false),
                    Prep::Index(a) => {
                        w.drain(16);
                        w.cpu.x = w.read(w.absolute(a), 1);
                        w.flags(w.cpu.x, 1);
                    }
                    Prep::Load(a) => {
                        w.drain(if a < 256 { 8 } else { 16 });
                        w.cpu.a = w.read(if a < 256 { u32::from(a) } else { w.absolute(a) }, 2);
                        w.flags(w.cpu.a, 2);
                    }
                    Prep::Table(a) => {
                        w.drain(24);
                        w.cpu.a = w.read(a + u32::from(w.cpu.x), 2);
                        w.flags(w.cpu.a, 2);
                    }
                    Prep::Mask(value) => {
                        w.drain(16);
                        w.cpu.a &= value;
                        w.flags(w.cpu.a, 2);
                    }
                    Prep::Base => {
                        w.drain(16);
                        w.adc(loader.base());
                    }
                    Prep::Save => {
                        w.drain(8);
                        w.write(0, w.cpu.a, 2);
                    }
                    Prep::Destination => w.immediate(loader.destination(), 2),
                    Prep::X(value, width) => {
                        w.drain(8 * u32::from(width));
                        w.cpu.x = value;
                        w.flags(value, width);
                    }
                    Prep::Y(value) => {
                        w.drain(16);
                        w.cpu.y = value;
                        w.flags(value, 2);
                    }
                    Prep::Shift => {
                        w.drain(6);
                        w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a & 0x8000 != 0);
                        w.cpu.a <<= 1;
                        w.flags(w.cpu.a, 2);
                    }
                    Prep::Tax => {
                        w.drain(6);
                        w.cpu.x = w.cpu.a & 255;
                        w.flags(w.cpu.x, 1);
                    }
                    Prep::Increment => {
                        w.drain(6);
                        w.cpu.x = u16::from((w.cpu.x as u8).wrapping_add(1));
                        w.flags(w.cpu.x, 1);
                    }
                    Prep::Swap => {
                        w.drain(16);
                        w.cpu.x = w.read(w.absolute(0xabd), 2);
                        w.flags(w.cpu.x, 2);
                    }
                    Prep::Alternate => w.immediate(
                        if loader == Loader::Dungeon {
                            0x1f2
                        } else {
                            0x1e2
                        },
                        2,
                    ),
                    Prep::PushA => {
                        w.drain(6);
                        w.push(w.cpu.a, 2);
                    }
                    Prep::PullA => {
                        w.drain(12);
                        let value = w.pull(2);
                        w.load(value, 2);
                    }
                    Prep::SecondDestination => w.immediate(0x112, 2),
                    Prep::Branch(skip) => {
                        w.drain(8);
                        if w.cpu.p & ZERO != 0 {
                            w.drain(6);
                            next = Prefix(loader, i + skip);
                        }
                    }
                    Prep::Copy => {
                        w.drain(16);
                        w.drain(6);
                        w.push(loader.prefix_pc(i) + 2, 2);
                        self.copy_return = Some((loader, i + 1));
                        next = if loader.single() || loader == Loader::Dungeon && i == 18 {
                            Single(0)
                        } else {
                            Multiple(0)
                        };
                    }
                    Prep::Return => {
                        w.rtl();
                        next = if self.call == u8::MAX {
                            Complete
                        } else if self.call == 8 {
                            Copy(0)
                        } else {
                            Call(self.call + 1)
                        };
                    }
                }
                next
            }
            Single(i) => {
                let mut next = Single(i + 1);
                match i {
                    0 => {
                        w.drain(6);
                        w.cpu.y = w.cpu.x;
                        w.flags(w.cpu.y, 2);
                    }
                    1 => w.clear_carry(),
                    2 => {
                        w.drain(16);
                        let rhs = w.read(w.absolute(0xaa8), 2);
                        w.adc(rhs);
                    }
                    3 => w.tax_word(),
                    4 => w.immediate(27, 2),
                    5 => {
                        w.drain(8);
                        w.write(2, w.cpu.a, 2);
                    }
                    6 => w.read_color(),
                    7 => w.write_color(),
                    8 | 9 => w.increment_pointer(),
                    10 | 11 => w.increment_x(),
                    12 => w.decrement_y(),
                    13 => {
                        w.drain(8);
                        if w.cpu.p & NEGATIVE == 0 {
                            w.drain(6);
                            next = Single(6);
                        }
                    }
                    14 => {
                        w.rts();
                        next = if let Some((l, s)) = self.copy_return.take() {
                            Prefix(l, s)
                        } else {
                            assert_eq!(self.call, u8::MAX);
                            Complete
                        };
                    }
                    _ => unreachable!(),
                }
                next
            }
            Multiple(i) => {
                let mut next = Multiple(i + 1);
                match i {
                    0..=2 => {
                        w.drain(8);
                        w.write(
                            4 + u32::from(i) * 2,
                            [w.cpu.a, w.cpu.x, w.cpu.y][usize::from(i)],
                            2,
                        );
                    }
                    3 => w.immediate(27, 2),
                    4 => {
                        w.drain(8);
                        w.write(2, w.cpu.a, 2);
                    }
                    5 => {
                        w.drain(16);
                        w.cpu.a = w.read(w.absolute(0xaa8), 2);
                        w.flags(w.cpu.a, 2);
                    }
                    6 | 19 => w.clear_carry(),
                    7 => {
                        w.drain(8);
                        let rhs = w.read(4, 2);
                        w.adc(rhs);
                    }
                    8 => w.tax_word(),
                    9 => {
                        w.drain(8);
                        w.cpu.y = w.read(6, 2);
                        w.flags(w.cpu.y, 2);
                    }
                    10 => w.read_color(),
                    11 => w.write_color(),
                    12 | 13 => w.increment_pointer(),
                    14 | 15 => w.increment_x(),
                    16 => w.decrement_y(),
                    17 => {
                        w.drain(8);
                        if w.cpu.p & NEGATIVE == 0 {
                            w.drain(6);
                            next = Multiple(10);
                        }
                    }
                    18 => {
                        w.drain(8);
                        w.cpu.a = w.read(4, 2);
                        w.flags(w.cpu.a, 2);
                    }
                    20 => {
                        w.drain(16);
                        w.adc(0x20);
                    }
                    21 => {
                        w.drain(8);
                        w.write(4, w.cpu.a, 2);
                    }
                    22 => {
                        w.drain(8);
                        let value = w.read(8, 2).wrapping_sub(1);
                        w.drain(6);
                        w.flags(value, 2);
                        w.write(8, value, 2);
                    }
                    23 => {
                        w.drain(8);
                        if w.cpu.p & NEGATIVE == 0 {
                            w.drain(6);
                            next = Multiple(5);
                        }
                    }
                    24 => {
                        w.rts();
                        next = if let Some((l, s)) = self.copy_return.take() {
                            Prefix(l, s)
                        } else {
                            assert_eq!(self.call, u8::MAX);
                            Complete
                        };
                    }
                    _ => unreachable!(),
                }
                next
            }
            Copy(i) => {
                let mut next = Copy(i + 1);
                match i {
                    0 => w.width(0x20, true),
                    1 => {
                        w.drain(8);
                        w.cpu.x = 0;
                        w.flags(0, 1);
                    }
                    2 => {
                        w.drain(24);
                        w.cpu.a = w.read(0x7ec4d0 + u32::from(w.cpu.x), 2);
                        w.flags(w.cpu.a, 2);
                    }
                    3 => {
                        w.drain(24);
                        w.write(0x7ec6b0 + u32::from(w.cpu.x), w.cpu.a, 2);
                    }
                    4 | 5 => {
                        w.drain(6);
                        w.cpu.x = u16::from((w.cpu.x as u8).wrapping_add(1));
                        w.flags(w.cpu.x, 1);
                    }
                    6 => {
                        w.drain(8);
                        w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.x >= 16);
                        w.flags(u16::from((w.cpu.x as u8).wrapping_sub(16)), 1);
                    }
                    7 => {
                        w.drain(8);
                        if w.cpu.p & ZERO == 0 {
                            w.drain(6);
                            next = Copy(2);
                        }
                    }
                    8 => {
                        w.width(0x20, false);
                        next = ReturnPalette;
                    }
                    _ => unreachable!(),
                }
                next
            }
            ReturnPalette => {
                w.rts();
                ReturnCaller
            }
            ReturnCaller => {
                w.rtl();
                Complete
            }
            Complete => unreachable!(),
        };
        if !self.complete() {
            if returns {
                assert_eq!(
                    (w.cpu.bank, w.cpu.pc),
                    ((self.routine() >> 16) as u8, self.pc()),
                    "palette return stack disagrees with its retained caller"
                );
            }
            w.cpu.pc = self.pc();
        }
        Some(w.cycles)
    }
}

enum Prep {
    Width(u8),
    ByteWidth,
    Index(u16),
    Load(u16),
    Table(u32),
    Mask(u16),
    Base,
    Save,
    Destination,
    X(u16, u8),
    Y(u16),
    Shift,
    Tax,
    Increment,
    Swap,
    Alternate,
    Branch(u8),
    Copy,
    Return,
    PushA,
    PullA,
    SecondDestination,
}
fn prefix_action(loader: Loader, i: u8) -> Prep {
    use Loader::*;
    use Prep::*;
    match loader {
        Dungeon => match i {
            0 => Width(0x21),
            1 => Index(0xab6),
            2 => Table(0x1bec4b),
            3 => Base,
            4 | 12 => Save,
            5 => PushA,
            6 => Width(0x10),
            7 => Destination,
            8 => X(14, 2),
            9 => Y(5),
            10 | 18 => Copy,
            11 => PullA,
            13 => SecondDestination,
            14 => Swap,
            15 => Branch(2),
            16 => Alternate,
            17 => X(6, 2),
            19 => ByteWidth,
            20 => Return,
            _ => unreachable!(),
        },
        SpriteLeft => match i {
            0 => Width(0x21),
            1 => Index(0xaac),
            2 => Table(0x1bebc6),
            3 => Mask(255),
            4 => Base,
            5 => Save,
            6 => Width(0x10),
            7 => Destination,
            8 => Swap,
            9 => Branch(2),
            10 => Alternate,
            11 => X(6, 2),
            12 => Copy,
            13 => ByteWidth,
            14 => Return,
            _ => unreachable!(),
        },
        SpriteMain => match i {
            0 => Width(0x21),
            1 => X(0, 1),
            2 => Load(0x8a),
            3 => Mask(0x40),
            4 => Branch(3),
            5 | 6 => Increment,
            7 => Table(0x1bec73),
            8 => Base,
            9 => Save,
            10 => Width(0x10),
            11 => Destination,
            12 => X(14, 2),
            13 => Y(3),
            14 => Copy,
            15 => ByteWidth,
            16 => Return,
            _ => unreachable!(),
        },
        BgMain => match i {
            0 => Width(0x21),
            1 => Load(0xab3),
            2 => Shift,
            3 => Tax,
            4 => Table(0x1bec3b),
            5 => Base,
            6 => Save,
            7 => Width(0x10),
            8 => Destination,
            9 => X(6, 2),
            10 => Y(4),
            11 => Copy,
            12 => ByteWidth,
            13 => Return,
            _ => unreachable!(),
        },
        Bg1 | Bg2 => match i {
            0 => Width(0x21),
            1 => Load(if loader == Bg1 { 0xab4 } else { 0xab5 }),
            2 => Mask(255),
            3 => Shift,
            4 => Tax,
            5 => Table(0x1bec13),
            6 => Base,
            7 => Save,
            8 => Width(0x10),
            9 => Destination,
            10 => X(6, 2),
            11 => Y(2),
            12 => Copy,
            13 => ByteWidth,
            14 => Return,
            _ => unreachable!(),
        },
        Bg3 | Environment | Hud => match i {
            0 => Width(0x21),
            1 => Index(match loader {
                Bg3 => 0xab8,
                Environment => 0xab1,
                _ => 0xab2,
            }),
            2 => Table(if loader == Hud { 0x1bec47 } else { 0x1bebc6 }),
            3 => Mask(255),
            4 => Base,
            5 => Save,
            6 => Width(0x10),
            7 => Destination,
            8 => X(if loader == Hud { 15 } else { 6 }, 2),
            9 if loader == Hud => Y(1),
            9 => Copy,
            10 if loader == Hud => Copy,
            10 => ByteWidth,
            11 if loader == Hud => ByteWidth,
            11 | 12 => Return,
            _ => unreachable!(),
        },
    }
}
impl<B: WorkBus> Work<'_, B> {
    fn read_color(&mut self) {
        self.drain(8);
        let low = self.read(0, 2);
        let bank = self.read(2, 1);
        self.cpu.a = self.read((u32::from(bank) << 16) | u32::from(low), 2);
        self.flags(self.cpu.a, 2);
    }
    fn write_color(&mut self) {
        self.drain(24);
        self.write(0x7ec300 + u32::from(self.cpu.x), self.cpu.a, 2);
    }
    fn increment_pointer(&mut self) {
        self.drain(8);
        let value = self.read(0, 2).wrapping_add(1);
        self.drain(6);
        self.flags(value, 2);
        self.write(0, value, 2);
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
    fn palette_loaders_match_every_instruction_and_live_control_branch() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        assert_eq!(&OFFSETS[..], &rom[0xdebc6..0xdec77]);
        // Whole caller plus each independently entered loader. Direct entries
        // exercise live inputs that the intro caller otherwise initializes.
        for entry in 0..=11 {
            for variant in 0..4 {
                for (v, h) in [(235, 128), (224, 1338), (261, 1350)] {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 8)) as u8;
                    }
                    machine.ram[0xaa8..0xabe].fill(0);
                    machine.ram[0x8a] = if variant & 1 != 0 { 0x40 } else { 0 };
                    machine.ram[0xaac] = [0, 3, 11, 15][variant];
                    machine.ram[0xab1] = variant as u8;
                    machine.ram[0xab2] = (variant & 1) as u8;
                    machine.ram[0xab3] = (variant + 2) as u8;
                    machine.ram[0xab4] = (variant + 3) as u8;
                    machine.ram[0xab5] = (variant + 4) as u8;
                    machine.ram[0xab6] = [0, 2, 6, 38][variant];
                    machine.ram[0xab8] = (variant + 1) as u8;
                    machine.ram[0xabd] = (variant & 1) as u8;
                    machine.ram[0xaa9] = if variant & 2 != 0 { 2 } else { 0 };
                    let mut work = IntroPalettes::default();
                    let mut native = NativeNmiContext {
                        a: 0,
                        x: 0x80,
                        y: 0xff,
                        d: 0,
                        db: if variant & 2 != 0 { 1 } else { 0 },
                        bank: 2,
                        pc: 0xc5d3,
                        sp: 0x1fa,
                        p: 0x13,
                    };
                    machine.ram[0x1fb..0x200].copy_from_slice(&[0x1c, 0x81, 0x59, 0x80, 0]);
                    if entry >= 9 {
                        let address = [0x1bee74, 0x1beea8, 0x1bee52][usize::from(entry - 9)];
                        work = IntroPalettes::from_entry(address);
                        native.bank = 27;
                        native.pc = address as u16;
                        native.sp = 0x1f7;
                        native.p = 0x33;
                        machine.ram[0x1f8..0x1fb].copy_from_slice(&[0xbe, 0xcd, 12]);
                    } else if entry != 0 {
                        let loader = Loader::from_call(entry);
                        work.phase = Phase::Prefix(loader, 0);
                        work.call = entry;
                        native.bank = 27;
                        native.pc = loader.entry();
                        native.sp = 0x1f7;
                        native.p = 0x33;
                        let return_word = 0xc5fau16 + u16::from(entry) * 4;
                        machine.ram[0x1f8..0x1fa].copy_from_slice(&return_word.to_le_bytes());
                        machine.ram[0x1fa] = 2;
                    }
                    machine.cpu.a = native.a;
                    machine.cpu.x = native.x;
                    machine.cpu.y = native.y;
                    machine.cpu.db = native.db;
                    machine.cpu.k = native.bank;
                    machine.cpu.pc = native.pc;
                    machine.cpu.sp = native.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(native.p);
                    let mut ram = machine.ram.clone();
                    let mut clock = CpuMasterTimeline::at_raster(
                        0,
                        CpuRasterPosition::new(v, h),
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
                    let normalize = |a: u32| {
                        if a >> 16 < 0x40 && a & 65535 < 0x2000 {
                            a & 65535
                        } else {
                            a
                        }
                    };
                    let mut count = 0;
                    while !work.complete() {
                        let phase = work.phase;
                        let operand = (u32::from(native.bank) << 16) | u32::from(native.pc + 1);
                        let mut first_read = true;
                        let expected = source
                            .step()
                            .unwrap()
                            .accesses
                            .into_iter()
                            .filter_map(|a| {
                                let address = normalize(a.address);
                                let (access, value) = match a.kind {
                                    Kind::Read { value, width } => {
                                        let is_operand = first_read && a.address == operand;
                                        first_read = false;
                                        if is_operand {
                                            return None;
                                        }
                                        (Access::Read(address, width), value)
                                    }
                                    Kind::Write { value, width } => {
                                        (Access::Write(address, value, width), 0)
                                    }
                                    _ => return None,
                                };
                                Some((a.timestamp.master_cycles(), access, value))
                            })
                            .collect::<Vec<_>>();
                        let mut actual = Vec::new();
                        work.step(&mut native, &mut clock, |at, access| {
                            let read = |a: u32| {
                                let a = normalize(a);
                                if a < 0x2000 {
                                    ram[a as usize]
                                } else if a >= 0x7e0000 {
                                    ram[a as usize - 0x7e0000]
                                } else {
                                    rom[((a >> 16) as usize & 127) * 0x8000 + (a as usize & 0x7fff)]
                                }
                            };
                            let (access, value) = match access {
                                Access::Read(a, width) => {
                                    let value = if width == 1 {
                                        u16::from(read(a))
                                    } else {
                                        u16::from_le_bytes([read(a), read(a + 1)])
                                    };
                                    (Access::Read(normalize(a), width), value)
                                }
                                Access::Write(a, value, width) => {
                                    let a = normalize(a);
                                    let offset = if a >= 0x7e0000 {
                                        a as usize - 0x7e0000
                                    } else {
                                        a as usize
                                    };
                                    ram[offset..offset + usize::from(width)].copy_from_slice(
                                        &value.to_le_bytes()[..usize::from(width)],
                                    );
                                    (Access::Write(a, value, width), 0)
                                }
                                Access::Vblank { .. } => return 0,
                            };
                            actual.push((at, access, value));
                            value
                        });
                        assert_eq!(
                            actual, expected,
                            "entry={entry} variant={variant} phase={phase:?}"
                        );
                        let c = &source.snes().cpu;
                        assert_eq!(
                            native,
                            NativeNmiContext {
                                a: c.a,
                                x: c.x,
                                y: c.y,
                                d: c.dp,
                                db: c.db,
                                bank: c.k,
                                pc: c.pc,
                                sp: c.sp,
                                p: c.pack_flags()
                            },
                            "phase={phase:?}"
                        );
                        assert_eq!(clock, *source.timeline(), "phase={phase:?}");
                        work = work.clone();
                        count += 1;
                        assert!(count < 10000);
                        if entry != 0 && matches!(work.phase, Phase::Call(_) | Phase::Copy(_)) {
                            break;
                        }
                    }
                    assert_eq!(ram, source.snes().ram);
                    if entry == 0 {
                        assert_eq!((native.bank, native.pc, native.sp), (0, 0x805a, 0x1ff));
                    }
                }
            }
        }
    }
}
