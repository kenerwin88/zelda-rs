//! Retained native work for the shared Sprite_DrawShadowEx body ($06:DC66).
//! The caller supplies live native inputs at each instruction. Loaded values,
//! the selected shape and the return work belong to this continuation.

#[derive(Clone, Copy, Debug)]
pub(crate) struct Inputs {
    pub world_y: u16,
    pub pause: u8,
    pub state: u8,
    pub draw_work: u8,
    pub scroll_y: u16,
    pub flags2: u8,
    pub flags3: u8,
    pub x: u16,
    pub y: u16,
    pub flags: u8,
    pub oam_pointer: u16,
    pub extended_pointer: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Publication {
    YLow(u8),
    YHigh(u8),
    Y(u16),
    OamByte { address: u16, value: u8 },
    ExtendedByte { address: u16, value: u8 },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub publication: Option<Publication>,
    pub pc: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Stage {
    ClearWorldCarry,
    AddWorldLow,
    StoreWorldLow,
    LoadWorldHigh,
    AddWorldCarry,
    StoreWorldHigh,
    LoadPause,
    TestPause,
    LoadState,
    CompareCarried,
    TestCarried,
    LoadDrawWork,
    CompareHidden,
    TestHidden,
    Widen,
    LoadWorldY,
    SetScrollCarry,
    SubtractScroll,
    StoreScreenY,
    ClearClipCarry,
    AddClipMargin,
    CompareClip,
    Narrow,
    TestClip,
    LoadOffset,
    MaskOffset,
    DoubleOffset,
    QuadrupleOffset,
    SelectOffset,
    LoadX,
    StoreX,
    LoadShape,
    MaskShape,
    SelectShape,
    SmallNextY,
    SmallDiscardRead,
    SmallLoadY,
    SmallIncrementY,
    SmallStoreY,
    SmallNextChar,
    SmallLoadChar,
    SmallStoreChar,
    SmallLoadFlags,
    SmallMaskFlags,
    SmallPriority,
    SmallNextFlags,
    SmallStoreFlags,
    LargeLoadY,
    LargeNextY,
    LargeStoreY,
    LargeLoadChar,
    LargeNextChar,
    LargeStoreChar,
    LargeLoadFlags,
    LargeMaskFlags,
    LargePriority,
    LargeNextFlags,
    LargeStoreFlags,
    CopyOffset,
    HalfOffset,
    QuarterOffset,
    SelectExtendedOffset,
    LoadXHigh,
    MaskXHigh,
    SetLargeSize,
    StoreExtended,
    Return,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ShadowWork {
    stage: Stage,
    value: u16,
    index: u8,
    carry: bool,
    zero: bool,
    large: bool,
}

impl ShadowWork {
    pub(crate) fn new(offset: u8) -> Self {
        Self {
            stage: Stage::ClearWorldCarry,
            value: u16::from(offset),
            index: 0,
            carry: false,
            zero: false,
            large: false,
        }
    }

    pub(crate) fn at_extended_tail(&self) -> bool {
        self.stage == Stage::CopyOffset
    }

    pub(crate) fn step(&mut self, input: Inputs) -> Option<Instruction> {
        use Stage::*;
        let mut publication = None;
        macro_rules! go {
            ($pc:expr, $cycles:expr, $next:expr) => {
                ($pc, $cycles, $next)
            };
        }
        macro_rules! byte_store {
            ($pointer:expr) => {
                Publication::OamByte {
                    address: $pointer.wrapping_add(u16::from(self.index)),
                    value: self.value as u8,
                }
            };
        }
        let (pc, master_cycles, next) = match self.stage {
            ClearWorldCarry => {
                self.carry = false;
                go!(0xdc66, 14, AddWorldLow)
            }
            AddWorldLow => {
                let sum = self.value + u16::from(input.world_y as u8);
                self.value = sum & 255;
                self.carry = sum > 255;
                go!(0xdc67, 32, StoreWorldLow)
            }
            StoreWorldLow => {
                publication = Some(Publication::YLow(self.value as u8));
                go!(0xdc6a, 24, LoadWorldHigh)
            }
            LoadWorldHigh => {
                self.value = input.world_y >> 8;
                go!(0xdc6c, 32, AddWorldCarry)
            }
            AddWorldCarry => {
                self.value = (self.value + u16::from(self.carry)) & 255;
                go!(0xdc6f, 16, StoreWorldHigh)
            }
            StoreWorldHigh => {
                publication = Some(Publication::YHigh(self.value as u8));
                go!(0xdc71, 24, LoadPause)
            }
            LoadPause => {
                self.value = u16::from(input.pause);
                go!(0xdc73, 32, TestPause)
            }
            TestPause => {
                let taken = self.value != 0;
                go!(
                    0xdc76,
                    if taken { 22 } else { 16 },
                    if taken { Return } else { LoadState }
                )
            }
            LoadState => {
                self.value = u16::from(input.state);
                go!(0xdc78, 32, CompareCarried)
            }
            CompareCarried => {
                self.zero = self.value == 10;
                go!(0xdc7b, 16, TestCarried)
            }
            TestCarried => {
                go!(
                    0xdc7d,
                    if self.zero { 16 } else { 22 },
                    if self.zero { LoadDrawWork } else { Widen }
                )
            }
            LoadDrawWork => {
                self.value = u16::from(input.draw_work);
                go!(0xdc7f, 40, CompareHidden)
            }
            CompareHidden => {
                self.zero = self.value == 3;
                go!(0xdc83, 16, TestHidden)
            }
            TestHidden => {
                go!(
                    0xdc85,
                    if self.zero { 22 } else { 16 },
                    if self.zero { Return } else { Widen }
                )
            }
            Widen => go!(0xdc87, 22, LoadWorldY),
            LoadWorldY => {
                self.value = input.y;
                go!(0xdc89, 32, SetScrollCarry)
            }
            SetScrollCarry => go!(0xdc8b, 14, SubtractScroll),
            SubtractScroll => {
                self.value = self.value.wrapping_sub(input.scroll_y);
                go!(0xdc8c, 32, StoreScreenY)
            }
            StoreScreenY => {
                publication = Some(Publication::Y(self.value));
                go!(0xdc8e, 32, ClearClipCarry)
            }
            ClearClipCarry => go!(0xdc90, 14, AddClipMargin),
            AddClipMargin => {
                self.value = self.value.wrapping_add(16);
                go!(0xdc91, 24, CompareClip)
            }
            CompareClip => {
                self.carry = self.value >= 256;
                go!(0xdc94, 24, Narrow)
            }
            Narrow => go!(0xdc97, 22, TestClip),
            TestClip => {
                self.large = self.carry;
                go!(
                    0xdc99,
                    if self.carry { 22 } else { 16 },
                    if self.carry { Return } else { LoadOffset }
                )
            }
            LoadOffset => {
                self.value = u16::from(input.flags2);
                go!(0xdc9b, 32, MaskOffset)
            }
            MaskOffset => {
                self.value &= 31;
                go!(0xdc9e, 16, DoubleOffset)
            }
            DoubleOffset => {
                self.value <<= 1;
                go!(0xdca0, 14, QuadrupleOffset)
            }
            QuadrupleOffset => {
                self.value <<= 1;
                go!(0xdca1, 14, SelectOffset)
            }
            SelectOffset => {
                self.index = self.value as u8;
                go!(0xdca2, 14, LoadX)
            }
            LoadX => {
                self.value = u16::from(input.x as u8);
                go!(0xdca3, 24, StoreX)
            }
            StoreX => {
                publication = Some(byte_store!(input.oam_pointer));
                go!(0xdca5, 46, LoadShape)
            }
            LoadShape => {
                self.value = u16::from(input.flags3);
                go!(0xdca7, 32, MaskShape)
            }
            MaskShape => {
                self.value &= 32;
                go!(0xdcaa, 16, SelectShape)
            }
            SelectShape => {
                self.large = self.value == 0;
                go!(
                    0xdcac,
                    if self.large { 22 } else { 16 },
                    if self.large { LargeLoadY } else { SmallNextY }
                )
            }
            SmallNextY => {
                self.index += 1;
                go!(0xdcae, 14, SmallDiscardRead)
            }
            SmallDiscardRead => {
                // The overwritten RAM read has no data dependency, but its
                // indexed page crossing is still real bus work.
                let crossed = (input.oam_pointer & 255) + u16::from(self.index) > 255;
                go!(0xdcaf, if crossed { 46 } else { 40 }, SmallLoadY)
            }
            SmallLoadY => {
                self.value = u16::from(input.y as u8);
                go!(0xdcb1, 24, SmallIncrementY)
            }
            SmallIncrementY => {
                self.value = (self.value + 1) & 255;
                go!(0xdcb3, 14, SmallStoreY)
            }
            SmallStoreY => {
                publication = Some(byte_store!(input.oam_pointer));
                go!(0xdcb4, 46, SmallNextChar)
            }
            SmallNextChar => {
                self.index += 1;
                go!(0xdcb6, 14, SmallLoadChar)
            }
            SmallLoadChar => {
                self.value = 0x38;
                go!(0xdcb7, 16, SmallStoreChar)
            }
            SmallStoreChar => {
                publication = Some(byte_store!(input.oam_pointer));
                go!(0xdcb9, 46, SmallLoadFlags)
            }
            SmallLoadFlags => {
                self.value = u16::from(input.flags);
                go!(0xdcbb, 24, SmallMaskFlags)
            }
            SmallMaskFlags => {
                self.value &= 0x30;
                go!(0xdcbd, 16, SmallPriority)
            }
            SmallPriority => {
                self.value |= 8;
                go!(0xdcbf, 16, SmallNextFlags)
            }
            SmallNextFlags => {
                self.index += 1;
                go!(0xdcc1, 14, SmallStoreFlags)
            }
            SmallStoreFlags => {
                publication = Some(byte_store!(input.oam_pointer));
                go!(0xdcc2, 46, CopyOffset)
            }
            LargeLoadY => {
                self.value = u16::from(input.y as u8);
                go!(0xdccf, 24, LargeNextY)
            }
            LargeNextY => {
                self.index += 1;
                go!(0xdcd1, 14, LargeStoreY)
            }
            LargeStoreY => {
                publication = Some(byte_store!(input.oam_pointer));
                go!(0xdcd2, 46, LargeLoadChar)
            }
            LargeLoadChar => {
                self.value = 0x6c;
                go!(0xdcd4, 16, LargeNextChar)
            }
            LargeNextChar => {
                self.index += 1;
                go!(0xdcd6, 14, LargeStoreChar)
            }
            LargeStoreChar => {
                publication = Some(byte_store!(input.oam_pointer));
                go!(0xdcd7, 46, LargeLoadFlags)
            }
            LargeLoadFlags => {
                self.value = u16::from(input.flags);
                go!(0xdcd9, 24, LargeMaskFlags)
            }
            LargeMaskFlags => {
                self.value &= 0x30;
                go!(0xdcdb, 16, LargePriority)
            }
            LargePriority => {
                self.value |= 8;
                go!(0xdcdd, 16, LargeNextFlags)
            }
            LargeNextFlags => {
                self.index += 1;
                go!(0xdcdf, 14, LargeStoreFlags)
            }
            LargeStoreFlags => {
                publication = Some(byte_store!(input.oam_pointer));
                go!(0xdce0, 46, CopyOffset)
            }
            CopyOffset => {
                self.value = u16::from(self.index);
                go!(if self.large { 0xdce2 } else { 0xdcc4 }, 14, HalfOffset)
            }
            HalfOffset => {
                self.value >>= 1;
                go!(if self.large { 0xdce3 } else { 0xdcc5 }, 14, QuarterOffset)
            }
            QuarterOffset => {
                self.value >>= 1;
                go!(
                    if self.large { 0xdce4 } else { 0xdcc6 },
                    14,
                    SelectExtendedOffset
                )
            }
            SelectExtendedOffset => {
                self.index = self.value as u8;
                go!(if self.large { 0xdce5 } else { 0xdcc7 }, 14, LoadXHigh)
            }
            LoadXHigh => {
                self.value = input.x >> 8;
                go!(if self.large { 0xdce6 } else { 0xdcc8 }, 24, MaskXHigh)
            }
            MaskXHigh => {
                self.value &= 1;
                go!(
                    if self.large { 0xdce8 } else { 0xdcca },
                    16,
                    if self.large {
                        SetLargeSize
                    } else {
                        StoreExtended
                    }
                )
            }
            SetLargeSize => {
                self.value |= 2;
                go!(0xdcea, 16, StoreExtended)
            }
            StoreExtended => {
                publication = Some(Publication::ExtendedByte {
                    address: input.extended_pointer.wrapping_add(u16::from(self.index)),
                    value: self.value as u8,
                });
                go!(if self.large { 0xdcec } else { 0xdccc }, 46, Return)
            }
            Return => go!(if self.large { 0xdcee } else { 0xdcce }, 42, Complete),
            Complete => return None,
        };
        self.stage = next;
        let _ = pc;
        Some(Instruction {
            master_cycles,
            publication,
            pc: 0x06_0000 | pc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loaded_y_and_selected_shape_survive_restore_but_later_operands_remain_live() {
        let mut input = Inputs {
            world_y: 0x1234,
            pause: 0,
            state: 9,
            draw_work: 0,
            scroll_y: 0x1200,
            flags2: 1,
            flags3: 0,
            x: 0x123,
            y: 0,
            flags: 0xb7,
            oam_pointer: 0x800,
            extended_pointer: 0xa20,
        };
        let mut work = ShadowWork::new(10);
        loop {
            let retired = work.step(input).unwrap();
            match retired.publication {
                Some(Publication::YLow(value)) => input.y = (input.y & 0xff00) | u16::from(value),
                Some(Publication::YHigh(value)) => {
                    input.y = (input.y & 0xff) | (u16::from(value) << 8)
                }
                Some(Publication::Y(value)) => input.y = value,
                _ => {}
            }
            if retired.pc == 0x06_dcd1 {
                break;
            }
        }
        // Suspend after LDA Y / INY but before the ordinary OAM Y store.
        work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
        input.y = 0xee;
        input.flags3 = 0x20;
        input.oam_pointer = 0x900;
        assert_eq!(
            work.step(input).unwrap().publication,
            Some(Publication::OamByte {
                address: 0x905,
                value: 0x3e
            })
        );
        while !work.at_extended_tail() {
            let retired = work.step(input).unwrap();
            assert!(!matches!(
                retired.publication,
                Some(Publication::ExtendedByte { .. })
            ));
        }
        // The source boundary at DCE2 has not loaded X high or the extended
        // destination. Those reads belong to the resumed work, not its prefix.
        work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
        input.x = 0;
        let mut remaining_cycles = 0;
        let mut extended = Vec::new();
        while let Some(retired) = work.step(input) {
            remaining_cycles += retired.master_cycles;
            if retired.pc == 0x06_dce6 {
                input.x = 0x100; // The already-loaded value must remain zero.
                input.extended_pointer = 0xa40;
            }
            if let Some(Publication::ExtendedByte { address, value }) = retired.publication {
                extended.push((address, value));
            }
            work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
        }
        assert_eq!(extended, [(0xa41, 2)]);
        assert_eq!(remaining_cycles, 200); // Includes the final RTS.
        assert!(work.step(input).is_none());
    }
}
