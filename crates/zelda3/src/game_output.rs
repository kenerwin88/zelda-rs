use crate::zelda_rtl::ZeldaState;

pub const AUDIO_INTERNAL_SAMPLES_PER_FRAME: usize = 534;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioSampleStats {
    pub samples_per_channel: usize,
    pub channels: usize,
    pub peak: i16,
    pub first_nonzero: Option<usize>,
    pub mean_abs: u32,
    pub checksum: u32,
}

impl AudioSampleStats {
    pub fn from_interleaved(samples: &[i16], channels: usize) -> Self {
        let mut sum = 0u64;
        let mut peak = 0i16;
        let mut first_nonzero = None;
        let mut checksum = FNV1A32_OFFSET;
        for (index, &sample) in samples.iter().enumerate() {
            let abs = sample.saturating_abs();
            if abs > peak {
                peak = abs;
            }
            if sample != 0 && first_nonzero.is_none() {
                first_nonzero = Some(index);
            }
            sum += abs as u64;
            checksum = fnv1a32_bytes(checksum, &sample.to_le_bytes());
        }
        let mean_abs = if samples.is_empty() {
            0
        } else {
            (sum / samples.len() as u64) as u32
        };
        Self {
            samples_per_channel: if channels == 0 {
                0
            } else {
                samples.len() / channels
            },
            channels,
            peak,
            first_nonzero,
            mean_abs,
            checksum,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DspWriteEvent {
    pub addr: u8,
    pub value: u8,
    pub sample_offset: i32,
    pub timer_cycles: u8,
}

impl DspWriteEvent {
    pub fn new(addr: u8, value: u8, sample_offset: i32, timer_cycles: u8) -> Self {
        Self {
            addr,
            value,
            sample_offset,
            timer_cycles,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MusicControlState {
    pub apui00: u8,
    pub music_control: u8,
    pub sound_effect_ambient: u8,
    pub sound_effect_1: u8,
    pub sound_effect_2: u8,
    pub queued_music_control: u8,
    pub last_music_control: u8,
}

impl MusicControlState {
    pub fn from_game(game: &ZeldaState) -> Self {
        Self {
            apui00: game.ram[0x0648],
            music_control: game.ram[0x012c],
            sound_effect_ambient: game.ram[0x012d],
            sound_effect_1: game.ram[0x012e],
            sound_effect_2: game.ram[0x012f],
            queued_music_control: game.ram[0x0132],
            last_music_control: game.ram[0x0133],
        }
    }

    pub fn legacy_apui_fields(&self) -> [u8; 4] {
        [
            self.apui00,
            self.music_control,
            self.sound_effect_ambient,
            self.sound_effect_1,
        ]
    }

    pub fn legacy_music_fields(&self) -> [u8; 3] {
        [
            self.sound_effect_2,
            self.queued_music_control,
            self.last_music_control,
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioQueueState {
    pub pos: u8,
    pub count: u8,
    pub total: u8,
    pub write: [u8; 4],
    pub pending: [u8; 4],
    pub input: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpcSequencerState {
    pub spc_in: [u8; 4],
    pub spc_out: [u8; 4],
    pub timer_cycles: u8,
    pub main_tempo_accum: u8,
    pub block_count: u8,
    pub key_on: u8,
    pub key_off: u8,
    pub current_bit: u8,
    pub port1_active: u8,
    pub port2_active: u8,
    pub port3_active: u8,
    pub is_chan_on: u8,
    pub vol_dirty: u8,
    pub ch7_sfx: u8,
    pub ch7_sfx_ptr: u16,
    pub ch7_pattern: u16,
    pub ch7_ticks: u8,
    pub ch7_keyoff_ticks: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioRouteState {
    pub music: MusicControlState,
    pub queue: AudioQueueState,
    pub spc: Option<SpcSequencerState>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioEventFrame {
    pub music: MusicControlState,
    pub queue: AudioQueueState,
    pub events: Vec<AudioEvent>,
    pub unresolved_dsp_writes: usize,
}

impl AudioEventFrame {
    pub fn from_route_and_dsp_writes(route: AudioRouteState, writes: &[DspWriteEvent]) -> Self {
        let mut events = Vec::with_capacity(writes.len() + 2);
        events.push(AudioEvent {
            sample_offset: 0,
            timer_cycles: route.spc.map(|spc| spc.timer_cycles).unwrap_or_default(),
            kind: AudioEventKind::MusicState(route.music),
            parity_dsp: None,
        });
        events.push(AudioEvent {
            sample_offset: 0,
            timer_cycles: route.spc.map(|spc| spc.timer_cycles).unwrap_or_default(),
            kind: AudioEventKind::ApuPorts {
                write: route.queue.write,
                pending: route.queue.pending,
                input: route.queue.input,
                spc_in: route.spc.map(|spc| spc.spc_in).unwrap_or_default(),
                spc_out: route.spc.map(|spc| spc.spc_out).unwrap_or_default(),
            },
            parity_dsp: None,
        });

        let mut unresolved_dsp_writes = 0usize;
        for &write in writes {
            let kind = classify_dsp_write(write);
            if matches!(kind, AudioEventKind::UnresolvedDspWrite { .. }) {
                unresolved_dsp_writes += 1;
            }
            events.push(AudioEvent {
                sample_offset: write.sample_offset,
                timer_cycles: write.timer_cycles,
                kind,
                parity_dsp: Some(write),
            });
        }
        Self {
            music: route.music,
            queue: route.queue,
            events,
            unresolved_dsp_writes,
        }
    }

    pub fn command_hash(&self) -> u32 {
        let mut hash = FNV1A32_OFFSET;
        hash = hash_music_state(hash, self.music);
        hash = hash_bytes(hash, &self.queue.write);
        hash = hash_bytes(hash, &self.queue.pending);
        hash = hash_bytes(hash, &self.queue.input);
        for event in &self.events {
            hash = hash_audio_event(hash, event);
        }
        hash
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioEvent {
    pub sample_offset: i32,
    pub timer_cycles: u8,
    pub kind: AudioEventKind,
    pub parity_dsp: Option<DspWriteEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioEventKind {
    MusicState(MusicControlState),
    ApuPorts {
        write: [u8; 4],
        pending: [u8; 4],
        input: [u8; 4],
        spc_in: [u8; 4],
        spc_out: [u8; 4],
    },
    PlayMusic {
        track: u8,
    },
    StopMusic,
    PlaySfx {
        bank: u8,
        id: u8,
    },
    SetTempo {
        value: u8,
    },
    NoteOn {
        voice: u8,
        pitch: u8,
        instrument: u8,
        volume: u8,
    },
    NoteOff {
        voice: u8,
    },
    SetDuration {
        voice: u8,
        frames: u8,
    },
    PitchSlide {
        voice: u8,
        target_pitch: u8,
        frames: u8,
    },
    SetNoise {
        voice: u8,
        enabled: bool,
    },
    SetEnvelope {
        voice: u8,
        attack: u8,
        decay: u8,
        sustain: u8,
        release: u8,
    },
    VoiceKeyOn {
        mask: u8,
    },
    VoiceKeyOff {
        mask: u8,
    },
    VoiceParameter {
        voice: u8,
        parameter: VoiceParameterKind,
        value: u8,
    },
    EchoParameter {
        parameter: EchoParameterKind,
        value: u8,
    },
    GlobalParameter {
        register: u8,
        value: u8,
    },
    UnresolvedDspWrite {
        register: u8,
        value: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VoiceParameterKind {
    VolumeLeft,
    VolumeRight,
    PitchLow,
    PitchHigh,
    Source,
    Adsr1,
    Adsr2,
    Gain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EchoParameterKind {
    VolumeLeft,
    VolumeRight,
    Feedback,
    EnableMask,
    Fir(u8),
    Delay,
    StartAddress,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RenderOutputFacts {
    pub mode: u8,
    pub forced_blank: bool,
    pub brightness: u8,
    pub screen_enabled: [u8; 2],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeOutputFacts {
    pub frame_counter: u8,
    pub main_module: u8,
    pub submodule: u8,
    pub subsubmodule: u8,
    pub inidisp: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GameFrameOutput {
    pub runtime: RuntimeOutputFacts,
    pub render: RenderOutputFacts,
    pub audio: AudioEventFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioBackendMode {
    DspParity,
    TraceOnly,
    Modern,
}

impl Default for AudioBackendMode {
    fn default() -> Self {
        Self::DspParity
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioTraceFrameSummary {
    pub sample_stats: AudioSampleStats,
    pub dsp_pre_hash: u32,
    pub dsp_post_hash: u32,
    pub dsp_write_count: u32,
    pub dsp_write_hash: u32,
    pub dsp_write_values_hash: u32,
    pub command_event_count: u32,
    pub command_event_hash: u32,
    pub unresolved_dsp_writes: u32,
}

impl AudioTraceFrameSummary {
    pub fn from_parts(
        samples: &[i16],
        channels: usize,
        dsp_pre_hash: u32,
        dsp_post_hash: u32,
        writes: &[DspWriteEvent],
        event_frame: &AudioEventFrame,
    ) -> Self {
        Self {
            sample_stats: AudioSampleStats::from_interleaved(samples, channels),
            dsp_pre_hash,
            dsp_post_hash,
            dsp_write_count: writes.len() as u32,
            dsp_write_hash: checksum_dsp_writes(writes),
            dsp_write_values_hash: checksum_dsp_write_values(writes),
            command_event_count: event_frame.events.len() as u32,
            command_event_hash: event_frame.command_hash(),
            unresolved_dsp_writes: event_frame.unresolved_dsp_writes as u32,
        }
    }
}

pub fn checksum_samples(samples: &[i16]) -> u32 {
    AudioSampleStats::from_interleaved(samples, 2).checksum
}

pub fn checksum_dsp_writes(writes: &[DspWriteEvent]) -> u32 {
    let mut hash = FNV1A32_OFFSET;
    for write in writes {
        hash = fnv1a32_byte(hash, write.addr);
        hash = fnv1a32_byte(hash, write.value);
        hash = fnv1a32_bytes(hash, &write.sample_offset.to_le_bytes());
        hash = fnv1a32_byte(hash, write.timer_cycles);
    }
    hash
}

pub fn checksum_dsp_write_values(writes: &[DspWriteEvent]) -> u32 {
    let mut hash = FNV1A32_OFFSET;
    for write in writes {
        hash = fnv1a32_byte(hash, write.addr);
        hash = fnv1a32_byte(hash, write.value);
    }
    hash
}

fn classify_dsp_write(write: DspWriteEvent) -> AudioEventKind {
    match write.addr & 0x7f {
        0x4c => AudioEventKind::VoiceKeyOn { mask: write.value },
        0x5c => AudioEventKind::VoiceKeyOff { mask: write.value },
        0x2c => AudioEventKind::EchoParameter {
            parameter: EchoParameterKind::VolumeLeft,
            value: write.value,
        },
        0x3c => AudioEventKind::EchoParameter {
            parameter: EchoParameterKind::VolumeRight,
            value: write.value,
        },
        0x0d => AudioEventKind::EchoParameter {
            parameter: EchoParameterKind::Feedback,
            value: write.value,
        },
        0x4d => AudioEventKind::EchoParameter {
            parameter: EchoParameterKind::EnableMask,
            value: write.value,
        },
        0x0f | 0x1f | 0x2f | 0x3f | 0x4f | 0x5f | 0x6f | 0x7f => AudioEventKind::EchoParameter {
            parameter: EchoParameterKind::Fir((write.addr >> 4) & 7),
            value: write.value,
        },
        0x6d => AudioEventKind::EchoParameter {
            parameter: EchoParameterKind::StartAddress,
            value: write.value,
        },
        0x7d => AudioEventKind::EchoParameter {
            parameter: EchoParameterKind::Delay,
            value: write.value,
        },
        0x0c | 0x1c | 0x2d | 0x3d | 0x5d | 0x6c => AudioEventKind::GlobalParameter {
            register: write.addr & 0x7f,
            value: write.value,
        },
        reg if (reg & 0x0f) <= 0x07 => {
            let voice = reg >> 4;
            let parameter = match reg & 0x0f {
                0x00 => VoiceParameterKind::VolumeLeft,
                0x01 => VoiceParameterKind::VolumeRight,
                0x02 => VoiceParameterKind::PitchLow,
                0x03 => VoiceParameterKind::PitchHigh,
                0x04 => VoiceParameterKind::Source,
                0x05 => VoiceParameterKind::Adsr1,
                0x06 => VoiceParameterKind::Adsr2,
                0x07 => VoiceParameterKind::Gain,
                _ => unreachable!(),
            };
            AudioEventKind::VoiceParameter {
                voice,
                parameter,
                value: write.value,
            }
        }
        register => AudioEventKind::UnresolvedDspWrite {
            register,
            value: write.value,
        },
    }
}

const FNV1A32_OFFSET: u32 = 2166136261;
const FNV1A32_PRIME: u32 = 16777619;

fn fnv1a32_byte(hash: u32, byte: u8) -> u32 {
    (hash ^ u32::from(byte)).wrapping_mul(FNV1A32_PRIME)
}

fn fnv1a32_bytes(mut hash: u32, bytes: &[u8]) -> u32 {
    for &byte in bytes {
        hash = fnv1a32_byte(hash, byte);
    }
    hash
}

fn hash_bytes(hash: u32, bytes: &[u8]) -> u32 {
    fnv1a32_bytes(hash, bytes)
}

fn hash_music_state(mut hash: u32, music: MusicControlState) -> u32 {
    hash = fnv1a32_byte(hash, music.apui00);
    hash = fnv1a32_byte(hash, music.music_control);
    hash = fnv1a32_byte(hash, music.sound_effect_ambient);
    hash = fnv1a32_byte(hash, music.sound_effect_1);
    hash = fnv1a32_byte(hash, music.sound_effect_2);
    hash = fnv1a32_byte(hash, music.queued_music_control);
    fnv1a32_byte(hash, music.last_music_control)
}

fn hash_audio_event(mut hash: u32, event: &AudioEvent) -> u32 {
    hash = fnv1a32_bytes(hash, &event.sample_offset.to_le_bytes());
    hash = fnv1a32_byte(hash, event.timer_cycles);
    match &event.kind {
        AudioEventKind::MusicState(music) => {
            hash = fnv1a32_byte(hash, 1);
            hash_music_state(hash, *music)
        }
        AudioEventKind::ApuPorts {
            write,
            pending,
            input,
            spc_in,
            spc_out,
        } => {
            hash = fnv1a32_byte(hash, 2);
            hash = hash_bytes(hash, write);
            hash = hash_bytes(hash, pending);
            hash = hash_bytes(hash, input);
            hash = hash_bytes(hash, spc_in);
            hash_bytes(hash, spc_out)
        }
        AudioEventKind::VoiceKeyOn { mask } => {
            hash = fnv1a32_byte(hash, 3);
            fnv1a32_byte(hash, *mask)
        }
        AudioEventKind::VoiceKeyOff { mask } => {
            hash = fnv1a32_byte(hash, 4);
            fnv1a32_byte(hash, *mask)
        }
        AudioEventKind::VoiceParameter {
            voice,
            parameter,
            value,
        } => {
            hash = fnv1a32_byte(hash, 5);
            hash = fnv1a32_byte(hash, *voice);
            hash = fnv1a32_byte(hash, *parameter as u8);
            fnv1a32_byte(hash, *value)
        }
        AudioEventKind::EchoParameter { parameter, value } => {
            hash = fnv1a32_byte(hash, 6);
            hash = match parameter {
                EchoParameterKind::VolumeLeft => fnv1a32_byte(hash, 0),
                EchoParameterKind::VolumeRight => fnv1a32_byte(hash, 1),
                EchoParameterKind::Feedback => fnv1a32_byte(hash, 2),
                EchoParameterKind::EnableMask => fnv1a32_byte(hash, 3),
                EchoParameterKind::Fir(index) => fnv1a32_byte(fnv1a32_byte(hash, 4), *index),
                EchoParameterKind::Delay => fnv1a32_byte(hash, 5),
                EchoParameterKind::StartAddress => fnv1a32_byte(hash, 6),
            };
            fnv1a32_byte(hash, *value)
        }
        AudioEventKind::GlobalParameter { register, value } => {
            hash = fnv1a32_byte(hash, 7);
            hash = fnv1a32_byte(hash, *register);
            fnv1a32_byte(hash, *value)
        }
        AudioEventKind::UnresolvedDspWrite { register, value } => {
            hash = fnv1a32_byte(hash, 8);
            hash = fnv1a32_byte(hash, *register);
            fnv1a32_byte(hash, *value)
        }
        AudioEventKind::PlayMusic { track } => {
            hash = fnv1a32_byte(hash, 9);
            fnv1a32_byte(hash, *track)
        }
        AudioEventKind::StopMusic => fnv1a32_byte(hash, 10),
        AudioEventKind::PlaySfx { bank, id } => {
            hash = fnv1a32_byte(hash, 11);
            hash = fnv1a32_byte(hash, *bank);
            fnv1a32_byte(hash, *id)
        }
        AudioEventKind::SetTempo { value } => {
            hash = fnv1a32_byte(hash, 12);
            fnv1a32_byte(hash, *value)
        }
        AudioEventKind::NoteOn {
            voice,
            pitch,
            instrument,
            volume,
        } => {
            hash = fnv1a32_byte(hash, 13);
            hash = fnv1a32_byte(hash, *voice);
            hash = fnv1a32_byte(hash, *pitch);
            hash = fnv1a32_byte(hash, *instrument);
            fnv1a32_byte(hash, *volume)
        }
        AudioEventKind::NoteOff { voice } => {
            hash = fnv1a32_byte(hash, 14);
            fnv1a32_byte(hash, *voice)
        }
        AudioEventKind::SetEnvelope {
            voice,
            attack,
            decay,
            sustain,
            release,
        } => {
            hash = fnv1a32_byte(hash, 15);
            hash = fnv1a32_byte(hash, *voice);
            hash = fnv1a32_byte(hash, *attack);
            hash = fnv1a32_byte(hash, *decay);
            hash = fnv1a32_byte(hash, *sustain);
            fnv1a32_byte(hash, *release)
        }
        AudioEventKind::SetDuration { voice, frames } => {
            hash = fnv1a32_byte(hash, 16);
            hash = fnv1a32_byte(hash, *voice);
            fnv1a32_byte(hash, *frames)
        }
        AudioEventKind::PitchSlide {
            voice,
            target_pitch,
            frames,
        } => {
            hash = fnv1a32_byte(hash, 17);
            hash = fnv1a32_byte(hash, *voice);
            hash = fnv1a32_byte(hash, *target_pitch);
            fnv1a32_byte(hash, *frames)
        }
        AudioEventKind::SetNoise { voice, enabled } => {
            hash = fnv1a32_byte(hash, 18);
            hash = fnv1a32_byte(hash, *voice);
            fnv1a32_byte(hash, u8::from(*enabled))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_stats_match_legacy_audio_stats_shape() {
        let stats = AudioSampleStats::from_interleaved(&[0, -3, 7, 1], 2);

        assert_eq!(stats.samples_per_channel, 2);
        assert_eq!(stats.channels, 2);
        assert_eq!(stats.peak, 7);
        assert_eq!(stats.first_nonzero, Some(1));
        assert_eq!(stats.mean_abs, 2);
    }

    #[test]
    fn dsp_write_checksums_match_legacy_hash_contract() {
        let writes = [
            DspWriteEvent::new(0x4c, 0x80, 321, 17),
            DspWriteEvent::new(0x5c, 0x40, 322, 18),
        ];

        let legacy = {
            let mut hash = FNV1A32_OFFSET;
            for write in writes {
                hash = fnv1a32_byte(hash, write.addr);
                hash = fnv1a32_byte(hash, write.value);
                hash = fnv1a32_bytes(hash, &write.sample_offset.to_le_bytes());
                hash = fnv1a32_byte(hash, write.timer_cycles);
            }
            hash
        };

        assert_eq!(checksum_dsp_writes(&writes), legacy);
        assert_ne!(
            checksum_dsp_writes(&writes),
            checksum_dsp_write_values(&writes)
        );
    }

    #[test]
    fn event_frame_classifies_known_dsp_writes_and_keeps_unresolved_annotations() {
        let route = AudioRouteState {
            music: MusicControlState {
                music_control: 0x12,
                sound_effect_1: 0x34,
                ..MusicControlState::default()
            },
            queue: AudioQueueState {
                write: [1, 2, 3, 4],
                pending: [5, 6, 7, 8],
                input: [9, 10, 11, 12],
                ..AudioQueueState::default()
            },
            spc: None,
        };
        let writes = [
            DspWriteEvent::new(0x4c, 0x80, 10, 11),
            DspWriteEvent::new(0x02, 0x44, 12, 13),
            DspWriteEvent::new(0x7c, 0xff, 14, 15),
        ];

        let frame = AudioEventFrame::from_route_and_dsp_writes(route, &writes);

        assert_eq!(frame.events.len(), 5);
        assert_eq!(frame.unresolved_dsp_writes, 1);
        assert!(matches!(
            frame.events[2].kind,
            AudioEventKind::VoiceKeyOn { mask: 0x80 }
        ));
        assert!(matches!(
            frame.events[3].kind,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::PitchLow,
                value: 0x44
            }
        ));
        assert!(matches!(
            frame.events[4].kind,
            AudioEventKind::UnresolvedDspWrite {
                register: 0x7c,
                value: 0xff
            }
        ));
        assert_ne!(frame.command_hash(), 0);
    }
}
