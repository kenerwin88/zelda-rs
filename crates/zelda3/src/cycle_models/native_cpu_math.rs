//! Retained CPU multiply/divide registers shared by main and poly work.
//! Bus semantics follow the pinned source device, independently of CPU callers.
use super::intro_clear::Access;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeCpuMath {
    pub(crate) multiplicand: u8,
    pub(crate) dividend: u16,
    pub(crate) quotient: u16,
    pub(crate) remainder: u16,
}
impl NativeCpuMath {
    pub(crate) fn snes9x_reset() -> Self {
        // Pinned ppu.cpp:S9xSoftResetPPU clears FillRAM[$4200..$42ff].
        // The legacy C-port Snes::reset arithmetic seed is different.
        Self::default()
    }

    pub(crate) fn access(&mut self, access: Access) -> Option<u16> {
        match access {
            Access::Read(a, 1) if (0x4214..=0x4217).contains(&a) => {
                let value = if a < 0x4216 {
                    self.quotient
                } else {
                    self.remainder
                };
                Some((value >> ((a & 1) * 8)) & 255)
            }
            Access::Write(a, value, 1) if (0x4202..=0x4206).contains(&a) => {
                match a {
                    0x4202 => self.multiplicand = value as u8,
                    0x4203 => self.remainder = u16::from(self.multiplicand) * (value & 255),
                    0x4204 => self.dividend = (self.dividend & 0xff00) | (value & 255),
                    0x4205 => self.dividend = (self.dividend & 255) | (value << 8),
                    0x4206 => {
                        (self.quotient, self.remainder) = if value == 0 {
                            (65535, self.dividend)
                        } else {
                            (self.dividend / value, self.dividend % value)
                        };
                    }
                    _ => unreachable!(),
                }
                Some(0)
            }
            _ => None,
        }
    }
}
