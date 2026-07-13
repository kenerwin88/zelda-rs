use crate::game_output::{
    AudioEventFrame, AudioEventKind, AudioSampleStats, EchoParameterKind, MusicControlState,
    VoiceParameterKind,
};

const MODERN_AUDIO_VOICES: usize = 8;
const DEFAULT_FRAME_RATE: usize = 60;
const DSP_SAMPLES_PER_FRAME: usize = 534;

const fn default_echo_mix() -> i8 {
    0
}

const fn default_echo_feedback() -> i8 {
    64
}

const fn default_master_volume() -> i8 {
    96
}

const fn default_noise_sample() -> i16 {
    -0x4000
}

#[derive(Clone, Debug, Default)]
struct DebugVoiceSamples([Vec<i16>; MODERN_AUDIO_VOICES]);

impl PartialEq for DebugVoiceSamples {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for DebugVoiceSamples {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernAudioFrameStats {
    pub samples_per_channel: usize,
    pub channels: usize,
    pub understood_events: u32,
    pub ignored_events: u32,
    pub triggered_voices: u32,
    pub active_voices: u32,
    pub peak: i16,
    pub checksum: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernAudioEngine {
    voices: [ModernVoice; MODERN_AUDIO_VOICES],
    last_music: MusicControlState,
    last_ports: [u8; 4],
    /// Canonical instrument/echo context selected by the game state.
    #[serde(skip, default)]
    sample_bank_id: u8,
    #[serde(default)]
    echo_left: Vec<i16>,
    #[serde(default)]
    echo_right: Vec<i16>,
    #[serde(default = "default_echo_mix")]
    echo_mix_left: i8,
    #[serde(default = "default_echo_mix")]
    echo_mix_right: i8,
    #[serde(default = "default_echo_feedback")]
    echo_feedback: i8,
    #[serde(default = "default_master_volume")]
    master_volume_left: i8,
    #[serde(default = "default_master_volume")]
    master_volume_right: i8,
    #[serde(default)]
    fir_coefficients: [i8; 8],
    #[serde(default)]
    echo_delay_samples: u16,
    #[serde(default)]
    dsp_flags: u8,
    #[serde(default)]
    pitch_modulation_mask: u8,
    #[serde(default)]
    noise_enable_mask: u8,
    #[serde(default)]
    fir_history_left: [i16; 8],
    #[serde(default)]
    fir_history_right: [i16; 8],
    #[serde(default)]
    fir_history_index: u8,
    #[serde(default)]
    echo_ring_index: usize,
    #[serde(default)]
    echo_remaining_samples: u16,
    #[serde(default)]
    echo_start_page: u8,
    #[serde(default)]
    echo_ram_initialized: bool,
    #[serde(default = "default_noise_sample")]
    noise_sample: i16,
    #[serde(default)]
    noise_counter: u16,
    last_stats: ModernAudioFrameStats,
    #[serde(skip, default)]
    debug_voice_samples: DebugVoiceSamples,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModernVoiceDebugState {
    pub active: bool,
    pub volume_left: i8,
    pub volume_right: i8,
    pub echo_send: bool,
    pub pitch: u16,
    pub pitch_counter: u16,
    pub gain: u16,
    pub envelope_state: u8,
    pub envelope_rate_counter: u16,
    pub adsr1: u8,
    pub adsr2: u8,
    pub gain_config: u8,
    pub sample_out: i16,
    pub sample_backed: bool,
    pub sample_length: usize,
    pub sample_loops: bool,
    pub brr_block_start: usize,
}

impl Default for ModernAudioEngine {
    fn default() -> Self {
        Self {
            voices: std::array::from_fn(|_| ModernVoice::default()),
            last_music: MusicControlState::default(),
            last_ports: [0; 4],
            sample_bank_id: 0,
            echo_left: Vec::new(),
            echo_right: Vec::new(),
            echo_mix_left: default_echo_mix(),
            echo_mix_right: default_echo_mix(),
            echo_feedback: default_echo_feedback(),
            master_volume_left: default_master_volume(),
            master_volume_right: default_master_volume(),
            fir_coefficients: [0; 8],
            echo_delay_samples: 512,
            dsp_flags: 0x20,
            pitch_modulation_mask: 0,
            noise_enable_mask: 0,
            fir_history_left: [0; 8],
            fir_history_right: [0; 8],
            fir_history_index: 0,
            echo_ring_index: 0,
            echo_remaining_samples: 512,
            echo_start_page: 0xc8,
            echo_ram_initialized: false,
            noise_sample: default_noise_sample(),
            noise_counter: 0,
            last_stats: ModernAudioFrameStats::default(),
            debug_voice_samples: DebugVoiceSamples::default(),
        }
    }
}

impl ModernAudioEngine {
    pub fn select_sample_bank(&mut self, bank_id: u8) {
        // Validate eagerly so a corrupt route cannot fail much later in mixing.
        let _ = crate::modern_sample_bank::bank_name(bank_id);
        if self.sample_bank_id != bank_id {
            self.sample_bank_id = bank_id;
            self.echo_ram_initialized = false;
        }
    }

    pub fn sample_bank_id(&self) -> u8 {
        self.sample_bank_id
    }
    pub fn seed_dsp_checkpoint_state(&mut self, sample_ram: &[u8], dsp: &[u8]) {
        if dsp.len() < 884 {
            return;
        }
        self.master_volume_left = dsp[821] as i8;
        self.master_volume_right = dsp[822] as i8;
        self.echo_mix_left = dsp[831] as i8;
        self.echo_mix_right = dsp[832] as i8;
        self.echo_feedback = dsp[833] as i8;
        self.dsp_flags = dsp[0x6c];
        self.pitch_modulation_mask = dsp[0x2d];
        self.noise_enable_mask = dsp[0x3d];
        self.echo_start_page = dsp[0x6d];
        self.echo_delay_samples = u16::from(dsp[0x7d] & 0x0f).saturating_mul(512).max(1);
        self.fir_coefficients = std::array::from_fn(|index| dsp[843 + index] as i8);

        for voice_index in 0..MODERN_AUDIO_VOICES {
            let base = 0x80 + voice_index * 86;
            let word = |offset| u16::from_le_bytes([dsp[base + offset], dsp[base + offset + 1]]);
            let source = dsp[base + 44];
            let voice = &mut self.voices[voice_index];
            voice.exact_pitch_word = word(0);
            voice.render_pitch_word = word(0);
            voice.dsp_pitch_configured = true;
            voice.pitch_register_only = false;
            voice.dsp_pitch_counter = word(2);
            voice.dsp_adsr1 = dsp[voice_index * 0x10 + 5];
            voice.dsp_adsr2 = dsp[voice_index * 0x10 + 6];
            voice.dsp_gain = dsp[voice_index * 0x10 + 7];
            voice.dsp_envelope_configured = true;
            voice.dsp_rate_counter = word(64);
            voice.dsp_envelope_state = dsp[base + 66];
            voice.dsp_gain_level = word(76);
            voice.volume_left = dsp[base + 82] as i8;
            voice.volume_right = dsp[base + 83] as i8;
            voice.stereo_volume_configured = true;
            voice.echo_send = dsp[base + 84] != 0;
            voice.noise_enabled = self.noise_enable_mask & (1 << voice_index) != 0;
            voice.last_output_sample = word(80) as i16;
            voice.active = voice.dsp_envelope_state != 4 || voice.dsp_gain_level != 0;
            voice.start_delay_samples = 0;
            voice.pitch_slide_frames = 0;

            if let Some(sample) = decode_brr_sample(sample_ram, source) {
                let classic_window: [i16; 19] = std::array::from_fn(|index| {
                    let offset = base + 6 + index * 2;
                    i16::from_le_bytes([dsp[offset], dsp[offset + 1]])
                });
                let decode_offset = usize::from(word(46));
                let matched_block_start = sample
                    .pcm
                    .chunks_exact(16)
                    .enumerate()
                    .rev()
                    .find(|(index, block)| {
                        let start = *index * 16;
                        **block == classic_window[3..]
                            && start >= 3
                            && sample.pcm[start - 3..start + 16] == classic_window
                            && (sample.block_addresses[*index].wrapping_add(9) & 0xffff)
                                == decode_offset
                    })
                    .or_else(|| {
                        sample
                            .pcm
                            .chunks_exact(16)
                            .enumerate()
                            .rev()
                            .find(|(index, block)| {
                                let start = *index * 16;
                                **block == classic_window[3..]
                                    && start >= 3
                                    && sample.pcm[start - 3..start + 16] == classic_window
                            })
                    })
                    .map(|(index, _)| index * 16);
                let (sample, block_start, checkpoint_decoder) =
                    if let Some(block_start) = matched_block_start {
                        (sample, block_start, None)
                    } else {
                        let previous_flags = dsp[base + 48];
                        let old = word(50) as i16;
                        let older = word(52) as i16;
                        decode_brr_checkpoint_continuation(
                            sample_ram,
                            source,
                            decode_offset,
                            previous_flags,
                            old,
                            older,
                            classic_window,
                        )
                        .map_or((sample, 0, None), |(continuation, decoder)| {
                            (continuation, 3, Some(decoder))
                        })
                    };
                if std::env::var_os("ZELDA3_AUDIO_CHECKPOINT_DEBUG").is_some() {
                    eprintln!(
                        "modern checkpoint voice={voice_index} source={source} decode_offset={decode_offset:04x} block_start={block_start} block_address={:04x}",
                        sample.block_addresses[block_start / 16]
                    );
                }
                voice.sample_data = sample.pcm;
                voice.sample_loop_start = sample.loop_start;
                voice.sample_loops = sample.loops;
                voice.sample_backed = true;
                voice.brr_block_start = block_start;
                voice.brr_started = true;
                voice.checkpoint_brr_decoder = checkpoint_decoder;
                voice.sample_position = (block_start as u64) << 32;
                voice.sample_step = sample_step_for_pitch_word(voice.exact_pitch_word, 32_000);
            } else {
                voice.sample_data.clear();
                voice.sample_backed = false;
                voice.brr_block_start = 0;
                voice.brr_started = false;
                voice.checkpoint_brr_decoder = None;
            }
        }
    }

    /// Notify the renderer that the SPC address space backing samples and echo
    /// has been replaced or modified by a song-bank upload.
    ///
    /// Zelda's uploads can overlap the active echo window. The hardware DSP
    /// reads those bytes on its next echo pass, so the modern cache must be
    /// rebuilt from the updated RAM before rendering again.
    pub fn sample_ram_changed(&mut self) {
        self.echo_ram_initialized = false;
    }

    pub fn render_frame(
        &mut self,
        frame: &AudioEventFrame,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
    ) -> ModernAudioFrameStats {
        self.render_frame_with_sample_ram(frame, audio_buffer, samples, channels, None)
    }

    pub fn render_frame_with_sample_ram(
        &mut self,
        frame: &AudioEventFrame,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
        sample_ram: Option<&[u8]>,
    ) -> ModernAudioFrameStats {
        let samples_per_channel = samples.max(0) as usize;
        let channels = channels.max(0) as usize;
        let count = samples_per_channel.saturating_mul(channels);
        for value in audio_buffer.iter_mut().take(count) {
            *value = 0;
        }

        let mut stats = ModernAudioFrameStats {
            samples_per_channel,
            channels,
            ..ModernAudioFrameStats::default()
        };
        let mut deferred_globals = Vec::new();
        let mut deferred_voice_events = Vec::new();
        let has_modern_intent = frame.events.iter().any(|event| {
            matches!(
                event.kind,
                AudioEventKind::PlayMusic { .. }
                    | AudioEventKind::StopMusic
                    | AudioEventKind::PlaySfx { .. }
                    | AudioEventKind::SetTempo { .. }
                    | AudioEventKind::SetNoteOrigin { .. }
                    | AudioEventKind::SetPitchWord { .. }
                    | AudioEventKind::SetStereoVolume { .. }
                    | AudioEventKind::SetDspEnvelope { .. }
                    | AudioEventKind::NoteOn { .. }
                    | AudioEventKind::KeyOnVoice { .. }
                    | AudioEventKind::NoteOff { .. }
                    | AudioEventKind::SetDuration { .. }
                    | AudioEventKind::PitchSlide { .. }
                    | AudioEventKind::SetNoise { .. }
                    | AudioEventKind::SetPan { .. }
                    | AudioEventKind::SetEchoSend { .. }
                    | AudioEventKind::SetEnvelope { .. }
            )
        });

        for event in &frame.events {
            if event.sample_offset > 0 && is_deferred_voice_event(&event.kind) {
                deferred_voice_events.push(event.clone());
                stats.understood_events += 1;
                if matches!(
                    event.kind,
                    AudioEventKind::NoteOn { .. } | AudioEventKind::KeyOnVoice { .. }
                ) {
                    stats.triggered_voices += 1;
                }
                continue;
            }
            match &event.kind {
                AudioEventKind::MusicState(music) => {
                    stats.understood_events += 1;
                    if !frame.sequenced && !has_modern_intent {
                        self.handle_music_state(*music, &mut stats);
                    }
                }
                AudioEventKind::ApuPorts {
                    input,
                    write,
                    pending,
                    ..
                } => {
                    stats.understood_events += 1;
                    if !frame.sequenced && !has_modern_intent {
                        self.handle_ports(*input, *write, *pending, &mut stats);
                    }
                }
                AudioEventKind::PlayMusic { .. }
                | AudioEventKind::PlaySfx { .. }
                | AudioEventKind::SetTempo { .. } => {
                    stats.understood_events += 1;
                }
                AudioEventKind::StopMusic => {
                    stats.understood_events += 1;
                    if !frame.sequenced {
                        for voice in &mut self.voices {
                            voice.begin_release();
                        }
                    }
                }
                AudioEventKind::SetNoteOrigin { .. } => {
                    stats.understood_events += 1;
                }
                AudioEventKind::SetPitchWord { voice, pitch_word } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.exact_pitch_word = *pitch_word;
                        voice.dsp_pitch_configured = true;
                        voice.pitch_register_only = false;
                    }
                }
                AudioEventKind::SetPitchRegisterWord { voice, pitch_word } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.exact_pitch_word = *pitch_word;
                        voice.dsp_pitch_configured = true;
                        voice.pitch_register_only = true;
                    }
                }
                AudioEventKind::SetStereoVolume { voice, left, right } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.volume_left = *left;
                        voice.volume_right = *right;
                        voice.stereo_volume_configured = true;
                    }
                }
                AudioEventKind::SetDspEnvelope {
                    voice,
                    adsr1,
                    adsr2,
                    gain,
                } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.dsp_adsr1 = *adsr1;
                        voice.dsp_adsr2 = *adsr2;
                        voice.dsp_gain = *gain;
                        voice.dsp_envelope_configured = true;
                        voice.exact_parameters_pending = true;
                    }
                }
                AudioEventKind::NoteOn {
                    voice,
                    pitch,
                    instrument,
                    volume,
                } => {
                    stats.understood_events += 1;
                    self.trigger_voice_with_params(*voice as usize, *pitch, *instrument, *volume);
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.start_delay_samples = event.sample_offset.max(0) as usize;
                    }
                    self.load_voice_sample(*voice as usize, *instrument, *pitch, sample_ram);
                    stats.triggered_voices += 1;
                }
                AudioEventKind::KeyOnVoice {
                    voice,
                    source,
                    adsr1,
                    adsr2,
                    gain,
                    volume_left,
                    volume_right,
                    rate_counter,
                } => {
                    stats.understood_events += 1;
                    self.key_on_voice(
                        *voice as usize,
                        *source,
                        *adsr1,
                        *adsr2,
                        *gain,
                        *volume_left,
                        *volume_right,
                        *rate_counter,
                        sample_ram,
                    );
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.start_delay_samples = event.sample_offset.max(0) as usize;
                    }
                    stats.triggered_voices += 1;
                }
                AudioEventKind::NoteOff { voice } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.begin_release();
                    }
                }
                AudioEventKind::SetEnvelope {
                    attack,
                    voice,
                    decay,
                    sustain,
                    release,
                } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.envelope_configured = true;
                        voice.envelope_attack_frames = *attack;
                        voice.envelope_decay_frames = *decay;
                        voice.envelope_sustain = (*sustain).min(15);
                        voice.envelope_release_frames = *release;
                    }
                }
                AudioEventKind::SetDuration { voice, frames } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.remaining_frames = *frames;
                    }
                }
                AudioEventKind::PitchSlide {
                    voice,
                    target_pitch,
                    frames,
                } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        let target = phase_step_for_code(*target_pitch);
                        if *frames == 0 {
                            voice.base_phase_step = target;
                            voice.phase_step = target;
                            voice.pitch_slide_frames = 0;
                        } else {
                            voice.pitch_slide_target = target;
                            voice.pitch_slide_frames = *frames;
                        }
                    }
                }
                AudioEventKind::SetNoise { voice, enabled } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.noise_enabled = *enabled;
                        voice.timbre = if *enabled { 3 } else { voice.instrument_timbre };
                    }
                }
                AudioEventKind::SetPan { voice, pan } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.pan = *pan;
                    }
                }
                AudioEventKind::SetEchoSend { voice, enabled } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.echo_send = *enabled;
                    }
                }
                AudioEventKind::VoiceKeyOn { mask } => {
                    stats.understood_events += 1;
                    for voice in 0..MODERN_AUDIO_VOICES {
                        if mask & (1 << voice) != 0 {
                            self.trigger_voice(voice, 0x40 + voice as u8 * 3, 1200);
                            stats.triggered_voices += 1;
                        }
                    }
                }
                AudioEventKind::VoiceKeyOff { mask } => {
                    stats.understood_events += 1;
                    for voice in 0..MODERN_AUDIO_VOICES {
                        if mask & (1 << voice) != 0 {
                            self.voices[voice].begin_release();
                        }
                    }
                }
                AudioEventKind::VoiceParameter {
                    voice,
                    parameter,
                    value,
                } => {
                    stats.understood_events += 1;
                    self.handle_voice_parameter(*voice, *parameter, *value);
                }
                AudioEventKind::EchoParameter { parameter, value } => match parameter {
                    EchoParameterKind::VolumeLeft => {
                        stats.understood_events += 1;
                        self.echo_mix_left = *value as i8;
                    }
                    EchoParameterKind::VolumeRight => {
                        stats.understood_events += 1;
                        self.echo_mix_right = *value as i8;
                    }
                    EchoParameterKind::Feedback => {
                        stats.understood_events += 1;
                        self.echo_feedback = value.cast_signed();
                    }
                    EchoParameterKind::EnableMask => {
                        stats.understood_events += 1;
                        for voice in 0..MODERN_AUDIO_VOICES {
                            self.voices[voice].echo_send = value & (1 << voice) != 0;
                        }
                    }
                    EchoParameterKind::Fir(_)
                    | EchoParameterKind::Delay
                    | EchoParameterKind::StartAddress => stats.ignored_events += 1,
                },
                AudioEventKind::ResetEchoVolume { restore_offset } => {
                    stats.understood_events += 1;
                    let restore_left = self.echo_mix_left as u8;
                    let restore_right = self.echo_mix_right as u8;
                    self.echo_mix_left = 0;
                    self.echo_mix_right = 0;
                    deferred_globals.push((usize::from(*restore_offset), 0x2c, restore_left));
                    deferred_globals.push((usize::from(*restore_offset), 0x3c, restore_right));
                }
                AudioEventKind::GlobalParameter { register, value } => {
                    if event.sample_offset > 0 {
                        deferred_globals.push((event.sample_offset as usize, *register, *value));
                        stats.understood_events += 1;
                    } else if self.handle_global_parameter(*register, *value) {
                        stats.understood_events += 1;
                    } else {
                        stats.ignored_events += 1;
                    }
                }
                AudioEventKind::UnresolvedDspWrite { .. } => {
                    stats.ignored_events += 1;
                }
            }
        }

        self.advance_envelopes();
        deferred_globals.sort_unstable_by_key(|event| event.0);
        deferred_voice_events.sort_by_key(|event| event.sample_offset);
        if samples_per_channel != 0 && channels != 0 {
            self.initialize_echo_ring_from_source(sample_ram);
            let mut native_audio = vec![0i16; DSP_SAMPLES_PER_FRAME.saturating_mul(channels)];
            self.mix(
                &mut native_audio,
                DSP_SAMPLES_PER_FRAME,
                channels,
                &deferred_globals,
                &deferred_voice_events,
                sample_ram,
            );
            resample_nearest(
                &native_audio,
                audio_buffer,
                DSP_SAMPLES_PER_FRAME,
                samples_per_channel,
                channels,
            );
        }
        self.advance_pitch_slides();
        self.advance_voice_durations();
        stats.active_voices = self.voices.iter().filter(|voice| voice.active).count() as u32;

        let sample_stats = AudioSampleStats::from_interleaved(&audio_buffer[..count], channels);
        stats.peak = sample_stats.peak;
        stats.checksum = sample_stats.checksum;
        self.last_stats = stats;
        stats
    }

    pub fn last_stats(&self) -> ModernAudioFrameStats {
        self.last_stats
    }

    pub fn voice_debug_states(&self) -> [ModernVoiceDebugState; MODERN_AUDIO_VOICES] {
        std::array::from_fn(|voice| {
            let voice = &self.voices[voice];
            ModernVoiceDebugState {
                active: voice.active,
                volume_left: voice.volume_left,
                volume_right: voice.volume_right,
                echo_send: voice.echo_send,
                pitch: voice.exact_pitch_word,
                pitch_counter: voice.dsp_pitch_counter,
                gain: voice.dsp_gain_level,
                envelope_state: voice.dsp_envelope_state,
                envelope_rate_counter: voice.dsp_rate_counter,
                adsr1: voice.dsp_adsr1,
                adsr2: voice.dsp_adsr2,
                gain_config: voice.dsp_gain,
                sample_out: voice.last_output_sample,
                sample_backed: voice.sample_backed,
                sample_length: voice.sample_data.len(),
                sample_loops: voice.sample_loops,
                brr_block_start: voice.brr_block_start,
            }
        })
    }

    pub fn echo_debug_state(&self) -> (usize, u8, u16) {
        (
            self.echo_ring_index,
            self.fir_history_index,
            self.echo_remaining_samples,
        )
    }

    pub fn echo_debug_value(&self) -> (i16, i16) {
        (
            self.echo_left
                .get(self.echo_ring_index)
                .copied()
                .unwrap_or(0),
            self.echo_right
                .get(self.echo_ring_index)
                .copied()
                .unwrap_or(0),
        )
    }

    pub fn echo_debug_ring(&self) -> (&[i16], &[i16]) {
        (&self.echo_left, &self.echo_right)
    }

    pub fn echo_debug_fir_history(&self) -> (&[i16; 8], &[i16; 8]) {
        (&self.fir_history_left, &self.fir_history_right)
    }

    pub fn global_debug_state(&self) -> (i8, i8, i8, i8, i8, u8, u8, u8, [i8; 8]) {
        (
            self.master_volume_left,
            self.master_volume_right,
            self.echo_mix_left,
            self.echo_mix_right,
            self.echo_feedback,
            self.dsp_flags,
            self.pitch_modulation_mask,
            self.noise_enable_mask,
            self.fir_coefficients,
        )
    }

    pub fn debug_voice_samples(&self) -> &[Vec<i16>; MODERN_AUDIO_VOICES] {
        &self.debug_voice_samples.0
    }

    pub fn seed_echo_checkpoint_state(
        &mut self,
        sample_ram: &[u8],
        start_page: u8,
        delay_register: u8,
        ring_index: u16,
        echo_remaining: u16,
        fir_index: u8,
        fir_left: [i16; 8],
        fir_right: [i16; 8],
        rewind_samples: u16,
    ) {
        self.echo_start_page = start_page;
        self.echo_delay_samples = u16::from(delay_register & 0x0f).saturating_mul(512).max(1);
        self.echo_ram_initialized = false;
        self.initialize_echo_ring_from_source(Some(sample_ram));
        let echo_len = self.echo_left.len().max(1);
        self.echo_ring_index = (usize::from(ring_index) + echo_len
            - usize::from(rewind_samples) % echo_len)
            % echo_len;
        self.echo_remaining_samples = if rewind_samples == 0 {
            echo_remaining.max(1)
        } else {
            self.echo_delay_samples
                .saturating_sub(self.echo_ring_index as u16)
                .max(1)
        };
        self.fir_history_index = fir_index.wrapping_sub(rewind_samples as u8) & 7;
        self.fir_history_left = fir_left;
        self.fir_history_right = fir_right;
    }

    fn handle_music_state(&mut self, music: MusicControlState, stats: &mut ModernAudioFrameStats) {
        let melody = music.music_control;
        if melody != 0 && melody != self.last_music.music_control {
            self.trigger_voice(0, melody, 700);
            stats.triggered_voices += 1;
        }
        for (voice, code) in [
            music.sound_effect_ambient,
            music.sound_effect_1,
            music.sound_effect_2,
            music.queued_music_control,
        ]
        .into_iter()
        .enumerate()
        {
            if code != 0 {
                self.trigger_voice((voice + 1).min(MODERN_AUDIO_VOICES - 1), code, 1000);
                stats.triggered_voices += 1;
            }
        }
        self.last_music = music;
    }

    fn handle_ports(
        &mut self,
        input: [u8; 4],
        write: [u8; 4],
        pending: [u8; 4],
        stats: &mut ModernAudioFrameStats,
    ) {
        for port in 0..4 {
            let code = [input[port], write[port], pending[port]]
                .into_iter()
                .find(|value| *value != 0)
                .unwrap_or(0);
            if code != 0 && code != self.last_ports[port] {
                self.trigger_voice(port, code, 1100);
                stats.triggered_voices += 1;
            }
            self.last_ports[port] = input[port];
        }
    }

    fn handle_voice_parameter(&mut self, voice: u8, parameter: VoiceParameterKind, value: u8) {
        let Some(voice) = self.voices.get_mut(voice as usize) else {
            return;
        };
        match parameter {
            VoiceParameterKind::VolumeLeft => {
                voice.volume_left = value as i8;
                voice.stereo_volume_configured = true;
            }
            VoiceParameterKind::VolumeRight => {
                voice.volume_right = value as i8;
                voice.stereo_volume_configured = true;
            }
            VoiceParameterKind::PitchLow => {
                voice.exact_pitch_word = (voice.exact_pitch_word & 0x3f00) | u16::from(value);
                voice.dsp_pitch_configured = true;
            }
            VoiceParameterKind::PitchHigh => {
                voice.exact_pitch_word =
                    (voice.exact_pitch_word & 0x00ff) | (u16::from(value & 0x3f) << 8);
                voice.dsp_pitch_configured = true;
            }
            VoiceParameterKind::Source => {
                voice.instrument_timbre = value & 3;
                if !voice.noise_enabled {
                    voice.timbre = voice.instrument_timbre;
                }
            }
            VoiceParameterKind::Adsr1 => voice.dsp_adsr1 = value,
            VoiceParameterKind::Adsr2 => voice.dsp_adsr2 = value,
            VoiceParameterKind::Gain => voice.dsp_gain = value,
        }
    }

    fn handle_global_parameter(&mut self, register: u8, value: u8) -> bool {
        match register & 0x7f {
            0x0c => self.master_volume_left = value as i8,
            0x1c => self.master_volume_right = value as i8,
            0x2c => self.echo_mix_left = value as i8,
            0x3c => self.echo_mix_right = value as i8,
            0x0d => self.echo_feedback = value as i8,
            0x2d => self.pitch_modulation_mask = value,
            0x3d => {
                self.noise_enable_mask = value;
                for voice in 0..MODERN_AUDIO_VOICES {
                    self.voices[voice].noise_enabled = value & (1 << voice) != 0;
                }
            }
            0x4d => {
                for voice in 0..MODERN_AUDIO_VOICES {
                    self.voices[voice].echo_send = value & (1 << voice) != 0;
                }
            }
            0x6c => self.dsp_flags = value,
            0x7d => {
                let delay = u16::from(value & 0x0f).saturating_mul(512).max(1);
                if delay != self.echo_delay_samples {
                    self.echo_delay_samples = delay;
                }
            }
            register if register & 0x0f == 0x0f => {
                self.fir_coefficients[usize::from(register >> 4)] = value as i8;
            }
            0x6d => {
                if value != self.echo_start_page {
                    self.echo_start_page = value;
                    self.echo_ram_initialized = false;
                }
            }
            _ => return false,
        }
        true
    }

    fn initialize_echo_ring_from_source(&mut self, sample_ram: Option<&[u8]>) {
        if self.echo_ram_initialized {
            return;
        }
        let delay = usize::from(self.echo_delay_samples.max(1));
        self.echo_left.resize(delay, 0);
        self.echo_right.resize(delay, 0);
        let start = usize::from(self.echo_start_page) << 8;
        for index in 0..delay {
            let address = start + index * 4;
            if let Some(bytes) = sample_source_bytes(self.sample_bank_id, sample_ram, address) {
                self.echo_left[index] = i16::from_le_bytes([bytes[0], bytes[1]]);
                self.echo_right[index] = i16::from_le_bytes([bytes[2], bytes[3]]);
            } else {
                self.echo_left[index] = 0;
                self.echo_right[index] = 0;
            }
        }
        self.echo_ring_index = 0;
        self.echo_remaining_samples = self.echo_delay_samples.max(1);
        self.fir_history_left = [0; 8];
        self.fir_history_right = [0; 8];
        self.fir_history_index = 0;
        self.echo_ram_initialized = true;
    }

    fn resize_echo_ring_from_ram(&mut self, sample_ram: Option<&[u8]>) {
        if !self.echo_ram_initialized {
            let ring_index = self.echo_ring_index;
            let remaining = self.echo_remaining_samples;
            let fir_index = self.fir_history_index;
            let fir_left = self.fir_history_left;
            let fir_right = self.fir_history_right;
            self.initialize_echo_ring_from_source(sample_ram);
            self.echo_ring_index = ring_index % self.echo_left.len().max(1);
            self.echo_remaining_samples = remaining.max(1);
            self.fir_history_index = fir_index;
            self.fir_history_left = fir_left;
            self.fir_history_right = fir_right;
        }
        let echo_delay = usize::from(self.echo_delay_samples.max(1));
        if self.echo_left.len() == echo_delay {
            return;
        }
        let previous_len = self.echo_left.len();
        self.echo_left.resize(echo_delay, 0);
        self.echo_right.resize(echo_delay, 0);
        if echo_delay > previous_len {
            let start = usize::from(self.echo_start_page) << 8;
            for index in previous_len..echo_delay {
                let address = start + index * 4;
                if let Some(bytes) = sample_source_bytes(self.sample_bank_id, sample_ram, address) {
                    self.echo_left[index] = i16::from_le_bytes([bytes[0], bytes[1]]);
                    self.echo_right[index] = i16::from_le_bytes([bytes[2], bytes[3]]);
                }
            }
        }
        self.echo_ring_index %= echo_delay;
    }

    fn apply_deferred_voice_event(&mut self, kind: &AudioEventKind, sample_ram: Option<&[u8]>) {
        match kind {
            AudioEventKind::VoiceParameter {
                voice,
                parameter,
                value,
            } => self.handle_voice_parameter(*voice, *parameter, *value),
            AudioEventKind::SetPitchWord { voice, pitch_word } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.exact_pitch_word = *pitch_word;
                    voice.dsp_pitch_configured = true;
                    voice.pitch_register_only = false;
                }
            }
            AudioEventKind::SetPitchRegisterWord { voice, pitch_word } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.exact_pitch_word = *pitch_word;
                    voice.dsp_pitch_configured = true;
                    voice.pitch_register_only = true;
                }
            }
            AudioEventKind::SetStereoVolume { voice, left, right } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.volume_left = *left;
                    voice.volume_right = *right;
                    voice.stereo_volume_configured = true;
                }
            }
            AudioEventKind::SetDspEnvelope {
                voice,
                adsr1,
                adsr2,
                gain,
            } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.dsp_adsr1 = *adsr1;
                    voice.dsp_adsr2 = *adsr2;
                    voice.dsp_gain = *gain;
                    voice.dsp_envelope_configured = true;
                    voice.exact_parameters_pending = true;
                }
            }
            AudioEventKind::SetEchoSend { voice, enabled } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.echo_send = *enabled;
                }
            }
            AudioEventKind::SetNoise { voice, enabled } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.noise_enabled = *enabled;
                    voice.timbre = if *enabled { 3 } else { voice.instrument_timbre };
                }
            }
            AudioEventKind::SetEnvelope {
                voice,
                attack,
                decay,
                sustain,
                release,
            } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.envelope_configured = true;
                    voice.envelope_attack_frames = *attack;
                    voice.envelope_decay_frames = *decay;
                    voice.envelope_sustain = (*sustain).min(15);
                    voice.envelope_release_frames = *release;
                }
            }
            AudioEventKind::PitchSlide {
                voice,
                target_pitch,
                frames,
            } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    let target = phase_step_for_code(*target_pitch);
                    if *frames == 0 {
                        voice.base_phase_step = target;
                        voice.phase_step = target;
                        voice.pitch_slide_frames = 0;
                    } else {
                        voice.pitch_slide_target = target;
                        voice.pitch_slide_frames = *frames;
                    }
                }
            }
            AudioEventKind::SetPan { voice, pan } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.pan = *pan;
                }
            }
            AudioEventKind::SetDuration { voice, frames } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.remaining_frames = *frames;
                }
            }
            AudioEventKind::NoteOn {
                voice,
                pitch,
                instrument,
                volume,
            } => {
                self.trigger_voice_with_params(*voice as usize, *pitch, *instrument, *volume);
                self.load_voice_sample(*voice as usize, *instrument, *pitch, sample_ram);
            }
            AudioEventKind::KeyOnVoice {
                voice,
                source,
                adsr1,
                adsr2,
                gain,
                volume_left,
                volume_right,
                rate_counter,
            } => {
                self.key_on_voice(
                    *voice as usize,
                    *source,
                    *adsr1,
                    *adsr2,
                    *gain,
                    *volume_left,
                    *volume_right,
                    *rate_counter,
                    sample_ram,
                );
            }
            AudioEventKind::NoteOff { voice } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.begin_release();
                }
            }
            _ => {}
        }
    }

    fn trigger_voice(&mut self, voice: usize, code: u8, amplitude: i32) {
        self.trigger_voice_with_params(voice, code, code >> 4, amplitude.saturating_div(12) as u8);
    }

    fn trigger_voice_with_params(&mut self, voice: usize, pitch: u8, instrument: u8, volume: u8) {
        let Some(voice) = self.voices.get_mut(voice) else {
            return;
        };
        let exact_parameters = voice.exact_parameters_pending;
        voice.exact_parameters_pending = false;
        if !exact_parameters {
            voice.dsp_envelope_configured = false;
            voice.stereo_volume_configured = false;
            voice.exact_pitch_word = 0;
            voice.dsp_pitch_configured = false;
        }
        voice.active = true;
        voice.base_phase_step = phase_step_for_code(pitch);
        voice.phase_step = voice.base_phase_step;
        voice.peak_amplitude = i32::from(volume).saturating_mul(12).min(1800);
        if voice.dsp_envelope_configured {
            voice.initialize_dsp_envelope();
            voice.amplitude = 0;
        } else if voice.envelope_configured && voice.envelope_attack_frames != 0 {
            voice.amplitude = 0;
            voice.envelope_stage = ENVELOPE_ATTACK;
            voice.envelope_frames_remaining = voice.envelope_attack_frames;
        } else {
            voice.amplitude = voice.peak_amplitude;
            voice.begin_decay_or_sustain();
        }
        voice.decay = 3 + (instrument & 3);
        voice.instrument_timbre = instrument % 3;
        voice.timbre = if voice.noise_enabled {
            3
        } else {
            voice.instrument_timbre
        };
        voice.pitch_slide_frames = 0;
        voice.sample_position = 0;
        voice.sample_backed = false;
        voice.brr_block_start = 0;
        voice.brr_started = false;
        voice.checkpoint_brr_decoder = None;
    }

    fn key_on_voice(
        &mut self,
        voice: usize,
        source: u8,
        adsr1: u8,
        adsr2: u8,
        gain: u8,
        volume_left: i8,
        volume_right: i8,
        rate_counter: u16,
        sample_ram: Option<&[u8]>,
    ) {
        let Some(state) = self.voices.get_mut(voice) else {
            return;
        };
        state.exact_parameters_pending = false;
        state.dsp_adsr1 = adsr1;
        state.dsp_adsr2 = adsr2;
        state.dsp_gain = gain;
        state.dsp_rate_counter = rate_counter;
        state.dsp_envelope_configured = true;
        state.volume_left = volume_left;
        state.volume_right = volume_right;
        state.stereo_volume_configured = true;
        state.active = true;
        state.instrument_timbre = source % 3;
        if !state.noise_enabled {
            state.timbre = state.instrument_timbre;
        }
        state.pitch_slide_frames = 0;
        state.sample_position = 0;
        state.sample_backed = false;
        state.brr_block_start = 0;
        state.brr_started = false;
        state.checkpoint_brr_decoder = None;
        if state.dsp_envelope_configured {
            state.initialize_dsp_envelope();
            state.amplitude = 0;
        } else {
            state.amplitude = state.peak_amplitude;
            state.begin_decay_or_sustain();
        }
        self.load_voice_sample(voice, source, 0, sample_ram);
    }

    fn load_voice_sample(
        &mut self,
        voice: usize,
        instrument: u8,
        pitch: u8,
        sample_ram: Option<&[u8]>,
    ) {
        let Some(voice) = self.voices.get_mut(voice) else {
            return;
        };
        let sample = sample_ram
            .and_then(|ram| decode_brr_sample(ram, instrument))
            .or_else(|| decode_brr_bank_sample(self.sample_bank_id, instrument));
        let Some(sample) = sample else {
            return;
        };
        if sample
            .pcm
            .iter()
            .map(|value| value.unsigned_abs())
            .max()
            .unwrap_or(0)
            < 64
        {
            return;
        }
        voice.sample_data = sample.pcm;
        voice.sample_loop_start = sample.loop_start;
        voice.sample_loops = sample.loops;
        voice.sample_position = 0;
        voice.sample_step = if voice.dsp_pitch_configured {
            sample_step_for_pitch_word(voice.exact_pitch_word, 32_000)
        } else {
            sample_step_for_pitch(pitch, 32_000)
        };
        voice.sample_backed = true;
    }

    fn mix(
        &mut self,
        audio_buffer: &mut [i16],
        samples_per_channel: usize,
        channels: usize,
        deferred_globals: &[(usize, u8, u8)],
        deferred_voice_events: &[crate::game_output::AudioEvent],
        sample_ram: Option<&[u8]>,
    ) {
        for samples in &mut self.debug_voice_samples.0 {
            samples.clear();
        }
        let frame_rate = samples_per_channel
            .saturating_mul(DEFAULT_FRAME_RATE)
            .max(1);
        for voice in &mut self.voices {
            if voice.active {
                voice.rescale_phase_step(frame_rate);
            }
        }

        self.resize_echo_ring_from_ram(sample_ram);

        let mut deferred_index = 0;
        let mut deferred_voice_index = 0;
        for sample_index in 0..samples_per_channel {
            while deferred_globals
                .get(deferred_index)
                .is_some_and(|event| event.0 <= sample_index)
            {
                let (_, register, value) = deferred_globals[deferred_index];
                self.handle_global_parameter(register, value);
                deferred_index += 1;
            }
            self.resize_echo_ring_from_ram(sample_ram);
            while deferred_voice_events
                .get(deferred_voice_index)
                .is_some_and(|event| event.sample_offset.max(0) as usize <= sample_index)
            {
                self.apply_deferred_voice_event(
                    &deferred_voice_events[deferred_voice_index].kind,
                    sample_ram,
                );
                deferred_voice_index += 1;
            }
            let mut mixed_left = 0i32;
            let mut mixed_right = 0i32;
            let mut echo_input_left = 0i32;
            let mut echo_input_right = 0i32;
            let mut previous_voice_sample = 0i32;
            for voice_index in 0..self.voices.len() {
                let voice = &mut self.voices[voice_index];
                if !voice.active {
                    voice.advance_inactive_pitch_counter();
                    self.debug_voice_samples.0[voice_index].push(0);
                    previous_voice_sample = 0;
                    continue;
                }
                let unmodulated_step = voice.sample_step;
                voice.render_pitch_word = voice.exact_pitch_word;
                if voice_index != 0
                    && self.pitch_modulation_mask & (1 << voice_index) != 0
                    && voice.exact_pitch_word != 0
                {
                    let factor = (previous_voice_sample >> 4) + 0x400;
                    let modulated =
                        (i32::from(voice.exact_pitch_word) * factor >> 10).clamp(0, 0x3fff) as u16;
                    voice.sample_step = sample_step_for_pitch_word(modulated, 32_000);
                    voice.render_pitch_word = modulated;
                }
                let sample = if voice.noise_enabled {
                    voice.next_noise_sample(self.noise_sample)
                } else {
                    voice.next_sample_with_ram(sample_ram)
                };
                voice.last_output_sample = sample as i16;
                self.debug_voice_samples.0[voice_index].push(sample as i16);
                voice.sample_step = unmodulated_step;
                previous_voice_sample = sample;
                let (voice_left, voice_right) = if voice.stereo_volume_configured {
                    (
                        sample * i32::from(voice.volume_left) >> 6,
                        sample * i32::from(voice.volume_right) >> 6,
                    )
                } else {
                    let pan = i32::from(voice.pan);
                    let left_gain = 127 - pan.max(0);
                    let right_gain = 127 + pan.min(0);
                    (sample * left_gain / 127, sample * right_gain / 127)
                };
                mixed_left += voice_left;
                mixed_right += voice_right;
                mixed_left = mixed_left.clamp(i16::MIN as i32, i16::MAX as i32);
                mixed_right = mixed_right.clamp(i16::MIN as i32, i16::MAX as i32);
                if voice.echo_send {
                    echo_input_left += voice_left;
                    echo_input_right += voice_right;
                    echo_input_left = echo_input_left.clamp(i16::MIN as i32, i16::MAX as i32);
                    echo_input_right = echo_input_right.clamp(i16::MIN as i32, i16::MAX as i32);
                }
            }
            mixed_left = (mixed_left * i32::from(self.master_volume_left) >> 7)
                .clamp(i16::MIN as i32, i16::MAX as i32);
            mixed_right = (mixed_right * i32::from(self.master_volume_right) >> 7)
                .clamp(i16::MIN as i32, i16::MAX as i32);
            let history_index = usize::from(self.fir_history_index);
            self.fir_history_left[history_index] = self.echo_left[self.echo_ring_index] >> 1;
            self.fir_history_right[history_index] = self.echo_right[self.echo_ring_index] >> 1;
            let mut filtered_left = 0i32;
            let mut filtered_right = 0i32;
            for tap in 0..8 {
                let index = (history_index + tap + 1) & 7;
                filtered_left += i32::from(self.fir_history_left[index])
                    * i32::from(self.fir_coefficients[tap])
                    >> 6;
                filtered_right += i32::from(self.fir_history_right[index])
                    * i32::from(self.fir_coefficients[tap])
                    >> 6;
                if tap == 6 {
                    filtered_left = i32::from(filtered_left as i16);
                    filtered_right = i32::from(filtered_right as i16);
                }
            }
            filtered_left = filtered_left.clamp(i16::MIN as i32, i16::MAX as i32);
            filtered_right = filtered_right.clamp(i16::MIN as i32, i16::MAX as i32);
            mixed_left += filtered_left * i32::from(self.echo_mix_left) >> 7;
            mixed_right += filtered_right * i32::from(self.echo_mix_right) >> 7;
            let echo_write_left = (echo_input_left
                + (filtered_left * i32::from(self.echo_feedback) >> 7))
                .clamp(i16::MIN as i32, i16::MAX as i32)
                & !1;
            let echo_write_right = (echo_input_right
                + (filtered_right * i32::from(self.echo_feedback) >> 7))
                .clamp(i16::MIN as i32, i16::MAX as i32)
                & !1;
            if self.dsp_flags & 0x20 == 0 {
                self.echo_left[self.echo_ring_index] = echo_write_left as i16;
                self.echo_right[self.echo_ring_index] = echo_write_right as i16;
            }
            self.fir_history_index = (self.fir_history_index + 1) & 7;
            self.echo_ring_index += 1;
            self.echo_remaining_samples = self.echo_remaining_samples.saturating_sub(1);
            if self.echo_remaining_samples == 0 {
                self.echo_remaining_samples = self.echo_delay_samples.max(1);
                self.echo_ring_index = 0;
            } else if self.echo_ring_index >= self.echo_left.len() {
                self.echo_ring_index = 0;
            }
            if self.dsp_flags & 0x40 != 0 {
                mixed_left = 0;
                mixed_right = 0;
            }
            self.advance_noise();
            let mixed_left = mixed_left.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            let mixed_right = mixed_right.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            for channel in 0..channels {
                let index = sample_index * channels + channel;
                if let Some(slot) = audio_buffer.get_mut(index) {
                    *slot = if channels == 1 {
                        ((i32::from(mixed_left) + i32::from(mixed_right)) / 2) as i16
                    } else if channel & 1 == 0 {
                        mixed_left
                    } else {
                        mixed_right
                    };
                }
            }
        }
    }

    fn advance_noise(&mut self) {
        let rate = DSP_ENVELOPE_RATE_VALUES[usize::from(self.dsp_flags & 0x1f)];
        if rate == 0 {
            return;
        }
        self.noise_counter = self.noise_counter.wrapping_add(1);
        if self.noise_counter >= rate {
            let sample = i32::from(self.noise_sample);
            let bit = (sample & 1) ^ ((sample >> 1) & 1);
            let shifted = ((sample >> 1) & 0x3fff) | (bit << 14);
            self.noise_sample = (((shifted & 0x7fff) << 1) as i16) >> 1;
            self.noise_counter = 0;
        }
    }

    fn advance_voice_durations(&mut self) {
        for voice in &mut self.voices {
            if !voice.active {
                continue;
            }
            if voice.remaining_frames != 0 {
                voice.remaining_frames = voice.remaining_frames.saturating_sub(1);
                if voice.remaining_frames == 0 {
                    voice.begin_release();
                }
            }
        }
    }

    fn advance_envelopes(&mut self) {
        for voice in &mut self.voices {
            voice.advance_envelope();
        }
    }

    fn advance_pitch_slides(&mut self) {
        for voice in &mut self.voices {
            if !voice.active || voice.pitch_slide_frames == 0 {
                continue;
            }
            if voice.pitch_slide_frames == 1 {
                voice.base_phase_step = voice.pitch_slide_target;
            } else {
                let current = i64::from(voice.base_phase_step);
                let target = i64::from(voice.pitch_slide_target);
                let delta = (target - current) / i64::from(voice.pitch_slide_frames);
                voice.base_phase_step = (current + delta).clamp(0, i64::from(u32::MAX)) as u32;
            }
            voice.phase_step = voice.base_phase_step;
            voice.pitch_slide_frames -= 1;
        }
    }
}

const ENVELOPE_SUSTAIN: u8 = 0;
const ENVELOPE_ATTACK: u8 = 1;
const ENVELOPE_DECAY: u8 = 2;
const ENVELOPE_RELEASE: u8 = 3;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ModernVoice {
    active: bool,
    phase: u32,
    phase_step: u32,
    base_phase_step: u32,
    amplitude: i32,
    decay: u8,
    timbre: u8,
    #[serde(default)]
    instrument_timbre: u8,
    #[serde(default)]
    noise_enabled: bool,
    #[serde(default)]
    pan: i8,
    #[serde(default)]
    echo_send: bool,
    remaining_frames: u16,
    #[serde(default)]
    pitch_slide_target: u32,
    #[serde(default)]
    pitch_slide_frames: u8,
    #[serde(default)]
    peak_amplitude: i32,
    #[serde(default)]
    envelope_configured: bool,
    #[serde(default)]
    envelope_attack_frames: u8,
    #[serde(default)]
    envelope_decay_frames: u8,
    #[serde(default)]
    envelope_sustain: u8,
    #[serde(default)]
    envelope_release_frames: u8,
    #[serde(default)]
    envelope_stage: u8,
    #[serde(default)]
    envelope_frames_remaining: u8,
    #[serde(default)]
    sample_data: Vec<i16>,
    #[serde(default)]
    sample_loop_start: usize,
    #[serde(default)]
    sample_loops: bool,
    #[serde(default)]
    sample_position: u64,
    #[serde(default)]
    sample_step: u64,
    #[serde(default)]
    sample_backed: bool,
    #[serde(default)]
    dsp_pitch_counter: u16,
    #[serde(default)]
    render_pitch_word: u16,
    #[serde(default)]
    brr_block_start: usize,
    #[serde(default)]
    brr_started: bool,
    #[serde(default)]
    checkpoint_brr_decoder: Option<CheckpointBrrDecoder>,
    #[serde(default)]
    exact_pitch_word: u16,
    #[serde(default)]
    dsp_pitch_configured: bool,
    #[serde(default)]
    pitch_register_only: bool,
    #[serde(default)]
    start_delay_samples: usize,
    #[serde(default)]
    stereo_volume_configured: bool,
    #[serde(default)]
    volume_left: i8,
    #[serde(default)]
    volume_right: i8,
    #[serde(default)]
    dsp_adsr1: u8,
    #[serde(default)]
    dsp_adsr2: u8,
    #[serde(default)]
    dsp_gain: u8,
    #[serde(default)]
    dsp_envelope_configured: bool,
    #[serde(default)]
    exact_parameters_pending: bool,
    #[serde(default)]
    dsp_gain_level: u16,
    #[serde(default)]
    dsp_envelope_state: u8,
    #[serde(default)]
    dsp_rate_counter: u16,
    #[serde(default)]
    last_output_sample: i16,
    #[serde(default)]
    dsp_rate_phase: u32,
}

impl ModernVoice {
    fn begin_decay_or_sustain(&mut self) {
        if self.envelope_configured && self.envelope_decay_frames != 0 {
            self.envelope_stage = ENVELOPE_DECAY;
            self.envelope_frames_remaining = self.envelope_decay_frames;
        } else {
            self.envelope_stage = ENVELOPE_SUSTAIN;
            self.envelope_frames_remaining = 0;
            if self.envelope_configured {
                self.amplitude = self.sustain_amplitude();
            }
        }
    }

    fn begin_release(&mut self) {
        if self.dsp_envelope_configured {
            self.remaining_frames = 0;
            self.pitch_slide_frames = 0;
            self.dsp_envelope_state = 4;
            return;
        }
        if !self.active {
            return;
        }
        self.remaining_frames = 0;
        self.pitch_slide_frames = 0;
        if self.envelope_configured && self.envelope_release_frames != 0 {
            self.envelope_stage = ENVELOPE_RELEASE;
            self.envelope_frames_remaining = self.envelope_release_frames;
        } else {
            self.active = false;
            self.amplitude = 0;
        }
    }

    fn sustain_amplitude(&self) -> i32 {
        self.peak_amplitude
            .saturating_mul(i32::from(self.envelope_sustain))
            / 15
    }

    fn advance_envelope(&mut self) {
        if !self.active {
            return;
        }
        if self.dsp_envelope_configured {
            return;
        }
        match self.envelope_stage {
            ENVELOPE_ATTACK => {
                let remaining = i32::from(self.envelope_frames_remaining.max(1));
                self.amplitude += (self.peak_amplitude - self.amplitude) / remaining;
                self.envelope_frames_remaining = self.envelope_frames_remaining.saturating_sub(1);
                if self.envelope_frames_remaining == 0 {
                    self.amplitude = self.peak_amplitude;
                    self.begin_decay_or_sustain();
                }
            }
            ENVELOPE_DECAY => {
                let target = self.sustain_amplitude();
                let remaining = i32::from(self.envelope_frames_remaining.max(1));
                self.amplitude += (target - self.amplitude) / remaining;
                self.envelope_frames_remaining = self.envelope_frames_remaining.saturating_sub(1);
                if self.envelope_frames_remaining == 0 {
                    self.amplitude = target;
                    self.envelope_stage = ENVELOPE_SUSTAIN;
                }
            }
            ENVELOPE_RELEASE => {
                let remaining = i32::from(self.envelope_frames_remaining.max(1));
                self.amplitude -= self.amplitude / remaining;
                self.envelope_frames_remaining = self.envelope_frames_remaining.saturating_sub(1);
                if self.envelope_frames_remaining == 0 {
                    self.amplitude = 0;
                    self.active = false;
                    self.envelope_stage = ENVELOPE_SUSTAIN;
                }
            }
            _ => {}
        }
    }

    fn rescale_phase_step(&mut self, sample_rate: usize) {
        let sample_rate = sample_rate.max(1) as u64;
        self.phase_step = ((u64::from(self.base_phase_step) * 44_100) / sample_rate) as u32;
    }

    fn next_sample(&mut self) -> i32 {
        self.next_sample_with_ram(None)
    }

    fn next_sample_with_ram(&mut self, sample_ram: Option<&[u8]>) -> i32 {
        if self.start_delay_samples != 0 {
            self.start_delay_samples -= 1;
            return 0;
        }
        self.advance_dsp_envelope_for_output_sample();
        if self.sample_backed && !self.sample_data.is_empty() {
            let (mut index, fraction) = if self.dsp_pitch_configured {
                let pitch = self.render_pitch_word;
                let sum = u32::from(self.dsp_pitch_counter) + u32::from(pitch);
                let overflow = sum > 0xffff;
                self.dsp_pitch_counter = sum as u16;
                if overflow {
                    if let (Some(decoder), Some(sample_ram)) =
                        (&mut self.checkpoint_brr_decoder, sample_ram)
                    {
                        if let Some(block) = decoder.decode_next(sample_ram) {
                            let history_start = self.brr_block_start.saturating_add(13);
                            let history = [
                                self.sample_data.get(history_start).copied().unwrap_or(0),
                                self.sample_data
                                    .get(history_start + 1)
                                    .copied()
                                    .unwrap_or(0),
                                self.sample_data
                                    .get(history_start + 2)
                                    .copied()
                                    .unwrap_or(0),
                            ];
                            self.sample_data.clear();
                            self.sample_data.extend_from_slice(&history);
                            self.sample_data.extend_from_slice(&block);
                            self.brr_block_start = 3;
                        } else {
                            self.brr_block_start = self.sample_data.len();
                        }
                    } else if self.brr_started {
                        self.brr_block_start = self.brr_block_start.saturating_add(16);
                    } else {
                        self.brr_started = true;
                        self.brr_block_start = 0;
                    }
                }
                if !self.brr_started {
                    return 0;
                }
                (
                    self.brr_block_start + usize::from(self.dsp_pitch_counter >> 12),
                    ((self.dsp_pitch_counter >> 4) & 0xff) as u8,
                )
            } else {
                self.sample_position = self.sample_position.wrapping_add(self.sample_step);
                (
                    (self.sample_position >> 32) as usize,
                    (self.sample_position >> 24) as u8,
                )
            };
            if index >= self.sample_data.len() {
                if self.sample_loops && self.sample_loop_start < self.sample_data.len() {
                    let loop_len = self.sample_data.len() - self.sample_loop_start;
                    index = self.sample_loop_start + (index - self.sample_loop_start) % loop_len;
                    if self.dsp_pitch_configured {
                        self.brr_block_start = index & !15;
                    } else {
                        self.sample_position = (index as u64) << 32;
                    }
                } else {
                    // On the decode attempt after a non-looping BRR END, the
                    // DSP raises ENDX and forces the channel to release with
                    // zero gain. Preserve the rate counter: a later KON does
                    // not reset it.
                    if self.dsp_envelope_configured {
                        self.dsp_envelope_state = 4;
                        self.dsp_gain_level = 0;
                    }
                    self.active = false;
                    return 0;
                }
            }
            let sample = snes::apu::dsp_gaussian_interpolate(
                self.sample_at(index as isize - 3),
                self.sample_at(index as isize - 2),
                self.sample_at(index as isize - 1),
                self.sample_at(index as isize),
                fraction,
            );
            let gain = if self.dsp_envelope_configured {
                i32::from(self.dsp_gain_level)
            } else {
                self.amplitude
            };
            let sample = i32::from(sample) * gain >> 11;
            return sample;
        }
        self.advance_pitch_counter_without_decode();
        self.phase = self.phase.wrapping_add(self.phase_step);
        let amplitude = if self.dsp_envelope_configured {
            i32::from(self.dsp_gain_level)
        } else {
            self.amplitude
        };
        match self.timbre {
            0 => {
                if self.phase & 0x8000_0000 == 0 {
                    amplitude
                } else {
                    -amplitude
                }
            }
            1 => {
                let ramp = ((self.phase >> 20) as i32 & 0xfff) - 2048;
                ramp * amplitude / 2048
            }
            2 => {
                let tri = if self.phase & 0x8000_0000 == 0 {
                    (self.phase >> 19) as i32 & 0xfff
                } else {
                    4095 - ((self.phase >> 19) as i32 & 0xfff)
                };
                (tri - 2048) * amplitude / 2048
            }
            _ => {
                let bit = ((self.phase >> 31) ^ (self.phase >> 27) ^ (self.phase >> 23)) & 1;
                if bit == 0 {
                    amplitude / 2
                } else {
                    -amplitude / 2
                }
            }
        }
    }

    fn next_noise_sample(&mut self, noise_sample: i16) -> i32 {
        if self.start_delay_samples != 0 {
            self.start_delay_samples -= 1;
            return 0;
        }
        self.advance_pitch_counter_without_decode();
        self.advance_dsp_envelope_for_output_sample();
        let gain = if self.dsp_envelope_configured {
            i32::from(self.dsp_gain_level)
        } else {
            self.amplitude
        };
        i32::from(noise_sample) * gain >> 11
    }

    fn advance_inactive_pitch_counter(&mut self) {
        if !self.pitch_register_only {
            self.render_pitch_word = self.exact_pitch_word;
        }
        self.advance_pitch_counter_without_decode();
    }

    fn advance_pitch_counter_without_decode(&mut self) {
        if self.dsp_pitch_configured {
            self.dsp_pitch_counter = self.dsp_pitch_counter.wrapping_add(self.render_pitch_word);
        }
    }

    fn sample_at(&self, index: isize) -> i16 {
        if index < 0 || self.sample_data.is_empty() {
            return 0;
        }
        let index = index as usize;
        if index < self.sample_data.len() {
            return self.sample_data[index];
        }
        if self.sample_loops && self.sample_loop_start < self.sample_data.len() {
            let loop_len = self.sample_data.len() - self.sample_loop_start;
            return self.sample_data
                [self.sample_loop_start + (index - self.sample_loop_start) % loop_len];
        }
        0
    }

    fn initialize_dsp_envelope(&mut self) {
        self.dsp_gain_level = 0;
        self.dsp_envelope_state = if self.dsp_adsr1 & 0x80 == 0 { 3 } else { 0 };
    }

    fn advance_dsp_envelope_for_output_sample(&mut self) {
        if !self.dsp_envelope_configured {
            return;
        }
        self.advance_dsp_envelope_tick();
    }

    fn advance_dsp_envelope_tick(&mut self) {
        let use_gain = self.dsp_adsr1 & 0x80 == 0;
        let direct_gain = use_gain && self.dsp_gain & 0x80 == 0;
        if direct_gain && self.dsp_envelope_state != 4 {
            self.dsp_gain_level = u16::from(self.dsp_gain & 0x7f) * 16;
            return;
        }
        let rate_index = match self.dsp_envelope_state {
            0 => usize::from((self.dsp_adsr1 & 0x0f) * 2 + 1),
            1 => usize::from(((self.dsp_adsr1 & 0x70) >> 4) * 2 + 16),
            2 => usize::from(self.dsp_adsr2 & 0x1f),
            3 => usize::from(self.dsp_gain & 0x1f),
            _ => 0,
        };
        let rate = DSP_ENVELOPE_RATE_VALUES[rate_index];
        if self.dsp_envelope_state != 4 && rate != 0 {
            self.dsp_rate_counter = self.dsp_rate_counter.wrapping_add(1);
            if self.dsp_rate_counter < rate {
                return;
            }
            self.dsp_rate_counter = 0;
        } else if self.dsp_envelope_state != 4 {
            return;
        }
        match self.dsp_envelope_state {
            0 => {
                self.dsp_gain_level =
                    self.dsp_gain_level
                        .wrapping_add(if rate == 1 { 1024 } else { 32 });
                if self.dsp_gain_level >= 0x7e0 {
                    self.dsp_envelope_state = 1;
                }
                self.dsp_gain_level = self.dsp_gain_level.min(0x7ff);
            }
            1 => {
                self.dsp_gain_level = dsp_exp_decrease(self.dsp_gain_level);
                let sustain = (u16::from(self.dsp_adsr2 >> 5) + 1) * 0x100;
                if self.dsp_gain_level < sustain {
                    self.dsp_envelope_state = 2;
                }
            }
            2 => self.dsp_gain_level = dsp_exp_decrease(self.dsp_gain_level),
            3 => match (self.dsp_gain >> 5) & 3 {
                0 => self.dsp_gain_level = self.dsp_gain_level.saturating_sub(32),
                1 => self.dsp_gain_level = dsp_exp_decrease(self.dsp_gain_level),
                2 => self.dsp_gain_level = (self.dsp_gain_level + 32).min(0x7ff),
                _ => {
                    self.dsp_gain_level = (self.dsp_gain_level
                        + if self.dsp_gain_level < 0x600 { 32 } else { 8 })
                    .min(0x7ff);
                }
            },
            _ => {
                self.dsp_gain_level = self.dsp_gain_level.saturating_sub(8);
                if self.dsp_gain_level == 0 {
                    self.active = false;
                }
            }
        }
    }
}

const DSP_ENVELOPE_RATE_VALUES: [u16; 32] = [
    0, 2048, 1536, 1280, 1024, 768, 640, 512, 384, 320, 256, 192, 160, 128, 96, 80, 64, 48, 40, 32,
    24, 20, 16, 12, 10, 8, 6, 5, 4, 3, 2, 1,
];

fn dsp_exp_decrease(gain: u16) -> u16 {
    let step = (((i32::from(gain) - 1) >> 8) + 1) as u16;
    gain.saturating_sub(step)
}

struct DecodedBrrSample {
    pcm: Vec<i16>,
    block_addresses: Vec<usize>,
    loop_start: usize,
    loops: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CheckpointBrrDecoder {
    address: u16,
    previous_flags: u8,
    old: i16,
    older: i16,
    loop_address: u16,
}

impl CheckpointBrrDecoder {
    fn decode_next(&mut self, ram: &[u8]) -> Option<[i16; 16]> {
        if self.previous_flags & 1 != 0 {
            if self.previous_flags == 1 {
                return None;
            }
            self.address = self.loop_address;
        }
        let address = usize::from(self.address);
        let block = ram.get(address..address + 9)?;
        let header = block[0];
        self.previous_flags = header & 3;
        self.address = self.address.wrapping_add(9);
        let shift = header >> 4;
        let filter = (header >> 2) & 3;
        let mut output = [0i16; 16];
        for (nibble_index, decoded) in output.iter_mut().enumerate() {
            let byte = block[1 + nibble_index / 2];
            let mut value = i32::from(if nibble_index & 1 == 0 {
                byte >> 4
            } else {
                byte & 0x0f
            });
            if value > 7 {
                value -= 16;
            }
            value = if shift <= 12 {
                (value << shift) >> 1
            } else {
                (value >> 3) << 12
            };
            let old = i32::from(self.old);
            let older = i32::from(self.older);
            match filter {
                1 => value += old + (-old >> 4),
                2 => value += 2 * old + ((-3 * old) >> 5) - older + (older >> 4),
                3 => value += 2 * old + ((-13 * old) >> 6) - older + ((3 * older) >> 4),
                _ => {}
            }
            value = value.clamp(i16::MIN as i32, i16::MAX as i32);
            *decoded = (((value & 0x7fff) << 1) as i16) >> 1;
            self.older = self.old;
            self.old = *decoded;
        }
        Some(output)
    }
}

/// Rebuild a sample stream from the DSP's live BRR decoder state.
///
/// A static decode from the directory entry is insufficient after a loop whose
/// filter history differs from the first pass. Keep the 19 samples currently in
/// the hardware window, then decode subsequent blocks using the imported
/// `old`/`older` history and next-block address.
fn decode_brr_checkpoint_continuation(
    ram: &[u8],
    source: u8,
    address: usize,
    previous_flags: u8,
    old: i16,
    older: i16,
    window: [i16; 19],
) -> Option<(DecodedBrrSample, CheckpointBrrDecoder)> {
    const DIRECTORY: usize = 0x3c00;
    if ram.len() < 0x10000 {
        return None;
    }
    let entry = DIRECTORY + usize::from(source) * 4;
    let loop_address = usize::from(ram[entry + 2]) | (usize::from(ram[entry + 3]) << 8);
    Some((
        DecodedBrrSample {
            pcm: window.to_vec(),
            block_addresses: vec![address.wrapping_sub(9) & 0xffff],
            loop_start: 0,
            loops: false,
        },
        CheckpointBrrDecoder {
            address: address as u16,
            previous_flags,
            old,
            older,
            loop_address: loop_address as u16,
        },
    ))
}

fn sample_source_bytes(bank_id: u8, ram: Option<&[u8]>, address: usize) -> Option<[u8; 4]> {
    if let Some(ram) = ram {
        return Some(ram.get(address..address + 4)?.try_into().unwrap());
    }
    crate::modern_sample_bank::echo_bytes(bank_id, address)
}

fn decode_brr_bank_sample(bank_id: u8, source: u8) -> Option<DecodedBrrSample> {
    let sample = crate::modern_sample_bank::sample(bank_id, source)?;
    decode_brr_asset(sample.brr, sample.loop_offset)
}

fn decode_brr_asset(brr: &[u8], loop_offset: usize) -> Option<DecodedBrrSample> {
    const MAX_BLOCKS: usize = 4096;
    if brr.is_empty() || brr.len() % 9 != 0 || loop_offset % 9 != 0 {
        return None;
    }
    let mut pcm = Vec::new();
    let mut block_addresses = Vec::new();
    let mut address = 0usize;
    let mut loop_states = Vec::new();
    let mut old = 0i32;
    let mut older = 0i32;
    for _ in 0..MAX_BLOCKS {
        if address + 9 > brr.len() {
            return None;
        }
        if address == loop_offset {
            if let Some((_, loop_start)) = loop_states
                .iter()
                .find(|((seen_old, seen_older), _)| *seen_old == old && *seen_older == older)
            {
                if pcm.iter().all(|sample| *sample == 0) {
                    return None;
                }
                return Some(DecodedBrrSample {
                    pcm,
                    block_addresses,
                    loop_start: *loop_start,
                    loops: true,
                });
            }
            loop_states.push(((old, older), pcm.len()));
        }
        let header = brr[address];
        block_addresses.push(address);
        decode_brr_block(
            &brr[address + 1..address + 9],
            header,
            &mut old,
            &mut older,
            &mut pcm,
        );
        address += 9;
        if header & 1 != 0 {
            let loops = header & 2 != 0;
            if loops && loop_offset + 9 <= brr.len() {
                address = loop_offset;
                continue;
            }
            if pcm.iter().all(|sample| *sample == 0) {
                return None;
            }
            return Some(DecodedBrrSample {
                pcm,
                block_addresses,
                loop_start: 0,
                loops,
            });
        }
    }
    None
}

fn decode_brr_block(data: &[u8], header: u8, old: &mut i32, older: &mut i32, pcm: &mut Vec<i16>) {
    let shift = header >> 4;
    let filter = (header >> 2) & 3;
    for nibble_index in 0..16 {
        let byte = data[nibble_index / 2];
        let mut sample = if nibble_index & 1 == 0 {
            i32::from(byte >> 4)
        } else {
            i32::from(byte & 0x0f)
        };
        if sample > 7 {
            sample -= 16;
        }
        sample = if shift <= 12 {
            (sample << shift) >> 1
        } else {
            (sample >> 3) << 12
        };
        match filter {
            1 => sample += *old + (-*old >> 4),
            2 => sample += 2 * *old + ((-3 * *old) >> 5) - *older + (*older >> 4),
            3 => sample += 2 * *old + ((-13 * *old) >> 6) - *older + ((3 * *older) >> 4),
            _ => {}
        }
        sample = sample.clamp(i16::MIN as i32, i16::MAX as i32);
        let decoded = (((sample & 0x7fff) << 1) as i16 >> 1) as i16;
        *older = *old;
        *old = i32::from(decoded);
        pcm.push(decoded);
    }
}

fn decode_brr_sample(ram: &[u8], source: u8) -> Option<DecodedBrrSample> {
    const DIRECTORY: usize = 0x3c00;
    const MAX_BLOCKS: usize = 4096;
    if ram.len() < 0x10000 {
        return None;
    }
    let entry = DIRECTORY + usize::from(source) * 4;
    let start = usize::from(ram[entry]) | (usize::from(ram[entry + 1]) << 8);
    let loop_address = usize::from(ram[entry + 2]) | (usize::from(ram[entry + 3]) << 8);
    if start == 0 || start + 9 > ram.len() {
        return None;
    }

    let mut pcm = Vec::new();
    let mut block_addresses = Vec::new();
    let mut address = start;
    let mut loop_states = Vec::new();
    let mut old = 0i32;
    let mut older = 0i32;
    for _ in 0..MAX_BLOCKS {
        if address + 9 > ram.len() {
            return None;
        }
        if address == loop_address {
            if let Some((_, loop_start)) = loop_states
                .iter()
                .find(|((seen_old, seen_older), _)| *seen_old == old && *seen_older == older)
            {
                if pcm.iter().all(|sample| *sample == 0) {
                    return None;
                }
                return Some(DecodedBrrSample {
                    pcm,
                    block_addresses,
                    loop_start: *loop_start,
                    loops: true,
                });
            }
            loop_states.push(((old, older), pcm.len()));
        }
        let header = ram[address];
        block_addresses.push(address);
        decode_brr_block(
            &ram[address + 1..address + 9],
            header,
            &mut old,
            &mut older,
            &mut pcm,
        );
        address += 9;
        if header & 1 != 0 {
            let loops = header & 2 != 0;
            if loops && loop_address + 9 <= ram.len() {
                address = loop_address;
                continue;
            }
            if pcm.iter().all(|sample| *sample == 0) {
                return None;
            }
            return Some(DecodedBrrSample {
                pcm,
                block_addresses,
                loop_start: 0,
                loops,
            });
        }
    }
    None
}

fn sample_step_for_pitch(pitch: u8, output_rate: usize) -> u64 {
    let semitones = f64::from(pitch) - 60.0;
    let ratio = 2.0f64.powf(semitones / 12.0);
    (ratio * 32_000.0 / output_rate.max(1) as f64 * (1u64 << 32) as f64) as u64
}

fn sample_step_for_pitch_word(pitch_word: u16, output_rate: usize) -> u64 {
    let numerator = u128::from(pitch_word) * 32_000 * (1u128 << 32);
    (numerator / (4096 * output_rate.max(1) as u128)) as u64
}

fn is_deferred_voice_event(kind: &AudioEventKind) -> bool {
    matches!(
        kind,
        AudioEventKind::SetPitchWord { .. }
            | AudioEventKind::SetPitchRegisterWord { .. }
            | AudioEventKind::VoiceParameter { .. }
            | AudioEventKind::SetStereoVolume { .. }
            | AudioEventKind::SetDspEnvelope { .. }
            | AudioEventKind::SetEchoSend { .. }
            | AudioEventKind::SetNoise { .. }
            | AudioEventKind::SetEnvelope { .. }
            | AudioEventKind::PitchSlide { .. }
            | AudioEventKind::SetPan { .. }
            | AudioEventKind::SetDuration { .. }
            | AudioEventKind::NoteOn { .. }
            | AudioEventKind::KeyOnVoice { .. }
            | AudioEventKind::NoteOff { .. }
    )
}

fn resample_nearest(
    source: &[i16],
    destination: &mut [i16],
    source_samples: usize,
    destination_samples: usize,
    channels: usize,
) {
    if source_samples == 0 || destination_samples == 0 || channels == 0 {
        return;
    }
    let adder = source_samples as f32 / destination_samples as f32;
    let mut location = 0.0f32;
    for output_index in 0..destination_samples {
        let source_index = (location as usize).min(source_samples - 1);
        for channel in 0..channels {
            let source_offset = source_index * channels + channel;
            let destination_offset = output_index * channels + channel;
            if let (Some(&sample), Some(output)) = (
                source.get(source_offset),
                destination.get_mut(destination_offset),
            ) {
                *output = sample;
            }
        }
        location += adder;
    }
}

fn phase_step_for_code(code: u8) -> u32 {
    let note = u32::from(code & 0x3f);
    let freq_hz = 82 + note * 11;
    ((u64::from(freq_hz) << 32) / 44_100) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_output::{AudioEvent, AudioQueueState, AudioRouteState, DspWriteEvent};

    fn empty_frame_with_writes(writes: &[DspWriteEvent]) -> AudioEventFrame {
        AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), writes)
    }

    fn single_block_sample_ram(source: u8, header: u8, data: [u8; 8]) -> Vec<u8> {
        let mut ram = vec![0u8; 0x10000];
        let address = 0x5000usize;
        let entry = 0x3c00 + usize::from(source) * 4;
        ram[entry..entry + 2].copy_from_slice(&(address as u16).to_le_bytes());
        ram[entry + 2..entry + 4].copy_from_slice(&(address as u16).to_le_bytes());
        ram[address] = header;
        ram[address + 1..address + 9].copy_from_slice(&data);
        ram
    }

    #[test]
    fn decodes_brr_nibbles_from_the_spc_sample_directory() {
        let ram = single_block_sample_ram(3, 0x01, [0x17; 8]);
        let sample = decode_brr_sample(&ram, 3).expect("valid BRR sample");

        assert_eq!(sample.pcm.len(), 16);
        assert_eq!(&sample.pcm[..4], &[0, 3, 0, 3]);
        assert!(!sample.loops);
    }

    #[test]
    fn canonical_bank_decoder_matches_spc_directory_decoder() {
        let source = 3u8;
        let asset = crate::modern_sample_bank::sample(0, source).unwrap();
        let start = 0x5000usize;
        let mut ram = vec![0u8; 0x10000];
        let entry = 0x3c00 + usize::from(source) * 4;
        ram[entry..entry + 2].copy_from_slice(&(start as u16).to_le_bytes());
        ram[entry + 2..entry + 4]
            .copy_from_slice(&((start + asset.loop_offset) as u16).to_le_bytes());
        ram[start..start + asset.brr.len()].copy_from_slice(asset.brr);

        let canonical = decode_brr_bank_sample(0, source).unwrap();
        let legacy = decode_brr_sample(&ram, source).unwrap();

        assert_eq!(canonical.pcm, legacy.pcm);
        assert_eq!(canonical.loop_start, legacy.loop_start);
        assert_eq!(canonical.loops, legacy.loops);
    }

    #[test]
    fn decodes_out_of_line_brr_loop_body_after_first_end_block() {
        let source = 3u8;
        let mut ram = single_block_sample_ram(source, 0x03, [0x17; 8]);
        let first = 0x5000usize;
        let loop_address = first + 9;
        let entry = 0x3c00 + usize::from(source) * 4;
        ram[entry + 2..entry + 4].copy_from_slice(&(loop_address as u16).to_le_bytes());
        ram[loop_address] = 0x03;
        ram[loop_address + 1..loop_address + 9].copy_from_slice(&[0x71; 8]);

        let sample = decode_brr_sample(&ram, source).expect("looping BRR sample");

        assert!(sample.loops);
        assert_eq!(sample.pcm.len(), 48);
        assert_eq!(sample.loop_start, 32);
    }

    #[test]
    fn checkpoint_continuation_preserves_live_brr_filter_history() {
        let source = 3u8;
        let ram = single_block_sample_ram(source, 0x05, [0; 8]);
        let address = 0x5000usize;
        let window = std::array::from_fn(|index| index as i16);

        let (sample, mut decoder) =
            decode_brr_checkpoint_continuation(&ram, source, address, 0, 1000, 500, window)
                .expect("checkpoint continuation");

        assert_eq!(&sample.pcm[..19], &window);
        let block = decoder.decode_next(&ram).expect("next filtered block");
        assert_eq!(block[0], 937);
        assert_eq!(sample.pcm.len(), 19);
        assert!(!sample.loops);

        // Zero history produces a different next block, proving a checkpoint
        // continuation is not interchangeable with the static catalog sample.
        let (_, mut zero_history) =
            decode_brr_checkpoint_continuation(&ram, source, address, 0, 0, 0, window)
                .expect("zero-history continuation");
        let zero_block = zero_history.decode_next(&ram).expect("zero-history block");
        assert_eq!(zero_block[0], 0);
        assert_ne!(zero_block[0], block[0]);
    }

    #[test]
    fn checkpoint_brr_loop_decodes_on_demand_without_growing_pcm_storage() {
        let source = 3u8;
        let ram = single_block_sample_ram(source, 0x07, [0x11; 8]);
        let address = 0x5000usize;
        let window = [0i16; 19];
        let (sample, decoder) =
            decode_brr_checkpoint_continuation(&ram, source, address, 0, 0, 0, window)
                .expect("checkpoint continuation");
        let mut voice = ModernVoice {
            active: true,
            amplitude: 2048,
            sample_data: sample.pcm,
            sample_backed: true,
            dsp_pitch_configured: true,
            exact_pitch_word: 0xffff,
            render_pitch_word: 0xffff,
            dsp_pitch_counter: 1,
            brr_block_start: 3,
            brr_started: true,
            checkpoint_brr_decoder: Some(decoder),
            ..ModernVoice::default()
        };

        for _ in 0..10_000 {
            voice.next_sample_with_ram(Some(&ram));
        }

        assert!(voice.active);
        assert_eq!(voice.sample_data.len(), 19);
        assert_eq!(voice.brr_block_start, 3);
    }

    #[test]
    fn note_on_uses_brr_sample_when_sample_ram_is_available() {
        let ram = single_block_sample_ram(3, 0x83, [0x77; 8]);
        let mut engine = ModernAudioEngine::default();
        let mut frame = empty_frame_with_writes(&[]);
        frame.events.push(AudioEvent {
            sample_offset: 0,
            timer_cycles: 0,
            kind: AudioEventKind::SetPitchWord {
                voice: 0,
                pitch_word: 0x1000,
            },
            parity_dsp: None,
        });
        frame.events.push(AudioEvent {
            sample_offset: 4,
            timer_cycles: 0,
            kind: AudioEventKind::NoteOn {
                voice: 0,
                pitch: 60,
                instrument: 3,
                volume: 127,
            },
            parity_dsp: None,
        });
        let mut audio = [0i16; 1470];

        engine.render_frame_with_sample_ram(&frame, &mut audio, 735, 2, Some(&ram));

        assert!(engine.voices[0].sample_backed);
        assert_eq!(engine.voices[0].sample_data.len(), 32);
        assert_eq!(
            engine.voices[0].sample_step,
            sample_step_for_pitch_word(0x1000, 32_000)
        );
        assert_eq!(&audio[..10], &[0; 10]);
        assert!(audio.iter().any(|sample| *sample != 0));
    }

    #[test]
    fn normal_render_uses_canonical_bank_without_sample_ram() {
        let mut engine = ModernAudioEngine::default();
        engine.select_sample_bank(1);
        let mut frame = empty_frame_with_writes(&[]);
        frame.events.push(AudioEvent {
            sample_offset: 0,
            timer_cycles: 0,
            kind: AudioEventKind::NoteOn {
                voice: 0,
                pitch: 60,
                instrument: 3,
                volume: 127,
            },
            parity_dsp: None,
        });
        let mut audio = [0i16; 1470];

        engine.render_frame(&frame, &mut audio, 735, 2);

        assert_eq!(engine.sample_bank_id(), 1);
        assert!(engine.voices[0].sample_backed);
        assert!(audio.iter().any(|sample| *sample != 0));
    }

    #[test]
    fn zero_dsp_pitch_freezes_the_current_brr_interpolation_position() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[0];
        voice.active = true;
        voice.sample_backed = true;
        voice.sample_data = (0..32).map(|sample| sample * 100).collect();
        voice.brr_started = true;
        voice.dsp_pitch_configured = true;
        voice.exact_pitch_word = 0x0100;
        voice.render_pitch_word = 0x0100;
        voice.amplitude = 2048;

        let before_zero = voice.next_sample();
        voice.exact_pitch_word = 0;
        voice.render_pitch_word = 0;
        let after_zero = voice.next_sample();

        assert_eq!(after_zero, before_zero);
        assert_eq!(voice.dsp_pitch_counter, 0x0100);
    }

    #[test]
    fn non_looping_brr_end_forces_zero_gain_release_state() {
        let mut voice = ModernVoice {
            active: true,
            sample_backed: true,
            sample_data: vec![1; 16],
            brr_started: true,
            brr_block_start: 16,
            dsp_pitch_configured: true,
            render_pitch_word: 0,
            dsp_envelope_configured: true,
            dsp_adsr1: 0xff,
            dsp_adsr2: 0xff,
            dsp_envelope_state: 2,
            dsp_gain_level: 1024,
            ..ModernVoice::default()
        };

        assert_eq!(voice.next_sample(), 0);
        assert!(!voice.active);
        assert_eq!(voice.dsp_gain_level, 0);
        assert_eq!(voice.dsp_envelope_state, 4);

        voice.dsp_envelope_state = 2;
        voice.begin_release();
        assert_eq!(voice.dsp_envelope_state, 4);
    }

    #[test]
    fn renders_nonzero_audio_from_apu_port_event_without_dsp() {
        let mut engine = ModernAudioEngine::default();
        let mut frame = empty_frame_with_writes(&[]);
        frame.events.push(AudioEvent {
            sample_offset: 0,
            timer_cycles: 0,
            kind: AudioEventKind::ApuPorts {
                write: [0x12, 0, 0, 0],
                pending: [0, 0, 0, 0],
                input: [0x12, 0, 0, 0],
                spc_in: [0; 4],
                spc_out: [0; 4],
            },
            parity_dsp: None,
        });
        frame.queue = AudioQueueState {
            input: [0x12, 0, 0, 0],
            ..AudioQueueState::default()
        };
        let mut audio = [0i16; 16];

        let stats = engine.render_frame(&frame, &mut audio, 8, 2);

        assert!(audio.iter().any(|sample| *sample != 0));
        assert_eq!(stats.triggered_voices, 1);
        assert_eq!(stats.ignored_events, 0);
        assert_eq!(stats.active_voices, 1);
        assert_eq!(engine.last_stats(), stats);
    }

    #[test]
    fn sequenced_quiet_frame_does_not_reinterpret_pending_apu_port() {
        let mut engine = ModernAudioEngine::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                pending: [0, 0, 0, 0x0a],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(route, &[]);
        frame.sequenced = true;
        let mut audio = [1i16; 16];

        let stats = engine.render_frame(&frame, &mut audio, 8, 2);

        assert_eq!(audio, [0; 16]);
        assert_eq!(stats.triggered_voices, 0);
        assert_eq!(stats.active_voices, 0);
    }

    #[test]
    fn key_on_and_key_off_events_control_modern_voices() {
        let mut engine = ModernAudioEngine::default();
        let mut audio = [0i16; 16];
        let key_on = empty_frame_with_writes(&[DspWriteEvent::new(0x4c, 0x03, 0, 0)]);

        let on_stats = engine.render_frame(&key_on, &mut audio, 8, 2);

        assert!(audio.iter().any(|sample| *sample != 0));
        assert_eq!(on_stats.triggered_voices, 2);

        let key_off = empty_frame_with_writes(&[DspWriteEvent::new(0x5c, 0x03, 0, 0)]);
        let off_stats = engine.render_frame(&key_off, &mut audio, 8, 2);

        assert_eq!(off_stats.active_voices, 0);
    }

    #[test]
    fn pitch_slide_advances_over_its_declared_frames_instead_of_jumping() {
        let mut engine = ModernAudioEngine::default();
        let mut frame = empty_frame_with_writes(&[]);
        frame.events.extend([
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::NoteOn {
                    voice: 0,
                    pitch: 10,
                    instrument: 0,
                    volume: 80,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::SetDuration {
                    voice: 0,
                    frames: 4,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::PitchSlide {
                    voice: 0,
                    target_pitch: 20,
                    frames: 2,
                },
                parity_dsp: None,
            },
        ]);
        let start = phase_step_for_code(10);
        let target = phase_step_for_code(20);
        let mut audio = [0i16; 16];

        engine.render_frame(&frame, &mut audio, 8, 2);

        assert!(engine.voices[0].base_phase_step > start);
        assert!(engine.voices[0].base_phase_step < target);

        engine.render_frame(&empty_frame_with_writes(&[]), &mut audio, 8, 2);
        assert_eq!(engine.voices[0].base_phase_step, target);
    }

    #[test]
    fn typed_envelope_attacks_then_releases_instead_of_being_overwritten() {
        let mut engine = ModernAudioEngine::default();
        let mut note_on = empty_frame_with_writes(&[]);
        note_on.events.extend([
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::SetEnvelope {
                    voice: 0,
                    attack: 4,
                    decay: 2,
                    sustain: 8,
                    release: 2,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::NoteOn {
                    voice: 0,
                    pitch: 24,
                    instrument: 0,
                    volume: 100,
                },
                parity_dsp: None,
            },
        ]);
        let mut audio = [0i16; 16];

        engine.render_frame(&note_on, &mut audio, 8, 2);
        let first_attack = engine.voices[0].amplitude;
        engine.render_frame(&empty_frame_with_writes(&[]), &mut audio, 8, 2);
        let second_attack = engine.voices[0].amplitude;

        assert!(first_attack > 0 && first_attack < 1200);
        assert!(second_attack > first_attack && second_attack < 1200);

        let mut note_off = empty_frame_with_writes(&[]);
        note_off.events.push(AudioEvent {
            sample_offset: 0,
            timer_cycles: 0,
            kind: AudioEventKind::NoteOff { voice: 0 },
            parity_dsp: None,
        });
        engine.render_frame(&note_off, &mut audio, 8, 2);
        assert!(engine.voices[0].active);
        assert!(engine.voices[0].amplitude > 0);

        engine.render_frame(&empty_frame_with_writes(&[]), &mut audio, 8, 2);
        assert!(!engine.voices[0].active);
        assert_eq!(engine.voices[0].amplitude, 0);
    }

    #[test]
    fn noise_selection_survives_note_on_and_restores_instrument_waveform() {
        let mut engine = ModernAudioEngine::default();
        let mut noise_note = empty_frame_with_writes(&[]);
        noise_note.events.extend([
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::SetNoise {
                    voice: 0,
                    enabled: true,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::NoteOn {
                    voice: 0,
                    pitch: 24,
                    instrument: 0,
                    volume: 80,
                },
                parity_dsp: None,
            },
        ]);
        let mut audio = [0i16; 16];

        engine.render_frame(&noise_note, &mut audio, 8, 2);
        assert_eq!(engine.voices[0].timbre, 3);

        let mut tonal_note = empty_frame_with_writes(&[]);
        tonal_note.events.extend([
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::SetNoise {
                    voice: 0,
                    enabled: false,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::NoteOn {
                    voice: 0,
                    pitch: 24,
                    instrument: 1,
                    volume: 80,
                },
                parity_dsp: None,
            },
        ]);

        engine.render_frame(&tonal_note, &mut audio, 8, 2);
        assert_eq!(engine.voices[0].timbre, 1);
    }

    #[test]
    fn tonal_instrument_fifteen_does_not_alias_the_noise_waveform() {
        let mut engine = ModernAudioEngine::default();
        let mut frame = empty_frame_with_writes(&[]);
        frame.events.extend([
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::SetNoise {
                    voice: 0,
                    enabled: false,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::NoteOn {
                    voice: 0,
                    pitch: 60,
                    instrument: 15,
                    volume: 80,
                },
                parity_dsp: None,
            },
        ]);
        let mut audio = [0i16; 16];

        engine.render_frame(&frame, &mut audio, 8, 2);

        assert_eq!(engine.voices[0].instrument_timbre, 0);
        assert_eq!(engine.voices[0].timbre, 0);
    }

    #[test]
    fn typed_pan_produces_distinct_stereo_channels() {
        let mut engine = ModernAudioEngine::default();
        let mut frame = empty_frame_with_writes(&[]);
        frame.events.extend([
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::SetPan {
                    voice: 0,
                    pan: -127,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::NoteOn {
                    voice: 0,
                    pitch: 24,
                    instrument: 0,
                    volume: 80,
                },
                parity_dsp: None,
            },
        ]);
        let mut audio = [0i16; 16];

        engine.render_frame(&frame, &mut audio, 8, 2);

        assert!(audio.chunks_exact(2).any(|pair| pair[0] != 0));
        assert!(audio.chunks_exact(2).all(|pair| pair[1] == 0));
    }

    #[test]
    fn typed_echo_send_produces_a_checkpointed_stereo_tail() {
        let mut engine = ModernAudioEngine::default();
        let mut note = empty_frame_with_writes(&[]);
        note.events.extend([
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::GlobalParameter {
                    register: 0x6c,
                    value: 0,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::GlobalParameter {
                    register: 0x2c,
                    value: 64,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::GlobalParameter {
                    register: 0x3c,
                    value: 64,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::GlobalParameter {
                    register: 0x0f,
                    value: 127,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::SetEchoSend {
                    voice: 0,
                    enabled: true,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: 0,
                timer_cycles: 0,
                kind: AudioEventKind::NoteOn {
                    voice: 0,
                    pitch: 24,
                    instrument: 0,
                    volume: 80,
                },
                parity_dsp: None,
            },
        ]);
        let mut first = [0i16; 16];
        engine.render_frame(&note, &mut first, 8, 2);

        let mut stop = empty_frame_with_writes(&[]);
        stop.events.push(AudioEvent {
            sample_offset: 0,
            timer_cycles: 0,
            kind: AudioEventKind::NoteOff { voice: 0 },
            parity_dsp: None,
        });
        let mut tail = [0i16; 16];
        engine.render_frame(&stop, &mut tail, 8, 2);

        assert!(first.iter().any(|sample| *sample != 0));
        assert!(tail.iter().any(|sample| *sample != 0));
        assert!(engine.echo_left.iter().any(|sample| *sample != 0));
        assert!(engine.echo_right.iter().any(|sample| *sample != 0));
    }
}
