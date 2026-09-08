use super::*;

#[test]
fn probe_geometry_matches_pre_refactor_formulas_across_coordinate_wraps() {
    // Frozen from tile_detect.rs at 869fdb0c. In particular, slope -1
    // wraps before masking, and all four directions are valid on either axis.
    let cardinal = [8u16, 24, 0, 15];
    let slopes = [7i16, 24, -1, 16];
    let low = [0u16, 0, 8, 8];
    let center = [8u16, 8, 16, 16];
    let high = [15u16, 15, 23, 23];
    for x in 0..=u16::MAX {
        let y = x.rotate_left(7);
        for d in 0..4 {
            for axis in [CollisionAxis::Vertical, CollisionAxis::Horizontal] {
                for kind in [MovementProbeKind::Cardinal, MovementProbeKind::Slope] {
                    let (edge, sides) = match kind {
                        MovementProbeKind::Cardinal => {
                            (cardinal[d], vec![low[d], center[d], high[d]])
                        }
                        MovementProbeKind::Slope => (slopes[d] as u16, vec![low[d], high[d]]),
                    };
                    let probe = MovementProbe::new(
                        x,
                        y,
                        axis,
                        CollisionDirection::from_legacy(d as u8),
                        kind,
                    );
                    assert_eq!(probe.points().len(), sides.len());
                    for (point, side) in probe.points().iter().zip(sides) {
                        let expected = match axis {
                            CollisionAxis::Vertical => (x.wrapping_add(side), y.wrapping_add(edge)),
                            CollisionAxis::Horizontal => {
                                (x.wrapping_add(edge), y.wrapping_add(side))
                            }
                        };
                        assert_eq!((point.x, point.y), expected);
                    }
                }
            }
        }
    }
}

#[test]
fn moving_floor_order_matches_pre_refactor_decision_for_all_masks() {
    // Frozen decision from link_handle_cardinal_collision at 869fdb0c.
    // High bits must not be discarded before the exact-mask comparisons.
    for bits in 0..=u8::MAX {
        for x in [0, 1, 0xff] {
            for y in [0, 1, 0xff] {
                for floor in i8::MIN..=i8::MAX {
                    let horizontal = if bits == 12 || bits == 3 {
                        false
                    } else if bits == 10 || bits == 5 {
                        true
                    } else if (bits & 0x0c) == 0 && (bits & 3) == 0 {
                        false
                    } else if y != 0 {
                        true
                    } else if x == 0 {
                        false
                    } else {
                        floor >= 0
                    };
                    assert_eq!(
                        CollisionOrder::for_moving_floor(bits, x, y, floor),
                        if horizontal {
                            CollisionOrder::HorizontalFirst
                        } else {
                            CollisionOrder::VerticalFirst
                        }
                    );
                }
            }
        }
    }
}
