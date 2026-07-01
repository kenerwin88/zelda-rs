//! Source-keyed HD overrides for the modern (off-VRAM) software renderer.
//!
//! HD art overrides a cell by its logical source key and is recolored through the
//! LIVE CGRAM every frame ("detail-modulate"): `final = live * (override / reference)`.
//! Art authored as `reference[cgram_idx]` gives `detail == 1` → exact parity; a
//! different HD color recolors while still tracking the runtime palette. Phase 1
//! samples at native 8×8 (nearest block top-left); Phase 2 will sample at N×.

/// Sentinel for a cell with no atlas source key (live-VRAM-decoded animation cells,
/// test cells): never has an override.
pub const NO_SOURCE_KEY: u64 = 0;

/// A decoded HD override image for one 8×8 logical cell. `width`/`height` are multiples
/// of 8 (an N× upscale; 8×8 == 1×). `rgba` is row-major RGBA8, `width*height*4` bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HdCell {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl HdCell {
    /// Sample the color for native 8×8 pixel `(lx, ly)` (each 0..8) as the top-left of
    /// its N× block — nearest downsample, matching the atlas convention. Alpha is
    /// dropped: transparency is decided by the base cell's slot index, not HD alpha.
    pub fn sample_native(&self, lx: u32, ly: u32) -> [u8; 3] {
        let scale_x = (self.width / 8).max(1);
        let scale_y = (self.height / 8).max(1);
        let px = (lx * scale_x).min(self.width.saturating_sub(1));
        let py = (ly * scale_y).min(self.height.saturating_sub(1));
        let idx = ((py * self.width + px) * 4) as usize;
        [self.rgba[idx], self.rgba[idx + 1], self.rgba[idx + 2]]
    }
}

/// `final = clamp(live * (hd / max(reference, 1)))` per RGB channel; alpha from `live`.
/// Reference is guarded away from 0 to avoid divide-by-zero on dark slots. Authoring HD
/// as `reference[idx]` yields detail 1 → `final == live` (exact parity).
pub fn detail_modulate(live: [u8; 4], hd: [u8; 3], reference: [u8; 3]) -> [u8; 4] {
    let mut out = [0u8, 0, 0, live[3]];
    for c in 0..3 {
        let detail = hd[c] as f32 / (reference[c] as f32).max(1.0);
        out[c] = (live[c] as f32 * detail).round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// Resolve the final RGBA for one cell pixel. `None` = transparent (base slot index 0;
/// HD art never changes tile shape). With an override, detail-modulate; without,
/// return the live color unchanged (byte-identical to the direct CGRAM lookup).
#[allow(clippy::too_many_arguments)]
pub fn resolve_pixel_color(
    base_index: u8,
    cgram_idx: usize,
    live_rgba: [u8; 4],
    override_cell: Option<&HdCell>,
    reference: &[[u8; 4]; 256],
    lx: u32,
    ly: u32,
) -> Option<[u8; 4]> {
    if base_index == 0 {
        return None;
    }
    match override_cell {
        Some(hd) => {
            let hd_rgb = hd.sample_native(lx, ly);
            let r = reference[cgram_idx];
            Some(detail_modulate(live_rgba, hd_rgb, [r[0], r[1], r[2]]))
        }
        None => Some(live_rgba),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detail_one_identity_returns_live() {
        // hd == reference → detail 1 → final == live (exact, CPU math).
        let out = detail_modulate([200, 100, 50, 0xff], [128, 128, 128], [128, 128, 128]);
        assert_eq!(out, [200, 100, 50, 0xff]);
    }

    #[test]
    fn detail_modulate_recolors_by_ratio() {
        // live 100 * (hd 64 / ref 128) = 50.
        let out = detail_modulate([100, 100, 100, 0xff], [64, 64, 64], [128, 128, 128]);
        assert_eq!(out, [50, 50, 50, 0xff]);
    }

    #[test]
    fn detail_modulate_clamps_and_guards_zero_reference() {
        // ref 0 → guarded to 1; huge detail clamps to 255. alpha preserved from live.
        let out = detail_modulate([200, 0, 0, 0xff], [255, 0, 0], [0, 0, 0]);
        assert_eq!(out, [255, 0, 0, 0xff]);
    }

    #[test]
    fn sample_native_nearest_block_top_left() {
        // 16×16 HD cell (scale 2). Native (1,0) → HD (2,0).
        let mut rgba = vec![0u8; 16 * 16 * 4];
        let idx = ((0 * 16 + 2) * 4) as usize;
        rgba[idx..idx + 4].copy_from_slice(&[9, 8, 7, 0xff]);
        let cell = HdCell { width: 16, height: 16, rgba };
        assert_eq!(cell.sample_native(1, 0), [9, 8, 7]);
    }

    #[test]
    fn resolve_transparent_when_base_index_zero() {
        let reference = [[0u8; 4]; 256];
        assert_eq!(
            resolve_pixel_color(0, 5, [1, 2, 3, 0xff], None, &reference, 0, 0),
            None
        );
    }

    #[test]
    fn resolve_returns_live_without_override() {
        let reference = [[0u8; 4]; 256];
        assert_eq!(
            resolve_pixel_color(1, 5, [1, 2, 3, 0xff], None, &reference, 0, 0),
            Some([1, 2, 3, 0xff])
        );
    }

    #[test]
    fn resolve_detail_modulates_with_override() {
        let mut reference = [[0u8; 4]; 256];
        reference[5] = [128, 128, 128, 0xff];
        let cell = HdCell { width: 8, height: 8, rgba: vec![64u8; 8 * 8 * 4] };
        // live 100 * (hd 64 / ref 128) = 50.
        assert_eq!(
            resolve_pixel_color(1, 5, [100, 100, 100, 0xff], Some(&cell), &reference, 0, 0),
            Some([50, 50, 50, 0xff])
        );
    }
}
