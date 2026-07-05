use std::env;

use zelda3::ZeldaState;

use crate::{TRACE_MAIN_MODULE_INDEX, TRACE_SUBMODULE_INDEX, TRACE_SUBSUBMODULE_INDEX};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AudioFrameStats {
    pub(crate) samples_per_channel: usize,
    pub(crate) peak: i16,
    pub(crate) first_nonzero: Option<usize>,
    pub(crate) mean_abs: u32,
}

impl AudioFrameStats {
    pub(crate) fn from_interleaved_stereo(samples: &[i16]) -> Self {
        let mut sum = 0u64;
        let mut peak = 0i16;
        let mut first_nonzero = None;
        for (i, &sample) in samples.iter().enumerate() {
            let abs = sample.saturating_abs();
            if abs > peak {
                peak = abs;
            }
            if sample != 0 && first_nonzero.is_none() {
                first_nonzero = Some(i);
            }
            sum += abs as u64;
        }
        let mean_abs = if samples.is_empty() {
            0
        } else {
            (sum / samples.len() as u64) as u32
        };
        Self {
            samples_per_channel: samples.len() / 2,
            peak,
            first_nonzero,
            mean_abs,
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
    let mut hash = 2166136261u32;
    for sample in samples {
        for byte in sample.to_le_bytes() {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
    }
    hash
}

pub(crate) fn should_write_fingerprint(fingerprint_frame: Option<u32>, frame: u32) -> bool {
    fingerprint_frame.is_none_or(|target| frame == target)
}

pub(crate) fn print_replay_audio_trace(
    frame: u32,
    game: &ZeldaState,
    audio: &[i16],
    samples: usize,
    channels: usize,
    dsp_pre_hash: u32,
    dsp_writes: &[(u8, u8, i32, u8)],
) {
    let stats = AudioFrameStats::from_interleaved_stereo(audio);
    let mean_abs = if audio.is_empty() {
        0.0
    } else {
        audio
            .iter()
            .map(|sample| i64::from(sample.saturating_abs()))
            .sum::<i64>() as f64
            / audio.len() as f64
    };
    print!(
        "{{\"frame\":{frame},\"samples\":{samples},\"channels\":{channels},\"peak\":{},\"first_nonzero\":",
        stats.peak
    );
    if let Some(first_nonzero) = stats.first_nonzero {
        print!("{first_nonzero}");
    } else {
        print!("null");
    }
    println!(
        ",\"mean_abs\":{mean_abs:.6},\"hash\":\"0x{:08x}\",\"apui\":[{},{},{},{}],\"music\":[{},{},{}],\"main\":{},\"sub\":{},\"subsub\":{},\"inidisp\":{},\"dsp_pre\":\"0x{dsp_pre_hash:08x}\",\"dsp_post\":\"0x{:08x}\",\"dsp_writes\":{},\"dsp_write_hash\":\"0x{:08x}\",\"dsp_write_values_hash\":\"0x{:08x}\"{},{}{}",
        replay_checksum_samples(audio),
        game.ram[0x0648],
        game.ram[0x012c],
        game.ram[0x012d],
        game.ram[0x012e],
        game.ram[0x012f],
        game.ram[0x0132],
        game.ram[0x0133],
        game.ram[TRACE_MAIN_MODULE_INDEX],
        game.ram[TRACE_SUBMODULE_INDEX],
        game.ram[TRACE_SUBSUBMODULE_INDEX],
        game.ram[0x13],
        game.zelda_audio_dsp_hash(),
        dsp_writes.len(),
        replay_checksum_dsp_writes(dsp_writes),
        replay_checksum_dsp_write_values(dsp_writes),
        replay_dsp_write_events_json(frame, dsp_writes),
        game.zelda_audio_route_debug_json(),
        "}",
    );
}

fn replay_dsp_write_events_json(frame: u32, writes: &[(u8, u8, i32, u8)]) -> String {
    let target = env::var("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());
    if target != Some(frame) {
        return String::new();
    }
    let events = writes
        .iter()
        .map(|(addr, val, sample_offset, timer)| format!("[{addr},{val},{sample_offset},{timer}]"))
        .collect::<Vec<_>>()
        .join(",");
    format!(",\"dsp_write_events\":[{events}]")
}

pub(crate) fn replay_checksum_dsp_writes(writes: &[(u8, u8, i32, u8)]) -> u32 {
    let mut hash = 2166136261u32;
    for &(addr, val, sample_offset, timer_cycles) in writes {
        hash = (hash ^ u32::from(addr)).wrapping_mul(16777619);
        hash = (hash ^ u32::from(val)).wrapping_mul(16777619);
        for byte in sample_offset.to_le_bytes() {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
        hash = (hash ^ u32::from(timer_cycles)).wrapping_mul(16777619);
    }
    hash
}

pub(crate) fn replay_checksum_dsp_write_values(writes: &[(u8, u8, i32, u8)]) -> u32 {
    let mut hash = 2166136261u32;
    for &(addr, val, _, _) in writes {
        hash = (hash ^ u32::from(addr)).wrapping_mul(16777619);
        hash = (hash ^ u32::from(val)).wrapping_mul(16777619);
    }
    hash
}

/// Per-frame audio leaf hash: folds the same DSP/sample quantities the audio
/// trace prints, into one u32. Mirrored exactly in C (FingerprintAudioHash).
pub(crate) fn fingerprint_audio_hash(
    sample_checksum: u32,
    dsp_pre: u32,
    dsp_post: u32,
    dsp_write_count: u32,
    dsp_write_hash: u32,
    dsp_write_values_hash: u32,
) -> u32 {
    parity::fnv1a_u32s(&[
        sample_checksum,
        dsp_pre,
        dsp_post,
        dsp_write_count,
        dsp_write_hash,
        dsp_write_values_hash,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_frame_filter_preserves_default_all_frames_behavior() {
        assert!(should_write_fingerprint(None, 41));
        assert!(should_write_fingerprint(Some(42), 42));
        assert!(!should_write_fingerprint(Some(42), 41));
    }

    #[test]
    fn audio_stats_reports_peak_and_first_nonzero_sample() {
        let stats = AudioFrameStats::from_interleaved_stereo(&[0, -3, 7, 1]);

        assert_eq!(stats.samples_per_channel, 2);
        assert_eq!(stats.peak, 7);
        assert_eq!(stats.first_nonzero, Some(1));
        assert_eq!(stats.mean_abs, 2);
    }
}
