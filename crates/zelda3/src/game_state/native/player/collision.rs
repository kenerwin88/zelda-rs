//! Player probe geometry and collision order, independent of WRAM publication.

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CollisionAxis {
    Horizontal,
    Vertical,
}

impl CollisionAxis {
    pub(crate) fn blocked_by_slope(self, flags: u8) -> bool {
        let mask = match self {
            Self::Horizontal => 0x10,
            Self::Vertical => 0x20,
        };
        flags & mask != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CollisionDirection {
    North,
    South,
    West,
    East,
}

impl CollisionDirection {
    pub(crate) fn from_legacy(value: u8) -> Self {
        match value {
            0 => Self::North,
            1 => Self::South,
            2 => Self::West,
            3 => Self::East,
            _ => panic!("invalid player collision direction {value}"),
        }
    }

    pub(crate) fn legacy_value(self) -> u8 {
        match self {
            Self::North => 0,
            Self::South => 1,
            Self::West => 2,
            Self::East => 3,
        }
    }

    pub(crate) fn along(axis: CollisionAxis, negative: bool) -> Self {
        match (axis, negative) {
            (CollisionAxis::Vertical, true) => Self::North,
            (CollisionAxis::Vertical, false) => Self::South,
            (CollisionAxis::Horizontal, true) => Self::West,
            (CollisionAxis::Horizontal, false) => Self::East,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MovementProbeKind {
    Cardinal,
    Slope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProbePoint {
    pub(crate) x: u16,
    pub(crate) y: u16,
}

/// Ordered low-side, center, high-side samples (slopes omit the center).
/// Positions remain pixels; tile addressing belongs to the execution adapter.
pub(crate) struct MovementProbe {
    points: [ProbePoint; 3],
    count: usize,
}

impl MovementProbe {
    pub(crate) fn new(
        x: u16,
        y: u16,
        axis: CollisionAxis,
        direction: CollisionDirection,
        kind: MovementProbeKind,
    ) -> Self {
        use CollisionDirection::*;
        // The cardinal footprint is x=0..15, y=8..23. The south and
        // east leading edges, and slope reach, intentionally differ.
        let (cardinal_edge, slope_edge, low, center, high) = match direction {
            North => (8, 7, 0, 8, 15),
            South => (24, 24, 0, 8, 15),
            West => (0, -1, 8, 16, 23),
            East => (15, 16, 8, 16, 23),
        };
        let (edge, sides, count) = match kind {
            MovementProbeKind::Cardinal => (cardinal_edge, [low, center, high], 3),
            MovementProbeKind::Slope => (slope_edge, [low, high, high], 2),
        };
        // Keep axis and direction independent: legacy ledge callers can use
        // an orthogonal direction's offsets for an explicitly selected axis.
        let points = sides.map(|side| match axis {
            CollisionAxis::Vertical => ProbePoint {
                x: x.wrapping_add(side),
                y: y.wrapping_add(edge as u16),
            },
            CollisionAxis::Horizontal => ProbePoint {
                x: x.wrapping_add(edge as u16),
                y: y.wrapping_add(side),
            },
        });
        Self { points, count }
    }

    pub(crate) fn points(&self) -> &[ProbePoint] {
        &self.points[..self.count]
    }
}

/// Footprint query order also defines the accumulated result bits.
#[derive(Clone, Copy, Debug)]
pub(crate) enum FootprintCorner {
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
}

impl FootprintCorner {
    pub(crate) const ORDER: [Self; 4] = [
        Self::TopLeft,
        Self::BottomLeft,
        Self::TopRight,
        Self::BottomRight,
    ];

    pub(crate) const fn result_bit(self) -> u16 {
        match self {
            Self::TopLeft => 8,
            Self::BottomLeft => 2,
            Self::TopRight => 4,
            Self::BottomRight => 1,
        }
    }

    fn point(self, left: u16, right: u16, top: u16, bottom: u16) -> ProbePoint {
        match self {
            Self::TopLeft => ProbePoint { x: left, y: top },
            Self::BottomLeft => ProbePoint { x: left, y: bottom },
            Self::TopRight => ProbePoint { x: right, y: top },
            Self::BottomRight => ProbePoint {
                x: right,
                y: bottom,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PlayerFootprint {
    Body,
    MirrorClearance,
}

impl PlayerFootprint {
    pub(crate) fn corners(self, x: u16, y: u16) -> [(FootprintCorner, ProbePoint); 4] {
        let inset = match self {
            Self::Body => 0,
            Self::MirrorClearance => 2,
        };
        FootprintCorner::ORDER.map(|corner| {
            (
                corner,
                corner.point(
                    x.wrapping_add(inset),
                    x.wrapping_add(15 - inset),
                    y.wrapping_add(8 + inset),
                    y.wrapping_add(23 - inset),
                ),
            )
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CollisionOrder {
    VerticalFirst,
    HorizontalFirst,
}

impl CollisionOrder {
    pub(crate) fn axes(self) -> [CollisionAxis; 2] {
        use CollisionAxis::*;
        match self {
            Self::VerticalFirst => [Vertical, Horizontal],
            Self::HorizontalFirst => [Horizontal, Vertical],
        }
    }

    pub(crate) fn for_moving_floor(bits: u8, x_velocity: u8, y_velocity: u8, floor_y: i8) -> Self {
        use FootprintCorner::*;
        const TOP: u8 = (TopLeft.result_bit() | TopRight.result_bit()) as u8;
        const BOTTOM: u8 = (BottomLeft.result_bit() | BottomRight.result_bit()) as u8;
        const LEFT: u8 = (TopLeft.result_bit() | BottomLeft.result_bit()) as u8;
        const RIGHT: u8 = (TopRight.result_bit() | BottomRight.result_bit()) as u8;
        let horizontal_first = match bits {
            TOP | BOTTOM => false,
            LEFT | RIGHT => true,
            _ if bits & (TOP | BOTTOM) == 0 => false,
            _ if y_velocity != 0 => true,
            _ if x_velocity == 0 => false,
            _ => floor_y >= 0,
        };
        if horizontal_first {
            Self::HorizontalFirst
        } else {
            Self::VerticalFirst
        }
    }
}
