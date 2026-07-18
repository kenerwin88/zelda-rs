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

/// Stereo placement encoded by the original engine in the high two bits of an
/// SFX command byte.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioPan {
    #[default]
    Center,
    Right,
    Left,
    Reserved,
}

impl AudioPan {
    pub const fn from_legacy_bits(value: u8) -> Self {
        match value & 0xc0 {
            0x40 => Self::Right,
            0x80 => Self::Left,
            0xc0 => Self::Reserved,
            _ => Self::Center,
        }
    }

    pub const fn legacy_bits(self) -> u8 {
        match self {
            Self::Center => 0,
            Self::Right => 0x40,
            Self::Left => 0x80,
            Self::Reserved => 0xc0,
        }
    }
}

/// Semantic SFX latches used by the engine. Their numeric order intentionally
/// matches the modern catalog banks and APUI ports 1 through 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioSfxBank {
    Ambient,
    Effect1,
    Effect2,
}

impl AudioSfxBank {
    pub const ALL: [Self; 3] = [Self::Ambient, Self::Effect1, Self::Effect2];

    pub const fn catalog_id(self) -> u8 {
        match self {
            Self::Ambient => 0,
            Self::Effect1 => 1,
            Self::Effect2 => 2,
        }
    }

    pub const fn port_index(self) -> usize {
        self.catalog_id() as usize + 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioSfxCommand {
    pub effect: u8,
    pub pan: AudioPan,
}

impl AudioSfxCommand {
    pub const fn from_legacy_value(value: u8) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self {
                effect: value & 0x3f,
                pan: AudioPan::from_legacy_bits(value),
            })
        }
    }

    pub const fn legacy_value(self) -> u8 {
        (self.effect & 0x3f) | self.pan.legacy_bits()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioMusicCommand {
    #[default]
    Clear,
    Play {
        track: u8,
    },
    Stop,
    Control {
        value: u8,
    },
}

impl AudioMusicCommand {
    pub const fn from_legacy_value(value: u8) -> Self {
        match value {
            0 => Self::Clear,
            0xf0 => Self::Stop,
            1..=0xef => Self::Play { track: value },
            _ => Self::Control { value },
        }
    }

    pub const fn legacy_value(self) -> u8 {
        match self {
            Self::Clear => 0,
            Self::Play { track } => track,
            Self::Stop => 0xf0,
            Self::Control { value } => value,
        }
    }
}

/// A gameplay-authored audio command. The modern path consumes these commands
/// directly; APUI bytes are only a compatibility projection for the oracle and
/// legacy save formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EngineAudioCommand {
    ClearMusic,
    PlayMusic {
        track: u8,
    },
    StopMusic,
    MusicControl {
        value: u8,
    },
    ClearSfx {
        bank: AudioSfxBank,
    },
    PlaySfx {
        bank: AudioSfxBank,
        effect: u8,
        pan: AudioPan,
    },
}

impl EngineAudioCommand {
    pub const fn from_music_port_value(value: u8) -> Self {
        match AudioMusicCommand::from_legacy_value(value) {
            AudioMusicCommand::Clear => Self::ClearMusic,
            AudioMusicCommand::Play { track } => Self::PlayMusic { track },
            AudioMusicCommand::Stop => Self::StopMusic,
            AudioMusicCommand::Control { value } => Self::MusicControl { value },
        }
    }

    pub const fn from_sfx_port_value(bank: AudioSfxBank, value: u8) -> Self {
        match AudioSfxCommand::from_legacy_value(value) {
            Some(command) => Self::PlaySfx {
                bank,
                effect: command.effect,
                pan: command.pan,
            },
            None => Self::ClearSfx { bank },
        }
    }

    pub const fn from_apui_write(port: usize, value: u8) -> Self {
        match port & 3 {
            0 => Self::from_music_port_value(value),
            1 => Self::from_sfx_port_value(AudioSfxBank::Ambient, value),
            2 => Self::from_sfx_port_value(AudioSfxBank::Effect1, value),
            _ => Self::from_sfx_port_value(AudioSfxBank::Effect2, value),
        }
    }

    pub const fn legacy_port_write(self) -> (usize, u8) {
        match self {
            Self::ClearMusic => (0, 0),
            Self::PlayMusic { track } => (0, track),
            Self::StopMusic => (0, 0xf0),
            Self::MusicControl { value } => (0, value),
            Self::ClearSfx { bank } => (bank.port_index(), 0),
            Self::PlaySfx { bank, effect, pan } => (
                bank.port_index(),
                AudioSfxCommand { effect, pan }.legacy_value(),
            ),
        }
    }
}

/// One frame's four engine command latches. Each slot is last-write-wins, just
/// like the original hardware ports, so replacing a command before NMI does not
/// accidentally play both effects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EngineAudioCommandBatch {
    music: AudioMusicCommand,
    sfx: [Option<AudioSfxCommand>; 3],
}

impl EngineAudioCommandBatch {
    pub fn from_legacy_ports(ports: [u8; 4]) -> Self {
        let mut batch = Self::default();
        for (port, value) in ports.into_iter().enumerate() {
            batch.apply(EngineAudioCommand::from_apui_write(port, value));
        }
        batch
    }

    pub fn apply(&mut self, command: EngineAudioCommand) {
        match command {
            EngineAudioCommand::ClearMusic => self.music = AudioMusicCommand::Clear,
            EngineAudioCommand::PlayMusic { track } => {
                self.music = AudioMusicCommand::from_legacy_value(track);
            }
            EngineAudioCommand::StopMusic => self.music = AudioMusicCommand::Stop,
            EngineAudioCommand::MusicControl { value } => {
                self.music = AudioMusicCommand::from_legacy_value(value);
            }
            EngineAudioCommand::ClearSfx { bank } => {
                self.sfx[usize::from(bank.catalog_id())] = None;
            }
            EngineAudioCommand::PlaySfx { bank, effect, pan } => {
                self.sfx[usize::from(bank.catalog_id())] = Some(AudioSfxCommand { effect, pan });
            }
        }
    }

    pub const fn music(self) -> AudioMusicCommand {
        self.music
    }

    pub fn sfx(self, bank: AudioSfxBank) -> Option<AudioSfxCommand> {
        self.sfx[usize::from(bank.catalog_id())]
    }

    pub fn legacy_ports(self) -> [u8; 4] {
        let mut ports = [self.music.legacy_value(), 0, 0, 0];
        for (index, command) in self.sfx.into_iter().enumerate() {
            ports[index + 1] = command.map_or(0, AudioSfxCommand::legacy_value);
        }
        ports
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpcSequencerState {
    pub spc_in: [u8; 4],
    pub spc_out: [u8; 4],
    pub timer_cycles: u8,
    pub sfx_timer_accum: u8,
    pub main_tempo_accum: u8,
    pub block_count: u8,
    pub key_on: u8,
    pub key_off: u8,
    pub current_bit: u8,
    pub port1_active: u8,
    pub port2_active: u8,
    pub port3_active: u8,
    pub is_chan_on: u8,
    pub echo_enable_mask: u8,
    pub echo_enable_frame_start: u8,
    pub echo_enable_values: [u8; 16],
    pub echo_enable_offsets: [u16; 16],
    pub echo_enable_count: u8,
    pub echo_volume_registers: [u8; 32],
    pub echo_volume_values: [u8; 32],
    pub echo_volume_offsets: [u16; 32],
    pub echo_volume_count: u8,
    pub global_registers: [u8; 32],
    pub global_values: [u8; 32],
    pub global_offsets: [u16; 32],
    pub global_count: u8,
    pub voice_sources: [u8; 8],
    pub voice_adsr1: [u8; 8],
    pub voice_adsr2: [u8; 8],
    pub voice_gain: [u8; 8],
    pub voice_volume_left: [i8; 8],
    pub voice_volume_right: [i8; 8],
    pub vol_dirty: u8,
    pub ch7_sfx: u8,
    pub ch7_sfx_ptr: u16,
    pub ch7_pattern: u16,
    pub ch7_ticks: u8,
    pub ch7_keyoff_ticks: u8,
    pub sfx_kof_masks: [u8; 8],
    pub sfx_kof_offsets: [u16; 8],
    pub sfx_kof_count: u8,
    pub raw_kof_masks: [u8; 32],
    pub raw_kof_offsets: [u16; 32],
    pub raw_kof_count: u8,
    pub sfx_kon_masks: [u8; 8],
    pub sfx_kon_owned_masks: [u8; 8],
    pub sfx_kon_offsets: [u16; 8],
    pub sfx_kon_rate_counters: [[u16; 8]; 8],
    pub sfx_kon_sources: [[u8; 8]; 8],
    pub sfx_kon_adsr1: [[u8; 8]; 8],
    pub sfx_kon_adsr2: [[u8; 8]; 8],
    pub sfx_kon_gain: [[u8; 8]; 8],
    pub sfx_kon_volume_left: [[i8; 8]; 8],
    pub sfx_kon_volume_right: [[i8; 8]; 8],
    pub sfx_kon_count: u8,
    pub sfx_echo_masks: [u8; 8],
    pub sfx_echo_enabled: [bool; 8],
    pub sfx_echo_offsets: [u16; 8],
    pub sfx_echo_count: u8,
    pub sfx_pitch_masks: [u8; 32],
    pub sfx_pitch_words: [u16; 32],
    pub sfx_pitch_offsets: [u16; 32],
    pub sfx_pitch_count: u8,
    pub raw_pitch_masks: [u8; 32],
    pub raw_pitch_words: [u16; 32],
    pub raw_pitch_offsets: [u16; 32],
    pub raw_pitch_masks_hi: [u8; 32],
    pub raw_pitch_words_hi: [u16; 32],
    pub raw_pitch_offsets_hi: [u16; 32],
    pub raw_pitch_masks_hi2: [u8; 32],
    pub raw_pitch_words_hi2: [u16; 32],
    pub raw_pitch_offsets_hi2: [u16; 32],
    pub raw_pitch_masks_hi3: [u8; 32],
    pub raw_pitch_words_hi3: [u16; 32],
    pub raw_pitch_offsets_hi3: [u16; 32],
    pub raw_pitch_count: u8,
    pub sfx_volume_masks: [u8; 32],
    pub sfx_volume_left: [i8; 32],
    pub sfx_volume_right: [i8; 32],
    pub sfx_volume_offsets: [u16; 32],
    pub sfx_volume_count: u8,
    pub raw_volume_masks: [u8; 32],
    pub raw_volume_left: [i8; 32],
    pub raw_volume_right: [i8; 32],
    pub raw_volume_offsets: [u16; 32],
    pub raw_volume_count: u8,
    pub raw_envelope_masks: [u8; 32],
    pub raw_envelope_registers: [u8; 32],
    pub raw_envelope_values: [u8; 32],
    pub raw_envelope_offsets: [u16; 32],
    pub raw_envelope_count: u8,
    pub sfx_setup_masks: [u8; 8],
    pub sfx_setup_sources: [u8; 8],
    pub sfx_setup_adsr1: [u8; 8],
    pub sfx_setup_adsr2: [u8; 8],
    pub sfx_setup_gain: [u8; 8],
    pub sfx_setup_offsets: [u16; 8],
    pub sfx_setup_count: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioRouteState {
    pub music: MusicControlState,
    pub queue: AudioQueueState,
    pub spc: Option<SpcSequencerState>,
    pub sample_bank_id: u8,
    pub sample_bank_generation: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioEventFrame {
    pub music: MusicControlState,
    pub queue: AudioQueueState,
    pub events: Vec<AudioEvent>,
    pub unresolved_dsp_writes: usize,
    /// True when a semantic sequencer has already interpreted the route state.
    ///
    /// The renderer uses this to avoid interpreting the raw APU ports a second
    /// time on quiet frames that happen not to contain a note command.
    #[serde(default)]
    pub sequenced: bool,
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
            kind: AudioEventKind::SampleBankState {
                bank_id: route.sample_bank_id,
                generation: route.sample_bank_generation,
            },
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
            sequenced: false,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioNoteOrigin {
    Music,
    Sfx,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioEventKind {
    MusicState(MusicControlState),
    SampleBankState {
        bank_id: u8,
        generation: u32,
    },
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
    /// SPC music-master volume. Unlike DSP MVOL, this must not attenuate SFX
    /// voices which temporarily occupy music channels.
    SetMusicVolume {
        value: u8,
    },
    SetNoteOrigin {
        voice: u8,
        origin: AudioNoteOrigin,
    },
    SetPitchWord {
        voice: u8,
        pitch_word: u16,
    },
    SetPitchRegisterWord {
        voice: u8,
        pitch_word: u16,
    },
    SetStereoVolume {
        voice: u8,
        left: i8,
        right: i8,
    },
    SetDspEnvelope {
        voice: u8,
        adsr1: u8,
        adsr2: u8,
        gain: u8,
    },
    SetEnvelopeRateCounter {
        voice: u8,
        rate_counter: u16,
    },
    NoteOn {
        voice: u8,
        pitch: u8,
        instrument: u8,
        volume: u8,
    },
    /// A raw DSP KON write for a fully staged music voice. Unlike `NoteOn`,
    /// this event is timestamped at the register write; the renderer owns the
    /// KON polling and voice-start pipeline that follows it.
    DspKeyOn {
        voice: u8,
        pitch: u8,
        instrument: u8,
        volume: u8,
    },
    /// A raw DSP KOFF write. Unlike `NoteOff`, this event is timestamped at
    /// the register write; the renderer owns the envelope-release boundary.
    DspKeyOff {
        voice: u8,
    },
    /// DSP KON using the voice's already-programmed SRCN, ADSR, pitch, and
    /// volume registers.
    RetriggerVoice {
        voice: u8,
    },
    KeyOnVoice {
        voice: u8,
        source: u8,
        adsr1: u8,
        adsr2: u8,
        gain: u8,
        volume_left: i8,
        volume_right: i8,
        rate_counter: u16,
    },
    ResetEchoVolume {
        restore_offset: u16,
    },
    NoteOff {
        voice: u8,
    },
    SetDuration {
        voice: u8,
        frames: u16,
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
    SetPan {
        voice: u8,
        pan: i8,
    },
    SetEchoSend {
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
        AudioEventKind::SampleBankState {
            bank_id,
            generation,
        } => {
            hash = fnv1a32_byte(hash, 21);
            hash = fnv1a32_byte(hash, *bank_id);
            fnv1a32_bytes(hash, &generation.to_le_bytes())
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
        AudioEventKind::SetMusicVolume { value } => {
            hash = fnv1a32_byte(hash, 29);
            fnv1a32_byte(hash, *value)
        }
        AudioEventKind::SetNoteOrigin { voice, origin } => {
            hash = fnv1a32_byte(hash, 21);
            hash = fnv1a32_byte(hash, *voice);
            fnv1a32_byte(
                hash,
                match origin {
                    AudioNoteOrigin::Music => 0,
                    AudioNoteOrigin::Sfx => 1,
                },
            )
        }
        AudioEventKind::SetPitchWord { voice, pitch_word } => {
            hash = fnv1a32_byte(hash, 22);
            hash = fnv1a32_byte(hash, *voice);
            let [lo, hi] = pitch_word.to_le_bytes();
            fnv1a32_byte(fnv1a32_byte(hash, lo), hi)
        }
        AudioEventKind::SetPitchRegisterWord { voice, pitch_word } => {
            hash = fnv1a32_byte(hash, 26);
            hash = fnv1a32_byte(hash, *voice);
            let [lo, hi] = pitch_word.to_le_bytes();
            fnv1a32_byte(fnv1a32_byte(hash, lo), hi)
        }
        AudioEventKind::SetStereoVolume { voice, left, right } => {
            hash = fnv1a32_byte(hash, 23);
            hash = fnv1a32_byte(hash, *voice);
            hash = fnv1a32_byte(hash, *left as u8);
            fnv1a32_byte(hash, *right as u8)
        }
        AudioEventKind::SetDspEnvelope {
            voice,
            adsr1,
            adsr2,
            gain,
        } => {
            hash = fnv1a32_byte(hash, 24);
            hash = fnv1a32_byte(hash, *voice);
            hash = fnv1a32_byte(hash, *adsr1);
            hash = fnv1a32_byte(hash, *adsr2);
            fnv1a32_byte(hash, *gain)
        }
        AudioEventKind::SetEnvelopeRateCounter {
            voice,
            rate_counter,
        } => {
            hash = fnv1a32_byte(hash, 28);
            hash = fnv1a32_byte(hash, *voice);
            let [lo, hi] = rate_counter.to_le_bytes();
            fnv1a32_byte(fnv1a32_byte(hash, lo), hi)
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
        AudioEventKind::DspKeyOn {
            voice,
            pitch,
            instrument,
            volume,
        } => {
            hash = fnv1a32_byte(hash, 30);
            hash = fnv1a32_byte(hash, *voice);
            hash = fnv1a32_byte(hash, *pitch);
            hash = fnv1a32_byte(hash, *instrument);
            fnv1a32_byte(hash, *volume)
        }
        AudioEventKind::DspKeyOff { voice } => {
            hash = fnv1a32_byte(hash, 31);
            fnv1a32_byte(hash, *voice)
        }
        AudioEventKind::RetriggerVoice { voice } => {
            hash = fnv1a32_byte(hash, 27);
            fnv1a32_byte(hash, *voice)
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
            hash = fnv1a32_byte(hash, 25);
            hash = fnv1a32_byte(hash, *voice);
            hash = fnv1a32_byte(hash, *source);
            hash = fnv1a32_byte(hash, *adsr1);
            hash = fnv1a32_byte(hash, *adsr2);
            hash = fnv1a32_byte(hash, *gain);
            hash = fnv1a32_byte(hash, *volume_left as u8);
            hash = fnv1a32_byte(hash, *volume_right as u8);
            hash = fnv1a32_byte(hash, *rate_counter as u8);
            fnv1a32_byte(hash, (*rate_counter >> 8) as u8)
        }
        AudioEventKind::ResetEchoVolume { restore_offset } => {
            hash = fnv1a32_byte(hash, 27);
            let [lo, hi] = restore_offset.to_le_bytes();
            fnv1a32_byte(fnv1a32_byte(hash, lo), hi)
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
            let [lo, hi] = frames.to_le_bytes();
            fnv1a32_byte(fnv1a32_byte(hash, lo), hi)
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
        AudioEventKind::SetPan { voice, pan } => {
            hash = fnv1a32_byte(hash, 19);
            hash = fnv1a32_byte(hash, *voice);
            fnv1a32_byte(hash, *pan as u8)
        }
        AudioEventKind::SetEchoSend { voice, enabled } => {
            hash = fnv1a32_byte(hash, 20);
            hash = fnv1a32_byte(hash, *voice);
            fnv1a32_byte(hash, u8::from(*enabled))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_audio_commands_round_trip_legacy_ports_without_losing_semantics() {
        let ports = [0x12, 0, 0x4a, 0x8b];
        let batch = EngineAudioCommandBatch::from_legacy_ports(ports);

        assert_eq!(batch.legacy_ports(), ports);
        assert_eq!(batch.music(), AudioMusicCommand::Play { track: 0x12 });
        assert_eq!(
            batch.sfx(AudioSfxBank::Effect1),
            Some(AudioSfxCommand {
                effect: 0x0a,
                pan: AudioPan::Right,
            })
        );
        assert_eq!(
            batch.sfx(AudioSfxBank::Effect2),
            Some(AudioSfxCommand {
                effect: 0x0b,
                pan: AudioPan::Left,
            })
        );
    }

    #[test]
    fn engine_audio_command_batch_is_last_write_wins_per_semantic_bank() {
        let mut batch = EngineAudioCommandBatch::default();
        batch.apply(EngineAudioCommand::PlaySfx {
            bank: AudioSfxBank::Effect1,
            effect: 1,
            pan: AudioPan::Center,
        });
        batch.apply(EngineAudioCommand::PlaySfx {
            bank: AudioSfxBank::Effect1,
            effect: 2,
            pan: AudioPan::Right,
        });

        assert_eq!(batch.legacy_ports(), [0, 0, 0x42, 0]);
    }

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
            ..AudioRouteState::default()
        };
        let writes = [
            DspWriteEvent::new(0x4c, 0x80, 10, 11),
            DspWriteEvent::new(0x02, 0x44, 12, 13),
            DspWriteEvent::new(0x7c, 0xff, 14, 15),
        ];

        let frame = AudioEventFrame::from_route_and_dsp_writes(route, &writes);

        assert_eq!(frame.events.len(), 6);
        assert_eq!(frame.unresolved_dsp_writes, 1);
        assert!(matches!(
            frame.events[3].kind,
            AudioEventKind::VoiceKeyOn { mask: 0x80 }
        ));
        assert!(matches!(
            frame.events[4].kind,
            AudioEventKind::VoiceParameter {
                voice: 0,
                parameter: VoiceParameterKind::PitchLow,
                value: 0x44
            }
        ));
        assert!(matches!(
            frame.events[5].kind,
            AudioEventKind::UnresolvedDspWrite {
                register: 0x7c,
                value: 0xff
            }
        ));
        assert_ne!(frame.command_hash(), 0);
    }
}
