use super::*;
use sha2::{Digest, Sha256};

#[test]
fn player_motion_matches_frozen_byte_publication() {
    let mut cases = String::new();
    for seed in 0..32u32 {
        let mut rng = seed + 1;
        let mut ram = vec![0; WRAM_SIZE];
        for byte in &mut ram {
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            *byte = (rng >> 24) as u8;
        }
        let mut player = FollowerLinkState::load_from_ram(&ram);
        let mut digest = Sha256::new();
        for (axis, coordinate) in [
            (PlayerAxis::Y, LINK_Y_COORD),
            (PlayerAxis::X, LINK_X_COORD),
            (PlayerAxis::Z, LINK_Z_COORD),
        ] {
            for velocity in [0, 1, 15, 16, 127, 128, 240, 255u8] {
                let value = {
                    let mut bridge = NativeFollowerLinkBridgeMut::new(&mut player, &mut ram);
                    match axis {
                        PlayerAxis::Y => bridge.move_y_by_velocity(velocity),
                        PlayerAxis::X => bridge.move_x_by_velocity(velocity),
                        PlayerAxis::Z => bridge.move_z_by_velocity(velocity),
                    }
                };
                digest.update(value.to_le_bytes());
                digest.update(&ram);
                let pending = NativeFollowerLinkBridgeMut::new(&mut player, &mut ram)
                    .move_axis_subpixel_only_by_velocity(axis, velocity);
                digest.update(pending.to_le_bytes());
                digest.update(&ram);
                // A suspended operation must observe intervening compatibility
                // writes, while retaining the delta computed before suspension.
                ram[coordinate + 1] ^= 0x80;
                let high = NativeFollowerLinkBridgeMut::new(&mut player, &mut ram)
                    .apply_axis_pixel_delta_low(axis, pending);
                digest.update([high]);
                digest.update(&ram);
                ram[coordinate] = ram[coordinate].wrapping_add(3);
                ram[LINK_Z_SUBPIXEL] ^= 0x5a;
                let value = NativeFollowerLinkBridgeMut::new(&mut player, &mut ram)
                    .apply_axis_coordinate_high(axis, high);
                digest.update(value.to_le_bytes());
                digest.update(&ram);
                let value = NativeFollowerLinkBridgeMut::new(&mut player, &mut ram)
                    .apply_axis_pixel_delta(axis, 0xff80);
                digest.update(value.to_le_bytes());
                digest.update(&ram);
            }
        }
        for delta in [0, 1, 255, 256, 32767, 32768, 65535] {
            let mut bridge = NativeFollowerLinkBridgeMut::new(&mut player, &mut ram);
            digest.update(bridge.move_x_by_subpixel_delta(delta).to_le_bytes());
            digest.update(bridge.move_y_by_subpixel_delta(delta).to_le_bytes());
            digest.update(&ram);
        }
        let mut projected = vec![0xa5; WRAM_SIZE];
        player.write_to_ram(&mut projected);
        digest.update(&projected);
        cases.push_str(&format!("{seed}: {:x}\n", digest.finalize()));
    }
    assert_eq!(
        cases,
        include_str!("../../../../testdata/player-motion-69183423.txt")
    );
}
