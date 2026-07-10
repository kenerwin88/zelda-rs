use crate::game_output::{
    AudioEventFrame, AudioEventKind, AudioSampleStats, MusicControlState, VoiceParameterKind,
};

const MODERN_AUDIO_VOICES: usize = 8;
const DEFAULT_FRAME_RATE: usize = 60;

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
    last_stats: ModernAudioFrameStats,
}

impl Default for ModernAudioEngine {
    fn default() -> Self {
        Self {
            voices: [ModernVoice::default(); MODERN_AUDIO_VOICES],
            last_music: MusicControlState::default(),
            last_ports: [0; 4],
            last_stats: ModernAudioFrameStats::default(),
        }
    }
}

impl ModernAudioEngine {
    pub fn render_frame(
        &mut self,
        frame: &AudioEventFrame,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
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
        let has_modern_intent = frame.events.iter().any(|event| {
            matches!(
                event.kind,
                AudioEventKind::PlayMusic { .. }
                    | AudioEventKind::StopMusic
                    | AudioEventKind::PlaySfx { .. }
                    | AudioEventKind::SetTempo { .. }
                    | AudioEventKind::NoteOn { .. }
                    | AudioEventKind::NoteOff { .. }
                    | AudioEventKind::SetDuration { .. }
                    | AudioEventKind::PitchSlide { .. }
                    | AudioEventKind::SetNoise { .. }
                    | AudioEventKind::SetEnvelope { .. }
            )
        });

        for event in &frame.events {
            match &event.kind {
                AudioEventKind::MusicState(music) => {
                    stats.understood_events += 1;
                    if !has_modern_intent {
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
                    if !has_modern_intent {
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
                    for voice in &mut self.voices {
                        voice.active = false;
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
                    stats.triggered_voices += 1;
                }
                AudioEventKind::NoteOff { voice } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.active = false;
                    }
                }
                AudioEventKind::SetEnvelope {
                    voice,
                    decay,
                    sustain,
                    ..
                } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.decay = (*decay).max(1);
                        voice.amplitude = voice.amplitude.max(i32::from(*sustain) * 80);
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
                    ..
                } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        voice.base_phase_step = phase_step_for_code(*target_pitch);
                        voice.phase_step = voice.base_phase_step;
                    }
                }
                AudioEventKind::SetNoise { voice, enabled } => {
                    stats.understood_events += 1;
                    if let Some(voice) = self.voices.get_mut(*voice as usize) {
                        if *enabled {
                            voice.timbre = 3;
                        }
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
                            self.voices[voice].active = false;
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
                AudioEventKind::EchoParameter { .. }
                | AudioEventKind::GlobalParameter { .. }
                | AudioEventKind::UnresolvedDspWrite { .. } => {
                    stats.ignored_events += 1;
                }
            }
        }

        if samples_per_channel != 0 && channels != 0 {
            self.mix(audio_buffer, samples_per_channel, channels);
        }
        self.decay_voices();
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
            VoiceParameterKind::VolumeLeft | VoiceParameterKind::VolumeRight => {
                voice.amplitude = i32::from(value).saturating_mul(10).min(1600);
                voice.active |= value != 0;
            }
            VoiceParameterKind::PitchLow | VoiceParameterKind::PitchHigh => {
                voice.base_phase_step = phase_step_for_code(value);
                voice.phase_step = voice.base_phase_step;
            }
            VoiceParameterKind::Source => {
                voice.timbre = value & 3;
            }
            VoiceParameterKind::Adsr1 | VoiceParameterKind::Adsr2 | VoiceParameterKind::Gain => {
                voice.decay = 1 + (value >> 5).min(6);
            }
        }
    }

    fn trigger_voice(&mut self, voice: usize, code: u8, amplitude: i32) {
        self.trigger_voice_with_params(voice, code, code >> 4, amplitude.saturating_div(12) as u8);
    }

    fn trigger_voice_with_params(&mut self, voice: usize, pitch: u8, instrument: u8, volume: u8) {
        let Some(voice) = self.voices.get_mut(voice) else {
            return;
        };
        voice.active = true;
        voice.base_phase_step = phase_step_for_code(pitch);
        voice.phase_step = voice.base_phase_step;
        voice.amplitude = i32::from(volume).saturating_mul(12).min(1800);
        voice.decay = 3 + (instrument & 3);
        voice.timbre = instrument & 3;
    }

    fn mix(&mut self, audio_buffer: &mut [i16], samples_per_channel: usize, channels: usize) {
        let frame_rate = samples_per_channel
            .saturating_mul(DEFAULT_FRAME_RATE)
            .max(1);
        for voice in &mut self.voices {
            if voice.active {
                voice.rescale_phase_step(frame_rate);
            }
        }

        for sample_index in 0..samples_per_channel {
            let mut mixed = 0i32;
            for voice in &mut self.voices {
                if !voice.active {
                    continue;
                }
                mixed += voice.next_sample();
            }
            let mixed = mixed.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            for channel in 0..channels {
                let index = sample_index * channels + channel;
                if let Some(slot) = audio_buffer.get_mut(index) {
                    *slot = mixed;
                }
            }
        }
    }

    fn decay_voices(&mut self) {
        for voice in &mut self.voices {
            if !voice.active {
                continue;
            }
            voice.amplitude = voice
                .amplitude
                .saturating_sub(i32::from(voice.decay) * 32)
                .max(0);
            if voice.amplitude == 0 {
                voice.active = false;
            }
            if voice.remaining_frames != 0 {
                voice.remaining_frames = voice.remaining_frames.saturating_sub(1);
                if voice.remaining_frames == 0 {
                    voice.active = false;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ModernVoice {
    active: bool,
    phase: u32,
    phase_step: u32,
    base_phase_step: u32,
    amplitude: i32,
    decay: u8,
    timbre: u8,
    remaining_frames: u8,
}

impl ModernVoice {
    fn rescale_phase_step(&mut self, sample_rate: usize) {
        let sample_rate = sample_rate.max(1) as u64;
        self.phase_step = ((u64::from(self.base_phase_step) * 44_100) / sample_rate) as u32;
    }

    fn next_sample(&mut self) -> i32 {
        self.phase = self.phase.wrapping_add(self.phase_step);
        match self.timbre {
            0 => {
                if self.phase & 0x8000_0000 == 0 {
                    self.amplitude
                } else {
                    -self.amplitude
                }
            }
            1 => {
                let ramp = ((self.phase >> 20) as i32 & 0xfff) - 2048;
                ramp * self.amplitude / 2048
            }
            2 => {
                let tri = if self.phase & 0x8000_0000 == 0 {
                    (self.phase >> 19) as i32 & 0xfff
                } else {
                    4095 - ((self.phase >> 19) as i32 & 0xfff)
                };
                (tri - 2048) * self.amplitude / 2048
            }
            _ => {
                let bit = ((self.phase >> 31) ^ (self.phase >> 27) ^ (self.phase >> 23)) & 1;
                if bit == 0 {
                    self.amplitude / 2
                } else {
                    -self.amplitude / 2
                }
            }
        }
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
}
