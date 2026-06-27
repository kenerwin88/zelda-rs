use crate::modern_frame::{ModernFrame, MODERN_FRAME_HEIGHT, MODERN_FRAME_WIDTH};

pub fn render_modern_frame_software(
    frame: &ModernFrame,
    atlas_rgba: &[u8],
    atlas_width: u16,
    atlas_height: u16,
) -> Vec<u8> {
    let mut out = vec![0u8; usize::from(MODERN_FRAME_WIDTH) * usize::from(MODERN_FRAME_HEIGHT) * 4];
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&frame.backdrop_color_rgba);
    }
    if frame.forced_blank {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return out;
    }
    for layer in &frame.bg_layers {
        if !layer.enabled_main {
            continue;
        }
        for tile in &layer.tiles {
            if tile.screen_width_px == 0 || tile.screen_height_px == 0 {
                continue; // degenerate footprint — nothing to draw
            }
            // Downsample factor from the (upscaled) atlas source rect to the
            // on-screen footprint. Nearest: sample the top-left of each block.
            let scale_x = tile.atlas_width_px / tile.screen_width_px;
            let scale_y = tile.atlas_height_px / tile.screen_height_px;
            for y in 0..tile.screen_height_px {
                for x in 0..tile.screen_width_px {
                    // Mirror on the SCREEN coordinate, then scale up into the source.
                    let src_x = if tile.hflip {
                        tile.screen_width_px - 1 - x
                    } else {
                        x
                    };
                    let src_y = if tile.vflip {
                        tile.screen_height_px - 1 - y
                    } else {
                        y
                    };
                    let atlas_x =
                        u32::from(tile.atlas_x_px) + u32::from(src_x) * u32::from(scale_x);
                    let atlas_y =
                        u32::from(tile.atlas_y_px) + u32::from(src_y) * u32::from(scale_y);
                    if atlas_x >= u32::from(atlas_width) || atlas_y >= u32::from(atlas_height) {
                        continue;
                    }
                    let dst_x = tile.screen_x + x as i16;
                    let dst_y = tile.screen_y + y as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue;
                    }
                    let src = (atlas_y as usize * usize::from(atlas_width) + atlas_x as usize) * 4;
                    let dst = (dst_y as usize * 256 + dst_x as usize) * 4;
                    if tile.transparent_color_zero && atlas_rgba[src + 3] == 0 {
                        continue;
                    }
                    out[dst..dst + 4].copy_from_slice(&atlas_rgba[src..src + 4]);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_frame::{ModernBgLayer, ModernBlendMode, ModernFrame, ModernTileInstance};

    #[test]
    fn software_renderer_blits_one_opaque_tile_from_atlas() {
        let mut atlas = vec![0u8; 8 * 8 * 4];
        for px in atlas.chunks_exact_mut(4) {
            px.copy_from_slice(&[10, 20, 30, 0xff]);
        }
        let mut frame = ModernFrame::empty();
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.blend_mode = ModernBlendMode::Opaque;
        layer.tiles.push(ModernTileInstance {
            atlas_id: 0,
            atlas_x_px: 0,
            atlas_y_px: 0,
            atlas_width_px: 8,
            atlas_height_px: 8,
            screen_width_px: 8,
            screen_height_px: 8,
            screen_x: 4,
            screen_y: 5,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            transparent_color_zero: false,
        });
        frame.bg_layers[0] = layer;

        let rgba = render_modern_frame_software(&frame, &atlas, 8, 8);
        let offset = ((5usize * 256) + 4usize) * 4;

        assert_eq!(&rgba[offset..offset + 4], &[10, 20, 30, 0xff]);
    }

    /// A pixel-distinct 8x8 pattern: color depends on BOTH x and y so the test
    /// catches scale being ignored, x/y swaps, and mirror mistakes.
    fn pattern_8x8(x: usize, y: usize) -> [u8; 4] {
        [(x as u8) * 30 + 5, (y as u8) * 30 + 7, 100, 0xff]
    }

    #[test]
    fn software_renderer_downsamples_scaled_atlas_tile() {
        const SCALE: usize = 4;
        const SRC: usize = 8 * SCALE; // 32
                                      // Build a 32x32 atlas that is a 4x nearest upscale of the 8x8 pattern.
        let mut atlas = vec![0u8; SRC * SRC * 4];
        for ay in 0..SRC {
            for ax in 0..SRC {
                let px = pattern_8x8(ax / SCALE, ay / SCALE);
                let o = (ay * SRC + ax) * 4;
                atlas[o..o + 4].copy_from_slice(&px);
            }
        }

        let mut frame = ModernFrame::empty();
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.blend_mode = ModernBlendMode::Opaque;
        layer.tiles.push(ModernTileInstance {
            atlas_id: 0,
            atlas_x_px: 0,
            atlas_y_px: 0,
            atlas_width_px: SRC as u16,
            atlas_height_px: SRC as u16,
            screen_width_px: 8,
            screen_height_px: 8,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            transparent_color_zero: false,
        });
        frame.bg_layers[0] = layer;

        let rgba = render_modern_frame_software(&frame, &atlas, SRC as u16, SRC as u16);

        // The 8x8 footprint at screen (0,0) must equal the original 8x8 pattern.
        for y in 0..8usize {
            for x in 0..8usize {
                let o = (y * 256 + x) * 4;
                assert_eq!(
                    &rgba[o..o + 4],
                    &pattern_8x8(x, y),
                    "screen pixel ({x},{y}) should equal downsampled pattern"
                );
            }
        }

        // Pixels just outside the 8x8 footprint must remain backdrop (not 32x32).
        let backdrop = frame.backdrop_color_rgba;
        for &(x, y) in &[(8usize, 0usize), (0, 8), (8, 8), (20, 20)] {
            let o = (y * 256 + x) * 4;
            assert_eq!(&rgba[o..o + 4], &backdrop, "({x},{y}) should be backdrop");
        }
    }
}
