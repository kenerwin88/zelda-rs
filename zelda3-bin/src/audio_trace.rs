use zelda3::game_output::{checksum_samples, AudioSampleStats};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AudioFrameStats {
    pub(crate) samples_per_channel: usize,
    pub(crate) peak: i16,
    pub(crate) first_nonzero: Option<usize>,
    pub(crate) mean_abs: u32,
}

impl AudioFrameStats {
    pub(crate) fn from_interleaved_stereo(samples: &[i16]) -> Self {
        let stats = AudioSampleStats::from_interleaved(samples, 2);
        Self {
            samples_per_channel: stats.samples_per_channel,
            peak: stats.peak,
            first_nonzero: stats.first_nonzero,
            mean_abs: stats.mean_abs,
        }
    }
}

pub(crate) fn first_peak_frame(stats: &[AudioFrameStats], threshold: i16) -> Option<usize> {
    stats.iter().position(|stats| stats.peak >= threshold)
}

pub(crate) fn max_peak_frame(stats: &[AudioFrameStats]) -> Option<(usize, i16)> {
    stats
        .iter()
        .enumerate()
        .max_by_key(|(_, stats)| stats.peak)
        .map(|(i, stats)| (i, stats.peak))
}

pub(crate) fn print_audio_window(
    label: &str,
    stats: &[AudioFrameStats],
    debug: &[String],
    center: Option<usize>,
) {
    let Some(center) = center else {
        println!("{label}: no non-silent frames captured");
        return;
    };
    let start = center.saturating_sub(4);
    let end = (center + 12).min(stats.len().saturating_sub(1));
    println!("{label} window frames {start}..={end}:");
    for i in start..=end {
        let stats = stats[i];
        if let Some(debug) = debug.get(i) {
            println!(
                "  {i:>5}: peak={:>5} mean_abs={:>4} first={:?} samples={} {debug}",
                stats.peak, stats.mean_abs, stats.first_nonzero, stats.samples_per_channel,
            );
        } else {
            println!(
                "  {i:>5}: peak={:>5} mean_abs={:>4} first={:?} samples={}",
                stats.peak, stats.mean_abs, stats.first_nonzero, stats.samples_per_channel,
            );
        }
    }
}

pub(crate) fn replay_checksum_samples(samples: &[i16]) -> u32 {
    checksum_samples(samples)
}

