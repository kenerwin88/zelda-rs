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

const fn default_music_volume() -> u8 {
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

#[derive(Clone, Debug, Default)]
struct DebugMixSamples(Vec<[i32; 4]>);

impl PartialEq for DebugMixSamples {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for DebugMixSamples {}

#[derive(Clone, Debug, Default)]
struct DebugVoicePositions([Vec<u64>; MODERN_AUDIO_VOICES]);

impl PartialEq for DebugVoicePositions {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for DebugVoicePositions {}

#[derive(Clone, Debug, Default)]
struct DebugVoicePitchWords([Vec<u16>; MODERN_AUDIO_VOICES]);

impl PartialEq for DebugVoicePitchWords {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for DebugVoicePitchWords {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct EchoRamRegion {
    start: usize,
    left: Vec<i16>,
    right: Vec<i16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingDspKeyOn {
    pitch: u8,
    instrument: u8,
    volume: u8,
    #[serde(default)]
    write_phase: u8,
}

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
    sample_bank_generation: u32,
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
    #[serde(default = "default_music_volume")]
    music_volume: u8,
    #[serde(default)]
    fir_coefficients: [i8; 8],
    #[serde(default)]
    echo_delay_samples: u16,
    /// EDL register value waiting for phase 29 at echo offset zero.
    #[serde(default)]
    echo_delay_register_samples: u16,
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
    #[serde(default)]
    echo_preserved_regions: Vec<EchoRamRegion>,
    #[serde(default = "default_noise_sample")]
    noise_sample: i16,
    #[serde(default)]
    noise_counter: u16,
    #[serde(default)]
    dsp_global_counter: u16,
    #[serde(default)]
    dsp_rendered_samples: u64,
    /// Final S-DSP mix waiting in the one-sample DAC/output staging register.
    #[serde(default)]
    dsp_output_left: i16,
    #[serde(default)]
    dsp_output_right: i16,
    /// Inputs retained for a final-output register write that lands after the
    /// internal mix but before this staged sample reaches the DAC.
    #[serde(default)]
    dsp_output_raw_main_left: i16,
    #[serde(default)]
    dsp_output_raw_main_right: i16,
    #[serde(default)]
    dsp_output_filtered_left: i16,
    #[serde(default)]
    dsp_output_filtered_right: i16,
    /// Voice mix accumulated for the in-flight DSP cycle's echo write.
    #[serde(default)]
    dsp_output_echo_input_left: i16,
    #[serde(default)]
    dsp_output_echo_input_right: i16,
    /// S-DSP polls KON after every other output sample.
    #[serde(default)]
    dsp_even_cycle: bool,
    /// KOFF writes remain pending until the same every-other-sample register
    /// poll used by the hardware voice pipeline.
    #[serde(default)]
    pending_dsp_key_off_mask: u8,
    #[serde(default)]
    pending_dsp_key_off_delays: [u8; MODERN_AUDIO_VOICES],
    #[serde(default)]
    checkpoint_sample_prefix: Vec<i16>,
    #[serde(default)]
    checkpoint_sample_offset: u16,
    last_stats: ModernAudioFrameStats,
    #[serde(skip, default)]
    debug_voice_samples: DebugVoiceSamples,
    #[serde(skip, default)]
    debug_voice_gains: DebugVoiceSamples,
    #[serde(skip, default)]
    debug_mix_samples: DebugMixSamples,
    #[serde(skip, default)]
    debug_voice_positions: DebugVoicePositions,
    #[serde(skip, default)]
    debug_voice_pitch_words: DebugVoicePitchWords,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct ModernVoiceDebugState {
    pub active: bool,
    pub volume_left: i8,
    pub volume_right: i8,
    pub echo_send: bool,
    pub pitch: u16,
    pub pitch_counter: u16,
    pub dsp_sample_position: u64,
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
    pub sample_loop_start: usize,
    pub brr_block_start: usize,
}

impl Default for ModernAudioEngine {
    fn default() -> Self {
        Self {
            voices: std::array::from_fn(|_| ModernVoice::default()),
            last_music: MusicControlState::default(),
            last_ports: [0; 4],
            sample_bank_id: 0,
            sample_bank_generation: 0,
            echo_left: Vec::new(),
            echo_right: Vec::new(),
            echo_mix_left: default_echo_mix(),
            echo_mix_right: default_echo_mix(),
            echo_feedback: default_echo_feedback(),
            master_volume_left: default_master_volume(),
            master_volume_right: default_master_volume(),
            music_volume: default_music_volume(),
            fir_coefficients: [0; 8],
            // Hardware EDL starts at zero. Represent its zero-byte ring as a
            // one-sample sentinel so the cursor remains pinned at zero.
            echo_delay_samples: 1,
            echo_delay_register_samples: 1,
            dsp_flags: 0x20,
            pitch_modulation_mask: 0,
            noise_enable_mask: 0,
            fir_history_left: [0; 8],
            fir_history_right: [0; 8],
            fir_history_index: 0,
            echo_ring_index: 0,
            echo_remaining_samples: 1,
            echo_start_page: 0xc8,
            echo_ram_initialized: false,
            echo_preserved_regions: Vec::new(),
            noise_sample: default_noise_sample(),
            noise_counter: 0,
            dsp_global_counter: 0,
            dsp_rendered_samples: 0,
            dsp_output_left: 0,
            dsp_output_right: 0,
            dsp_output_raw_main_left: 0,
            dsp_output_raw_main_right: 0,
            dsp_output_filtered_left: 0,
            dsp_output_filtered_right: 0,
            dsp_output_echo_input_left: 0,
            dsp_output_echo_input_right: 0,
            dsp_even_cycle: false,
            pending_dsp_key_off_mask: 0,
            pending_dsp_key_off_delays: [0; MODERN_AUDIO_VOICES],
            checkpoint_sample_prefix: Vec::new(),
            checkpoint_sample_offset: 0,
            last_stats: ModernAudioFrameStats::default(),
            debug_voice_samples: DebugVoiceSamples::default(),
            debug_voice_gains: DebugVoiceSamples::default(),
            debug_mix_samples: DebugMixSamples::default(),
            debug_voice_positions: DebugVoicePositions::default(),
            debug_voice_pitch_words: DebugVoicePitchWords::default(),
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

    /// Publish a completed SPC upload without replacing DSP-owned echo state.
    ///
    /// The song-bank stream changes instrument data in shared APU RAM while
    /// the S-DSP continues advancing its echo cursor and FIR history. Those
    /// live values belong to the renderer, so completing the host transfer
    /// changes the BRR bank identity but must not re-seed the echo ring.
    pub fn complete_sample_bank_upload(&mut self, bank_id: u8, generation: u32) {
        // Validate eagerly so a corrupt upload cannot fail later in mixing.
        let _ = crate::modern_sample_bank::bank_name(bank_id);
        self.sample_bank_id = bank_id;
        self.sample_bank_generation = generation;
    }

    pub fn sample_bank_id(&self) -> u8 {
        self.sample_bank_id
    }
    pub fn seed_dsp_checkpoint_state(&mut self, sample_ram: &[u8], dsp: &[u8]) {
        if dsp.len() < 3022 {
            return;
        }
        self.checkpoint_sample_offset =
            u16::from_le_bytes([dsp[3020], dsp[3021]]).min(DSP_SAMPLES_PER_FRAME as u16);
        let prefix_samples = usize::from(self.checkpoint_sample_offset) * 2;
        self.checkpoint_sample_prefix = (0..prefix_samples)
            .map(|index| {
                let offset = 884 + index * 2;
                i16::from_le_bytes([dsp[offset], dsp[offset + 1]])
            })
            .collect();
        self.master_volume_left = dsp[821] as i8;
        self.dsp_even_cycle = dsp[818] != 0;
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
                voice.dsp_terminal_block_start = sample.terminal_block_start;
                voice.dsp_end_pipeline = 0;
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
        let mut deferred_music_volumes = Vec::new();
        let mut deferred_voice_events = Vec::new();
        let has_modern_intent = frame.events.iter().any(|event| {
            matches!(
                event.kind,
                AudioEventKind::PlayMusic { .. }
                    | AudioEventKind::StopMusic
                    | AudioEventKind::PlaySfx { .. }
                    | AudioEventKind::SetTempo { .. }
                    | AudioEventKind::SetMusicVolume { .. }
                    | AudioEventKind::SetNoteOrigin { .. }
                    | AudioEventKind::SetPitchWord { .. }
                    | AudioEventKind::SetStereoVolume { .. }
                    | AudioEventKind::SetDspEnvelope { .. }
                    | AudioEventKind::NoteOn { .. }
                    | AudioEventKind::DspKeyOn { .. }
                    | AudioEventKind::DspKeyOff { .. }
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
                let mut deferred = event.clone();
                let retrigger_shift = deferred_event_voice(&event.kind)
                    .and_then(|voice| {
                        let state = self.voices.get(voice)?;
                        Some(deferred_retrigger_shift(
                            frame,
                            event,
                            state.key_on_count,
                            state.active,
                            state.note_origin,
                        ))
                    })
                    .unwrap_or(0);
                let retriggered_note_off = matches!(event.kind, AudioEventKind::NoteOff { .. })
                    && deferred_event_voice(&event.kind).is_some_and(|voice| {
                        self.voices
                            .get(voice)
                            .is_some_and(|voice| voice.key_on_count > 1)
                    });
                deferred.sample_offset = deferred_voice_sample_offset_with_bank_generation(
                    event,
                    retrigger_shift,
                    retriggered_note_off,
                    self.dsp_global_counter,
                    self.sample_bank_generation,
                );
                deferred_voice_events.push(deferred);
                stats.understood_events += 1;
                if matches!(
                    event.kind,
                    AudioEventKind::NoteOn { .. }
                        | AudioEventKind::DspKeyOn { .. }
                        | AudioEventKind::KeyOnVoice { .. }
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
                AudioEventKind::SampleBankState {
                    bank_id,
                    generation,
                } => {
                    stats.understood_events += 1;
                    if *generation != self.sample_bank_generation {
                        self.sample_bank_generation = *generation;
                        let echo_ram_initialized = self.echo_ram_initialized;
                        self.select_sample_bank(*bank_id);
                        if sample_ram.is_some() {
                            // The exact bridge owns a live echo ring already
                            // synchronized with SPC RAM. A bank selection must
                            // not replace that runtime state with a catalog seed.
                            self.echo_ram_initialized = echo_ram_initialized;
                        }
                        self.echo_mix_left = 0;
                        self.echo_mix_right = 0;
                        for voice in &mut self.voices {
                            voice.begin_release();
                        }
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
                AudioEventKind::SetMusicVolume { value } => {
                    stats.understood_events += 1;
                    if event.sample_offset > 0 {
                        deferred_music_volumes.push((event.sample_offset as usize, *value));
                    } else {
                        self.music_volume = *value;
                    }
                }
                AudioEventKind::StopMusic => {
                    stats.understood_events += 1;
                    if !frame.sequenced {
                        for voice in &mut self.voices {
                            voice.begin_release();
                        }
                    }
                }
                AudioEventKind::SetNoteOrigin { voice, origin } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.note_origin = Some(*origin);
                    }
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
                AudioEventKind::SetEnvelopeRateCounter {
                    voice,
                    rate_counter,
                } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.dsp_rate_counter = *rate_counter;
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
                AudioEventKind::DspKeyOn {
                    voice,
                    pitch,
                    instrument,
                    volume,
                } => {
                    stats.understood_events += 1;
                    self.schedule_dsp_key_on(
                        *voice as usize,
                        *pitch,
                        *instrument,
                        *volume,
                        event.timer_cycles,
                    );
                    stats.triggered_voices += 1;
                }
                AudioEventKind::DspKeyOff { voice } => {
                    stats.understood_events += 1;
                    self.schedule_dsp_key_off(*voice as usize, event.timer_cycles);
                }
                AudioEventKind::RetriggerVoice { voice } => {
                    stats.understood_events += 1;
                    self.retrigger_voice(*voice as usize);
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.start_delay_samples = event.sample_offset.max(0) as usize;
                    }
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
                        voice.timbre = if *enabled {
                            3
                        } else {
                            voice.instrument_timbre % 3
                        };
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
                            self.schedule_staged_dsp_key_on(voice, event.timer_cycles);
                            stats.triggered_voices += 1;
                        }
                    }
                }
                AudioEventKind::VoiceKeyOff { mask } => {
                    stats.understood_events += 1;
                    for voice in 0..MODERN_AUDIO_VOICES {
                        if mask & (1 << voice) != 0 {
                            self.schedule_dsp_key_off(voice, event.timer_cycles);
                        }
                    }
                }
                AudioEventKind::VoiceParameter {
                    voice,
                    parameter,
                    value,
                } => {
                    stats.understood_events += 1;
                    let application_offset = deferred_voice_sample_offset_with_bank_generation(
                        event,
                        0,
                        false,
                        self.dsp_global_counter,
                        self.sample_bank_generation,
                    );
                    if application_offset < 0 {
                        self.handle_backdated_voice_parameter(*voice, *parameter, *value);
                    } else if application_offset > 0 {
                        let mut deferred = event.clone();
                        deferred.sample_offset = application_offset;
                        deferred_voice_events.push(deferred);
                    } else {
                        self.handle_voice_parameter(*voice, *parameter, *value);
                    }
                }
                AudioEventKind::EchoParameter { parameter, value } => {
                    let register = match parameter {
                        EchoParameterKind::VolumeLeft => 0x2c,
                        EchoParameterKind::VolumeRight => 0x3c,
                        EchoParameterKind::Feedback => 0x0d,
                        EchoParameterKind::EnableMask => 0x4d,
                        EchoParameterKind::Fir(index) => (index << 4) | 0x0f,
                        EchoParameterKind::Delay => 0x7d,
                        EchoParameterKind::StartAddress => 0x6d,
                    };
                    let application_offset = dsp_global_application_sample_offset(event, register);
                    if application_offset < 0 {
                        if self.handle_backdated_output_parameter(register, *value) {
                            stats.understood_events += 1;
                        } else {
                            stats.ignored_events += 1;
                        }
                    } else if application_offset > 0 {
                        deferred_globals.push((application_offset as usize, register, *value));
                        stats.understood_events += 1;
                    } else if self.handle_global_parameter(register, *value) {
                        stats.understood_events += 1;
                    } else {
                        stats.ignored_events += 1;
                    }
                }
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
                    let application_offset = dsp_global_application_sample_offset(event, *register);
                    if application_offset < 0 {
                        if self.handle_backdated_output_parameter(*register, *value) {
                            stats.understood_events += 1;
                        } else {
                            stats.ignored_events += 1;
                        }
                    } else if application_offset > 0 {
                        deferred_globals.push((application_offset as usize, *register, *value));
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
        deferred_music_volumes.sort_unstable_by_key(|event| event.0);
        deferred_voice_events.sort_by_key(|event| event.sample_offset);
        if samples_per_channel != 0 && channels != 0 {
            // Uploads change echo RAM contents, not the DSP's live cursor or
            // FIR phase. Rebuild the cached ring without resetting that state.
            self.resize_echo_ring_from_ram(sample_ram);
            let native_samples = if (400..=DSP_SAMPLES_PER_FRAME).contains(&samples_per_channel) {
                samples_per_channel
            } else {
                DSP_SAMPLES_PER_FRAME
            };
            let mut native_audio = vec![0i16; native_samples.saturating_mul(channels)];
            let sample_start = if channels == 2 {
                let prefix_len = self.checkpoint_sample_prefix.len().min(native_audio.len());
                native_audio[..prefix_len]
                    .copy_from_slice(&self.checkpoint_sample_prefix[..prefix_len]);
                usize::from(self.checkpoint_sample_offset).min(native_samples)
            } else {
                0
            };
            self.mix(
                &mut native_audio,
                sample_start,
                native_samples,
                channels,
                &deferred_globals,
                &deferred_music_volumes,
                &deferred_voice_events,
                sample_ram,
            );
            self.checkpoint_sample_prefix.clear();
            self.checkpoint_sample_offset = 0;
            if native_samples == samples_per_channel {
                audio_buffer[..count].copy_from_slice(&native_audio[..count]);
            } else {
                resample_nearest(
                    &native_audio,
                    audio_buffer,
                    native_samples,
                    samples_per_channel,
                    channels,
                );
            }
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
                dsp_sample_position: voice.dsp_sample_position,
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
                sample_loop_start: voice.sample_loop_start,
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

    pub fn echo_debug_config(&self) -> (u8, u16, bool) {
        (
            self.echo_start_page,
            self.echo_delay_samples,
            self.echo_ram_initialized,
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

    pub fn debug_voice_gains(&self) -> &[Vec<i16>; MODERN_AUDIO_VOICES] {
        &self.debug_voice_gains.0
    }

    pub fn debug_mix_samples(&self) -> &[[i32; 4]] {
        &self.debug_mix_samples.0
    }

    pub fn debug_staged_output_components(&self) -> [i16; 6] {
        [
            self.dsp_output_left,
            self.dsp_output_right,
            self.dsp_output_raw_main_left,
            self.dsp_output_raw_main_right,
            self.dsp_output_filtered_left,
            self.dsp_output_filtered_right,
        ]
    }

    pub fn debug_voice_positions(&self) -> &[Vec<u64>; MODERN_AUDIO_VOICES] {
        &self.debug_voice_positions.0
    }

    pub fn debug_voice_pitch_words(&self) -> &[Vec<u16>; MODERN_AUDIO_VOICES] {
        &self.debug_voice_pitch_words.0
    }

    pub fn debug_dsp_global_counter(&self) -> u16 {
        self.dsp_global_counter
    }

    pub fn debug_dsp_rendered_samples(&self) -> u64 {
        self.dsp_rendered_samples
    }

    pub fn debug_checkpoint_sample_offset(&self) -> u16 {
        self.checkpoint_sample_offset
    }

    pub fn debug_voice_sample_data(&self, voice: usize) -> Option<&[i16]> {
        self.voices
            .get(voice)
            .map(|voice| voice.sample_data.as_slice())
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
        self.echo_delay_register_samples = self.echo_delay_samples;
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
                voice.pitch_register_only = false;
            }
            VoiceParameterKind::PitchHigh => {
                voice.exact_pitch_word =
                    (voice.exact_pitch_word & 0x00ff) | (u16::from(value & 0x3f) << 8);
                voice.dsp_pitch_configured = true;
                voice.pitch_register_only = false;
            }
            VoiceParameterKind::Source => {
                voice.instrument_timbre = value;
                if !voice.noise_enabled {
                    voice.timbre = voice.instrument_timbre % 3;
                }
            }
            VoiceParameterKind::Adsr1 => {
                voice.dsp_adsr1 = value;
                voice.dsp_envelope_configured = true;
            }
            VoiceParameterKind::Adsr2 => {
                voice.dsp_adsr2 = value;
                voice.dsp_envelope_configured = true;
            }
            VoiceParameterKind::Gain => {
                voice.dsp_gain = value;
                voice.dsp_envelope_configured = true;
            }
        }
    }

    /// Apply a register write consumed by a voice phase in the partial DSP
    /// cycle before this host frame's first output sample.
    ///
    /// Audio from the preceding host frame has already been presented, but a
    /// pitch write in that partial cycle has also advanced the hardware BRR
    /// cursor. Reconcile that persistent cursor by the pitch delta while
    /// keeping final-output staging at the renderer boundary.
    fn handle_backdated_voice_parameter(
        &mut self,
        voice_index: u8,
        parameter: VoiceParameterKind,
        value: u8,
    ) {
        let voice_slot = usize::from(voice_index);
        let old_pitch = self
            .voices
            .get(voice_slot)
            .map(|voice| voice.exact_pitch_word)
            .unwrap_or_default();
        let old_volume_mix = self.voices.get(voice_slot).and_then(|voice| {
            let volume = match parameter {
                VoiceParameterKind::VolumeLeft => voice.volume_left,
                VoiceParameterKind::VolumeRight => voice.volume_right,
                _ => return None,
            };
            Some((
                staged_voice_channel_mix(voice, volume, self.music_volume),
                voice.echo_send,
            ))
        });
        self.handle_voice_parameter(voice_index, parameter, value);
        if let Some((old_mix, echo_send)) = old_volume_mix {
            let voice = &self.voices[voice_slot];
            let volume = match parameter {
                VoiceParameterKind::VolumeLeft => voice.volume_left,
                VoiceParameterKind::VolumeRight => voice.volume_right,
                _ => unreachable!(),
            };
            let new_mix = staged_voice_channel_mix(voice, volume, self.music_volume);
            self.reconcile_backdated_volume_mix(parameter, new_mix - old_mix, echo_send);
            return;
        }
        if !matches!(
            parameter,
            VoiceParameterKind::PitchLow | VoiceParameterKind::PitchHigh
        ) {
            return;
        }
        let Some(voice) = self.voices.get_mut(usize::from(voice_index)) else {
            return;
        };
        if !voice.active || !voice.dsp_key_on_timed || voice.checkpoint_brr_decoder.is_some() {
            return;
        }
        let new_pitch = voice.exact_pitch_word;
        if new_pitch >= old_pitch {
            voice.dsp_sample_position = voice
                .dsp_sample_position
                .saturating_add(u64::from(new_pitch - old_pitch));
        } else {
            voice.dsp_sample_position = voice
                .dsp_sample_position
                .saturating_sub(u64::from(old_pitch - new_pitch));
        }
        voice.dsp_pitch_counter = voice.dsp_sample_position as u16;
        voice.brr_block_start = (voice.dsp_sample_position >> 12) as usize & !15;
    }

    fn reconcile_backdated_volume_mix(
        &mut self,
        parameter: VoiceParameterKind,
        delta: i32,
        echo_send: bool,
    ) {
        let (raw_main, echo_input, filtered, echo_ring) = match parameter {
            VoiceParameterKind::VolumeLeft => (
                &mut self.dsp_output_raw_main_left,
                &mut self.dsp_output_echo_input_left,
                self.dsp_output_filtered_left,
                &mut self.echo_left,
            ),
            VoiceParameterKind::VolumeRight => (
                &mut self.dsp_output_raw_main_right,
                &mut self.dsp_output_echo_input_right,
                self.dsp_output_filtered_right,
                &mut self.echo_right,
            ),
            _ => return,
        };
        *raw_main = (i32::from(*raw_main) + delta).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        if echo_send {
            *echo_input =
                (i32::from(*echo_input) + delta).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            if self.dsp_flags & 0x20 == 0 && !echo_ring.is_empty() {
                let previous = (self.echo_ring_index + echo_ring.len() - 1) % echo_ring.len();
                echo_ring[previous] = (i32::from(*echo_input)
                    + (i32::from(filtered) * i32::from(self.echo_feedback) >> 7))
                    .clamp(i16::MIN as i32, i16::MAX as i32)
                    as i16
                    & !1;
            }
        }
        self.rebuild_staged_output();
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
                self.echo_delay_register_samples = delay;
            }
            register if register & 0x0f == 0x0f => {
                self.fir_coefficients[usize::from(register >> 4)] = value as i8;
            }
            0x6d => {
                if value != self.echo_start_page {
                    self.preserve_current_echo_region();
                    self.echo_start_page = value;
                    self.echo_ram_initialized = false;
                }
            }
            _ => return false,
        }
        true
    }

    fn handle_backdated_output_parameter(&mut self, register: u8, value: u8) -> bool {
        if !self.handle_global_parameter(register, value) {
            return false;
        }
        if matches!(register & 0x7f, 0x0c | 0x1c | 0x2c | 0x3c) {
            self.rebuild_staged_output();
        }
        true
    }

    fn rebuild_staged_output(&mut self) {
        let left = (i32::from(self.dsp_output_raw_main_left) * i32::from(self.master_volume_left)
            >> 7)
            + (i32::from(self.dsp_output_filtered_left) * i32::from(self.echo_mix_left) >> 7);
        let right =
            (i32::from(self.dsp_output_raw_main_right) * i32::from(self.master_volume_right) >> 7)
                + (i32::from(self.dsp_output_filtered_right) * i32::from(self.echo_mix_right) >> 7);
        self.dsp_output_left = left.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        self.dsp_output_right = right.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
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
            if let Some((left, right)) = self.preserved_echo_value(address) {
                self.echo_left[index] = left;
                self.echo_right[index] = right;
            } else if let Some(bytes) =
                sample_source_bytes(self.sample_bank_id, sample_ram, address)
            {
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

    fn preserve_current_echo_region(&mut self) {
        if !self.echo_ram_initialized || self.echo_left.is_empty() {
            return;
        }
        let start = usize::from(self.echo_start_page) << 8;
        self.echo_preserved_regions
            .retain(|region| region.start != start);
        self.echo_preserved_regions.push(EchoRamRegion {
            start,
            left: self.echo_left.clone(),
            right: self.echo_right.clone(),
        });
        if self.echo_preserved_regions.len() > 8 {
            self.echo_preserved_regions.remove(0);
        }
    }

    fn preserved_echo_value(&self, address: usize) -> Option<(i16, i16)> {
        self.echo_preserved_regions.iter().rev().find_map(|region| {
            let index = address.checked_sub(region.start)? / 4;
            Some((*region.left.get(index)?, *region.right.get(index)?))
        })
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
                if let Some((left, right)) = self.preserved_echo_value(address) {
                    self.echo_left[index] = left;
                    self.echo_right[index] = right;
                } else if let Some(bytes) =
                    sample_source_bytes(self.sample_bank_id, sample_ram, address)
                {
                    self.echo_left[index] = i16::from_le_bytes([bytes[0], bytes[1]]);
                    self.echo_right[index] = i16::from_le_bytes([bytes[2], bytes[3]]);
                }
            }
        }
        self.echo_ring_index %= echo_delay;
    }

    fn apply_deferred_voice_event(
        &mut self,
        event: &crate::game_output::AudioEvent,
        sample_ram: Option<&[u8]>,
    ) {
        match &event.kind {
            AudioEventKind::SetNoteOrigin { voice, origin } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.note_origin = Some(*origin);
                }
            }
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
            AudioEventKind::SetEnvelopeRateCounter {
                voice,
                rate_counter,
            } => {
                if let Some(voice) = self.voices.get_mut(*voice as usize) {
                    voice.dsp_rate_counter = *rate_counter;
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
                    voice.timbre = if *enabled {
                        3
                    } else {
                        voice.instrument_timbre % 3
                    };
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
            AudioEventKind::DspKeyOn {
                voice,
                pitch,
                instrument,
                volume,
            } => self.schedule_dsp_key_on(
                *voice as usize,
                *pitch,
                *instrument,
                *volume,
                event.timer_cycles,
            ),
            AudioEventKind::DspKeyOff { voice } => {
                self.schedule_dsp_key_off(*voice as usize, event.timer_cycles);
            }
            AudioEventKind::RetriggerVoice { voice } => {
                self.retrigger_voice(*voice as usize);
            }
            AudioEventKind::VoiceKeyOn { mask } => {
                for voice in 0..MODERN_AUDIO_VOICES {
                    if mask & (1 << voice) != 0 {
                        self.schedule_staged_dsp_key_on(voice, event.timer_cycles);
                    }
                }
            }
            AudioEventKind::VoiceKeyOff { mask } => {
                for voice in 0..MODERN_AUDIO_VOICES {
                    if mask & (1 << voice) != 0 {
                        self.schedule_dsp_key_off(voice, event.timer_cycles);
                    }
                }
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
        voice.dsp_key_on_timed = false;
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
        // Keep the raw SRCN register value. A later KON can legally reuse it
        // without another SRCN write; reducing it to the three synthetic
        // timbres here made those retriggers decode an unrelated BRR sample.
        voice.instrument_timbre = instrument;
        voice.timbre = if voice.noise_enabled {
            3
        } else {
            voice.instrument_timbre % 3
        };
        voice.pitch_slide_frames = 0;
        voice.sample_position = 0;
        voice.dsp_sample_position = 0;
        voice.sample_backed = false;
        voice.brr_block_start = 0;
        voice.brr_started = false;
        voice.checkpoint_brr_decoder = None;
        voice.dsp_terminal_block_start = None;
        voice.dsp_end_pipeline = 0;
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
        if rate_counter != u16::MAX {
            state.dsp_rate_counter = rate_counter;
        }
        state.dsp_envelope_configured = true;
        state.volume_left = volume_left;
        state.volume_right = volume_right;
        state.stereo_volume_configured = true;
        state.active = true;
        state.instrument_timbre = source;
        if !state.noise_enabled {
            state.timbre = state.instrument_timbre % 3;
        }
        state.pitch_slide_frames = 0;
        state.sample_position = 0;
        state.dsp_sample_position = 0;
        state.sample_backed = false;
        state.brr_block_start = 0;
        state.brr_started = false;
        state.checkpoint_brr_decoder = None;
        state.dsp_terminal_block_start = None;
        state.dsp_end_pipeline = 0;
        if state.dsp_envelope_configured {
            state.initialize_dsp_envelope();
            state.amplitude = 0;
        } else {
            state.amplitude = state.peak_amplitude;
            state.begin_decay_or_sustain();
        }
        self.load_voice_sample(voice, source, 0, sample_ram);
    }

    fn retrigger_voice(&mut self, voice: usize) {
        let Some(state) = self.voices.get_mut(voice) else {
            return;
        };
        state.active = true;
        state.remaining_frames = 0;
        state.pitch_slide_frames = 0;
        state.sample_position = 0;
        state.dsp_sample_position = 0;
        state.brr_block_start = 0;
        state.brr_started = false;
        state.checkpoint_brr_decoder = None;
        state.dsp_terminal_block_start = None;
        state.dsp_end_pipeline = 0;
        if state.dsp_envelope_configured {
            // KON resets the envelope and BRR decoder, but the DSP rate and
            // pitch counters are hardware state and intentionally survive.
            state.initialize_dsp_envelope();
            state.amplitude = 0;
        }
    }

    fn key_on_staged_voice(&mut self, voice: usize, sample_ram: Option<&[u8]>) {
        let Some(state) = self.voices.get(voice) else {
            return;
        };
        let source = state.instrument_timbre;
        self.retrigger_voice(voice);
        self.load_voice_sample(voice, source, 0, sample_ram);
    }

    fn schedule_dsp_key_on(
        &mut self,
        voice: usize,
        pitch: u8,
        instrument: u8,
        volume: u8,
        write_phase: u8,
    ) {
        if let Some(voice) = self.voices.get_mut(voice) {
            voice.pending_dsp_key_on = Some(PendingDspKeyOn {
                pitch,
                instrument,
                volume,
                write_phase,
            });
        }
    }

    fn schedule_staged_dsp_key_on(&mut self, voice: usize, write_phase: u8) {
        let Some(state) = self.voices.get_mut(voice) else {
            return;
        };
        let instrument = state.instrument_timbre;
        let volume = state
            .volume_left
            .unsigned_abs()
            .max(state.volume_right.unsigned_abs());
        // Raw DSP parameter writes have already staged the exact pitch,
        // envelope, and stereo gains on the voice. Preserve them when the
        // hardware KON polling pipeline eventually starts the voice.
        state.exact_parameters_pending = true;
        self.schedule_dsp_key_on(voice, 0, instrument, volume, write_phase);
    }

    fn schedule_dsp_key_off(&mut self, voice: usize, write_phase: u8) {
        if voice < MODERN_AUDIO_VOICES {
            self.pending_dsp_key_off_mask |= 1 << voice;
            let missed_current_poll = self.dsp_even_cycle && write_phase > 30;
            self.pending_dsp_key_off_delays[voice] = if self.dsp_even_cycle {
                u8::from(missed_current_poll) * 2
            } else {
                1
            };
        }
    }

    fn latch_dsp_key_off_register(&mut self) {
        let mask = self.pending_dsp_key_off_mask;
        for voice in 0..MODERN_AUDIO_VOICES {
            if mask & (1 << voice) != 0 {
                if self.pending_dsp_key_off_delays[voice] == 0 {
                    self.voices[voice].begin_release();
                    self.pending_dsp_key_off_mask &= !(1 << voice);
                } else {
                    self.pending_dsp_key_off_delays[voice] -= 1;
                }
            }
        }
    }

    fn advance_dsp_key_on_pipelines(&mut self, sample_ram: Option<&[u8]>) {
        for voice_index in 0..MODERN_AUDIO_VOICES {
            let starting = {
                let voice = &mut self.voices[voice_index];
                (voice.dsp_key_on_pipeline == 5)
                    .then(|| voice.latched_dsp_key_on.take())
                    .flatten()
            };
            if let Some(key_on) = starting {
                self.trigger_voice_with_params(
                    voice_index,
                    key_on.pitch,
                    key_on.instrument,
                    key_on.volume,
                );
                self.voices[voice_index].dsp_key_on_timed = true;
                self.voices[voice_index].dsp_key_on_pipeline = 5;
                self.load_voice_sample(voice_index, key_on.instrument, key_on.pitch, sample_ram);
            }
        }
    }

    fn poll_dsp_key_on_register(&mut self) {
        if self.dsp_even_cycle {
            for voice in &mut self.voices {
                if voice
                    .pending_dsp_key_on
                    .is_some_and(|key_on| key_on.write_phase > 30)
                {
                    // A phase-31 write occurs after misc_30 sampled KON. Keep
                    // it pending for the next every-other-sample poll.
                    if let Some(key_on) = &mut voice.pending_dsp_key_on {
                        key_on.write_phase = 0;
                    }
                } else if let Some(key_on) = voice.pending_dsp_key_on.take() {
                    voice.latched_dsp_key_on = Some(key_on);
                    voice.dsp_key_on_pipeline = 5;
                }
            }
        } else {
            // No KON poll occurs in this output sample. Any pending write has
            // crossed the sample boundary before the next active phase-30 poll,
            // so its original within-sample phase can no longer miss that poll.
            for voice in &mut self.voices {
                if let Some(key_on) = &mut voice.pending_dsp_key_on {
                    key_on.write_phase = 0;
                }
            }
        }
        self.dsp_even_cycle = !self.dsp_even_cycle;
    }

    fn load_voice_sample(
        &mut self,
        voice: usize,
        instrument: u8,
        pitch: u8,
        sample_ram: Option<&[u8]>,
    ) {
        let voice_index = voice;
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
        voice.dsp_terminal_block_start = sample.terminal_block_start;
        voice.dsp_end_pipeline = 0;
        voice.sample_position = 0;
        voice.sample_step = if voice.dsp_pitch_configured {
            sample_step_for_pitch_word(voice.exact_pitch_word, 32_000)
        } else {
            sample_step_for_pitch(pitch, 32_000)
        };
        voice.sample_backed = true;
        let reused_music_voice = voice.key_on_count != 0
            && voice.note_origin == Some(crate::game_output::AudioNoteOrigin::Music);
        voice.mix_uses_previous_sample = if voice.dsp_key_on_timed {
            // S-DSP voice output reaches the host callback through the final
            // one-sample DAC staging register.
            true
        } else {
            matches!(voice_index, 0 | 2) || reused_music_voice
        };
        voice.key_on_count = voice.key_on_count.saturating_add(1);
        if voice.dsp_pitch_configured {
            if voice.dsp_key_on_timed {
                // The DSP forces interpolation phase through its five KON
                // stages. Decoded sample data is ready for the first pitched
                // sample after that pipeline completes.
                voice.brr_block_start = 0;
                voice.brr_started = true;
                voice.dsp_sample_position = 0;
                voice.dsp_pitch_counter = 0;
                voice.start_delay_samples = 0;
                voice.dsp_gain_level = 0;
                voice.dsp_hidden_gain_level = 0;
                return;
            }

            // Legacy semantic NoteOn compatibility. Raw DSP-timed music uses
            // the hardware pipeline above and does not enter this path.
            voice.dsp_sample_position =
                u64::from(voice.exact_pitch_word.saturating_add(0x1000)) * 3;
            voice.dsp_pitch_counter = voice.dsp_sample_position as u16;
            voice.brr_started = true;
            let pipelined_music_voice_zero = voice_index == 0
                && voice.note_origin == Some(crate::game_output::AudioNoteOrigin::Music);
            voice.start_delay_samples =
                usize::from(!reused_music_voice && !pipelined_music_voice_zero);
            let attack_rate_index = usize::from((voice.dsp_adsr1 & 0x0f) * 2 + 1);
            let attack_rate = DSP_ENVELOPE_RATE_VALUES[attack_rate_index];
            let completed_fast_attack_preroll = voice.note_origin
                == Some(crate::game_output::AudioNoteOrigin::Music)
                && voice.dsp_adsr1 & 0x8f == 0x8f
                && voice.dsp_adsr2 >> 5 == 7;
            if completed_fast_attack_preroll {
                voice.dsp_gain_level = 0x7ff;
                voice.dsp_hidden_gain_level = 0x7f7;
                voice.dsp_envelope_state = 2;
            } else {
                voice.dsp_gain_level = 32;
                voice.dsp_hidden_gain_level = 32;
            }
            voice.dsp_rate_counter = attack_rate.saturating_sub(2);
        }
    }

    fn mix(
        &mut self,
        audio_buffer: &mut [i16],
        sample_start: usize,
        samples_per_channel: usize,
        channels: usize,
        deferred_globals: &[(usize, u8, u8)],
        deferred_music_volumes: &[(usize, u8)],
        deferred_voice_events: &[crate::game_output::AudioEvent],
        sample_ram: Option<&[u8]>,
    ) {
        for samples in &mut self.debug_voice_samples.0 {
            samples.clear();
        }
        for gains in &mut self.debug_voice_gains.0 {
            gains.clear();
        }
        for positions in &mut self.debug_voice_positions.0 {
            positions.clear();
        }
        for pitches in &mut self.debug_voice_pitch_words.0 {
            pitches.clear();
        }
        self.debug_mix_samples.0.clear();
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
        let mut deferred_music_volume_index = 0;
        let mut deferred_voice_index = 0;
        for sample_index in sample_start..samples_per_channel {
            while deferred_globals
                .get(deferred_index)
                .is_some_and(|event| event.0 <= sample_index)
            {
                let (_, register, value) = deferred_globals[deferred_index];
                self.handle_global_parameter(register, value);
                deferred_index += 1;
            }
            while deferred_music_volumes
                .get(deferred_music_volume_index)
                .is_some_and(|event| event.0 <= sample_index)
            {
                self.music_volume = deferred_music_volumes[deferred_music_volume_index].1;
                deferred_music_volume_index += 1;
            }
            if self.echo_delay_register_samples == 0 {
                self.echo_delay_register_samples = self.echo_delay_samples.max(1);
            }
            if self.echo_ring_index == 0
                && self.echo_delay_samples != self.echo_delay_register_samples
            {
                self.echo_delay_samples = self.echo_delay_register_samples;
                self.echo_remaining_samples = self.echo_delay_samples.max(1);
            }
            self.resize_echo_ring_from_ram(sample_ram);
            while deferred_voice_events
                .get(deferred_voice_index)
                .is_some_and(|event| event.sample_offset.max(0) as usize <= sample_index)
            {
                self.apply_deferred_voice_event(
                    &deferred_voice_events[deferred_voice_index],
                    sample_ram,
                );
                deferred_voice_index += 1;
            }
            self.latch_dsp_key_off_register();
            self.advance_dsp_key_on_pipelines(sample_ram);
            let mut mixed_left = 0i32;
            let mut mixed_right = 0i32;
            let mut echo_input_left = 0i32;
            let mut echo_input_right = 0i32;
            let mut semantic_dac_delta_left = 0i32;
            let mut semantic_dac_delta_right = 0i32;
            let mut previous_voice_sample = 0i32;
            let dsp_global_counter = self.dsp_global_counter;
            for voice_index in 0..self.voices.len() {
                let voice = &mut self.voices[voice_index];
                let was_active = voice.active;
                let previous_output_sample = i32::from(voice.last_output_sample);
                let mut debug_position = voice.dsp_sample_position;
                let mut debug_pitch = voice.render_pitch_word;
                let sample = if was_active {
                    let unmodulated_step = voice.sample_step;
                    voice.render_pitch_word = voice.exact_pitch_word;
                    if voice_index != 0
                        && self.pitch_modulation_mask & (1 << voice_index) != 0
                        && voice.exact_pitch_word != 0
                    {
                        let factor = (previous_voice_sample >> 4) + 0x400;
                        let modulated = (i32::from(voice.exact_pitch_word) * factor >> 10)
                            .clamp(0, 0x3fff) as u16;
                        voice.sample_step = sample_step_for_pitch_word(modulated, 32_000);
                        voice.render_pitch_word = modulated;
                    }
                    debug_position = voice.dsp_sample_position;
                    debug_pitch = voice.render_pitch_word;
                    let envelope_counter = if voice.dsp_key_on_timed {
                        dsp_counter_for_current_sample(dsp_global_counter)
                    } else {
                        envelope_counter_for_voice(
                            dsp_global_counter,
                            voice.mix_uses_previous_sample,
                        )
                    };
                    let sample = if voice.noise_enabled {
                        voice
                            .next_noise_sample_at_counter(self.noise_sample, Some(envelope_counter))
                    } else {
                        voice.next_sample_with_ram_at_counter(sample_ram, Some(envelope_counter))
                    };
                    voice.sample_step = unmodulated_step;
                    sample
                } else {
                    voice.advance_inactive_pitch_counter();
                    0
                };
                // Voices, echo input, and echo feedback share one internal DSP
                // sample clock. Presentation latency belongs at the final DAC
                // boundary, not on each voice independently.
                let mix_sample = sample;
                voice.last_output_sample = sample as i16;
                self.debug_voice_samples.0[voice_index].push(sample as i16);
                self.debug_voice_gains.0[voice_index].push(voice.dsp_gain_level as i16);
                self.debug_voice_positions.0[voice_index].push(debug_position);
                self.debug_voice_pitch_words.0[voice_index].push(debug_pitch);
                previous_voice_sample = sample;
                if !was_active && mix_sample == 0 {
                    continue;
                }
                let (mut voice_left, mut voice_right) = if voice.stereo_volume_configured {
                    (
                        mix_sample * i32::from(voice.volume_left) >> 7,
                        mix_sample * i32::from(voice.volume_right) >> 7,
                    )
                } else {
                    let pan = i32::from(voice.pan);
                    let left_gain = 127 - pan.max(0);
                    let right_gain = 127 + pan.min(0);
                    (mix_sample * left_gain / 127, mix_sample * right_gain / 127)
                };
                if voice.note_origin == Some(crate::game_output::AudioNoteOrigin::Music) {
                    voice_left = voice_left * i32::from(self.music_volume) / 96;
                    voice_right = voice_right * i32::from(self.music_volume) / 96;
                }
                if !voice.dsp_key_on_timed {
                    let (mut previous_left, mut previous_right) = if voice.stereo_volume_configured
                    {
                        (
                            previous_output_sample * i32::from(voice.volume_left) >> 7,
                            previous_output_sample * i32::from(voice.volume_right) >> 7,
                        )
                    } else {
                        let pan = i32::from(voice.pan);
                        let left_gain = 127 - pan.max(0);
                        let right_gain = 127 + pan.min(0);
                        (
                            previous_output_sample * left_gain / 127,
                            previous_output_sample * right_gain / 127,
                        )
                    };
                    if voice.note_origin == Some(crate::game_output::AudioNoteOrigin::Music) {
                        previous_left = previous_left * i32::from(self.music_volume) / 96;
                        previous_right = previous_right * i32::from(self.music_volume) / 96;
                    }
                    semantic_dac_delta_left += voice_left - previous_left;
                    semantic_dac_delta_right += voice_right - previous_right;
                }
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
            self.dsp_global_counter = if self.dsp_global_counter == 0 {
                30_719
            } else {
                self.dsp_global_counter - 1
            };
            let raw_main_left = mixed_left;
            let raw_main_right = mixed_right;
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
            filtered_left = filtered_left.clamp(i16::MIN as i32, i16::MAX as i32) & !1;
            filtered_right = filtered_right.clamp(i16::MIN as i32, i16::MAX as i32) & !1;
            let dry_left = mixed_left;
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
            self.debug_mix_samples
                .0
                .push([filtered_left, mixed_left, dry_left, raw_main_left]);
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
            self.dsp_rendered_samples = self.dsp_rendered_samples.saturating_add(1);
            let current_left = mixed_left.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            let current_right = mixed_right.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            let mixed_left = (i32::from(self.dsp_output_left)
                + (semantic_dac_delta_left * i32::from(self.master_volume_left) >> 7))
                .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            let mixed_right = (i32::from(self.dsp_output_right)
                + (semantic_dac_delta_right * i32::from(self.master_volume_right) >> 7))
                .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            self.dsp_output_left = current_left;
            self.dsp_output_right = current_right;
            self.dsp_output_raw_main_left = raw_main_left as i16;
            self.dsp_output_raw_main_right = raw_main_right as i16;
            self.dsp_output_filtered_left = filtered_left as i16;
            self.dsp_output_filtered_right = filtered_right as i16;
            self.dsp_output_echo_input_left = echo_input_left as i16;
            self.dsp_output_echo_input_right = echo_input_right as i16;
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
            self.poll_dsp_key_on_register();
        }

        // A hardware consumer can fall in the partial DSP cycle immediately
        // after this host buffer's final output sample. The audio for this
        // frame is complete, but those register latches are persistent state
        // for sample zero of the next callback and must not be discarded.
        while let Some(&(_, register, value)) = deferred_globals.get(deferred_index) {
            self.handle_global_parameter(register, value);
            deferred_index += 1;
        }
        while let Some(&(_, value)) = deferred_music_volumes.get(deferred_music_volume_index) {
            self.music_volume = value;
            deferred_music_volume_index += 1;
        }
        while let Some(event) = deferred_voice_events.get(deferred_voice_index) {
            self.apply_deferred_voice_event(event, sample_ram);
            deferred_voice_index += 1;
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
    #[serde(default)]
    note_origin: Option<crate::game_output::AudioNoteOrigin>,
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
    dsp_sample_position: u64,
    #[serde(default)]
    render_pitch_word: u16,
    #[serde(default)]
    brr_block_start: usize,
    #[serde(default)]
    brr_started: bool,
    #[serde(default)]
    checkpoint_brr_decoder: Option<CheckpointBrrDecoder>,
    /// Decoded-sample index corresponding to the non-looping BRR END block.
    #[serde(default)]
    dsp_terminal_block_start: Option<usize>,
    /// Samples remaining before a terminal header silences the voice pipeline.
    #[serde(default)]
    dsp_end_pipeline: u8,
    #[serde(default)]
    exact_pitch_word: u16,
    #[serde(default)]
    dsp_pitch_configured: bool,
    #[serde(default)]
    pitch_register_only: bool,
    #[serde(default)]
    start_delay_samples: usize,
    #[serde(default)]
    key_on_count: u32,
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
    dsp_hidden_gain_level: i32,
    #[serde(default)]
    dsp_envelope_state: u8,
    #[serde(default)]
    dsp_rate_counter: u16,
    #[serde(default)]
    last_output_sample: i16,
    #[serde(default)]
    mix_uses_previous_sample: bool,
    /// This voice was started by a raw KON event and therefore uses the
    /// free-running DSP pitch-counter/BRR pipeline.
    #[serde(default)]
    dsp_key_on_timed: bool,
    /// A raw KON write waiting for the S-DSP's every-other-sample poll.
    #[serde(default)]
    pending_dsp_key_on: Option<PendingDspKeyOn>,
    /// Parameters captured when the KON register is polled.
    #[serde(default)]
    latched_dsp_key_on: Option<PendingDspKeyOn>,
    /// Remaining samples in the internal voice-start pipeline after KON poll.
    #[serde(default)]
    dsp_key_on_pipeline: u8,
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
        self.next_sample_with_ram_at_counter(sample_ram, None)
    }

    fn advance_dsp_envelope_at_optional_counter(&mut self, counter: Option<u16>) {
        if let Some(counter) = counter {
            self.advance_dsp_envelope_at_counter(counter);
        } else {
            self.advance_dsp_envelope_for_output_sample();
        }
    }

    fn render_dsp_key_on_pipeline_sample(&mut self, envelope_counter: Option<u16>) -> bool {
        if !self.dsp_key_on_timed || self.dsp_key_on_pipeline == 0 {
            return false;
        }

        self.dsp_key_on_pipeline -= 1;
        self.dsp_sample_position = if self.dsp_key_on_pipeline & 3 != 0 {
            0x4000
        } else {
            0
        };
        self.dsp_pitch_counter = self.dsp_sample_position as u16;
        self.dsp_gain_level = 0;
        self.dsp_hidden_gain_level = 0;
        if self.dsp_key_on_pipeline == 0 {
            // S-DSP advances the envelope after emitting the final silent KON
            // sample, making that gain visible to the following sample.
            self.advance_dsp_envelope_at_optional_counter(envelope_counter);
        }
        true
    }

    fn next_sample_with_ram_at_counter(
        &mut self,
        sample_ram: Option<&[u8]>,
        envelope_counter: Option<u16>,
    ) -> i32 {
        if self.start_delay_samples != 0 {
            self.start_delay_samples -= 1;
            return 0;
        }
        if self.render_dsp_key_on_pipeline_sample(envelope_counter) {
            return 0;
        }
        if !self.dsp_key_on_timed {
            self.advance_dsp_envelope_at_optional_counter(envelope_counter);
        }
        if self.sample_backed && !self.sample_data.is_empty() {
            let (mut index, fraction) = if self.dsp_pitch_configured {
                let pitch = self.render_pitch_word;
                if self.checkpoint_brr_decoder.is_none() {
                    // Raw KON voices follow the S-DSP order exactly: render
                    // from the current BRR cursor, then add pitch. Semantic
                    // voices retain their historical pre-increment contract.
                    if !self.dsp_key_on_timed {
                        self.dsp_sample_position =
                            self.dsp_sample_position.saturating_add(u64::from(pitch));
                    }
                    self.dsp_pitch_counter = self.dsp_sample_position as u16;
                    let index = (self.dsp_sample_position >> 12) as usize;
                    self.brr_block_start = index & !15;
                    (index, ((self.dsp_sample_position >> 4) & 0xff) as u8)
                } else {
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
                    self.dsp_sample_position =
                        ((self.brr_block_start as u64) << 12) | u64::from(self.dsp_pitch_counter);
                    (
                        self.brr_block_start + usize::from(self.dsp_pitch_counter >> 12),
                        ((self.dsp_pitch_counter >> 4) & 0xff) as u8,
                    )
                }
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
                        if self.checkpoint_brr_decoder.is_none() {
                            self.dsp_sample_position =
                                ((index as u64) << 12) | (self.dsp_sample_position & 0x0fff);
                            self.dsp_pitch_counter = self.dsp_sample_position as u16;
                        }
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
            let flat_dsp_pipeline = self.dsp_key_on_timed && self.checkpoint_brr_decoder.is_none();
            let sample = if flat_dsp_pipeline {
                snes::apu::dsp_gaussian_interpolate(
                    self.sample_at(index as isize),
                    self.sample_at(index as isize + 1),
                    self.sample_at(index as isize + 2),
                    self.sample_at(index as isize + 3),
                    fraction,
                )
            } else if self.dsp_key_on_timed {
                snes::apu::dsp_gaussian_interpolate(
                    self.sample_at(index as isize - 2),
                    self.sample_at(index as isize - 1),
                    self.sample_at(index as isize),
                    self.sample_at(index as isize + 1),
                    fraction,
                )
            } else {
                snes::apu::dsp_gaussian_interpolate(
                    self.sample_at(index as isize - 3),
                    self.sample_at(index as isize - 2),
                    self.sample_at(index as isize - 1),
                    self.sample_at(index as isize),
                    fraction,
                )
            };
            let gain = if self.dsp_envelope_configured {
                i32::from(self.dsp_gain_level)
            } else {
                self.amplitude
            };
            let sample = (i32::from(sample) * gain >> 11) & !1;
            if flat_dsp_pipeline {
                self.dsp_sample_position = self
                    .dsp_sample_position
                    .saturating_add(u64::from(self.render_pitch_word));
                self.dsp_pitch_counter = self.dsp_sample_position as u16;
                self.brr_block_start = (self.dsp_sample_position >> 12) as usize & !15;
                self.advance_flat_brr_end_pipeline();
            }
            if self.dsp_key_on_timed {
                // The S-DSP applies the current envelope to this sample, then
                // runs the envelope generator for the following sample.
                self.advance_dsp_envelope_at_optional_counter(envelope_counter);
            }
            return sample;
        }
        self.advance_pitch_counter_without_decode();
        self.phase = self.phase.wrapping_add(self.phase_step);
        let amplitude = if self.dsp_envelope_configured {
            i32::from(self.dsp_gain_level)
        } else {
            self.amplitude
        };
        let sample = match self.timbre {
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
        };
        if self.dsp_key_on_timed {
            self.advance_dsp_envelope_at_optional_counter(envelope_counter);
        }
        sample
    }

    fn next_noise_sample(&mut self, noise_sample: i16) -> i32 {
        self.next_noise_sample_at_counter(noise_sample, None)
    }

    fn next_noise_sample_at_counter(
        &mut self,
        noise_sample: i16,
        envelope_counter: Option<u16>,
    ) -> i32 {
        if self.start_delay_samples != 0 {
            self.start_delay_samples -= 1;
            return 0;
        }
        if self.render_dsp_key_on_pipeline_sample(envelope_counter) {
            return 0;
        }
        self.advance_pitch_counter_without_decode();
        if !self.dsp_key_on_timed {
            self.advance_dsp_envelope_at_optional_counter(envelope_counter);
        }
        let gain = if self.dsp_envelope_configured {
            i32::from(self.dsp_gain_level)
        } else {
            self.amplitude
        };
        let sample = i32::from(noise_sample) * gain >> 11;
        if self.dsp_key_on_timed {
            self.advance_dsp_envelope_at_optional_counter(envelope_counter);
        }
        sample
    }

    fn advance_inactive_pitch_counter(&mut self) {
        if !self.pitch_register_only {
            self.render_pitch_word = self.exact_pitch_word;
        }
        self.advance_pitch_counter_without_decode();
    }

    fn advance_flat_brr_end_pipeline(&mut self) {
        if self.dsp_end_pipeline != 0 {
            self.dsp_end_pipeline -= 1;
            if self.dsp_end_pipeline == 0 {
                self.dsp_envelope_state = 4;
                self.dsp_gain_level = 0;
                self.dsp_hidden_gain_level = 0;
                self.active = false;
            }
            return;
        }
        let Some(terminal_block_start) = self.dsp_terminal_block_start else {
            return;
        };
        // The S-DSP decoder works twelve decoded samples ahead of the Gaussian
        // cursor. Crossing this boundary advances to the terminal BRR block;
        // its header reaches the envelope stage two output samples later.
        let decoder_boundary = terminal_block_start.saturating_sub(12);
        if (self.dsp_sample_position >> 12) as usize >= decoder_boundary {
            self.dsp_end_pipeline = 2;
        }
    }

    fn advance_pitch_counter_without_decode(&mut self) {
        if self.dsp_pitch_configured {
            if self.checkpoint_brr_decoder.is_none() {
                self.dsp_sample_position = self
                    .dsp_sample_position
                    .saturating_add(u64::from(self.render_pitch_word));
                self.dsp_pitch_counter = self.dsp_sample_position as u16;
            } else {
                self.dsp_pitch_counter =
                    self.dsp_pitch_counter.wrapping_add(self.render_pitch_word);
            }
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
        self.dsp_hidden_gain_level = 0;
        self.dsp_envelope_state = if self.dsp_adsr1 & 0x80 == 0 { 3 } else { 0 };
    }

    fn advance_dsp_envelope_for_output_sample(&mut self) {
        if !self.dsp_envelope_configured {
            return;
        }
        self.advance_dsp_envelope_tick();
    }

    fn advance_dsp_envelope_at_counter(&mut self, counter: u16) {
        if !self.dsp_envelope_configured {
            return;
        }
        if self.dsp_envelope_state == 4 {
            self.dsp_gain_level = self.dsp_gain_level.saturating_sub(8);
            self.dsp_hidden_gain_level = i32::from(self.dsp_gain_level);
            if self.dsp_gain_level == 0 {
                self.active = false;
            }
            return;
        }

        let use_gain = self.dsp_adsr1 & 0x80 == 0;
        let mut envelope = i32::from(self.dsp_gain_level);
        let rate_index;
        if !use_gain {
            if self.dsp_envelope_state >= 1 {
                envelope -= 1;
                envelope -= envelope >> 8;
                rate_index = if self.dsp_envelope_state == 1 {
                    usize::from(((self.dsp_adsr1 >> 3) & 0x0e) + 0x10)
                } else {
                    usize::from(self.dsp_adsr2 & 0x1f)
                };
            } else {
                rate_index = usize::from((self.dsp_adsr1 & 0x0f) * 2 + 1);
                envelope += if rate_index < 31 { 0x20 } else { 0x400 };
            }
        } else {
            let mode = self.dsp_gain >> 5;
            if mode < 4 {
                envelope = i32::from(self.dsp_gain) * 0x10;
                rate_index = 31;
            } else {
                rate_index = usize::from(self.dsp_gain & 0x1f);
                match mode {
                    4 => envelope -= 0x20,
                    5 => {
                        envelope -= 1;
                        envelope -= envelope >> 8;
                    }
                    _ => {
                        envelope += 0x20;
                        if mode > 6 && self.dsp_hidden_gain_level >= 0x600 {
                            envelope += 0x8 - 0x20;
                        }
                    }
                }
            }
        }

        if self.dsp_envelope_state == 1 && (envelope >> 8) == i32::from(self.dsp_adsr2 >> 5) {
            self.dsp_envelope_state = 2;
        }
        self.dsp_hidden_gain_level = envelope;
        if !(0..=0x7ff).contains(&envelope) {
            envelope = if envelope < 0 { 0 } else { 0x7ff };
            if self.dsp_envelope_state == 0 {
                self.dsp_envelope_state = 1;
            }
        }
        let rate = DSP_ENVELOPE_RATE_VALUES[rate_index];
        let offset = DSP_ENVELOPE_COUNTER_OFFSETS[rate_index];
        if rate != 0 && (u32::from(counter) + u32::from(offset)) % u32::from(rate) == 0 {
            self.dsp_gain_level = envelope as u16;
        }
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

const DSP_ENVELOPE_COUNTER_OFFSETS: [u16; 32] = [
    1, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 536, 0, 1040,
    536, 0, 1040, 536, 0, 1040, 536, 0, 1040, 0, 0,
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
    terminal_block_start: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CheckpointBrrDecoder {
    address: u16,
    previous_flags: u8,
    old: i16,
    older: i16,
    loop_address: u16,
    #[serde(default)]
    source: u8,
}

impl CheckpointBrrDecoder {
    fn decode_next(&mut self, ram: &[u8]) -> Option<[i16; 16]> {
        if self.previous_flags & 1 != 0 {
            if self.previous_flags == 1 {
                return None;
            }
            // The SNES DSP refreshes the source directory entry at every BRR
            // loop boundary. Song-bank uploads can replace that entry while a
            // voice is running, so a loop address latched only at KON is stale.
            const DIRECTORY: usize = 0x3c00;
            let entry = DIRECTORY + usize::from(self.source) * 4;
            let loop_bytes = ram.get(entry + 2..entry + 4)?;
            self.loop_address = u16::from_le_bytes([loop_bytes[0], loop_bytes[1]]);
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
                value & !0x7ff
            };
            let old = i32::from(self.old);
            let older = i32::from(self.older);
            match filter {
                1 => value += (old >> 1) + ((-old) >> 5),
                2 => {
                    let older = older >> 1;
                    value += old - older + (older >> 4) + ((old * -3) >> 6);
                }
                3 => {
                    let older = older >> 1;
                    value += old - older + ((old * -13) >> 7) + ((older * 3) >> 4);
                }
                _ => {}
            }
            value = value.clamp(i16::MIN as i32, i16::MAX as i32);
            *decoded = value.wrapping_mul(2) as i16;
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
            terminal_block_start: None,
        },
        CheckpointBrrDecoder {
            address: address as u16,
            previous_flags,
            old,
            older,
            loop_address: loop_address as u16,
            source,
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
                    terminal_block_start: None,
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
            let terminal_block_start = (!loops).then(|| pcm.len().saturating_sub(16));
            return Some(DecodedBrrSample {
                pcm,
                block_addresses,
                loop_start: 0,
                loops,
                terminal_block_start,
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
            sample & !0x7ff
        };
        match filter {
            1 => sample += (*old >> 1) + ((-*old) >> 5),
            2 => {
                let older_half = *older >> 1;
                sample += *old - older_half + (older_half >> 4) + ((*old * -3) >> 6);
            }
            3 => {
                let older_half = *older >> 1;
                sample += *old - older_half + ((*old * -13) >> 7) + ((older_half * 3) >> 4);
            }
            _ => {}
        }
        sample = sample.clamp(i16::MIN as i32, i16::MAX as i32);
        let decoded = sample.wrapping_mul(2) as i16;
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
                    terminal_block_start: None,
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
            let terminal_block_start = (!loops).then(|| pcm.len().saturating_sub(16));
            return Some(DecodedBrrSample {
                pcm,
                block_addresses,
                loop_start: 0,
                loops,
                terminal_block_start,
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

fn staged_voice_channel_mix(voice: &ModernVoice, volume: i8, music_volume: u8) -> i32 {
    let mut mixed = i32::from(voice.last_output_sample) * i32::from(volume) >> 7;
    if voice.note_origin == Some(crate::game_output::AudioNoteOrigin::Music) {
        mixed = mixed * i32::from(music_volume) / 96;
    }
    mixed
}

fn is_deferred_voice_event(kind: &AudioEventKind) -> bool {
    matches!(
        kind,
        AudioEventKind::SetNoteOrigin { .. }
            | AudioEventKind::SetPitchWord { .. }
            | AudioEventKind::SetPitchRegisterWord { .. }
            | AudioEventKind::SetEnvelopeRateCounter { .. }
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
            | AudioEventKind::DspKeyOn { .. }
            | AudioEventKind::DspKeyOff { .. }
            | AudioEventKind::KeyOnVoice { .. }
            | AudioEventKind::NoteOff { .. }
            | AudioEventKind::VoiceKeyOn { .. }
            | AudioEventKind::VoiceKeyOff { .. }
    )
}

/// Output sample at which a raw DSP register write can first affect the
/// corresponding hardware consumer.
///
/// ESA and EDL are sampled by the echo unit at phase 29, after the current
/// sample's echo read/output. A write through $F3 therefore cannot affect the
/// current output sample; a write after phase 29 also misses that latch.
fn dsp_global_application_sample_offset(
    event: &crate::game_output::AudioEvent,
    register: u8,
) -> i32 {
    let output_stage_offset = |read_phase: u8| {
        event.sample_offset.saturating_sub(1) + i32::from(event.timer_cycles > read_phase)
    };
    match register & 0x7f {
        // Left and right final-output multipliers run at phases 26 and 27.
        0x0c | 0x2c => output_stage_offset(26),
        0x1c | 0x3c => output_stage_offset(27),
        // Echo feedback is consumed alongside the left output at phase 26.
        0x0d => output_stage_offset(26),
        // These routing registers feed the internal voice/echo pipeline rather
        // than the staged DAC output.
        0x2d => event.sample_offset + i32::from(event.timer_cycles > 27),
        0x3d | 0x4d | 0x5d => event.sample_offset + i32::from(event.timer_cycles > 28),
        // FIR coefficients are consumed in four groups over phases 22..25.
        register if register & 0x0f == 0x0f => {
            let read_phase = match register >> 4 {
                0 => 22,
                1 | 2 => 23,
                3..=5 => 24,
                _ => 25,
            };
            output_stage_offset(read_phase)
        }
        // Echo address and length are latched at phase 29. The renderer's
        // internal DSP sample precedes the staged DAC sample by one slot.
        0x6d | 0x7d => output_stage_offset(29),
        _ => event.sample_offset,
    }
}

/// Event-clock phase at which V2 reads a voice's low pitch byte.
///
/// Lanes that have not crossed the parity frontier retain the historical
/// wrapped event-clock mapping. Oracle receipts bracket voice zero's mapped
/// V2 phase at raw S-DSP phase 21 for voice zero, phase 0 for voice one,
/// phase 6 for voice three, and phase 9 for voice four after their upload-epoch
/// conversion. The other lanes remain event-clock coordinates until raw-phase
/// receipts establish their epochs.
fn dsp_voice_pitch_low_read_phase(voice: u8) -> u8 {
    match voice {
        0 => 21,
        1 => 0,
        3 => 6,
        4 => 9,
        _ => voice.wrapping_mul(3).wrapping_add(21) % 24,
    }
}

/// The translated SPC driver clock keeps running across song-bank uploads,
/// while the modern renderer starts each uploaded bank at a new DSP event
/// epoch. Snes9x receipts show the first uploaded epoch four phases earlier
/// than the bootstrap epoch. Keep this conversion scoped to the proven voice-0,
/// voice-1, voice-3, and voice-4 pitch lanes until equivalent receipts establish the
/// others. Crossing phase zero also moves the write into the prior DSP sample.
fn dsp_voice_pitch_event_clock(
    event: &crate::game_output::AudioEvent,
    voice: u8,
    sample_bank_generation: u32,
) -> (u8, i32) {
    if matches!(voice, 0 | 1 | 3 | 4) {
        let phase_bias = (sample_bank_generation as u8).wrapping_mul(4) & 31;
        let shifted_phase = i16::from(event.timer_cycles) - i16::from(phase_bias);
        (
            shifted_phase.rem_euclid(32) as u8,
            i32::from(shifted_phase.div_euclid(32)),
        )
    } else {
        (event.timer_cycles, 0)
    }
}

fn deferred_voice_sample_offset_with_bank_generation(
    event: &crate::game_output::AudioEvent,
    retrigger_shift: i32,
    retriggered_note_off: bool,
    frame_start_counter: u16,
    sample_bank_generation: u32,
) -> i32 {
    if let AudioEventKind::VoiceParameter {
        voice, parameter, ..
    } = event.kind
    {
        if voice == 0 && parameter == VoiceParameterKind::VolumeRight && event.timer_cycles == 0 {
            // V5 for voice zero is the first operation after the DSP output
            // boundary. A write on that exact phase is visible in the output
            // slot already being assembled by the wrapped pipeline.
            return event.sample_offset.saturating_sub(1);
        }
        let read_phase = match parameter {
            // V4/V5 multiply the voice output by its left/right registers.
            VoiceParameterKind::VolumeLeft => Some(voice.wrapping_mul(3).wrapping_add(31) & 31),
            VoiceParameterKind::VolumeRight if voice == 0 => {
                // Voice zero's V5 phase wraps past the legacy output boundary.
                // A write after phase zero still reaches that wrapped multiply
                // before the current numbered output sample is committed.
                Some(31)
            }
            VoiceParameterKind::VolumeRight => Some(voice.wrapping_mul(3) & 31),
            // V2/V3a read each voice's low/high pitch bytes in a three-phase
            // pipeline, with voice zero wrapping to phases 21/22.
            VoiceParameterKind::PitchLow => Some(dsp_voice_pitch_low_read_phase(voice)),
            VoiceParameterKind::PitchHigh => {
                Some(dsp_voice_pitch_low_read_phase(voice).wrapping_add(1))
            }
            _ => None,
        };
        if let Some(read_phase) = read_phase {
            let pipeline_sample = if voice == 0 {
                event.sample_offset
            } else {
                event.sample_offset.saturating_sub(1)
            };
            let (event_phase, epoch_sample_adjustment) =
                dsp_voice_pitch_event_clock(event, voice, sample_bank_generation);
            return pipeline_sample + epoch_sample_adjustment + i32::from(event_phase > read_phase);
        }
        event.sample_offset
    } else if let AudioEventKind::NoteOff { voice } = event.kind {
        let voice_zero = i32::from(voice == 0);
        let first_key_phase_lead = i32::from(matches!(voice, 0 | 1));
        let sample_offset = event.sample_offset.saturating_sub(if retriggered_note_off {
            6 + voice_zero * 2
        } else {
            8 + first_key_phase_lead
        });
        let koff_latch_parity = i32::from(frame_start_counter & 1);
        if retriggered_note_off && voice == 0 && sample_offset & 1 != koff_latch_parity {
            sample_offset.saturating_sub(1)
        } else {
            sample_offset
        }
    } else if retrigger_shift != 0 {
        event.sample_offset.saturating_sub(retrigger_shift)
    } else {
        event.sample_offset
    }
}

#[cfg(test)]
fn deferred_voice_sample_offset(
    event: &crate::game_output::AudioEvent,
    retrigger_shift: i32,
    retriggered_note_off: bool,
    frame_start_counter: u16,
) -> i32 {
    deferred_voice_sample_offset_with_bank_generation(
        event,
        retrigger_shift,
        retriggered_note_off,
        frame_start_counter,
        0,
    )
}

fn envelope_counter_for_voice(global_counter: u16, use_previous_mix_sample: bool) -> u16 {
    if use_previous_mix_sample {
        global_counter
    } else if global_counter == 30_719 {
        0
    } else {
        global_counter + 1
    }
}

fn dsp_counter_for_current_sample(counter: u16) -> u16 {
    if counter == 0 {
        30_719
    } else {
        counter - 1
    }
}

fn deferred_retrigger_shift(
    frame: &AudioEventFrame,
    event: &crate::game_output::AudioEvent,
    key_on_count: u32,
    active: bool,
    current_origin: Option<crate::game_output::AudioNoteOrigin>,
) -> i32 {
    let Some(voice) = deferred_event_voice(&event.kind) else {
        return 0;
    };
    let has_raw_dsp_key_on = frame.events.iter().any(|candidate| {
        candidate.sample_offset == event.sample_offset
            && matches!(
                candidate.kind,
                AudioEventKind::DspKeyOn { voice: candidate, .. }
                    if usize::from(candidate) == voice
            )
    });
    if has_raw_dsp_key_on {
        return 0;
    }
    let has_key_on = frame.events.iter().any(|candidate| {
        candidate.sample_offset == event.sample_offset
            && matches!(
                candidate.kind,
                AudioEventKind::NoteOn { voice: candidate, .. }
                    | AudioEventKind::KeyOnVoice { voice: candidate, .. }
                    if usize::from(candidate) == voice
            )
    });
    if !has_key_on {
        return 0;
    }
    let has_music_origin = frame.events.iter().any(|candidate| {
        candidate.sample_offset == event.sample_offset
            && matches!(
                candidate.kind,
                AudioEventKind::SetNoteOrigin {
                    voice: candidate,
                    origin: crate::game_output::AudioNoteOrigin::Music,
                } if usize::from(candidate) == voice
            )
    });
    if key_on_count == 0 {
        0
    } else if current_origin == Some(crate::game_output::AudioNoteOrigin::Music) || has_music_origin
    {
        -1
    } else if active {
        10
    } else {
        7
    }
}

fn deferred_event_voice(kind: &AudioEventKind) -> Option<usize> {
    match kind {
        AudioEventKind::SetNoteOrigin { voice, .. }
        | AudioEventKind::SetPitchWord { voice, .. }
        | AudioEventKind::SetPitchRegisterWord { voice, .. }
        | AudioEventKind::SetEnvelopeRateCounter { voice, .. }
        | AudioEventKind::VoiceParameter { voice, .. }
        | AudioEventKind::SetStereoVolume { voice, .. }
        | AudioEventKind::SetDspEnvelope { voice, .. }
        | AudioEventKind::SetEchoSend { voice, .. }
        | AudioEventKind::SetNoise { voice, .. }
        | AudioEventKind::SetEnvelope { voice, .. }
        | AudioEventKind::PitchSlide { voice, .. }
        | AudioEventKind::SetPan { voice, .. }
        | AudioEventKind::SetDuration { voice, .. }
        | AudioEventKind::NoteOn { voice, .. }
        | AudioEventKind::DspKeyOn { voice, .. }
        | AudioEventKind::DspKeyOff { voice }
        | AudioEventKind::KeyOnVoice { voice, .. }
        | AudioEventKind::NoteOff { voice } => Some(usize::from(*voice)),
        _ => None,
    }
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

    fn timed_event(sample_offset: i32, timer_cycles: u8, kind: AudioEventKind) -> AudioEvent {
        AudioEvent {
            sample_offset,
            timer_cycles,
            kind,
            parity_dsp: None,
        }
    }

    #[test]
    fn echo_enable_written_after_phase_28_applies_on_the_next_sample() {
        let event = timed_event(
            100,
            30,
            AudioEventKind::GlobalParameter {
                register: 0x4d,
                value: 0xff,
            },
        );

        assert_eq!(dsp_global_application_sample_offset(&event, 0x4d), 101);
    }

    #[test]
    fn voice_four_pitch_write_before_its_latch_updates_the_prior_pipeline_sample() {
        let event = timed_event(
            100,
            0,
            AudioEventKind::VoiceParameter {
                voice: 4,
                parameter: VoiceParameterKind::PitchLow,
                value: 0x34,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 99);
    }

    #[test]
    fn pitch_register_read_phases_preserve_the_event_clock_mapping() {
        assert_eq!(
            std::array::from_fn::<_, 8, _>(|voice| { dsp_voice_pitch_low_read_phase(voice as u8) }),
            [21, 0, 3, 6, 9, 12, 15, 18]
        );
    }

    #[test]
    fn voice_three_timer_phase_eight_uses_the_uploaded_bank_dsp_epoch() {
        let event = timed_event(
            507,
            8,
            AudioEventKind::VoiceParameter {
                voice: 3,
                parameter: VoiceParameterKind::PitchLow,
                value: 143,
            },
        );

        assert_eq!(
            deferred_voice_sample_offset_with_bank_generation(&event, 0, false, 0, 1),
            506
        );
    }

    #[test]
    fn voice_three_timer_phase_ten_maps_to_the_uploaded_bank_latch() {
        let event = timed_event(
            249,
            10,
            AudioEventKind::VoiceParameter {
                voice: 3,
                parameter: VoiceParameterKind::PitchLow,
                value: 145,
            },
        );

        assert_eq!(
            deferred_voice_sample_offset_with_bank_generation(&event, 0, false, 0, 1),
            248
        );
    }

    #[test]
    fn voice_three_pitch_write_after_raw_phase_six_waits_for_the_next_pipeline_sample() {
        let event = timed_event(
            390,
            23,
            AudioEventKind::VoiceParameter {
                voice: 3,
                parameter: VoiceParameterKind::PitchLow,
                value: 145,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 390);
    }

    #[test]
    fn voice_three_timer_phase_eleven_maps_after_the_uploaded_bank_latch() {
        let event = timed_event(
            244,
            11,
            AudioEventKind::VoiceParameter {
                voice: 3,
                parameter: VoiceParameterKind::PitchLow,
                value: 143,
            },
        );

        assert_eq!(
            deferred_voice_sample_offset_with_bank_generation(&event, 0, false, 0, 1),
            244
        );
    }

    #[test]
    fn voice_three_raw_phase_nine_waits_for_the_next_pipeline_sample() {
        let event = timed_event(
            378,
            9,
            AudioEventKind::VoiceParameter {
                voice: 3,
                parameter: VoiceParameterKind::PitchLow,
                value: 124,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 378);
    }

    #[test]
    fn voice_four_pitch_write_on_raw_phase_nine_updates_the_prior_pipeline_sample() {
        let event = timed_event(
            260,
            9,
            AudioEventKind::VoiceParameter {
                voice: 4,
                parameter: VoiceParameterKind::PitchLow,
                value: 213,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 259);
    }

    #[test]
    fn voice_four_pitch_write_after_raw_phase_nine_waits_for_the_next_pipeline_sample() {
        let event = timed_event(
            260,
            11,
            AudioEventKind::VoiceParameter {
                voice: 4,
                parameter: VoiceParameterKind::PitchLow,
                value: 192,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 260);
    }

    #[test]
    fn voice_four_timer_phase_eleven_uses_the_uploaded_bank_dsp_epoch() {
        let event = timed_event(
            260,
            11,
            AudioEventKind::VoiceParameter {
                voice: 4,
                parameter: VoiceParameterKind::PitchLow,
                value: 213,
            },
        );

        assert_eq!(
            deferred_voice_sample_offset_with_bank_generation(&event, 0, false, 0, 1),
            259
        );
    }

    #[test]
    fn voice_four_timer_phase_fourteen_maps_after_the_uploaded_bank_latch() {
        let event = timed_event(
            288,
            14,
            AudioEventKind::VoiceParameter {
                voice: 4,
                parameter: VoiceParameterKind::PitchLow,
                value: 209,
            },
        );

        assert_eq!(
            deferred_voice_sample_offset_with_bank_generation(&event, 0, false, 0, 1),
            288
        );
    }

    #[test]
    fn voice_four_pitch_write_on_phase_twelve_waits_for_the_next_pipeline_sample() {
        let event = timed_event(
            260,
            12,
            AudioEventKind::VoiceParameter {
                voice: 4,
                parameter: VoiceParameterKind::PitchLow,
                value: 192,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 260);
    }

    #[test]
    fn voice_four_pitch_write_after_phase_twelve_waits_for_the_next_pipeline_sample() {
        let event = timed_event(
            288,
            14,
            AudioEventKind::VoiceParameter {
                voice: 4,
                parameter: VoiceParameterKind::PitchLow,
                value: 209,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 288);
    }

    #[test]
    fn voice_zero_pitch_write_uses_the_wrapped_pipeline_sample() {
        let event = timed_event(
            100,
            8,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::PitchLow,
                value: 0x34,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 100);
    }

    #[test]
    fn voice_zero_pitch_write_after_v2_but_before_wrap_waits_one_sample() {
        let event = timed_event(
            401,
            25,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::PitchLow,
                value: 122,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 402);
    }

    #[test]
    fn voice_zero_timer_phase_twenty_three_maps_before_the_uploaded_bank_latch() {
        let event = timed_event(
            450,
            23,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::PitchLow,
                value: 229,
            },
        );

        assert_eq!(
            deferred_voice_sample_offset_with_bank_generation(&event, 0, false, 0, 1),
            450
        );
    }

    #[test]
    fn voice_zero_raw_phase_twenty_two_waits_for_the_next_pipeline_sample() {
        let event = timed_event(
            214,
            22,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::PitchLow,
                value: 80,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 215);
    }

    #[test]
    fn voice_one_timer_phase_one_wraps_to_the_prior_uploaded_bank_sample() {
        let event = timed_event(
            518,
            1,
            AudioEventKind::VoiceParameter {
                voice: 1,
                parameter: VoiceParameterKind::PitchLow,
                value: 226,
            },
        );

        assert_eq!(
            deferred_voice_sample_offset_with_bank_generation(&event, 0, false, 0, 1),
            517
        );
    }

    #[test]
    fn voice_one_raw_phase_one_waits_for_the_next_pipeline_sample() {
        let event = timed_event(
            91,
            1,
            AudioEventKind::VoiceParameter {
                voice: 1,
                parameter: VoiceParameterKind::PitchLow,
                value: 94,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 91);
    }

    #[test]
    fn voice_one_pitch_write_after_phase_three_waits_for_the_next_pipeline_sample() {
        let event = timed_event(
            197,
            9,
            AudioEventKind::VoiceParameter {
                voice: 1,
                parameter: VoiceParameterKind::PitchLow,
                value: 222,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 197);
    }

    #[test]
    fn sample_zero_pitch_write_after_v2_is_deferred_by_the_frame_dispatcher() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[0];
        voice.active = true;
        voice.sample_backed = true;
        voice.sample_data = (0..32).map(|sample| sample * 100).collect();
        voice.brr_started = true;
        voice.dsp_key_on_timed = true;
        voice.dsp_pitch_configured = true;
        voice.exact_pitch_word = 2017;
        voice.render_pitch_word = 2017;
        voice.amplitude = 2048;
        let mut frame = empty_frame_with_writes(&[]);
        frame.events.push(timed_event(
            0,
            23,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::PitchLow,
                value: 223,
            },
        ));
        let mut output = [0i16; 4];

        engine.render_frame(&frame, &mut output, 2, 2);

        assert_eq!(&engine.debug_voice_pitch_words()[0][..2], [2017, 2015]);
    }

    #[test]
    fn register_latch_after_the_final_host_sample_is_preserved() {
        let mut engine = ModernAudioEngine::default();
        engine.voices[0].exact_pitch_word = 0x0601;
        let mut frame = empty_frame_with_writes(&[]);
        frame.events.push(timed_event(
            1,
            31,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::PitchLow,
                value: 3,
            },
        ));
        let mut output = [0i16; 4];

        engine.render_frame(&frame, &mut output, 2, 2);

        assert_eq!(engine.voices[0].exact_pitch_word, 0x0603);
    }

    #[test]
    fn voice_seven_left_volume_write_before_phase_20_updates_the_prior_pipeline_sample() {
        let event = timed_event(
            163,
            1,
            AudioEventKind::VoiceParameter {
                voice: 7,
                parameter: VoiceParameterKind::VolumeLeft,
                value: 75,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 162);
    }

    #[test]
    fn voice_zero_right_volume_write_uses_the_wrapped_pipeline_sample() {
        let event = timed_event(
            493,
            9,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::VolumeRight,
                value: 17,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 493);
    }

    #[test]
    fn voice_zero_right_volume_write_on_phase_zero_updates_the_prior_output_slot() {
        let event = timed_event(
            269,
            0,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::VolumeRight,
                value: 4,
            },
        );

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 0), 268);
    }

    #[test]
    fn backdated_pitch_write_reconciles_the_cross_frame_brr_cursor() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[7];
        voice.active = true;
        voice.dsp_key_on_timed = true;
        voice.dsp_pitch_configured = true;
        voice.exact_pitch_word = 6819;
        voice.render_pitch_word = 6819;
        voice.dsp_sample_position = 10_000;
        voice.dsp_pitch_counter = 10_000;

        engine.handle_backdated_voice_parameter(7, VoiceParameterKind::PitchLow, 160);

        assert_eq!(engine.voices[7].exact_pitch_word, 6816);
        assert_eq!(engine.voices[7].dsp_sample_position, 9_997);
        assert_eq!(engine.voices[7].dsp_pitch_counter, 9_997);
    }

    #[test]
    fn backdated_volume_write_reconciles_staged_output_and_echo_input() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[4];
        voice.last_output_sample = -1_888;
        voice.volume_right = 17;
        voice.stereo_volume_configured = true;
        voice.note_origin = Some(crate::game_output::AudioNoteOrigin::Music);
        voice.echo_send = true;
        engine.music_volume = 96;
        engine.master_volume_right = 96;
        engine.echo_mix_right = 40;
        engine.dsp_output_raw_main_right = 2_348;
        engine.dsp_output_filtered_right = 2_210;
        engine.dsp_output_right = 2_451;
        engine.dsp_output_echo_input_right = 100;
        engine.echo_feedback = 0;
        engine.dsp_flags = 0;
        engine.echo_right = vec![200, 0];
        engine.echo_ring_index = 1;

        engine.handle_backdated_voice_parameter(4, VoiceParameterKind::VolumeRight, 16);

        assert_eq!(engine.voices[4].volume_right, 16);
        assert_eq!(engine.dsp_output_raw_main_right, 2_363);
        assert_eq!(engine.dsp_output_right, 2_462);
        assert_eq!(engine.dsp_output_echo_input_right, 115);
        assert_eq!(engine.echo_right[0], 114);
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
        assert_eq!(&sample.pcm[..4], &[0, 6, 0, 6]);
        assert!(!sample.loops);
    }

    #[test]
    fn source_three_key_on_matches_snes9x_1_63_dsp_samples() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[7];
        voice.active = true;
        voice.exact_pitch_word = 15_626;
        voice.render_pitch_word = 15_626;
        voice.dsp_pitch_configured = true;
        voice.dsp_envelope_configured = true;
        voice.dsp_adsr1 = 0xfe;
        voice.dsp_adsr2 = 0xf8;
        voice.dsp_gain = 0xb8;
        engine.load_voice_sample(7, 3, 0, None);

        let mut counter = 16_374u16;
        let actual = (0..193)
            .map(|_| {
                let sample = engine.voices[7].next_sample_with_ram_at_counter(None, Some(counter));
                counter = if counter == 0 { 30_719 } else { counter - 1 };
                sample
            })
            .collect::<Vec<_>>();
        assert_eq!(
            &actual[..11],
            &[0, -60, -128, -32, 0, 22, 58, 296, 674, 778, 0]
        );
        assert_eq!(
            &actual[173..193],
            &[
                434, 2168, 7058, 10664, 4874, 0, 0, -992, -3712, -3252, -8216, -11818, -14628,
                -4032, -4034, 0, 0, -324, 3530, 8402,
            ]
        );
    }

    #[test]
    fn semantic_source_three_reaches_dac_on_the_snes9x_sample() {
        let mut engine = ModernAudioEngine::default();
        let mut frame = empty_frame_with_writes(&[]);
        let offset = 360;
        frame.events.extend([
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetNoteOrigin {
                    voice: 7,
                    origin: crate::game_output::AudioNoteOrigin::Sfx,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetNoise {
                    voice: 7,
                    enabled: false,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetPan { voice: 7, pan: 0 },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetEchoSend {
                    voice: 7,
                    enabled: false,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetEnvelope {
                    attack: 14,
                    voice: 7,
                    decay: 7,
                    sustain: 7,
                    release: 24,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetPitchWord {
                    voice: 7,
                    pitch_word: 15_626,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetStereoVolume {
                    voice: 7,
                    left: 50,
                    right: 50,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetDspEnvelope {
                    voice: 7,
                    adsr1: 0xfe,
                    adsr2: 0xf8,
                    gain: 0xb8,
                },
                parity_dsp: None,
            },
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::NoteOn {
                    voice: 7,
                    pitch: 127,
                    instrument: 3,
                    volume: 50,
                },
                parity_dsp: None,
            },
        ]);
        frame.sequenced = true;
        let mut audio = vec![0i16; 533 * 2];

        engine.render_frame(&frame, &mut audio, 533, 2);

        assert_eq!(&audio[718..722], &[0, 0, 0, 0]);
        assert_eq!(&audio[722..724], &[-18, -18]);
    }

    #[test]
    fn instrument_fifteen_brr_prefix_matches_snes9x_decoder() {
        let sample = decode_brr_bank_sample(0, 15).expect("instrument 15 in title bank");

        assert_eq!(
            &sample.pcm[..24],
            &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -128, -500, -1090, -1610,
                -1922, -2412,
            ]
        );
        assert_eq!(
            snes::apu::dsp_gaussian_interpolate(0, 0, -128, -500, 203),
            -128
        );
    }

    #[test]
    fn native_dsp_block_tracks_requested_533_sample_libretro_frame() {
        let mut engine = ModernAudioEngine::default();
        let mut audio = vec![0i16; 533 * 2];

        engine.render_frame(&empty_frame_with_writes(&[]), &mut audio, 533, 2);

        assert_eq!(engine.debug_voice_samples()[0].len(), 533);

        let mut startup_engine = ModernAudioEngine::default();
        let mut startup_audio = vec![0i16; 457 * 2];
        startup_engine.render_frame(&empty_frame_with_writes(&[]), &mut startup_audio, 457, 2);
        assert_eq!(startup_engine.debug_voice_samples()[0].len(), 457);
    }

    #[test]
    fn raw_dsp_key_on_waits_for_poll_and_runs_hardware_phase_sequence() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[0];
        voice.note_origin = Some(crate::game_output::AudioNoteOrigin::Music);
        voice.exact_pitch_word = 3822;
        voice.render_pitch_word = 3822;
        voice.dsp_pitch_counter = 0x4567;
        voice.dsp_pitch_configured = true;
        voice.dsp_envelope_configured = true;
        voice.dsp_adsr1 = 0xff;
        voice.dsp_adsr2 = 0xef;
        voice.dsp_gain = 0xb8;
        voice.exact_parameters_pending = true;

        engine.schedule_dsp_key_on(0, 30, 15, 23, 0);
        engine.advance_dsp_key_on_pipelines(None);
        engine.poll_dsp_key_on_register();
        assert!(engine.voices[0].pending_dsp_key_on.is_some());
        assert_eq!(engine.voices[0].dsp_key_on_pipeline, 0);

        engine.advance_dsp_key_on_pipelines(None);
        engine.poll_dsp_key_on_register();
        assert!(engine.voices[0].pending_dsp_key_on.is_none());
        assert_eq!(engine.voices[0].dsp_key_on_pipeline, 5);

        let mut phases = Vec::new();
        for _ in 0..5 {
            engine.advance_dsp_key_on_pipelines(None);
            assert!(engine.voices[0].active);
            assert!(engine.voices[0].render_dsp_key_on_pipeline_sample(Some(0)));
            phases.push(engine.voices[0].dsp_sample_position);
        }
        assert!(engine.voices[0].dsp_key_on_timed);
        assert_eq!(phases, [0, 0x4000, 0x4000, 0x4000, 0]);
        assert_eq!(engine.voices[0].dsp_pitch_counter, 0);
        assert!(engine.voices[0].brr_started);
        assert_eq!(engine.voices[0].dsp_gain_level, 1024);

        let mut outputs = Vec::new();
        for _ in 0..18 {
            outputs.push(engine.voices[0].next_sample_with_ram_at_counter(None, Some(0)));
        }
        let voice = &engine.voices[0];
        let index = (voice.dsp_sample_position >> 12) as isize;
        assert_eq!(voice.dsp_sample_position, 18 * 3822);
        assert_eq!(((voice.dsp_sample_position >> 4) & 0xff) as u8, 203);
        assert_eq!(
            [
                voice.sample_at(index),
                voice.sample_at(index + 1),
                voice.sample_at(index + 2),
                voice.sample_at(index + 3),
            ],
            [0, 0, -128, -500]
        );
        assert_eq!(outputs[17], -16);
    }

    #[test]
    fn raw_dsp_key_off_waits_for_the_every_other_sample_latch() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[0];
        voice.active = true;
        voice.dsp_key_on_timed = true;
        voice.dsp_envelope_configured = true;
        voice.dsp_envelope_state = 2;
        voice.dsp_adsr1 = 0xff;
        voice.dsp_adsr2 = 0;
        voice.dsp_gain_level = 100;
        voice.dsp_hidden_gain_level = 100;

        engine.schedule_dsp_key_off(0, 0);
        engine.latch_dsp_key_off_register();
        engine.voices[0].next_sample_with_ram_at_counter(None, Some(0));
        assert_eq!(engine.voices[0].dsp_gain_level, 100);

        engine.poll_dsp_key_on_register();
        engine.latch_dsp_key_off_register();
        engine.voices[0].next_sample_with_ram_at_counter(None, Some(0));
        assert_eq!(engine.voices[0].dsp_gain_level, 92);

        engine.poll_dsp_key_on_register();
        engine.latch_dsp_key_off_register();
        engine.voices[0].next_sample_with_ram_at_counter(None, Some(0));
        assert_eq!(engine.voices[0].dsp_gain_level, 84);
    }

    #[test]
    fn phase_31_koff_waits_for_the_next_every_other_register_poll() {
        let mut engine = ModernAudioEngine::default();
        engine.dsp_even_cycle = true;
        let voice = &mut engine.voices[0];
        voice.active = true;
        voice.dsp_envelope_configured = true;
        voice.dsp_envelope_state = 2;
        voice.dsp_gain_level = 100;

        engine.schedule_dsp_key_off(0, 31);
        engine.latch_dsp_key_off_register();
        assert_eq!(engine.voices[0].dsp_envelope_state, 2);
        engine.latch_dsp_key_off_register();
        assert_eq!(engine.voices[0].dsp_envelope_state, 2);
        engine.latch_dsp_key_off_register();
        assert_eq!(engine.voices[0].dsp_envelope_state, 4);
    }

    #[test]
    fn phase_31_kon_waits_for_the_next_every_other_register_poll() {
        let mut engine = ModernAudioEngine::default();
        engine.dsp_even_cycle = true;
        engine.schedule_dsp_key_on(0, 30, 15, 23, 31);

        engine.poll_dsp_key_on_register();
        assert!(engine.voices[0].pending_dsp_key_on.is_some());
        assert!(engine.voices[0].latched_dsp_key_on.is_none());
        engine.poll_dsp_key_on_register();
        assert!(engine.voices[0].pending_dsp_key_on.is_some());
        engine.poll_dsp_key_on_register();
        assert!(engine.voices[0].pending_dsp_key_on.is_none());
        assert!(engine.voices[0].latched_dsp_key_on.is_some());
        assert_eq!(engine.voices[0].dsp_key_on_pipeline, 5);
    }

    #[test]
    fn phase_31_kon_on_non_poll_sample_is_visible_at_next_poll() {
        let mut engine = ModernAudioEngine::default();
        engine.dsp_even_cycle = false;
        engine.schedule_dsp_key_on(0, 30, 15, 23, 31);

        engine.poll_dsp_key_on_register();
        assert!(engine.voices[0].pending_dsp_key_on.is_some());
        engine.poll_dsp_key_on_register();

        assert!(engine.voices[0].pending_dsp_key_on.is_none());
        assert!(engine.voices[0].latched_dsp_key_on.is_some());
        assert_eq!(engine.voices[0].dsp_key_on_pipeline, 5);
    }

    #[test]
    fn semantic_note_off_applies_at_the_snes_dsp_koff_boundary() {
        let event = crate::game_output::AudioEvent {
            sample_offset: 361,
            timer_cycles: 0,
            kind: AudioEventKind::NoteOff { voice: 7 },
            parity_dsp: None,
        };

        assert_eq!(deferred_voice_sample_offset(&event, 0, false, 1), 353);
        let voice_zero_off = crate::game_output::AudioEvent {
            kind: AudioEventKind::NoteOff { voice: 0 },
            ..event.clone()
        };
        assert_eq!(
            deferred_voice_sample_offset(&voice_zero_off, 0, false, 0),
            352
        );
        let voice_one_off = crate::game_output::AudioEvent {
            kind: AudioEventKind::NoteOff { voice: 1 },
            ..event.clone()
        };
        assert_eq!(
            deferred_voice_sample_offset(&voice_one_off, 0, false, 0),
            352
        );
        let repeated_off = crate::game_output::AudioEvent {
            sample_offset: 445,
            ..event.clone()
        };
        assert_eq!(deferred_voice_sample_offset(&repeated_off, 0, true, 1), 439);
        let repeated_voice_zero_off = crate::game_output::AudioEvent {
            kind: AudioEventKind::NoteOff { voice: 0 },
            ..repeated_off.clone()
        };
        assert_eq!(
            deferred_voice_sample_offset(&repeated_voice_zero_off, 0, true, 1),
            437
        );
        let even_phase_repeated_voice_zero_off = crate::game_output::AudioEvent {
            sample_offset: 512,
            ..repeated_voice_zero_off.clone()
        };
        assert_eq!(
            deferred_voice_sample_offset(&even_phase_repeated_voice_zero_off, 0, true, 1),
            503
        );
        let even_frame_start_repeated_voice_zero_off = crate::game_output::AudioEvent {
            sample_offset: 449,
            ..repeated_voice_zero_off.clone()
        };
        assert_eq!(
            deferred_voice_sample_offset(&even_frame_start_repeated_voice_zero_off, 0, true, 0,),
            440
        );

        let retrigger = crate::game_output::AudioEvent {
            sample_offset: 444,
            timer_cycles: 0,
            kind: AudioEventKind::NoteOn {
                voice: 7,
                pitch: 127,
                instrument: 3,
                volume: 50,
            },
            parity_dsp: None,
        };
        assert_eq!(deferred_voice_sample_offset(&retrigger, 10, false, 0), 434);
        let inactive_repeat = crate::game_output::AudioEvent {
            sample_offset: 58,
            ..retrigger.clone()
        };
        assert_eq!(
            deferred_voice_sample_offset(&inactive_repeat, 7, false, 0),
            51
        );

        let mut music_frame =
            AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        music_frame.events = vec![
            crate::game_output::AudioEvent {
                sample_offset: retrigger.sample_offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetNoteOrigin {
                    voice: 7,
                    origin: crate::game_output::AudioNoteOrigin::Music,
                },
                parity_dsp: None,
            },
            retrigger.clone(),
        ];
        assert_eq!(
            deferred_retrigger_shift(&music_frame, &retrigger, 2, false, None),
            -1
        );
        assert_eq!(
            deferred_retrigger_shift(&music_frame, &music_frame.events[0], 2, false, None),
            -1
        );

        let mut sfx_frame = music_frame;
        sfx_frame.events.remove(0);
        assert_eq!(
            deferred_retrigger_shift(
                &sfx_frame,
                &retrigger,
                2,
                false,
                Some(crate::game_output::AudioNoteOrigin::Music),
            ),
            -1
        );
        assert_eq!(
            deferred_retrigger_shift(&sfx_frame, &retrigger, 2, false, None),
            7
        );
        assert_eq!(
            deferred_retrigger_shift(&sfx_frame, &retrigger, 2, true, None),
            10
        );
    }

    #[test]
    fn music_voice_zero_skips_the_semantic_decoder_delay() {
        let mut engine = ModernAudioEngine::default();
        engine.voices[0].note_origin = Some(crate::game_output::AudioNoteOrigin::Music);
        engine.voices[0].dsp_pitch_configured = true;
        engine.voices[0].exact_pitch_word = 5724;
        engine.voices[0].dsp_envelope_configured = true;

        engine.load_voice_sample(0, 15, 45, None);

        assert_eq!(engine.voices[0].start_delay_samples, 0);

        engine.voices[0].key_on_count = 1;
        engine.load_voice_sample(0, 15, 45, None);
        assert_eq!(engine.voices[0].start_delay_samples, 0);
    }

    #[test]
    fn fast_attack_music_voice_starts_after_the_kon_envelope_preroll() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[2];
        voice.note_origin = Some(crate::game_output::AudioNoteOrigin::Music);
        voice.dsp_pitch_configured = true;
        voice.exact_pitch_word = 4290;
        voice.dsp_envelope_configured = true;
        voice.dsp_adsr1 = 0xff;
        voice.dsp_adsr2 = 0xef;

        engine.load_voice_sample(2, 15, 34, None);

        assert_eq!(engine.voices[2].dsp_gain_level, 0x7ff);
        assert_eq!(engine.voices[2].dsp_hidden_gain_level, 0x7f7);
        assert_eq!(engine.voices[2].dsp_envelope_state, 2);
    }

    #[test]
    fn dac_staged_voices_use_the_previous_pipeline_sample() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[1];
        voice.note_origin = Some(crate::game_output::AudioNoteOrigin::Music);
        voice.dsp_pitch_configured = true;
        voice.exact_pitch_word = 6810;
        voice.dsp_envelope_configured = true;
        voice.dsp_adsr1 = 0xff;
        voice.dsp_adsr2 = 0xef;
        engine.load_voice_sample(1, 15, 53, None);
        assert!(!engine.voices[1].mix_uses_previous_sample);
        engine.load_voice_sample(1, 15, 53, None);
        assert!(engine.voices[1].mix_uses_previous_sample);
    }

    #[test]
    fn inactive_voice_flushes_the_final_dac_stage_once() {
        let mut engine = ModernAudioEngine::default();
        engine.dsp_output_left = 1024;
        engine.dsp_output_right = -1024;
        let frame = empty_frame_with_writes(&[]);
        let mut tail = [0i16; 2];

        engine.render_frame(&frame, &mut tail, 1, 2);

        assert_eq!(tail, [1024, -1024]);
        assert_eq!(engine.voices[0].last_output_sample, 0);

        let mut after_tail = [0i16; 2];
        engine.render_frame(&frame, &mut after_tail, 1, 2);
        assert_eq!(after_tail, [0, 0]);
    }

    #[test]
    fn dac_staged_voices_observe_the_visible_dsp_counter_phase() {
        assert_eq!(envelope_counter_for_voice(26_769, true), 26_769);
        assert_eq!(envelope_counter_for_voice(26_769, false), 26_770);
        assert_eq!(envelope_counter_for_voice(0, true), 0);
        assert_eq!(envelope_counter_for_voice(30_719, false), 0);
        assert_eq!(dsp_counter_for_current_sample(2_933), 2_932);
        assert_eq!(dsp_counter_for_current_sample(0), 30_719);
    }

    #[test]
    fn looping_checkpoint_decoder_rereads_the_live_directory_entry() {
        const DIRECTORY: usize = 0x3c00;
        let source = 1u8;
        let old_loop = 0x5000usize;
        let new_loop = 0x6000usize;
        let entry = DIRECTORY + usize::from(source) * 4;
        let mut ram = vec![0u8; 0x10000];
        ram[entry + 2..entry + 4].copy_from_slice(&(new_loop as u16).to_le_bytes());
        ram[old_loop] = 0x03;
        ram[old_loop + 1..old_loop + 9].fill(0x11);
        ram[new_loop] = 0x01;
        ram[new_loop + 1..new_loop + 9].fill(0x22);
        let mut decoder = CheckpointBrrDecoder {
            previous_flags: 0x03,
            loop_address: old_loop as u16,
            source,
            ..CheckpointBrrDecoder::default()
        };

        let decoded = decoder.decode_next(&ram).expect("new loop target block");

        assert_eq!(decoder.address, (new_loop + 9) as u16);
        assert_eq!(decoder.loop_address, new_loop as u16);
        assert_ne!(decoded, [0; 16]);
        assert!(decoder.decode_next(&ram).is_none());
    }

    #[test]
    fn sample_bank_generation_change_releases_every_active_voice() {
        let mut engine = ModernAudioEngine::default();
        for voice in [2usize, 6, 7] {
            engine.voices[voice].active = true;
            engine.voices[voice].dsp_envelope_configured = true;
            engine.voices[voice].dsp_envelope_state = 2;
            engine.voices[voice].dsp_gain_level = 2000;
        }
        let frame = AudioEventFrame::from_route_and_dsp_writes(
            AudioRouteState {
                sample_bank_id: 1,
                sample_bank_generation: 1,
                ..AudioRouteState::default()
            },
            &[],
        );

        engine.render_frame(&frame, &mut [], 0, 2);

        assert_eq!(engine.sample_bank_id, 1);
        assert_eq!(engine.sample_bank_generation, 1);
        assert_eq!(engine.echo_mix_left, 0);
        assert_eq!(engine.echo_mix_right, 0);
        for voice in [2usize, 6, 7] {
            assert_eq!(engine.voices[voice].dsp_envelope_state, 4);
            assert!(engine.voices[voice].active);
        }
    }

    #[test]
    fn completed_sample_bank_upload_is_an_idempotent_publication() {
        let mut engine = ModernAudioEngine::default();
        engine.echo_left = vec![-2; 16];
        engine.echo_right = vec![-4; 16];
        engine.echo_ring_index = 7;
        engine.echo_ram_initialized = true;
        engine.fir_history_left = [-1; 8];
        engine.fir_history_right = [-2; 8];
        engine.echo_mix_left = 13;
        engine.echo_mix_right = 14;

        engine.complete_sample_bank_upload(1, 7);

        assert_eq!(engine.sample_bank_id, 1);
        assert_eq!(engine.sample_bank_generation, 7);
        assert_eq!(engine.echo_left, vec![-2; 16]);
        assert_eq!(engine.echo_right, vec![-4; 16]);
        assert_eq!(engine.echo_ring_index, 7);
        assert_eq!(engine.fir_history_left, [-1; 8]);
        assert_eq!(engine.fir_history_right, [-2; 8]);

        let already_published = AudioEventFrame::from_route_and_dsp_writes(
            AudioRouteState {
                sample_bank_id: 1,
                sample_bank_generation: 7,
                ..AudioRouteState::default()
            },
            &[],
        );
        engine.render_frame(&already_published, &mut [], 0, 2);

        assert_eq!(engine.echo_mix_left, 13);
        assert_eq!(engine.echo_mix_right, 14);
        assert_eq!(engine.echo_ring_index, 7);
        assert!(engine.echo_ram_initialized);
    }

    #[test]
    fn checkpoint_render_preserves_the_already_mixed_native_prefix() {
        let mut dsp = vec![0u8; 3024];
        let prefix = [101i16, -202, 303, -404];
        for (index, sample) in prefix.into_iter().enumerate() {
            let offset = 884 + index * 2;
            dsp[offset..offset + 2].copy_from_slice(&sample.to_le_bytes());
        }
        dsp[3020..3022].copy_from_slice(&2u16.to_le_bytes());
        let mut engine = ModernAudioEngine::default();
        engine.seed_dsp_checkpoint_state(&vec![0; 0x10000], &dsp);
        let frame = empty_frame_with_writes(&[]);
        let mut output = vec![0i16; DSP_SAMPLES_PER_FRAME * 2];

        engine.render_frame(&frame, &mut output, DSP_SAMPLES_PER_FRAME as i32, 2);

        assert_eq!(&output[..prefix.len()], &prefix);
        assert!(engine.checkpoint_sample_prefix.is_empty());
        assert_eq!(engine.checkpoint_sample_offset, 0);
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
        assert_eq!(block[0], 936);
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
    fn raw_pitch_write_replaces_register_only_pitch_for_inactive_counter() {
        let mut engine = ModernAudioEngine::default();
        let voice = &mut engine.voices[0];
        voice.active = false;
        voice.dsp_pitch_configured = true;
        voice.exact_pitch_word = 0x0100;
        voice.render_pitch_word = 0x0200;
        voice.pitch_register_only = true;
        voice.dsp_pitch_counter = 0;

        engine.handle_voice_parameter(0, VoiceParameterKind::PitchLow, 0xaf);
        engine.handle_voice_parameter(0, VoiceParameterKind::PitchHigh, 0x0d);
        engine.voices[0].advance_inactive_pitch_counter();

        assert!(!engine.voices[0].pitch_register_only);
        assert_eq!(engine.voices[0].render_pitch_word, 0x0daf);
        assert_eq!(engine.voices[0].dsp_pitch_counter, 0x0daf);
    }

    #[test]
    fn non_looping_brr_terminal_header_reaches_the_envelope_after_two_samples() {
        let mut voice = ModernVoice {
            active: true,
            sample_backed: true,
            sample_data: vec![1; 16],
            dsp_pitch_configured: true,
            render_pitch_word: 0,
            dsp_envelope_configured: true,
            dsp_adsr1: 0xff,
            dsp_adsr2: 0xff,
            dsp_envelope_state: 2,
            dsp_gain_level: 1024,
            dsp_key_on_timed: true,
            dsp_terminal_block_start: Some(12),
            ..ModernVoice::default()
        };

        assert_eq!(voice.next_sample(), 0);
        assert!(voice.active);
        assert_eq!(voice.next_sample(), 0);
        assert!(voice.active);
        assert_eq!(voice.next_sample(), 0);
        assert!(!voice.active);
        assert_eq!(voice.dsp_gain_level, 0);
        assert_eq!(voice.dsp_envelope_state, 4);

        voice.dsp_envelope_state = 2;
        voice.begin_release();
        assert_eq!(voice.dsp_envelope_state, 4);
    }

    #[test]
    fn flat_brr_pipeline_silences_after_terminal_header_reaches_envelope_stage() {
        let mut voice = ModernVoice {
            active: true,
            sample_backed: true,
            sample_data: vec![100; 32],
            sample_loops: false,
            dsp_pitch_configured: true,
            exact_pitch_word: 0x1000,
            render_pitch_word: 0x1000,
            dsp_sample_position: 3 << 12,
            dsp_key_on_timed: true,
            dsp_envelope_configured: true,
            dsp_envelope_state: 2,
            dsp_gain_level: 2047,
            dsp_terminal_block_start: Some(16),
            ..ModernVoice::default()
        };

        let _crosses_decoder_boundary = voice.next_sample();
        assert!(voice.active);
        let _decoder_advances_to_end_block = voice.next_sample();
        assert!(voice.active);
        let _terminal_header_reaches_envelope_stage = voice.next_sample();

        assert!(!voice.active);
        assert_eq!(voice.dsp_gain_level, 0);
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
    fn music_master_fade_does_not_attenuate_sfx_owned_voice() {
        let mut base = ModernAudioEngine::default();
        base.trigger_voice(0, 0x12, 1100);
        base.music_volume = 0x30;
        let mut music = base.clone();
        music.voices[0].note_origin = Some(crate::game_output::AudioNoteOrigin::Music);
        let mut sfx = base;
        sfx.voices[0].note_origin = Some(crate::game_output::AudioNoteOrigin::Sfx);
        let frame = empty_frame_with_writes(&[]);
        let mut music_audio = [0i16; 32];
        let mut sfx_audio = [0i16; 32];

        music.render_frame(&frame, &mut music_audio, 16, 2);
        sfx.render_frame(&frame, &mut sfx_audio, 16, 2);

        let music_peak = music_audio.iter().map(|sample| sample.abs()).max().unwrap();
        let sfx_peak = sfx_audio.iter().map(|sample| sample.abs()).max().unwrap();
        assert!(music_peak > 0);
        assert!(sfx_peak > music_peak);
        assert!(sfx_peak >= music_peak.saturating_mul(2).saturating_sub(2));
    }

    #[test]
    fn key_on_and_key_off_events_control_modern_voices() {
        let mut engine = ModernAudioEngine::default();
        let mut audio = [0i16; 16];
        let key_on = empty_frame_with_writes(&[DspWriteEvent::new(0x4c, 0x03, 0, 0)]);

        let on_stats = engine.render_frame(&key_on, &mut audio, 8, 2);

        assert_eq!(on_stats.triggered_voices, 2);
        assert_eq!(on_stats.active_voices, 2);

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

        assert_eq!(engine.voices[0].instrument_timbre, 15);
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

    #[test]
    fn timed_raw_dsp_writes_stage_parameters_before_key_on() {
        let writes = [
            DspWriteEvent::new(0x00, 64, 12, 0),
            DspWriteEvent::new(0x01, 64, 12, 0),
            DspWriteEvent::new(0x02, 0x00, 12, 0),
            DspWriteEvent::new(0x03, 0x10, 12, 0),
            DspWriteEvent::new(0x04, 0x00, 12, 0),
            DspWriteEvent::new(0x05, 0x8f, 12, 0),
            DspWriteEvent::new(0x06, 0xe0, 12, 0),
            DspWriteEvent::new(0x07, 0x7f, 12, 0),
            DspWriteEvent::new(0x4c, 0x01, 12, 0),
        ];
        let frame = empty_frame_with_writes(&writes);
        let mut engine = ModernAudioEngine::default();
        let mut output = [0i16; 1068];

        engine.render_frame(&frame, &mut output, 534, 2);

        let voice = engine.voice_debug_states()[0];
        assert!(voice.active);
        assert_eq!(voice.pitch, 0x1000);
        assert_eq!(voice.adsr1, 0x8f);
        assert_eq!(voice.adsr2, 0xe0);
        assert_eq!(voice.gain_config, 0x7f);
        assert!(engine.debug_voice_samples()[0][..12]
            .iter()
            .all(|sample| *sample == 0));
    }

    #[test]
    fn raw_dsp_source_register_survives_key_on_without_a_rewrite() {
        let first_key_on = empty_frame_with_writes(&[
            DspWriteEvent::new(0x04, 15, 12, 0),
            DspWriteEvent::new(0x4c, 0x01, 12, 0),
        ]);
        let repeated_key_on = empty_frame_with_writes(&[DspWriteEvent::new(0x4c, 0x01, 12, 0)]);
        let mut engine = ModernAudioEngine::default();
        let mut output = [0i16; 1068];

        engine.render_frame(&first_key_on, &mut output, 534, 2);
        assert_eq!(engine.voices[0].instrument_timbre, 15);

        engine.render_frame(&repeated_key_on, &mut output, 534, 2);
        assert_eq!(engine.voices[0].instrument_timbre, 15);
    }

    #[test]
    fn fir_result_clears_low_bit_before_echo_volume() {
        let mut engine = ModernAudioEngine::default();
        engine.echo_left = vec![4];
        engine.echo_right = vec![4];
        engine.echo_delay_samples = 1;
        engine.echo_remaining_samples = 1;
        engine.echo_ram_initialized = true;
        engine.echo_mix_left = 127;
        engine.echo_mix_right = 127;
        engine.fir_coefficients[7] = 127;
        engine.dsp_flags = 0x20;
        let mut output = [0i16; 4];

        engine.render_frame(&empty_frame_with_writes(&[]), &mut output, 2, 2);

        assert_eq!(output, [0, 0, 1, 1]);
    }

    #[test]
    fn frame_start_echo_volume_write_updates_the_staged_output_sample() {
        let mut engine = ModernAudioEngine::default();
        engine.echo_left = vec![-2_972];
        engine.echo_right = vec![0];
        engine.echo_delay_samples = 1;
        engine.echo_delay_register_samples = 1;
        engine.echo_remaining_samples = 1;
        engine.echo_ram_initialized = true;
        engine.echo_mix_left = 30;
        engine.echo_mix_right = 0;
        engine.fir_coefficients[7] = 64;
        engine.dsp_flags = 0x20;

        let mut preroll = vec![0; 533 * 2];
        engine.render_frame(&empty_frame_with_writes(&[]), &mut preroll, 533, 2);

        let frame = empty_frame_with_writes(&[DspWriteEvent::new(0x2c, 31, 0, 2)]);
        let mut audio = vec![0; 533 * 2];
        engine.render_frame(&frame, &mut audio, 533, 2);

        assert_eq!(audio[0], -360);
        assert_eq!(audio[1], 0);
    }

    #[test]
    fn zero_edl_holds_echo_pointer_at_zero() {
        let mut engine = ModernAudioEngine::default();
        let mut output = [0i16; 16];

        engine.render_frame(&empty_frame_with_writes(&[]), &mut output, 8, 2);

        assert_eq!(engine.echo_debug_state().0, 0);
    }

    #[test]
    fn timed_edl_write_enables_the_initial_echo_ring() {
        let mut engine = ModernAudioEngine::default();
        let mut output = [0i16; 2];
        let frame = empty_frame_with_writes(&[DspWriteEvent::new(0x7d, 1, 0, 19)]);

        engine.render_frame(&frame, &mut output, 1, 2);

        assert_eq!(engine.echo_debug_config(), (0xc8, 512, true));
        assert_eq!(engine.echo_debug_state(), (22, 6, 490));
    }

    #[test]
    fn echo_start_change_preserves_overlapping_live_ram() {
        let mut engine = ModernAudioEngine::default();
        engine.echo_start_page = 0xc8;
        engine.echo_delay_samples = 512;
        engine.echo_delay_register_samples = 512;
        engine.echo_left = vec![0; 512];
        engine.echo_right = vec![0; 512];
        engine.echo_left[337] = -124;
        engine.echo_right[337] = 246;
        engine.echo_ram_initialized = true;

        engine.handle_global_parameter(0x6d, 0xc0);
        engine.handle_global_parameter(0x7d, 2);
        let mut output = [0i16; 2];
        engine.render_frame(&empty_frame_with_writes(&[]), &mut output, 1, 2);

        assert_eq!(engine.echo_left[849], -124);
        assert_eq!(engine.echo_right[849], 246);
    }

    #[test]
    fn dirty_echo_ram_rebuild_preserves_dsp_cursor_and_fir_phase() {
        let mut engine = ModernAudioEngine::default();
        engine.echo_start_page = 0xc8;
        engine.echo_delay_samples = 512;
        engine.echo_delay_register_samples = 512;
        engine.echo_left = vec![0; 512];
        engine.echo_right = vec![0; 512];
        engine.echo_ring_index = 499;
        engine.echo_remaining_samples = 13;
        engine.fir_history_index = 3;
        engine.echo_ram_initialized = true;
        engine.sample_ram_changed();
        let mut output = [0i16; 800];

        engine.render_frame(&empty_frame_with_writes(&[]), &mut output, 400, 2);

        assert_eq!(engine.echo_ring_index, 387);
        assert_eq!(engine.echo_remaining_samples, 125);
        assert_eq!(engine.fir_history_index, 3);
    }

    #[test]
    fn timed_raw_echo_writes_reach_the_modern_dsp_state() {
        let writes = [
            DspWriteEvent::new(0x2c, 0x20, 20, 0),
            DspWriteEvent::new(0x3c, 0x30, 20, 0),
            DspWriteEvent::new(0x0d, 0x40, 20, 0),
            DspWriteEvent::new(0x4d, 0x03, 20, 0),
            DspWriteEvent::new(0x0f, 0x7f, 20, 0),
            DspWriteEvent::new(0x6d, 0x90, 20, 0),
            DspWriteEvent::new(0x7d, 0x02, 20, 0),
        ];
        let frame = empty_frame_with_writes(&writes);
        let mut engine = ModernAudioEngine::default();
        let mut output = [0i16; 1068];

        engine.render_frame(&frame, &mut output, 534, 2);

        assert_eq!(engine.echo_mix_left, 0x20);
        assert_eq!(engine.echo_mix_right, 0x30);
        assert_eq!(engine.echo_feedback, 0x40);
        assert!(engine.voices[0].echo_send);
        assert!(engine.voices[1].echo_send);
        assert_eq!(engine.fir_coefficients[0], 0x7f);
        assert_eq!(engine.echo_start_page, 0x90);
        assert_eq!(engine.echo_delay_samples, 1024);
    }
}
