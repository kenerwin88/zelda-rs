//! Source `gfx.cpp:SetupOBJ` range/time-over evaluation, independent of pixels.

pub(crate) const OBJ_LINES: usize = 239;
const TILE_LIMIT: i32 = 34;
const SPRITE_LIMIT: usize = 32;

/// Cumulative `GFX.OBJLines[Y].RTOFlags` for the pinned normal sprite limit.
pub(crate) fn range_time_flags(
    oam: &[u8],
    size_select: u8,
    first_sprite: u8,
    priority_rotation: bool,
    address: u16,
    flip: bool,
    interlace_obj: bool,
    odd_field: bool,
) -> Vec<u8> {
    assert_eq!(oam.len(), 0x220);
    let ((small_width, small_height), (large_width, large_height)) = match size_select & 7 {
        0 => ((8, 8), (16, 16)),
        1 => ((8, 8), (32, 32)),
        2 => ((8, 8), (64, 64)),
        3 => ((16, 16), (32, 32)),
        4 => ((16, 16), (64, 64)),
        5 => ((32, 32), (64, 64)),
        6 => ((16, 32), (32, 64)),
        _ => ((16, 32), (32, 32)),
    };
    let rotated_by_line = priority_rotation && flip && address & 1 != 0;
    let mut lines = vec![0u8; OBJ_LINES];
    let mut sprite_count = [0u8; OBJ_LINES];
    let mut tiles_left = [TILE_LIMIT; OBJ_LINES];
    let mut on_line = [[false; 128]; OBJ_LINES];
    let increment = if interlace_obj { 2 } else { 1 };
    let start_line = usize::from(interlace_obj && odd_field);

    for offset in 0..128 {
        let sprite = if rotated_by_line {
            offset
        } else {
            (usize::from(first_sprite) + offset) & 127
        };
        let high = oam[0x200 + sprite / 4] >> ((sprite & 3) * 2);
        let large = high & 2 != 0;
        let (width, height) = if large {
            (large_width, large_height)
        } else {
            (small_width, small_height)
        };
        let mut x = i32::from(oam[sprite * 4]) - if high & 1 != 0 { 256 } else { 0 };
        if x == -256 {
            x = if rotated_by_line { 256 } else { 0 };
        }
        if x <= -width || x > 256 {
            continue;
        }
        let visible_tiles = if x < 0 {
            (width + x + 7) >> 3
        } else if x + width > if rotated_by_line { 256 } else { 255 } {
            ((if rotated_by_line { 257 } else { 256 }) - x + 7) >> 3
        } else {
            width >> 3
        };
        let y_start = usize::from(oam[sprite * 4 + 1]);
        for line in (start_line..height as usize).step_by(increment) {
            let y = (y_start + (line - start_line) / increment) & 255;
            if y >= OBJ_LINES {
                continue;
            }
            if rotated_by_line {
                on_line[y][sprite] = true;
            } else {
                if usize::from(sprite_count[y]) >= SPRITE_LIMIT {
                    lines[y] |= 0x40;
                    continue;
                }
                tiles_left[y] -= visible_tiles;
                if tiles_left[y] < 0 {
                    lines[y] |= 0x80;
                }
                sprite_count[y] += 1;
            }
        }
    }
    if rotated_by_line {
        for y in 0..OBJ_LINES {
            let start = (usize::from(first_sprite) + y) & 127;
            for offset in 0..128 {
                let sprite = (start + offset) & 127;
                if !on_line[y][sprite] {
                    continue;
                }
                if usize::from(sprite_count[y]) >= SPRITE_LIMIT {
                    lines[y] |= 0x40;
                    break;
                }
                let high = oam[0x200 + sprite / 4] >> ((sprite & 3) * 2);
                let width = if high & 2 != 0 {
                    large_width
                } else {
                    small_width
                };
                let mut x = i32::from(oam[sprite * 4]) - if high & 1 != 0 { 256 } else { 0 };
                if x == -256 {
                    x = 256;
                }
                let visible_tiles = if x < 0 {
                    (width + x + 7) >> 3
                } else if x + width >= 257 {
                    (257 - x + 7) >> 3
                } else {
                    width >> 3
                };
                tiles_left[y] -= visible_tiles;
                if tiles_left[y] < 0 {
                    lines[y] |= 0x80;
                }
                sprite_count[y] += 1;
            }
        }
    }
    for y in 1..OBJ_LINES {
        lines[y] |= lines[y - 1];
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::range_time_flags;

    #[test]
    fn separate_sprite_and_tile_overflow_accumulate_downscreen() {
        let mut oam = vec![0; 0x220];
        for sprite in 0..128 {
            oam[sprite * 4 + 1] = 240;
        }
        for sprite in 0..33 {
            oam[sprite * 4 + 1] = 5;
        }
        let flags = range_time_flags(&oam, 0, 0, false, 0, false, false, false);
        assert_eq!(flags[4], 0);
        assert_eq!(flags[5], 0x40);
        assert_eq!(flags[6], 0x40);

        for sprite in 0..5 {
            oam[0x200 + sprite / 4] |= 2 << ((sprite & 3) * 2);
            oam[sprite * 4 + 1] = 20;
        }
        let flags = range_time_flags(&oam, 2, 0, false, 0, false, false, false);
        assert_eq!(flags[19], 0);
        assert_eq!(flags[20], 0x80);
    }
}
