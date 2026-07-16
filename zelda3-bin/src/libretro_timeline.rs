use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AudioComparisonMode {
    Exact,
    Timing,
}

impl AudioComparisonMode {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "exact" => Some(Self::Exact),
            "timing" => Some(Self::Timing),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Timing => "timing",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AudioTimingOptions {
    pub(crate) window_frames: usize,
    pub(crate) silence_threshold: i16,
    pub(crate) max_timing_error_frames: usize,
    pub(crate) max_envelope_error: f64,
}

impl AudioTimingOptions {
    pub(crate) fn from_sample_rate(
        sample_rate: f64,
        window_ms: f64,
        silence_threshold: i16,
        max_timing_error_ms: f64,
        max_envelope_error: f64,
    ) -> Self {
        let sample_rate = sample_rate.max(1.0);
        Self {
            window_frames: ((sample_rate * window_ms / 1000.0).round() as usize).max(1),
            silence_threshold,
            max_timing_error_frames: (sample_rate * max_timing_error_ms / 1000.0).round() as usize,
            max_envelope_error,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct AudioTimeline {
    #[serde(skip)]
    samples: Vec<i16>,
    frame_ends: Vec<u64>,
}

#[cfg(test)]
impl AudioTimeline {
    pub(crate) fn push_stereo_frame(&mut self, interleaved_samples: &[i16]) {
        self.samples.extend_from_slice(interleaved_samples);
        self.frame_ends.push((self.samples.len() / 2) as u64);
    }

    pub(crate) fn sample_frames(&self) -> usize {
        self.samples.len() / 2
    }

    pub(crate) fn waveform_hash(&self) -> String {
        let mut hash = 0xcbf29ce484222325u64;
        for sample in &self.samples {
            for byte in sample.to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        format!("{hash:016x}")
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AudioComparisonReport {
    pub(crate) mode: String,
    pub(crate) matched: bool,
    pub(crate) rust_sample_frames: usize,
    pub(crate) oracle_sample_frames: usize,
    pub(crate) first_mismatch_interleaved: Option<usize>,
    pub(crate) first_mismatch_sample_frame: Option<usize>,
    pub(crate) first_mismatch_channel: Option<usize>,
    pub(crate) first_mismatch_rust_sample: Option<i16>,
    pub(crate) first_mismatch_oracle_sample: Option<i16>,
    pub(crate) mismatched_interleaved_samples: usize,
    pub(crate) zero_lag_envelope_error: Option<f64>,
    pub(crate) best_lag_sample_frames: Option<isize>,
    pub(crate) best_lag_envelope_error: Option<f64>,
    pub(crate) max_activity_edge_lag_sample_frames: Option<usize>,
    pub(crate) rust_activity_edges: Vec<usize>,
    pub(crate) oracle_activity_edges: Vec<usize>,
    pub(crate) activity_mismatch_windows: usize,
    pub(crate) envelope_mismatch_windows: usize,
    pub(crate) envelope_mismatch_sample_frames: Vec<usize>,
    pub(crate) envelope_mismatch_details: Vec<EnvelopeMismatchWindow>,
    pub(crate) max_window_envelope_error: Option<f64>,
    pub(crate) rust_waveform_hash: String,
    pub(crate) oracle_waveform_hash: String,
    pub(crate) message: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct EnvelopeMismatchWindow {
    sample_frame: usize,
    rust_peak: i16,
    oracle_peak: i16,
    error: f64,
    rust_interleaved_samples: Vec<i16>,
    oracle_interleaved_samples: Vec<i16>,
}

#[cfg(test)]
pub(crate) fn compare_audio_timelines(
    rust: &AudioTimeline,
    oracle: &AudioTimeline,
    mode: AudioComparisonMode,
    timing: AudioTimingOptions,
) -> AudioComparisonReport {
    match mode {
        AudioComparisonMode::Exact => compare_exact(rust, oracle),
        AudioComparisonMode::Timing => compare_timing(rust, oracle, timing),
    }
}

pub(crate) struct StreamingAudioComparator {
    mode: AudioComparisonMode,
    timing: AudioTimingOptions,
    rust_sample_frames: usize,
    oracle_sample_frames: usize,
    first_mismatch_interleaved: Option<usize>,
    first_mismatch_rust_sample: Option<i16>,
    first_mismatch_oracle_sample: Option<i16>,
    mismatched_interleaved_samples: usize,
    rust_hash: u64,
    oracle_hash: u64,
    processed_timing_frames: usize,
    window_count: usize,
    window_peak_rust: i16,
    window_peak_oracle: i16,
    window_rust_interleaved_samples: Vec<i16>,
    window_oracle_interleaved_samples: Vec<i16>,
    envelope_error_sum: f64,
    envelope_mismatch_windows: usize,
    envelope_mismatch_sample_frames: Vec<usize>,
    envelope_mismatch_details: Vec<EnvelopeMismatchWindow>,
    max_window_envelope_error: f64,
    activity_mismatch_windows: usize,
    first_timing_mismatch_frame: Option<usize>,
    rust_activity_edges: Vec<usize>,
    oracle_activity_edges: Vec<usize>,
    last_rust_sample_active: bool,
    last_oracle_sample_active: bool,
    rust_silence_run: usize,
    oracle_silence_run: usize,
}

impl StreamingAudioComparator {
    pub(crate) fn new(mode: AudioComparisonMode, timing: AudioTimingOptions) -> Self {
        Self {
            mode,
            timing,
            rust_sample_frames: 0,
            oracle_sample_frames: 0,
            first_mismatch_interleaved: None,
            first_mismatch_rust_sample: None,
            first_mismatch_oracle_sample: None,
            mismatched_interleaved_samples: 0,
            rust_hash: 0xcbf29ce484222325,
            oracle_hash: 0xcbf29ce484222325,
            processed_timing_frames: 0,
            window_count: 0,
            window_peak_rust: 0,
            window_peak_oracle: 0,
            window_rust_interleaved_samples: Vec::with_capacity(timing.window_frames * 2),
            window_oracle_interleaved_samples: Vec::with_capacity(timing.window_frames * 2),
            envelope_error_sum: 0.0,
            envelope_mismatch_windows: 0,
            envelope_mismatch_sample_frames: Vec::new(),
            envelope_mismatch_details: Vec::new(),
            max_window_envelope_error: 0.0,
            activity_mismatch_windows: 0,
            first_timing_mismatch_frame: None,
            rust_activity_edges: Vec::new(),
            oracle_activity_edges: Vec::new(),
            last_rust_sample_active: false,
            last_oracle_sample_active: false,
            rust_silence_run: 0,
            oracle_silence_run: 0,
        }
    }

    pub(crate) fn push_stereo_frame(&mut self, rust: &[i16], oracle: &[i16]) {
        let rust_start = self.rust_sample_frames.saturating_mul(2);
        let common = rust.len().min(oracle.len());
        for index in 0..common {
            let mine = rust[index];
            let theirs = oracle[index];
            hash_sample(&mut self.rust_hash, mine);
            hash_sample(&mut self.oracle_hash, theirs);
            if mine != theirs {
                if self.first_mismatch_interleaved.is_none() {
                    self.first_mismatch_interleaved = Some(rust_start + index);
                    self.first_mismatch_rust_sample = Some(mine);
                    self.first_mismatch_oracle_sample = Some(theirs);
                }
                self.mismatched_interleaved_samples += 1;
            }
        }
        for &sample in &rust[common..] {
            hash_sample(&mut self.rust_hash, sample);
        }
        for &sample in &oracle[common..] {
            hash_sample(&mut self.oracle_hash, sample);
        }
        if rust.len() != oracle.len() {
            self.first_mismatch_interleaved
                .get_or_insert(rust_start + common);
            self.mismatched_interleaved_samples += rust.len().abs_diff(oracle.len());
        }

        let common_stereo_frames = common / 2;
        for frame in 0..common_stereo_frames {
            let offset = frame * 2;
            let rust_peak = rust[offset]
                .saturating_abs()
                .max(rust[offset + 1].saturating_abs());
            let oracle_peak = oracle[offset]
                .saturating_abs()
                .max(oracle[offset + 1].saturating_abs());
            let sample_frame = self.rust_sample_frames + frame;
            let rust_active = rust_peak >= self.timing.silence_threshold;
            let oracle_active = oracle_peak >= self.timing.silence_threshold;
            update_debounced_activity(
                &mut self.last_rust_sample_active,
                &mut self.rust_silence_run,
                &mut self.rust_activity_edges,
                rust_active,
                sample_frame,
                self.timing.window_frames,
            );
            update_debounced_activity(
                &mut self.last_oracle_sample_active,
                &mut self.oracle_silence_run,
                &mut self.oracle_activity_edges,
                oracle_active,
                sample_frame,
                self.timing.window_frames,
            );
            self.window_peak_rust = self.window_peak_rust.max(rust_peak);
            self.window_peak_oracle = self.window_peak_oracle.max(oracle_peak);
            self.window_rust_interleaved_samples
                .extend_from_slice(&rust[offset..offset + 2]);
            self.window_oracle_interleaved_samples
                .extend_from_slice(&oracle[offset..offset + 2]);
            self.window_count += 1;
            self.processed_timing_frames += 1;
            if self.window_count == self.timing.window_frames {
                self.finish_window();
            }
        }
        self.rust_sample_frames += rust.len() / 2;
        self.oracle_sample_frames += oracle.len() / 2;
    }

    pub(crate) fn finish(mut self) -> AudioComparisonReport {
        if self.window_count != 0 {
            self.finish_window();
        }
        let paired_edges = self
            .rust_activity_edges
            .len()
            .min(self.oracle_activity_edges.len());
        let max_edge_lag = self
            .rust_activity_edges
            .iter()
            .zip(&self.oracle_activity_edges)
            .map(|(&mine, &theirs)| mine.abs_diff(theirs))
            .max()
            .unwrap_or(0);
        let edge_count_mismatch = self
            .rust_activity_edges
            .len()
            .abs_diff(self.oracle_activity_edges.len());
        let windows = (self.rust_sample_frames.max(self.oracle_sample_frames)
            + self.timing.window_frames.saturating_sub(1))
            / self.timing.window_frames.max(1);
        let mean_envelope_error = if windows == 0 {
            0.0
        } else {
            self.envelope_error_sum / windows as f64
        };
        let first_mismatch = self.first_mismatch_interleaved.or_else(|| {
            self.first_timing_mismatch_frame
                .map(|sample_frame| sample_frame.saturating_mul(2))
        });
        let mut report = AudioComparisonReport {
            mode: self.mode.as_str().to_string(),
            matched: false,
            rust_sample_frames: self.rust_sample_frames,
            oracle_sample_frames: self.oracle_sample_frames,
            first_mismatch_interleaved: first_mismatch,
            first_mismatch_sample_frame: first_mismatch.map(|index| index / 2),
            first_mismatch_channel: first_mismatch.map(|index| index % 2),
            first_mismatch_rust_sample: self.first_mismatch_rust_sample,
            first_mismatch_oracle_sample: self.first_mismatch_oracle_sample,
            mismatched_interleaved_samples: self.mismatched_interleaved_samples,
            zero_lag_envelope_error: Some(mean_envelope_error),
            best_lag_sample_frames: None,
            best_lag_envelope_error: None,
            max_activity_edge_lag_sample_frames: (paired_edges != 0).then_some(max_edge_lag),
            rust_activity_edges: self.rust_activity_edges.clone(),
            oracle_activity_edges: self.oracle_activity_edges.clone(),
            activity_mismatch_windows: self.activity_mismatch_windows,
            envelope_mismatch_windows: self.envelope_mismatch_windows,
            envelope_mismatch_sample_frames: self.envelope_mismatch_sample_frames,
            envelope_mismatch_details: self.envelope_mismatch_details,
            max_window_envelope_error: Some(self.max_window_envelope_error),
            rust_waveform_hash: format!("{:016x}", self.rust_hash),
            oracle_waveform_hash: format!("{:016x}", self.oracle_hash),
            message: String::new(),
        };
        report.matched = match self.mode {
            AudioComparisonMode::Exact => {
                self.rust_sample_frames == self.oracle_sample_frames
                    && self.mismatched_interleaved_samples == 0
            }
            AudioComparisonMode::Timing => {
                self.rust_sample_frames == self.oracle_sample_frames
                    && mean_envelope_error <= self.timing.max_envelope_error
                    && self.envelope_mismatch_windows == 0
                    && self.activity_mismatch_windows == 0
                    && edge_count_mismatch == 0
                    && max_edge_lag <= self.timing.max_timing_error_frames
            }
        };
        report.message = if report.matched {
            format!(
                "{} continuous audio matched {} stereo sample frame(s)",
                self.mode.as_str(),
                self.rust_sample_frames,
            )
        } else {
            format!(
                "{} continuous audio diverged: rust_frames={} oracle_frames={} first_interleaved={:?} waveform_mismatches={} mean_envelope_error={mean_envelope_error:.6} max_window_envelope_error={:.6} envelope_mismatch_windows={} activity_mismatch_windows={} rust_edges={} oracle_edges={} max_edge_lag_frames={max_edge_lag}",
                self.mode.as_str(),
                self.rust_sample_frames,
                self.oracle_sample_frames,
                report.first_mismatch_interleaved,
                self.mismatched_interleaved_samples,
                self.max_window_envelope_error,
                self.envelope_mismatch_windows,
                self.activity_mismatch_windows,
                self.rust_activity_edges.len(),
                self.oracle_activity_edges.len(),
            )
        };
        report
    }

    fn finish_window(&mut self) {
        let rust_active = self.window_peak_rust >= self.timing.silence_threshold;
        let oracle_active = self.window_peak_oracle >= self.timing.silence_threshold;
        let window_end = self.processed_timing_frames;
        if rust_active != oracle_active {
            self.activity_mismatch_windows += 1;
            self.first_timing_mismatch_frame.get_or_insert(window_end);
        }
        let envelope_error = f64::from(self.window_peak_rust.abs_diff(self.window_peak_oracle))
            / f64::from(u16::MAX);
        self.envelope_error_sum += envelope_error;
        self.max_window_envelope_error = self.max_window_envelope_error.max(envelope_error);
        if envelope_error > self.timing.max_envelope_error {
            self.envelope_mismatch_windows += 1;
            let sample_frame = window_end.saturating_sub(self.window_count);
            self.envelope_mismatch_sample_frames.push(sample_frame);
            self.envelope_mismatch_details.push(EnvelopeMismatchWindow {
                sample_frame,
                rust_peak: self.window_peak_rust,
                oracle_peak: self.window_peak_oracle,
                error: envelope_error,
                rust_interleaved_samples: self.window_rust_interleaved_samples.clone(),
                oracle_interleaved_samples: self.window_oracle_interleaved_samples.clone(),
            });
            self.first_timing_mismatch_frame.get_or_insert(window_end);
        }
        self.window_count = 0;
        self.window_peak_rust = 0;
        self.window_peak_oracle = 0;
        self.window_rust_interleaved_samples.clear();
        self.window_oracle_interleaved_samples.clear();
    }
}

fn update_debounced_activity(
    active: &mut bool,
    silence_run: &mut usize,
    edges: &mut Vec<usize>,
    sample_is_active: bool,
    sample_frame: usize,
    silence_frames: usize,
) {
    if sample_is_active {
        *silence_run = 0;
        if !*active {
            edges.push(sample_frame);
            *active = true;
        }
    } else if *active {
        *silence_run += 1;
        if *silence_run >= silence_frames.max(1) {
            edges.push(sample_frame + 1 - *silence_run);
            *active = false;
            *silence_run = 0;
        }
    }
}

fn hash_sample(hash: &mut u64, sample: i16) {
    for byte in sample.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

#[cfg(test)]
fn common_report(
    rust: &AudioTimeline,
    oracle: &AudioTimeline,
    mode: AudioComparisonMode,
) -> AudioComparisonReport {
    AudioComparisonReport {
        mode: mode.as_str().to_string(),
        matched: false,
        rust_sample_frames: rust.sample_frames(),
        oracle_sample_frames: oracle.sample_frames(),
        first_mismatch_interleaved: None,
        first_mismatch_sample_frame: None,
        first_mismatch_channel: None,
        first_mismatch_rust_sample: None,
        first_mismatch_oracle_sample: None,
        mismatched_interleaved_samples: 0,
        zero_lag_envelope_error: None,
        best_lag_sample_frames: None,
        best_lag_envelope_error: None,
        max_activity_edge_lag_sample_frames: None,
        rust_activity_edges: Vec::new(),
        oracle_activity_edges: Vec::new(),
        activity_mismatch_windows: 0,
        envelope_mismatch_windows: 0,
        envelope_mismatch_sample_frames: Vec::new(),
        envelope_mismatch_details: Vec::new(),
        max_window_envelope_error: None,
        rust_waveform_hash: rust.waveform_hash(),
        oracle_waveform_hash: oracle.waveform_hash(),
        message: String::new(),
    }
}

#[cfg(test)]
fn compare_exact(rust: &AudioTimeline, oracle: &AudioTimeline) -> AudioComparisonReport {
    let mut report = common_report(rust, oracle, AudioComparisonMode::Exact);
    let common = rust.samples.len().min(oracle.samples.len());
    for index in 0..common {
        if rust.samples[index] != oracle.samples[index] {
            report.first_mismatch_interleaved.get_or_insert(index);
            report.mismatched_interleaved_samples += 1;
        }
    }
    report.mismatched_interleaved_samples += rust.samples.len().abs_diff(oracle.samples.len());
    if let Some(index) = report.first_mismatch_interleaved {
        report.first_mismatch_sample_frame = Some(index / 2);
        report.first_mismatch_channel = Some(index % 2);
        report.first_mismatch_rust_sample = rust.samples.get(index).copied();
        report.first_mismatch_oracle_sample = oracle.samples.get(index).copied();
    } else if rust.samples.len() != oracle.samples.len() {
        report.first_mismatch_interleaved = Some(common);
        report.first_mismatch_sample_frame = Some(common / 2);
        report.first_mismatch_channel = Some(common % 2);
    }
    report.matched = report.mismatched_interleaved_samples == 0;
    report.message = if report.matched {
        format!(
            "exact continuous waveform matched {} stereo sample frame(s)",
            rust.sample_frames()
        )
    } else {
        format!(
            "exact continuous waveform diverged: rust_frames={} oracle_frames={} first_interleaved={:?} mismatched_interleaved={}",
            rust.sample_frames(),
            oracle.sample_frames(),
            report.first_mismatch_interleaved,
            report.mismatched_interleaved_samples,
        )
    };
    report
}

#[cfg(test)]
fn compare_timing(
    rust: &AudioTimeline,
    oracle: &AudioTimeline,
    options: AudioTimingOptions,
) -> AudioComparisonReport {
    let mut report = common_report(rust, oracle, AudioComparisonMode::Timing);
    if rust.sample_frames() != oracle.sample_frames() {
        report.message = format!(
            "continuous audio duration diverged: rust_frames={} oracle_frames={}",
            rust.sample_frames(),
            oracle.sample_frames(),
        );
        return report;
    }

    let rust_envelope = normalized_peak_envelope(&rust.samples, options.window_frames);
    let oracle_envelope = normalized_peak_envelope(&oracle.samples, options.window_frames);
    let zero_lag = envelope_error(&rust_envelope, &oracle_envelope, 0);
    let max_lag_windows = options
        .max_timing_error_frames
        .saturating_add(options.window_frames - 1)
        / options.window_frames;
    let diagnostic_lag_windows = max_lag_windows.max(50);
    let (best_lag_windows, best_lag_error) = (-isize::try_from(diagnostic_lag_windows).unwrap()
        ..=isize::try_from(diagnostic_lag_windows).unwrap())
        .map(|lag| (lag, envelope_error(&rust_envelope, &oracle_envelope, lag)))
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .unwrap_or((0, zero_lag));

    let threshold = f64::from(options.silence_threshold.max(0)) / f64::from(i16::MAX);
    let activity_mismatch_windows = rust_envelope
        .iter()
        .zip(&oracle_envelope)
        .filter(|(mine, theirs)| (**mine >= threshold) != (**theirs >= threshold))
        .count();
    let allowed_activity_windows = max_lag_windows;
    let best_lag_sample_frames = best_lag_windows * options.window_frames as isize;
    let timing_ok = best_lag_sample_frames.unsigned_abs() <= options.max_timing_error_frames;
    let envelope_ok = zero_lag <= options.max_envelope_error;
    let activity_ok = activity_mismatch_windows <= allowed_activity_windows;

    report.zero_lag_envelope_error = Some(zero_lag);
    report.best_lag_sample_frames = Some(best_lag_sample_frames);
    report.best_lag_envelope_error = Some(best_lag_error);
    report.activity_mismatch_windows = activity_mismatch_windows;
    report.matched = timing_ok && envelope_ok && activity_ok;
    report.message = if report.matched {
        format!(
            "continuous audio timing matched: sample_frames={} zero_lag_error={zero_lag:.6} best_lag_frames={best_lag_sample_frames} activity_mismatch_windows={activity_mismatch_windows}",
            rust.sample_frames(),
        )
    } else {
        format!(
            "continuous audio timing diverged: sample_frames={} zero_lag_error={zero_lag:.6} max_error={:.6} best_lag_frames={best_lag_sample_frames} allowed_lag_frames={} activity_mismatch_windows={activity_mismatch_windows} allowed_activity_windows={allowed_activity_windows}",
            rust.sample_frames(),
            options.max_envelope_error,
            options.max_timing_error_frames,
        )
    };
    report
}

#[cfg(test)]
fn normalized_peak_envelope(samples: &[i16], window_frames: usize) -> Vec<f64> {
    let mut peaks = samples
        .chunks(window_frames.saturating_mul(2).max(2))
        .map(|window| {
            window
                .iter()
                .map(|sample| f64::from(sample.saturating_abs()))
                .fold(0.0, f64::max)
        })
        .collect::<Vec<_>>();
    let maximum = peaks.iter().copied().fold(1.0, f64::max);
    for peak in &mut peaks {
        *peak /= maximum;
    }
    peaks
}

#[cfg(test)]
fn envelope_error(mine: &[f64], theirs: &[f64], lag: isize) -> f64 {
    let (mine_start, theirs_start) = if lag >= 0 {
        (lag as usize, 0)
    } else {
        (0, lag.unsigned_abs())
    };
    let count = mine
        .len()
        .saturating_sub(mine_start)
        .min(theirs.len().saturating_sub(theirs_start));
    if count == 0 {
        return f64::INFINITY;
    }
    let overlap_error = mine[mine_start..mine_start + count]
        .iter()
        .zip(&theirs[theirs_start..theirs_start + count])
        .map(|(mine, theirs)| (mine - theirs).abs())
        .sum::<f64>()
        / count as f64;
    let full_count = mine.len().max(theirs.len()).max(1);
    let missing_fraction = full_count.saturating_sub(count) as f64 / full_count as f64;
    overlap_error + missing_fraction
}

pub(crate) fn format_input_history(input_history: &[(u32, u16)]) -> String {
    let mut text =
        String::from("# Deterministic controller stream captured once per game frame.\n");
    if input_history.is_empty() {
        return text;
    }
    let mut start = input_history[0].0;
    let mut end = start;
    let mut input = input_history[0].1;
    for &(frame, next_input) in &input_history[1..] {
        if frame == end.wrapping_add(1) && next_input == input {
            end = frame;
            continue;
        }
        push_input_run(&mut text, start, end, input);
        start = frame;
        end = frame;
        input = next_input;
    }
    push_input_run(&mut text, start, end, input);
    text
}

fn push_input_run(text: &mut String, start: u32, end: u32, input: u16) {
    if input == 0 {
        return;
    }
    if start == end {
        text.push_str(&format!("{start} 0x{input:04x}\n"));
    } else {
        text.push_str(&format!("{start}..{end} 0x{input:04x}\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timeline(samples: &[i16]) -> AudioTimeline {
        let mut timeline = AudioTimeline::default();
        timeline.push_stereo_frame(samples);
        timeline
    }

    #[test]
    fn exact_comparison_uses_the_entire_continuous_stream() {
        let mine = timeline(&[1, 2, 3, 4, 5, 6]);
        let theirs = timeline(&[1, 2, 3, 9, 5, 6]);
        let report = compare_audio_timelines(
            &mine,
            &theirs,
            AudioComparisonMode::Exact,
            AudioTimingOptions::from_sample_rate(32_000.0, 1.0, 64, 2.0, 0.25),
        );
        assert!(!report.matched);
        assert_eq!(report.first_mismatch_interleaved, Some(3));
        assert_eq!(report.first_mismatch_sample_frame, Some(1));
        assert_eq!(report.first_mismatch_channel, Some(1));
        assert_eq!(report.first_mismatch_rust_sample, Some(4));
        assert_eq!(report.first_mismatch_oracle_sample, Some(9));
    }

    #[test]
    fn timing_comparison_rejects_a_shift_instead_of_aligning_it_away() {
        let mut mine = vec![0; 400];
        let mut theirs = vec![0; 400];
        mine[80..120].fill(20_000);
        theirs[120..160].fill(20_000);
        let report = compare_audio_timelines(
            &timeline(&mine),
            &timeline(&theirs),
            AudioComparisonMode::Timing,
            AudioTimingOptions {
                window_frames: 10,
                silence_threshold: 64,
                max_timing_error_frames: 5,
                max_envelope_error: 0.25,
            },
        );
        assert!(!report.matched);
        assert_eq!(report.best_lag_sample_frames, Some(-20));
    }

    #[test]
    fn timing_comparison_rejects_a_hanging_tail() {
        let mut mine = vec![0; 400];
        let mut theirs = vec![0; 400];
        mine[80..120].fill(20_000);
        theirs[80..200].fill(20_000);
        let report = compare_audio_timelines(
            &timeline(&mine),
            &timeline(&theirs),
            AudioComparisonMode::Timing,
            AudioTimingOptions {
                window_frames: 10,
                silence_threshold: 64,
                max_timing_error_frames: 5,
                max_envelope_error: 0.25,
            },
        );
        assert!(!report.matched);
        assert!(report.activity_mismatch_windows > 0);
    }

    #[test]
    fn input_history_is_run_length_encoded_without_idle_noise() {
        assert_eq!(
            format_input_history(&[(0, 0), (1, 0x80), (2, 0x80), (3, 0), (4, 0x100)]),
            "# Deterministic controller stream captured once per game frame.\n1..2 0x0080\n4 0x0100\n"
        );
    }

    #[test]
    fn streaming_exact_comparison_keeps_absolute_offsets_across_frames() {
        let timing = AudioTimingOptions::from_sample_rate(32_000.0, 1.0, 64, 2.0, 0.25);
        let mut comparison = StreamingAudioComparator::new(AudioComparisonMode::Exact, timing);
        comparison.push_stereo_frame(&[1, 2, 3, 4], &[1, 2, 3, 4]);
        comparison.push_stereo_frame(&[5, 6, 7, 8], &[5, 6, 9, 8]);
        let report = comparison.finish();
        assert!(!report.matched);
        assert_eq!(report.first_mismatch_interleaved, Some(6));
        assert_eq!(report.first_mismatch_sample_frame, Some(3));
    }

    #[test]
    fn streaming_timing_comparison_rejects_a_late_edge_across_callback_blocks() {
        let timing = AudioTimingOptions {
            window_frames: 2,
            silence_threshold: 64,
            max_timing_error_frames: 1,
            max_envelope_error: 1.0,
        };
        let mut comparison = StreamingAudioComparator::new(AudioComparisonMode::Timing, timing);
        comparison.push_stereo_frame(&[0; 8], &[0; 8]);
        comparison.push_stereo_frame(
            &[10_000, 10_000, 10_000, 10_000, 0, 0, 0, 0],
            &[0, 0, 0, 0, 10_000, 10_000, 10_000, 10_000],
        );
        let report = comparison.finish();
        assert!(!report.matched);
        assert!(report.activity_mismatch_windows > 0);
        assert!(report.max_activity_edge_lag_sample_frames.unwrap() > 1);
    }

    #[test]
    fn streaming_activity_edges_keep_exact_sample_positions_inside_windows() {
        let timing = AudioTimingOptions {
            window_frames: 4,
            silence_threshold: 64,
            max_timing_error_frames: 0,
            max_envelope_error: 1.0,
        };
        let mut rust = [0; 22];
        rust[8..12].fill(100);
        let mut oracle = [0; 22];
        oracle[10..14].fill(100);
        let mut comparison = StreamingAudioComparator::new(AudioComparisonMode::Timing, timing);
        comparison.push_stereo_frame(&rust, &oracle);

        let report = comparison.finish();

        assert_eq!(report.rust_activity_edges, vec![4, 6]);
        assert_eq!(report.oracle_activity_edges, vec![5, 7]);
        assert_eq!(report.max_activity_edge_lag_sample_frames, Some(1));
    }

    #[test]
    fn streaming_timing_rejects_one_short_dropout_even_when_route_mean_is_small() {
        let timing = AudioTimingOptions {
            window_frames: 2,
            silence_threshold: 64,
            max_timing_error_frames: 1,
            max_envelope_error: 0.10,
        };
        let mut mine = vec![10_000i16; 400];
        let theirs = mine.clone();
        mine[200..204].fill(1_000);
        let mut comparison = StreamingAudioComparator::new(AudioComparisonMode::Timing, timing);
        comparison.push_stereo_frame(&mine, &theirs);
        let report = comparison.finish();
        assert!(!report.matched);
        assert_eq!(report.envelope_mismatch_windows, 1);
        assert_eq!(report.envelope_mismatch_sample_frames, vec![100]);
        assert_eq!(report.envelope_mismatch_details[0].rust_peak, 1_000);
        assert_eq!(report.envelope_mismatch_details[0].oracle_peak, 10_000);
        assert_eq!(
            report.envelope_mismatch_details[0].rust_interleaved_samples,
            vec![1_000; 4]
        );
        assert_eq!(
            report.envelope_mismatch_details[0].oracle_interleaved_samples,
            vec![10_000; 4]
        );
        assert!(report.zero_lag_envelope_error.unwrap() < timing.max_envelope_error);
        assert!(report.max_window_envelope_error.unwrap() > timing.max_envelope_error);
    }
}
