//! Source-keyed HD overrides for the modern (off-VRAM) software renderer.
//!
//! HD art overrides a cell by its logical source key and is recolored through the
//! LIVE CGRAM every frame ("detail-modulate"): `final = live * (override / reference)`.
//! Art authored as `reference[cgram_idx]` gives `detail == 1` → exact parity; a
//! different HD color recolors while still tracking the runtime palette. Phase 1
//! samples at native 8×8 (nearest block top-left); Phase 2 will sample at N×.

use std::collections::HashMap;
use std::io::BufReader;
use std::path::Path;
use std::sync::OnceLock;

use serde::Deserialize;

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

    /// Sample the HD texel for output-local pixel `(out_local_x, out_local_y)` within a
    /// tile footprint of `footprint_px` output pixels (= 8·scale): `hd = out_local *
    /// width / footprint_px`. At `footprint_px == 8` this is the native 8×8 sampling
    /// (equals `sample_native`). Alpha dropped (transparency is the base slot index's).
    pub fn sample_scaled(&self, out_local_x: u32, out_local_y: u32, footprint_px: u32) -> [u8; 3] {
        let fp = footprint_px.max(1);
        let px = (out_local_x * self.width / fp).min(self.width.saturating_sub(1));
        let py = (out_local_y * self.height / fp).min(self.height.saturating_sub(1));
        let idx = ((py * self.width + px) * 4) as usize;
        [self.rgba[idx], self.rgba[idx + 1], self.rgba[idx + 2]]
    }
}

/// Precomputed `detail[hd][ref] = hd / max(ref, 1)` for every u8 pair — the exact same
/// float expression the per-pixel path used, hoisted out of the hot loop so the override
/// compositor does a table lookup instead of a division per channel per pixel. Built once
/// on first use (256×256 f32 = 256 KiB). Bit-identical to the inline division, so the
/// `detail == 1 → final == live` guarantee and every kernel test are preserved exactly.
fn detail_lut() -> &'static [[f32; 256]; 256] {
    static LUT: OnceLock<Box<[[f32; 256]; 256]>> = OnceLock::new();
    LUT.get_or_init(|| {
        let mut t = Box::new([[0.0f32; 256]; 256]);
        for hd in 0..256usize {
            for r in 0..256usize {
                t[hd][r] = hd as f32 / (r as f32).max(1.0);
            }
        }
        t
    })
}

/// `final = clamp(live * (hd / max(reference, 1)))` per RGB channel; alpha from `live`.
/// Reference is guarded away from 0 to avoid divide-by-zero on dark slots. Authoring HD
/// as `reference[idx]` yields detail 1 → `final == live` (exact parity). The per-channel
/// `hd / max(ref, 1)` comes from `detail_lut()` (precomputed), removing the division from
/// the per-pixel hot path with no change to the result.
pub fn detail_modulate(live: [u8; 4], hd: [u8; 3], reference: [u8; 3]) -> [u8; 4] {
    let lut = detail_lut();
    let mut out = [0u8, 0, 0, live[3]];
    for c in 0..3 {
        let detail = lut[hd[c] as usize][reference[c] as usize];
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
    footprint_px: u32,
) -> Option<[u8; 4]> {
    if base_index == 0 {
        return None;
    }
    match override_cell {
        Some(hd) => {
            let hd_rgb = hd.sample_scaled(lx, ly, footprint_px);
            let r = reference[cgram_idx];
            Some(detail_modulate(live_rgba, hd_rgb, [r[0], r[1], r[2]]))
        }
        None => Some(live_rgba),
    }
}

#[derive(Debug, Deserialize)]
struct ManifestJson {
    reference_palette: String,
    #[serde(default)]
    overrides: Vec<OverrideJson>,
}

#[derive(Debug, Deserialize)]
struct OverrideJson {
    /// Logical source key, hex string (`0x…`) as emitted by `--dump-assets-by-source`.
    key: String,
    /// RGBA PNG path relative to the manifest; dims are multiples of 8.
    rgba: String,
}

/// Source-keyed HD override store: `source_key → HdCell` plus the reference palette the
/// HD art was authored against. Loaded once; empty/absent → the modern renderer is
/// byte-identical to today.
#[derive(Debug, Clone)]
pub struct ModernHdOverrides {
    by_key: HashMap<u64, HdCell>,
    reference: [[u8; 4]; 256],
}

impl ModernHdOverrides {
    pub fn from_parts(by_key: HashMap<u64, HdCell>, reference: [[u8; 4]; 256]) -> Self {
        Self { by_key, reference }
    }

    /// Load from `ZELDA3_MODERN_HD_OVERRIDES=<manifest path>`. Unset → `None` (disabled).
    pub fn from_env() -> Option<Self> {
        let path = std::env::var_os("ZELDA3_MODERN_HD_OVERRIDES")?;
        Self::load_manifest(Path::new(&path))
    }

    /// Parse a manifest and decode its art. Returns `None` (overrides disabled) if the
    /// manifest is unreadable/invalid or the reference palette is missing / not 256 px —
    /// never mis-recolor against a bad reference. Individual bad `rgba` entries are
    /// skipped (logged); other overrides still load.
    pub fn load_manifest(path: &Path) -> Option<Self> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| eprintln!("ZELDA3_MODERN_HD_OVERRIDES read {}: {e}", path.display()))
            .ok()?;
        let manifest: ManifestJson = serde_json::from_str(&json)
            .map_err(|e| eprintln!("ZELDA3_MODERN_HD_OVERRIDES parse {}: {e}", path.display()))
            .ok()?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));

        let reference = decode_reference(&base.join(&manifest.reference_palette))?;

        let mut by_key = HashMap::new();
        for ovr in &manifest.overrides {
            let Some(key) = parse_key(&ovr.key) else {
                eprintln!("ZELDA3_MODERN_HD_OVERRIDES bad key {:?}; skipping", ovr.key);
                continue;
            };
            match decode_rgba_cell(&base.join(&ovr.rgba)) {
                Some(cell) => {
                    by_key.insert(key, cell);
                }
                None => eprintln!(
                    "ZELDA3_MODERN_HD_OVERRIDES bad rgba {}; skipping",
                    base.join(&ovr.rgba).display()
                ),
            }
        }
        Some(Self { by_key, reference })
    }

    pub fn get(&self, key: u64) -> Option<&HdCell> {
        if key == NO_SOURCE_KEY {
            return None;
        }
        self.by_key.get(&key)
    }

    pub fn reference(&self) -> &[[u8; 4]; 256] {
        &self.reference
    }

    pub fn is_enabled(&self) -> bool {
        !self.by_key.is_empty()
    }
}

fn parse_key(s: &str) -> Option<u64> {
    let t = s.trim();
    match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(hex) => u64::from_str_radix(hex, 16).ok(),
        None => t.parse::<u64>().ok(),
    }
}

/// Decode a `width×height` RGBA8 PNG (dims multiples of 8) into an `HdCell`.
fn decode_rgba_cell(path: &Path) -> Option<HdCell> {
    let (width, height, rgba) = decode_rgba_png(path)?;
    if width == 0 || height == 0 || width % 8 != 0 || height % 8 != 0 {
        eprintln!(
            "ZELDA3_MODERN_HD_OVERRIDES rgba {} dims {width}×{height} not multiples of 8",
            path.display()
        );
        return None;
    }
    Some(HdCell {
        width,
        height,
        rgba,
    })
}

/// Decode a 256×1 RGBA PNG into a `[[u8;4];256]` reference palette. `None` if not 256 px.
fn decode_reference(path: &Path) -> Option<[[u8; 4]; 256]> {
    let (width, height, rgba) = decode_rgba_png(path).or_else(|| {
        eprintln!(
            "ZELDA3_MODERN_HD_OVERRIDES reference {} unreadable",
            path.display()
        );
        None
    })?;
    if (width * height) as usize != 256 || rgba.len() != 256 * 4 {
        eprintln!(
            "ZELDA3_MODERN_HD_OVERRIDES reference {} must be 256 RGBA px (got {width}×{height})",
            path.display()
        );
        return None;
    }
    let mut out = [[0u8; 4]; 256];
    for (i, px) in out.iter_mut().enumerate() {
        px.copy_from_slice(&rgba[i * 4..i * 4 + 4]);
    }
    Some(out)
}

/// Decode any RGBA8 PNG to `(width, height, rgba)`.
fn decode_rgba_png(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let file = std::fs::File::open(path).ok()?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some((info.width, info.height, buf))
}

/// Render-time override context threaded through the compositor. `disabled()` (no store)
/// makes every resolve a no-op → byte-identical to today.
#[derive(Clone, Copy)]
pub struct HdOverrideCtx<'a> {
    store: Option<&'a ModernHdOverrides>,
}

static ZERO_REFERENCE: [[u8; 4]; 256] = [[0, 0, 0, 0xff]; 256];

impl<'a> HdOverrideCtx<'a> {
    pub fn disabled() -> Self {
        Self { store: None }
    }

    pub fn new(store: &'a ModernHdOverrides) -> Self {
        Self { store: Some(store) }
    }

    pub fn resolve(&self, source_key: u64) -> Option<&'a HdCell> {
        self.store.and_then(|s| s.get(source_key))
    }

    pub fn reference(&self) -> &[[u8; 4]; 256] {
        match self.store {
            Some(s) => s.reference(),
            None => &ZERO_REFERENCE,
        }
    }
}

/// Integer HD scale factor for the modern N× compositor. `ZELDA3_HD_SCALE`,
/// default 2, clamped to 1..=4 (the CPU 60fps ceiling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HdScale(u32);

impl HdScale {
    pub const DEFAULT: u32 = 2;

    pub fn from_env() -> Self {
        Self::from_str_opt(std::env::var("ZELDA3_HD_SCALE").ok().as_deref())
    }

    /// Testable core: parse/clamp an optional string.
    pub fn from_str_opt(s: Option<&str>) -> Self {
        let v = match s {
            None => Self::DEFAULT,
            Some(t) => match t.trim().parse::<u32>() {
                Ok(n) => n.clamp(1, 4),
                Err(_) => Self::DEFAULT,
            },
        };
        HdScale(v)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn write_png(path: &std::path::Path, width: u32, height: u32, rgba: &[u8]) {
        let file = std::fs::File::create(path).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), width, height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(rgba).unwrap();
    }

    fn unique_dir(tag: &str) -> PathBuf {
        // No Date/random needed: process id + tag is unique enough per test run.
        let dir =
            std::env::temp_dir().join(format!("zelda3_hd_ovr_{}_{}", std::process::id(), tag));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

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
    fn detail_lut_is_bit_identical_to_inline_division() {
        // The LUT optimization MUST NOT change any output pixel. Sweep hd/ref/live and
        // compare detail_modulate against the original inline-division formula.
        let inline = |live: u8, hd: u8, r: u8| -> u8 {
            let detail = hd as f32 / (r as f32).max(1.0);
            (live as f32 * detail).round().clamp(0.0, 255.0) as u8
        };
        for &hd in &[0u8, 1, 7, 31, 64, 127, 128, 200, 254, 255] {
            for &r in &[0u8, 1, 3, 16, 63, 100, 128, 200, 255] {
                for &live in &[0u8, 1, 15, 50, 128, 200, 255] {
                    let out = detail_modulate([live, live, live, 0xff], [hd, hd, hd], [r, r, r]);
                    let want = inline(live, hd, r);
                    assert_eq!(out[0], want, "hd={hd} ref={r} live={live}");
                }
            }
        }
    }

    #[test]
    fn sample_native_nearest_block_top_left() {
        // 16×16 HD cell (scale 2). Native (1,0) → HD (2,0).
        let mut rgba = vec![0u8; 16 * 16 * 4];
        let idx = ((0 * 16 + 2) * 4) as usize;
        rgba[idx..idx + 4].copy_from_slice(&[9, 8, 7, 0xff]);
        let cell = HdCell {
            width: 16,
            height: 16,
            rgba,
        };
        assert_eq!(cell.sample_native(1, 0), [9, 8, 7]);
    }

    #[test]
    fn resolve_transparent_when_base_index_zero() {
        let reference = [[0u8; 4]; 256];
        assert_eq!(
            resolve_pixel_color(0, 5, [1, 2, 3, 0xff], None, &reference, 0, 0, 8),
            None
        );
    }

    #[test]
    fn resolve_returns_live_without_override() {
        let reference = [[0u8; 4]; 256];
        assert_eq!(
            resolve_pixel_color(1, 5, [1, 2, 3, 0xff], None, &reference, 0, 0, 8),
            Some([1, 2, 3, 0xff])
        );
    }

    #[test]
    fn resolve_detail_modulates_with_override() {
        let mut reference = [[0u8; 4]; 256];
        reference[5] = [128, 128, 128, 0xff];
        let cell = HdCell {
            width: 8,
            height: 8,
            rgba: vec![64u8; 8 * 8 * 4],
        };
        // live 100 * (hd 64 / ref 128) = 50.
        assert_eq!(
            resolve_pixel_color(
                1,
                5,
                [100, 100, 100, 0xff],
                Some(&cell),
                &reference,
                0,
                0,
                8
            ),
            Some([50, 50, 50, 0xff])
        );
    }

    #[test]
    fn ctx_disabled_resolves_nothing() {
        let ctx = HdOverrideCtx::disabled();
        assert!(ctx.resolve(0x0600_0000_1234_0000).is_none());
        assert_eq!(ctx.reference(), &[[0u8, 0, 0, 0xff]; 256]);
    }

    #[test]
    fn store_get_ignores_no_source_key() {
        let mut by_key = HashMap::new();
        by_key.insert(
            NO_SOURCE_KEY,
            HdCell {
                width: 8,
                height: 8,
                rgba: vec![0; 256],
            },
        );
        let store = ModernHdOverrides::from_parts(by_key, [[0u8; 4]; 256]);
        assert!(store.get(NO_SOURCE_KEY).is_none());
    }

    #[test]
    fn load_manifest_decodes_overrides_and_reference() {
        let dir = unique_dir("load_ok");
        let mut ref_rgba = vec![0u8; 256 * 4];
        for i in 0..256 {
            ref_rgba[i * 4] = 0x80;
            ref_rgba[i * 4 + 1] = 0x80;
            ref_rgba[i * 4 + 2] = 0x80;
            ref_rgba[i * 4 + 3] = 0xff;
        }
        write_png(&dir.join("ref.png"), 256, 1, &ref_rgba);
        write_png(&dir.join("grass.png"), 8, 8, &vec![0x40u8; 8 * 8 * 4]);
        let manifest = dir.join("m.json");
        std::fs::write(
            &manifest,
            r#"{"reference_palette":"ref.png","overrides":[{"key":"0x0000000100000000","rgba":"grass.png"}]}"#,
        )
        .unwrap();

        let store = ModernHdOverrides::load_manifest(&manifest).unwrap();
        assert!(store.is_enabled());
        assert_eq!(store.reference()[5], [0x80, 0x80, 0x80, 0xff]);
        let cell = store.get(0x0000_0001_0000_0000).unwrap();
        assert_eq!((cell.width, cell.height), (8, 8));
    }

    #[test]
    fn load_manifest_disables_when_reference_missing() {
        let dir = unique_dir("no_ref");
        write_png(&dir.join("grass.png"), 8, 8, &vec![0x40u8; 8 * 8 * 4]);
        let manifest = dir.join("m.json");
        std::fs::write(
            &manifest,
            r#"{"reference_palette":"missing.png","overrides":[{"key":"0x1","rgba":"grass.png"}]}"#,
        )
        .unwrap();

        // Reference missing/unreadable → overrides disabled entirely (returns None).
        assert!(ModernHdOverrides::load_manifest(&manifest).is_none());
    }

    #[test]
    fn end_to_end_manifest_load_resolves_override_cell() {
        let dir = unique_dir("e2e");
        write_png(&dir.join("ref.png"), 256, 1, &vec![0x80u8; 256 * 4]);
        // 16×16 (2×) HD cell, solid color 0x40.
        write_png(&dir.join("hd.png"), 16, 16, &vec![0x40u8; 16 * 16 * 4]);
        let manifest = dir.join("m.json");
        std::fs::write(
            &manifest,
            r#"{"reference_palette":"ref.png","overrides":[{"key":"0x00000002abcd0000","rgba":"hd.png"}]}"#,
        )
        .unwrap();

        let store = ModernHdOverrides::load_manifest(&manifest).unwrap();
        let ctx = HdOverrideCtx::new(&store);
        let cell = ctx.resolve(0x0000_0002_abcd_0000).unwrap();
        assert_eq!((cell.width, cell.height), (16, 16));
        assert_eq!(cell.sample_native(3, 3), [0x40, 0x40, 0x40]);
        assert!(ctx.resolve(NO_SOURCE_KEY).is_none());
    }

    #[test]
    fn hd_scale_parses_and_clamps() {
        assert_eq!(HdScale::from_str_opt(None).get(), 2); // default
        assert_eq!(HdScale::from_str_opt(Some("1")).get(), 1);
        assert_eq!(HdScale::from_str_opt(Some("4")).get(), 4);
        assert_eq!(HdScale::from_str_opt(Some("0")).get(), 1); // clamp low
        assert_eq!(HdScale::from_str_opt(Some("9")).get(), 4); // clamp high
        assert_eq!(HdScale::from_str_opt(Some("xyz")).get(), 2); // invalid → default
    }

    #[test]
    fn sample_scaled_maps_output_pixel_to_hd_texel() {
        // 16×16 HD cell (M=16). footprint 16 (scale 2): 1:1 mapping.
        let mut rgba = vec![0u8; 16 * 16 * 4];
        let put = |r: &mut Vec<u8>, x: usize, y: usize, c: [u8; 4]| {
            let i = (y * 16 + x) * 4;
            r[i..i + 4].copy_from_slice(&c);
        };
        put(&mut rgba, 9, 3, [7, 8, 9, 0xff]);
        let cell = HdCell {
            width: 16,
            height: 16,
            rgba,
        };
        assert_eq!(cell.sample_scaled(9, 3, 16), [7, 8, 9]); // 9*16/16=9, 3*16/16=3
                                                             // footprint 8 (scale 1) == native top-left of the whole cell region.
        assert_eq!(cell.sample_scaled(0, 0, 8), cell.sample_native(0, 0));
    }

    #[test]
    fn resolve_pixel_color_footprint_8_matches_phase1() {
        let mut reference = [[0u8; 4]; 256];
        reference[5] = [128, 128, 128, 0xff];
        let cell = HdCell {
            width: 8,
            height: 8,
            rgba: vec![64u8; 8 * 8 * 4],
        };
        // Same as the Phase 1 test, now with explicit footprint 8.
        assert_eq!(
            resolve_pixel_color(
                1,
                5,
                [100, 100, 100, 0xff],
                Some(&cell),
                &reference,
                0,
                0,
                8
            ),
            Some([50, 50, 50, 0xff])
        );
    }
}
