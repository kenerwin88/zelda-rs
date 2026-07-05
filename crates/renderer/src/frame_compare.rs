/// First differing pixel and mismatch count between two rendered frame buffers.
///
/// Coordinates use the SNES frame width used by the existing render diagnostics
/// (256 pixels). The compared buffers may contain more pixels, but the first
/// mismatch is reported in that coordinate space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuRenderDiff {
    pub mismatched_pixels: usize,
    pub first_x: usize,
    pub first_y: usize,
    pub cpu_rgb: (u8, u8, u8),
    pub gpu_rgb: (u8, u8, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuRenderComparison {
    pub cpu_hash: u32,
    pub gpu_hash: u32,
    pub diff: Option<GpuRenderDiff>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuRenderFrameComparison {
    comparison: GpuRenderComparison,
    divergence_line: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFrameHashReport {
    pub hash: u32,
    pub line: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderHashPair {
    pub cpu_hash: u32,
    pub gpu_hash: u32,
}

impl GpuRenderFrameComparison {
    pub fn diff(&self) -> Option<GpuRenderDiff> {
        self.comparison.diff
    }

    pub fn divergence_line(&self) -> Option<&str> {
        self.divergence_line.as_deref()
    }

    pub fn cpu_hash(&self) -> u32 {
        self.comparison.cpu_hash
    }
}

pub(crate) fn render_frame_rgb_hash_rgba(frame: &[u8]) -> u32 {
    let mut hash = 2166136261u32;
    for pixel in frame.chunks_exact(4) {
        hash = (hash ^ u32::from(pixel[0])).wrapping_mul(16777619); // R
        hash = (hash ^ u32::from(pixel[1])).wrapping_mul(16777619); // G
        hash = (hash ^ u32::from(pixel[2])).wrapping_mul(16777619); // B
    }
    hash
}

pub(crate) fn render_frame_rgb_hash_bgra(frame: &[u8]) -> u32 {
    let mut hash = 2166136261u32;
    for pixel in frame.chunks_exact(4) {
        hash = (hash ^ u32::from(pixel[2])).wrapping_mul(16777619); // R
        hash = (hash ^ u32::from(pixel[1])).wrapping_mul(16777619); // G
        hash = (hash ^ u32::from(pixel[0])).wrapping_mul(16777619); // B
    }
    hash
}

pub fn render_hash_frame_bgra(frame: u32, frame_bgra: &[u8]) -> RenderFrameHashReport {
    let hash = render_frame_rgb_hash_bgra(frame_bgra);
    RenderFrameHashReport {
        hash,
        line: render_hash_line("render-hash", frame, hash),
    }
}

pub fn gpu_render_hash_frame_rgba(frame: u32, frame_rgba: &[u8]) -> RenderFrameHashReport {
    let hash = render_frame_rgb_hash_rgba(frame_rgba);
    RenderFrameHashReport {
        hash,
        line: render_hash_line("gpu-render-hash", frame, hash),
    }
}

pub fn render_fingerprint_leaf_bgra(frame_bgra: &[u8]) -> u32 {
    render_frame_rgb_hash_bgra(frame_bgra)
}

pub fn render_hash_pair_bgra_rgba(cpu_bgra: &[u8], gpu_rgba: &[u8]) -> RenderHashPair {
    RenderHashPair {
        cpu_hash: render_frame_rgb_hash_bgra(cpu_bgra),
        gpu_hash: render_frame_rgb_hash_rgba(gpu_rgba),
    }
}

fn render_hash_line(label: &str, frame: u32, hash: u32) -> String {
    format!("{label} frame={frame} hash=0x{hash:08x}")
}

pub(crate) fn compare_bgra_to_rgba(cpu_bgra: &[u8], gpu_rgba: &[u8]) -> Option<GpuRenderDiff> {
    compare_frame_rgb_channels(cpu_bgra, gpu_rgba, |cpu| (cpu[2], cpu[1], cpu[0]))
}

pub(crate) fn compare_gpu_render_bgra_to_rgba(
    cpu_bgra: &[u8],
    gpu_rgba: &[u8],
) -> GpuRenderComparison {
    GpuRenderComparison {
        cpu_hash: render_frame_rgb_hash_bgra(cpu_bgra),
        gpu_hash: render_frame_rgb_hash_rgba(gpu_rgba),
        diff: compare_bgra_to_rgba(cpu_bgra, gpu_rgba),
    }
}

pub fn compare_gpu_render_frame_bgra_to_rgba(
    frame: u32,
    cpu_bgra: &[u8],
    gpu_rgba: &[u8],
) -> GpuRenderFrameComparison {
    let comparison = compare_gpu_render_bgra_to_rgba(cpu_bgra, gpu_rgba);
    let divergence_line = comparison
        .diff
        .map(|diff| gpu_render_divergence_line(frame, &comparison, diff));
    GpuRenderFrameComparison {
        comparison,
        divergence_line,
    }
}

fn gpu_render_divergence_line(
    frame: u32,
    comparison: &GpuRenderComparison,
    diff: GpuRenderDiff,
) -> String {
    format!(
        "gpu-render-divergence frame={frame} mismatched_pixels={} first_mismatch=({}, {}) cpu_rgb=({},{},{}) gpu_rgb=({},{},{}) cpu_hash=0x{:08x} gpu_hash=0x{:08x}",
        diff.mismatched_pixels,
        diff.first_x,
        diff.first_y,
        diff.cpu_rgb.0,
        diff.cpu_rgb.1,
        diff.cpu_rgb.2,
        diff.gpu_rgb.0,
        diff.gpu_rgb.1,
        diff.gpu_rgb.2,
        comparison.cpu_hash,
        comparison.gpu_hash
    )
}

pub(crate) fn compare_rgba_to_rgba(
    classic_rgba: &[u8],
    modern_rgba: &[u8],
) -> Option<GpuRenderDiff> {
    compare_frame_rgb_channels(classic_rgba, modern_rgba, |classic| {
        (classic[0], classic[1], classic[2])
    })
}

fn compare_frame_rgb_channels(
    expected: &[u8],
    actual_rgba: &[u8],
    expected_rgb: impl Fn(&[u8]) -> (u8, u8, u8),
) -> Option<GpuRenderDiff> {
    let mut mismatched_pixels = 0usize;
    let mut first = None;
    for (i, (expected, actual)) in expected
        .chunks_exact(4)
        .zip(actual_rgba.chunks_exact(4))
        .enumerate()
    {
        let cpu_rgb = expected_rgb(expected);
        let gpu_rgb = (actual[0], actual[1], actual[2]);
        if cpu_rgb != gpu_rgb {
            mismatched_pixels += 1;
            first.get_or_insert((i, cpu_rgb, gpu_rgb));
        }
    }

    first.map(|(i, cpu_rgb, gpu_rgb)| GpuRenderDiff {
        mismatched_pixels,
        first_x: i % 256,
        first_y: i / 256,
        cpu_rgb,
        gpu_rgb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_frame_diff_reports_first_mismatch() {
        let classic = [
            1, 2, 3, 0xff, //
            4, 5, 6, 0xff,
        ];
        let modern = [
            1, 2, 3, 0xff, //
            4, 7, 6, 0xff,
        ];

        let diff = compare_rgba_to_rgba(&classic, &modern).expect("one pixel differs");

        assert_eq!(diff.mismatched_pixels, 1);
        assert_eq!(diff.first_x, 1);
        assert_eq!(diff.first_y, 0);
        assert_eq!(diff.cpu_rgb, (4, 5, 6));
        assert_eq!(diff.gpu_rgb, (4, 7, 6));
    }

    #[test]
    fn bgra_frame_diff_reports_cpu_rgb_order() {
        let cpu_bgra = [
            3, 2, 1, 0xff, //
            6, 5, 4, 0xff,
        ];
        let gpu_rgba = [
            1, 2, 3, 0xff, //
            4, 7, 6, 0xff,
        ];

        let diff = compare_bgra_to_rgba(&cpu_bgra, &gpu_rgba).expect("one pixel differs");

        assert_eq!(diff.mismatched_pixels, 1);
        assert_eq!(diff.first_x, 1);
        assert_eq!(diff.cpu_rgb, (4, 5, 6));
        assert_eq!(diff.gpu_rgb, (4, 7, 6));
    }

    #[test]
    fn gpu_render_comparison_owns_hashes_and_diff() {
        let cpu_bgra = [
            3, 2, 1, 0xff, //
            6, 5, 4, 0xff,
        ];
        let gpu_rgba = [
            1, 2, 3, 0xff, //
            4, 7, 6, 0xff,
        ];

        let comparison = compare_gpu_render_bgra_to_rgba(&cpu_bgra, &gpu_rgba);

        assert_eq!(comparison.cpu_hash, render_frame_rgb_hash_bgra(&cpu_bgra));
        assert_eq!(comparison.gpu_hash, render_frame_rgb_hash_rgba(&gpu_rgba));
        assert_eq!(
            comparison.diff,
            Some(GpuRenderDiff {
                mismatched_pixels: 1,
                first_x: 1,
                first_y: 0,
                cpu_rgb: (4, 5, 6),
                gpu_rgb: (4, 7, 6),
            })
        );
    }

    #[test]
    fn gpu_render_frame_comparison_owns_divergence_line() {
        let cpu_bgra = [
            3, 2, 1, 0xff, //
            6, 5, 4, 0xff,
        ];
        let gpu_rgba = [
            1, 2, 3, 0xff, //
            4, 7, 6, 0xff,
        ];

        let report = compare_gpu_render_frame_bgra_to_rgba(42, &cpu_bgra, &gpu_rgba);

        assert_eq!(
            report.divergence_line(),
            Some(
                "gpu-render-divergence frame=42 mismatched_pixels=1 first_mismatch=(1, 0) cpu_rgb=(4,5,6) gpu_rgb=(4,7,6) cpu_hash=0x03d252aa gpu_hash=0x43d73498"
            )
        );
        assert_eq!(
            report.diff(),
            Some(GpuRenderDiff {
                mismatched_pixels: 1,
                first_x: 1,
                first_y: 0,
                cpu_rgb: (4, 5, 6),
                gpu_rgb: (4, 7, 6),
            })
        );
        assert_eq!(report.cpu_hash(), 0x03d252aa);
    }

    #[test]
    fn gpu_render_frame_comparison_omits_divergence_line_when_matching() {
        let cpu_bgra = [3, 2, 1, 0xff, 6, 5, 4, 0xff];
        let gpu_rgba = [1, 2, 3, 0xff, 4, 5, 6, 0xff];

        let report = compare_gpu_render_frame_bgra_to_rgba(42, &cpu_bgra, &gpu_rgba);

        assert_eq!(report.divergence_line(), None);
        assert_eq!(report.diff(), None);
    }

    #[test]
    fn render_hash_reports_own_legacy_output_lines() {
        let bgra = [3, 2, 1, 0xff, 6, 5, 4, 0xff];
        let rgba = [1, 2, 3, 0xff, 4, 5, 6, 0xff];

        let cpu = render_hash_frame_bgra(42, &bgra);
        let gpu = gpu_render_hash_frame_rgba(42, &rgba);

        assert_eq!(cpu.hash, render_frame_rgb_hash_bgra(&bgra));
        assert_eq!(gpu.hash, render_frame_rgb_hash_rgba(&rgba));
        assert_eq!(cpu.line, "render-hash frame=42 hash=0x03d252aa");
        assert_eq!(gpu.line, "gpu-render-hash frame=42 hash=0x03d252aa");
    }

    #[test]
    fn semantic_render_hash_helpers_own_raw_hash_reads() {
        let bgra = [3, 2, 1, 0xff, 6, 5, 4, 0xff];
        let rgba = [1, 2, 3, 0xff, 4, 5, 6, 0xff];

        let pair = render_hash_pair_bgra_rgba(&bgra, &rgba);

        assert_eq!(pair.cpu_hash, render_frame_rgb_hash_bgra(&bgra));
        assert_eq!(pair.gpu_hash, render_frame_rgb_hash_rgba(&rgba));
        assert_eq!(render_fingerprint_leaf_bgra(&bgra), pair.cpu_hash);
    }

    #[test]
    fn rgb_hash_ignores_alpha_and_honors_channel_order() {
        let rgba = [1, 2, 3, 0x00, 4, 5, 6, 0xff];
        let bgra = [3, 2, 1, 0xaa, 6, 5, 4, 0xbb];

        assert_eq!(
            render_frame_rgb_hash_rgba(&rgba),
            render_frame_rgb_hash_bgra(&bgra)
        );
    }
}
