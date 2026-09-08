//! Native fixed-point movement. Cartridge addresses belong to compatibility.

use super::PlayerMovementState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayerAxis {
    X,
    Y,
    Z,
}

impl PlayerAxis {
    /// Encode only when retaining a continuation in the compatibility scheduler.
    pub(crate) fn rom_pass(self) -> u8 {
        match self {
            Self::Y => 0,
            Self::X => 2,
            Self::Z => 4,
        }
    }

    /// Decode the X register at the ROM timing boundary, never in gameplay.
    pub(crate) fn from_rom_pass(pass: u8) -> Self {
        match pass {
            0 => Self::Y,
            2 => Self::X,
            4 => Self::Z,
            _ => panic!("invalid Link_MovePosition ROM pass {pass}"),
        }
    }
}

/// A coordinate with eight fractional bits. Z's fraction is supplied by the
/// compatibility boundary because its byte also belongs to an attract timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PlayerPosition {
    pub(super) coordinate: u16,
    pub(super) subpixel: u8,
}

impl PlayerPosition {
    pub(super) fn velocity_delta(velocity: u8) -> i16 {
        i16::from(velocity as i8) << 4
    }

    /// Advance the fraction only, retaining the signed carry for a later store.
    pub(super) fn advance_subpixel(&mut self, delta: i16) -> u16 {
        let sum = i32::from(self.subpixel) + i32::from(delta);
        self.subpixel = sum as u8;
        (sum >> 8) as u16
    }

    pub(super) fn advance(&mut self, delta: i16) {
        let carry = self.advance_subpixel(delta);
        self.apply_pixel_delta(carry);
    }

    pub(super) fn apply_pixel_delta(&mut self, delta: u16) {
        self.coordinate = self.coordinate.wrapping_add(delta);
    }

    /// Retain the old high byte until the suspended high-byte store resumes.
    pub(super) fn apply_pixel_delta_low(&mut self, delta: u16) -> u8 {
        let completed = self.coordinate.wrapping_add(delta);
        self.coordinate = (self.coordinate & 0xff00) | (completed & 0xff);
        (completed >> 8) as u8
    }

    pub(super) fn apply_coordinate_high(&mut self, high: u8) {
        self.coordinate = (self.coordinate & 0xff) | (u16::from(high) << 8);
    }
}

impl PlayerMovementState {
    pub(super) fn position(&self, axis: PlayerAxis, shared_z_subpixel: u8) -> PlayerPosition {
        let (coordinate, subpixel) = match axis {
            PlayerAxis::X => (self.x, self.x_subpixel),
            PlayerAxis::Y => (self.y, self.y_subpixel),
            PlayerAxis::Z => (self.z, shared_z_subpixel),
        };
        PlayerPosition {
            coordinate,
            subpixel,
        }
    }

    pub(super) fn set_axis_position(&mut self, axis: PlayerAxis, position: PlayerPosition) {
        match axis {
            PlayerAxis::X => self.set_x_with_subpixel(position.coordinate, position.subpixel),
            PlayerAxis::Y => self.set_y_with_subpixel(position.coordinate, position.subpixel),
            PlayerAxis::Z => self.set_z(position.coordinate),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_movement_wraps_at_coordinate_boundaries() {
        for (coordinate, subpixel, delta, expected_coordinate, expected_subpixel) in [
            (0xffff, 0xf8, 16, 0, 8),
            (0, 8, -16, 0xffff, 0xf8),
            (0, 0, i16::MIN, 0xff80, 0),
            (0xffff, 0xff, i16::MAX, 0x007f, 0xfe),
        ] {
            let mut position = PlayerPosition {
                coordinate,
                subpixel,
            };
            position.advance(delta);
            assert_eq!(
                position,
                PlayerPosition {
                    coordinate: expected_coordinate,
                    subpixel: expected_subpixel,
                }
            );
        }
        assert_eq!(PlayerPosition::velocity_delta(0x80), -2048);
        assert_eq!(PlayerPosition::velocity_delta(0xff), -16);
        assert_eq!(PlayerPosition::velocity_delta(0x7f), 2032);
    }

    #[test]
    fn split_coordinate_store_preserves_intervening_low_byte() {
        let mut position = PlayerPosition {
            coordinate: 0x12ff,
            subpixel: 0xf8,
        };
        let delta = position.advance_subpixel(16);
        assert_eq!(position.coordinate, 0x12ff);
        assert_eq!(position.subpixel, 8);
        let high = position.apply_pixel_delta_low(delta);
        assert_eq!(position.coordinate, 0x1200);
        assert_eq!(high, 0x13);
        position.coordinate = 0x1203;
        position.apply_coordinate_high(high);
        assert_eq!(position.coordinate, 0x1303);
    }
}
