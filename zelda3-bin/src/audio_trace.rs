use std::env;

use zelda3::{
    game_output::{
        checksum_dsp_write_values, checksum_dsp_writes, checksum_samples, AudioSampleStats,
        AudioTraceFrameSummary, DspWriteEvent,
    },
    modern_audio_sequence::ModernAudioSequencer,
    ZeldaState,
};

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
    dsp_writes: &[DspWriteEvent],
) {
    let event_frame = game.zelda_audio_event_frame_from_dsp_writes(&dsp_writes);
    let mut modern_sequence = ModernAudioSequencer::default();
    let modern_event_frame = modern_sequence.sequence_route(game.zelda_audio_route_state());
    let modern_sequence_stats = modern_sequence.last_stats();
    let summary = AudioTraceFrameSummary::from_parts(
        audio,
        channels,
        dsp_pre_hash,
        game.zelda_audio_dsp_hash(),
        dsp_writes,
        &event_frame,
    );
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
        ",\"mean_abs\":{mean_abs:.6},\"hash\":\"0x{:08x}\",\"apui\":[{},{},{},{}],\"music\":[{},{},{}],\"main\":{},\"sub\":{},\"subsub\":{},\"inidisp\":{},\"dsp_pre\":\"0x{:08x}\",\"dsp_post\":\"0x{:08x}\",\"dsp_writes\":{},\"dsp_write_hash\":\"0x{:08x}\",\"dsp_write_values_hash\":\"0x{:08x}\",\"command_events\":{},\"command_hash\":\"0x{:08x}\",\"unresolved_dsp_writes\":{},\"modern_sfx_known\":{},\"modern_sfx_unknown\":{},\"modern_program_hash\":\"0x{:08x}\",\"modern_command_events\":{},\"modern_command_hash\":\"0x{:08x}\"{},{}{}",
        summary.sample_stats.checksum,
        event_frame.music.apui00,
        event_frame.music.music_control,
        event_frame.music.sound_effect_ambient,
        event_frame.music.sound_effect_1,
        event_frame.music.sound_effect_2,
        event_frame.music.queued_music_control,
        event_frame.music.last_music_control,
        game.ram[TRACE_MAIN_MODULE_INDEX],
        game.ram[TRACE_SUBMODULE_INDEX],
        game.ram[TRACE_SUBSUBMODULE_INDEX],
        game.ram[0x13],
        summary.dsp_pre_hash,
        summary.dsp_post_hash,
        summary.dsp_write_count,
        summary.dsp_write_hash,
        summary.dsp_write_values_hash,
        summary.command_event_count,
        summary.command_event_hash,
        summary.unresolved_dsp_writes,
        modern_sequence_stats.known_sfx_commands,
        modern_sequence_stats.unknown_sfx_commands,
        modern_sequence_stats.program_hash,
        modern_event_frame.events.len(),
        modern_event_frame.command_hash(),
        replay_dsp_write_events_json(frame, dsp_writes),
        game.zelda_audio_route_debug_json(),
        "}",
    );
}

fn replay_dsp_write_events_json(frame: u32, writes: &[DspWriteEvent]) -> String {
    let target = env::var("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());
    let range = env::var("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME_RANGE").ok();
    if !should_write_dsp_write_events(target, range.as_deref(), frame) {
        return String::new();
    }
    let events = writes
        .iter()
        .map(|write| {
            format!(
                "[{},{},{},{}]",
                write.addr, write.value, write.sample_offset, write.timer_cycles
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(",\"dsp_write_events\":[{events}]")
}

fn should_write_dsp_write_events(target: Option<u32>, range: Option<&str>, frame: u32) -> bool {
    if target == Some(frame) {
        return true;
    }
    let Some(range) = range else {
        return false;
    };
    let Some((start, end)) = parse_dsp_write_frame_range(range) else {
        return false;
    };
    start <= frame && frame <= end
}

fn parse_dsp_write_frame_range(value: &str) -> Option<(u32, u32)> {
    let (start, end) = value
        .split_once(':')
        .or_else(|| value.split_once("..="))
        .or_else(|| value.split_once(".."))?;
    let start = start.parse::<u32>().ok()?;
    let end = end.parse::<u32>().ok()?;
    (start <= end).then_some((start, end))
}

pub(crate) fn replay_checksum_dsp_writes(writes: &[DspWriteEvent]) -> u32 {
    checksum_dsp_writes(writes)
}

pub(crate) fn replay_checksum_dsp_write_values(writes: &[DspWriteEvent]) -> u32 {
    checksum_dsp_write_values(writes)
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

    #[test]
    fn dsp_write_event_filter_accepts_exact_frame_or_range() {
        assert!(should_write_dsp_write_events(Some(41), None, 41));
        assert!(!should_write_dsp_write_events(Some(41), None, 42));
        assert!(should_write_dsp_write_events(None, Some("10:12"), 11));
        assert!(should_write_dsp_write_events(None, Some("10..=12"), 12));
        assert!(!should_write_dsp_write_events(None, Some("10..12"), 13));
        assert!(!should_write_dsp_write_events(None, Some("12:10"), 11));
        assert!(!should_write_dsp_write_events(None, Some("bad"), 11));
    }
}
