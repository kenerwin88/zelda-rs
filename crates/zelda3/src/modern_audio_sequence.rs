use crate::game_output::{
    AudioEvent, AudioEventFrame, AudioEventKind, AudioMusicCommand, AudioNoteOrigin,
    AudioRouteState, AudioSfxBank, DspWriteEvent, EngineAudioCommandBatch, MusicControlState,
    VoiceParameterKind,
};
use crate::modern_music_catalog::{
    decode_note, notes_starting_in, packed_track, ModernMusicNote, PACKED_NOTE_BYTES,
};
use crate::modern_music_globals::events_at as music_global_events_at;
use crate::modern_sfx_catalog::{
    lookup_sfx_program_for_context, sfx_program_hash, ModernSfxProgram, ModernSfxRuntimeContext,
    ModernSfxWaveform,
};
use crate::modern_sfx_dsp_catalog::{exact_sfx_dsp_step, ExactSfxDspStep};
use crate::modern_sfx_pitch_catalog::pitch_events as exact_sfx_pitch_events;

const SFX_SLOTS: usize = 7;
const MUSIC_NATIVE_FRAME_SAMPLES: u64 = 534;
// The translated engine publishes its audio command batch at the end of the
// game frame. On hardware, the CPU-to-APU port write reaches the DSP half a
// scheduler tick before that boundary.
const ENGINE_AUDIO_COMMAND_PHASE_SAMPLES: i32 = -32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingSfxStep {
    step: crate::modern_sfx_catalog::ModernSfxStep,
    exact: Option<ExactSfxDspStep>,
    #[serde(default)]
    engine_dsp_envelope: Option<[u8; 3]>,
    delay_after_previous: u8,
    #[serde(default)]
    preserve_existing_volume: bool,
    #[serde(default)]
    volume_via_parameters: bool,
    #[serde(default)]
    refresh_repeat_on_keyon: bool,
    #[serde(default)]
    preserve_inactive_pitch_latch: bool,
    #[serde(default)]
    engine_keyoff_owned: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingSfxPitchChange {
    samples_remaining: u32,
    pitch_word: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingSfxVolumeChange {
    samples_remaining: u32,
    left: i8,
    right: i8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Port3Id15Action {
    KeyOff {
        mask: u8,
        release: bool,
    },
    Chord {
        index: u8,
    },
    Voice3Note {
        pitch_word: u16,
        volume: i8,
        rate_counter: u16,
    },
    AuxVoice6Note {
        pitch_word: u16,
        rate_counter: u16,
        write_volume: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingPort3Id15Action {
    samples_remaining: u32,
    action: Port3Id15Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Bank1Id1Action {
    KeyOff { release: bool },
    KeyOn { pitch_word: u16, volume: i8 },
    Pitch { pitch_word: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingBank1Id1Action {
    samples_remaining: u32,
    action: Bank1Id1Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Bank1Id45Action {
    KeyOff { release: bool },
    KeyOn,
    Pitch { pitch_word: u16 },
    PitchAndVolume { pitch_word: u16, volume: i8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingBank1Id45Action {
    samples_remaining: u32,
    action: Bank1Id45Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Bank1Id95Action {
    KeyOff {
        release: bool,
    },
    KeyOn {
        source: u8,
        adsr1: u8,
        adsr2: u8,
        pitch_word: u16,
        left: i8,
        right: i8,
        write_volume: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingBank1Id95Action {
    samples_remaining: u32,
    action: Bank1Id95Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Bank1Id41Action {
    KeyOff {
        release: bool,
    },
    KeyOn {
        initial_pitch: u16,
        pitch_word: u16,
        volume: i8,
        rate_counter: u16,
    },
    Pitch {
        pitch_word: u16,
    },
    Volume {
        volume: i8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingBank1Id41Action {
    samples_remaining: u32,
    action: Bank1Id41Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Bank1Id60Action {
    KeyOff {
        release: bool,
    },
    KeyOn {
        pitch_word: u16,
        volume: i8,
        rate_counter: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingBank1Id60Action {
    samples_remaining: u32,
    action: Bank1Id60Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Bank2Id28Action {
    KeyOff { release: bool },
    KeyOn { pitch_word: u16, rate_counter: u16 },
    Pitch { pitch_word: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingBank2Id28Action {
    samples_remaining: u32,
    action: Bank2Id28Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Bank2Id9Action {
    KeyOff {
        release: bool,
    },
    KeyOn {
        initial_pitch: u16,
        pitch_word: u16,
        volume: i8,
    },
    Pitch {
        pitch_word: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingBank2Id9Action {
    samples_remaining: u32,
    action: Bank2Id9Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Bank2Id14Action {
    KeyOff { release: bool },
    KeyOn { pitch_word: u16, volume: i8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingBank2Id14Action {
    samples_remaining: u32,
    action: Bank2Id14Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Bank2Id11Action {
    KeyOff {
        release: bool,
    },
    KeyOn {
        initial_pitch: u16,
        pitch_word: u16,
        rate_counter: u16,
    },
    Pitch {
        pitch_word: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct PendingBank2Id11Action {
    samples_remaining: u32,
    action: Bank2Id11Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct LoopingSfxVoice {
    step: crate::modern_sfx_catalog::ModernSfxStep,
    exact: ExactSfxDspStep,
    overflows_remaining: u16,
    active: bool,
    active_overflows: u16,
    gap_overflows: u16,
    #[serde(default = "default_infinite_sfx_retriggers")]
    retriggers_remaining: u8,
    #[serde(default)]
    retrigger_index: u8,
    #[serde(default)]
    staggered_chime: bool,
}

const fn default_infinite_sfx_retriggers() -> u8 {
    u8::MAX
}

const fn default_port3_id15_voices() -> [u8; 5] {
    [2, 3, 4, 5, 6]
}

const RISING_WARBLE_SECOND_PITCHES: [u16; 96] = [
    4732, 4660, 4584, 4512, 4440, 4368, 4300, 4232, 4164, 4096, 4032, 3968, 3904, 3844, 3780, 3724,
    3664, 3604, 3656, 3708, 3764, 3816, 3872, 3928, 3988, 4044, 3872, 3708, 3552, 3400, 3256, 3120,
    2984, 2860, 2944, 3028, 3120, 3208, 3304, 3400, 3500, 3604, 3476, 3352, 3232, 3120, 3008, 2900,
    2800, 2700, 2760, 2820, 2880, 2944, 3008, 3072, 3140, 3208, 3096, 2984, 2880, 2780, 2680, 2584,
    2492, 2404, 2456, 2512, 2564, 2624, 2680, 2740, 2800, 2860, 2760, 2660, 2564, 2476, 2388, 2300,
    2220, 2140, 2204, 2268, 2336, 2404, 2476, 2548, 2624, 2700, 2572, 2452, 2336, 2228, 2120, 2020,
];
const PORT2_ID30_FIRST_PITCHES: [u16; 5] = [3_104, 2_996, 2_894, 2_796, 2_700];
const PORT2_ID30_SECOND_PITCHES: [u16; 45] = [
    2_902, 2_944, 2_988, 3_030, 3_076, 3_120, 3_166, 3_212, 3_098, 2_988, 2_880, 2_780, 2_680,
    2_586, 2_494, 2_404, 2_494, 2_586, 2_680, 2_780, 2_880, 2_988, 3_098, 3_212, 3_098, 2_988,
    2_880, 2_780, 2_680, 2_586, 2_494, 2_404, 2_494, 2_586, 2_680, 2_780, 2_880, 2_988, 3_098,
    3_212, 3_030, 2_860, 2_700, 2_548, 2_404,
];
const PORT2_ID30_SECOND_VOLUMES: [(usize, i8); 5] =
    [(8, 28), (16, 25), (24, 22), (32, 18), (40, 9)];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernAudioSequenceStats {
    pub music_commands: u32,
    pub sfx_commands: u32,
    pub note_events: u32,
    pub envelope_events: u32,
    pub ignored_commands: u32,
    pub known_sfx_commands: u32,
    pub unknown_sfx_commands: u32,
    pub fallback_sfx_commands: u32,
    #[serde(default)]
    pub exact_sfx_steps: u32,
    pub program_hash: u32,
    #[serde(default)]
    pub known_sfx_programs: [u16; SFX_SLOTS],
    #[serde(default)]
    pub unknown_sfx_programs: [u16; SFX_SLOTS],
    #[serde(default)]
    pub known_sfx_program_count: u8,
    #[serde(default)]
    pub unknown_sfx_program_count: u8,
    #[serde(default)]
    pub active_voice_mask: u8,
    #[serde(default)]
    pub sfx_voice_mask_start: u8,
    #[serde(default)]
    pub sfx_voice_mask: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModernAudioSequencer {
    last_music_track: u8,
    #[serde(default)]
    music_frame_position: u16,
    #[serde(default)]
    music_sample_position: u64,
    last_sfx: [u8; SFX_SLOTS],
    active_voice_mask: u8,
    #[serde(default)]
    music_voice_mask: u8,
    #[serde(default)]
    sfx_voice_mask: u8,
    #[serde(default)]
    sfx_voice_program: [u16; 8],
    #[serde(default)]
    voice_dsp_source: [u8; 8],
    #[serde(default)]
    persistent_sfx_pitch_words: [u16; 8],
    #[serde(default)]
    music_keyoff_frames_remaining: [u16; 8],
    #[serde(default)]
    music_keyoff_sample_offset: [u16; 8],
    #[serde(default)]
    voice_frames_remaining: [u16; 8],
    #[serde(default)]
    pending_voice_steps: [Vec<PendingSfxStep>; 8],
    #[serde(default)]
    sfx_keyoff_samples_remaining: [u32; 8],
    #[serde(default)]
    sfx_ownership_samples_remaining: [u32; 8],
    #[serde(default)]
    sfx_release_pending_mask: u8,
    #[serde(default)]
    sfx_ownership_release_overflows: [u8; 8],
    #[serde(default)]
    sfx_release_overflows_remaining: [u8; 8],
    #[serde(default)]
    sfx_keyoff_starts_ownership_mask: u8,
    #[serde(default)]
    pending_sfx_pitch_changes: [Vec<PendingSfxPitchChange>; 8],
    #[serde(default)]
    pending_sfx_volume_changes: [Vec<PendingSfxVolumeChange>; 8],
    #[serde(default)]
    sfx_voice_activation_count: [u8; 8],
    #[serde(default)]
    secondary_sfx_keyoff_samples_remaining: [u32; 8],
    #[serde(default)]
    rising_warble_long_pattern: [bool; 8],
    #[serde(default)]
    rising_warble_retrigger_samples_remaining: [u32; 8],
    #[serde(default)]
    rising_warble_command_clock: Option<(u8, u8)>,
    #[serde(default)]
    bank1_id29_voice7_hold_frames: u8,
    #[serde(skip)]
    frame_warble_pitch_events: Vec<(u8, i32, u16)>,
    #[serde(skip)]
    frame_warble_volume_events: Vec<(u8, i32, i8, i8)>,
    #[serde(default)]
    looping_sfx_voices: [Option<LoopingSfxVoice>; 8],
    #[serde(default)]
    previous_sfx_clock: Option<(u8, u8)>,
    #[serde(default)]
    semantic_sfx_pending_steps: [Vec<PendingSfxStep>; 8],
    #[serde(default)]
    semantic_sfx_repeat_steps: [Option<PendingSfxStep>; 8],
    #[serde(default)]
    semantic_pitch_latch_mask: u8,
    #[serde(default)]
    engine_automated_sfx_volume_mask: u8,
    #[serde(default)]
    semantic_bank1_command_voice: Option<u8>,
    #[serde(default)]
    semantic_bank2_command_voice: Option<u8>,
    #[serde(skip)]
    port23_id36_cluster_command: bool,
    #[serde(default)]
    port23_id36_cluster_active: bool,
    #[serde(default)]
    port23_id36_voice7_keyoffs: [u32; 2],
    #[serde(default)]
    port23_id36_voice7_retrigger: u32,
    #[serde(default)]
    port23_id36_voice6_final_keyoff: u32,
    #[serde(default)]
    port23_id36_voice7_late_keyoffs: [u32; 3],
    #[serde(default)]
    port23_id36_voice7_late_retrigger: u32,
    #[serde(default)]
    port23_id36_voice6_owned: bool,
    #[serde(default)]
    port23_id36_voice7_owned: bool,
    #[serde(skip)]
    frame_port23_id36_voice7_retrigger: bool,
    #[serde(default)]
    pending_port3_id15_actions: Vec<PendingPort3Id15Action>,
    #[serde(default)]
    port3_id15_owned_mask: u8,
    #[serde(default)]
    port3_id15_active: bool,
    #[serde(default)]
    port3_id15_voice3_volume: i8,
    #[serde(default = "default_port3_id15_voices")]
    port3_id15_voices: [u8; 5],
    #[serde(default)]
    pending_bank1_id1_actions: Vec<PendingBank1Id1Action>,
    #[serde(default)]
    bank1_id1_active: bool,
    #[serde(default)]
    pending_bank1_id45_actions: Vec<PendingBank1Id45Action>,
    #[serde(default)]
    bank1_id45_active: bool,
    #[serde(default)]
    bank1_id45_voice: u8,
    #[serde(default)]
    pending_bank1_id95_actions: Vec<PendingBank1Id95Action>,
    #[serde(default)]
    bank1_id95_active: bool,
    #[serde(default)]
    bank1_id95_voice: u8,
    #[serde(default)]
    pending_bank1_id41_actions: Vec<PendingBank1Id41Action>,
    #[serde(default)]
    bank1_id41_active: bool,
    #[serde(default)]
    pending_bank1_id60_actions: Vec<PendingBank1Id60Action>,
    #[serde(default)]
    bank1_id60_active: bool,
    #[serde(default)]
    pending_bank2_id28_actions: Vec<PendingBank2Id28Action>,
    #[serde(default)]
    bank2_id28_active: bool,
    #[serde(default)]
    pending_bank2_id9_actions: Vec<PendingBank2Id9Action>,
    #[serde(default)]
    bank2_id9_active: bool,
    #[serde(default)]
    bank2_id9_voice: u8,
    #[serde(default)]
    pending_bank2_id14_actions: Vec<PendingBank2Id14Action>,
    #[serde(default)]
    bank2_id14_active: bool,
    #[serde(default)]
    bank2_id14_voice: u8,
    #[serde(default)]
    pending_bank2_id11_actions: Vec<PendingBank2Id11Action>,
    #[serde(default)]
    bank2_id11_active: bool,
    #[serde(default)]
    bank2_id11_voice: u8,
    #[serde(default)]
    engine_receipt_mode: bool,
    #[serde(default)]
    music_echo_mask: u8,
    #[serde(default = "default_music_master_volume")]
    music_master_volume: u16,
    #[serde(default)]
    music_master_fade_ticks: u8,
    #[serde(default)]
    music_master_fade_add: i16,
    #[serde(default)]
    saved_music_master_volume: u8,
    #[serde(default = "default_music_tempo")]
    music_tempo: u8,
    #[serde(default)]
    music_tempo_accum: u8,
    #[serde(default)]
    last_music_control_command: u8,
    #[serde(default)]
    dsp_timer_cycles: u8,
    #[serde(default)]
    sfx_timer_accum: u8,
    #[serde(default)]
    sfx_clock_epoch: u32,
    #[serde(default)]
    timer_initialized: bool,
    last_stats: ModernAudioSequenceStats,
}

impl Default for ModernAudioSequencer {
    fn default() -> Self {
        Self {
            last_music_track: 0,
            music_frame_position: 0,
            music_sample_position: 0,
            last_sfx: [0; SFX_SLOTS],
            active_voice_mask: 0,
            music_voice_mask: 0,
            sfx_voice_mask: 0,
            sfx_voice_program: [0; 8],
            voice_dsp_source: [0; 8],
            persistent_sfx_pitch_words: [0; 8],
            music_keyoff_frames_remaining: [0; 8],
            music_keyoff_sample_offset: [0; 8],
            voice_frames_remaining: [0; 8],
            pending_voice_steps: std::array::from_fn(|_| Vec::new()),
            sfx_keyoff_samples_remaining: [0; 8],
            sfx_ownership_samples_remaining: [0; 8],
            sfx_release_pending_mask: 0,
            sfx_ownership_release_overflows: [0; 8],
            sfx_release_overflows_remaining: [0; 8],
            sfx_keyoff_starts_ownership_mask: 0,
            pending_sfx_pitch_changes: std::array::from_fn(|_| Vec::new()),
            pending_sfx_volume_changes: std::array::from_fn(|_| Vec::new()),
            sfx_voice_activation_count: [0; 8],
            secondary_sfx_keyoff_samples_remaining: [0; 8],
            rising_warble_long_pattern: [false; 8],
            rising_warble_retrigger_samples_remaining: [0; 8],
            rising_warble_command_clock: None,
            bank1_id29_voice7_hold_frames: 0,
            frame_warble_pitch_events: Vec::new(),
            frame_warble_volume_events: Vec::new(),
            looping_sfx_voices: [None; 8],
            previous_sfx_clock: None,
            semantic_sfx_pending_steps: std::array::from_fn(|_| Vec::new()),
            semantic_sfx_repeat_steps: [None; 8],
            semantic_pitch_latch_mask: 0,
            engine_automated_sfx_volume_mask: 0,
            semantic_bank1_command_voice: None,
            semantic_bank2_command_voice: None,
            port23_id36_cluster_command: false,
            port23_id36_cluster_active: false,
            port23_id36_voice7_keyoffs: [0; 2],
            port23_id36_voice7_retrigger: 0,
            port23_id36_voice6_final_keyoff: 0,
            port23_id36_voice7_late_keyoffs: [0; 3],
            port23_id36_voice7_late_retrigger: 0,
            port23_id36_voice6_owned: false,
            port23_id36_voice7_owned: false,
            frame_port23_id36_voice7_retrigger: false,
            pending_port3_id15_actions: Vec::new(),
            port3_id15_owned_mask: 0,
            port3_id15_active: false,
            port3_id15_voice3_volume: 0,
            port3_id15_voices: default_port3_id15_voices(),
            pending_bank1_id1_actions: Vec::new(),
            bank1_id1_active: false,
            pending_bank1_id45_actions: Vec::new(),
            bank1_id45_active: false,
            bank1_id45_voice: 0,
            pending_bank1_id95_actions: Vec::new(),
            bank1_id95_active: false,
            bank1_id95_voice: 0,
            pending_bank1_id41_actions: Vec::new(),
            bank1_id41_active: false,
            pending_bank1_id60_actions: Vec::new(),
            bank1_id60_active: false,
            pending_bank2_id28_actions: Vec::new(),
            bank2_id28_active: false,
            pending_bank2_id9_actions: Vec::new(),
            bank2_id9_active: false,
            bank2_id9_voice: 0,
            pending_bank2_id14_actions: Vec::new(),
            bank2_id14_active: false,
            bank2_id14_voice: 0,
            pending_bank2_id11_actions: Vec::new(),
            bank2_id11_active: false,
            bank2_id11_voice: 0,
            engine_receipt_mode: false,
            music_echo_mask: 0,
            music_master_volume: default_music_master_volume(),
            music_master_fade_ticks: 0,
            music_master_fade_add: 0,
            saved_music_master_volume: 0,
            music_tempo: default_music_tempo(),
            music_tempo_accum: 0,
            last_music_control_command: 0,
            dsp_timer_cycles: 0,
            sfx_timer_accum: 0,
            sfx_clock_epoch: 0,
            timer_initialized: false,
            last_stats: ModernAudioSequenceStats::default(),
        }
    }
}

impl ModernAudioSequencer {
    /// Seed the independent modern SFX scheduler clock at an audio-frame
    /// boundary. The accumulator is runtime state, so repeated instances of
    /// one program retain their phase instead of replaying harvested offsets.
    pub fn set_sfx_clock_phase(&mut self, timer_cycles: u8, sfx_timer_accum: u8) {
        self.dsp_timer_cycles = timer_cycles.min(64);
        self.sfx_timer_accum = sfx_timer_accum;
        self.sfx_clock_epoch = self.sfx_clock_epoch.wrapping_add(1);
        self.previous_sfx_clock = Some((self.dsp_timer_cycles, self.sfx_timer_accum));
        self.timer_initialized = true;
    }

    pub fn sfx_clock_checkpoint(&self) -> (u32, u8, u8) {
        (
            self.sfx_clock_epoch,
            self.dsp_timer_cycles,
            self.sfx_timer_accum,
        )
    }

    pub fn synchronize_sfx_clock_checkpoint(
        &mut self,
        epoch: u32,
        timer_cycles: u8,
        sfx_timer_accum: u8,
    ) {
        if self.sfx_clock_epoch == epoch {
            return;
        }
        self.sfx_clock_epoch = epoch;
        self.dsp_timer_cycles = timer_cycles.min(64);
        self.sfx_timer_accum = sfx_timer_accum;
        self.previous_sfx_clock = Some((self.dsp_timer_cycles, self.sfx_timer_accum));
        self.timer_initialized = true;
    }

    /// Compatibility entry point for oracle traces and older callers that only
    /// expose APUI state. The playable runtime uses `sequence_engine_commands`.
    pub fn sequence_route(&mut self, route: AudioRouteState) -> AudioEventFrame {
        let commands = EngineAudioCommandBatch::from_legacy_ports(route.queue.input);
        self.sequence_commands_with_writes(route, commands, &[], true)
    }

    /// Expand gameplay-authored commands without decoding the APUI projection.
    pub fn sequence_engine_commands(
        &mut self,
        route: AudioRouteState,
        commands: EngineAudioCommandBatch,
    ) -> AudioEventFrame {
        self.sequence_commands_with_writes(route, commands, &[], false)
    }

    pub fn sequence_parity_writes(
        &mut self,
        route: AudioRouteState,
        writes: &[DspWriteEvent],
    ) -> AudioEventFrame {
        let commands = EngineAudioCommandBatch::from_legacy_ports(route.queue.input);
        self.sequence_commands_with_writes(route, commands, writes, true)
    }

    fn sequence_commands_with_writes(
        &mut self,
        route: AudioRouteState,
        commands: EngineAudioCommandBatch,
        writes: &[DspWriteEvent],
        include_legacy_music_sfx: bool,
    ) -> AudioEventFrame {
        let port2_id30_command = commands
            .sfx(AudioSfxBank::Effect1)
            .is_some_and(|command| command.legacy_value() == 0x1e);
        self.port23_id36_cluster_command = commands
            .sfx(AudioSfxBank::Effect1)
            .is_some_and(|command| command.legacy_value() == 0x24)
            && commands
                .sfx(AudioSfxBank::Effect2)
                .is_some_and(|command| command.legacy_value() == 0x24);
        let port3_id15_command = commands
            .sfx(AudioSfxBank::Effect2)
            .is_some_and(|command| command.legacy_value() == 0x0f);
        if port3_id15_command {
            let first_voice = if self.sfx_voice_mask == 0 { 1 } else { 2 };
            self.port3_id15_voices = std::array::from_fn(|index| first_voice + index as u8);
        }
        let bank1_id1_command = commands
            .sfx(AudioSfxBank::Effect1)
            .is_some_and(|command| command.legacy_value() == 0x01);
        let bank1_id41_command = commands
            .sfx(AudioSfxBank::Effect1)
            .is_some_and(|command| command.legacy_value() == 0x29);
        let bank1_id29_command = commands
            .sfx(AudioSfxBank::Effect1)
            .is_some_and(|command| command.legacy_value() == 0x1d);
        if bank1_id29_command {
            self.rising_warble_command_clock = self.previous_sfx_clock;
            self.bank1_id29_voice7_hold_frames = 2;
        }
        let bank1_id45_command = commands
            .sfx(AudioSfxBank::Effect1)
            .is_some_and(|command| command.legacy_value() == 0x2d);
        if bank1_id45_command {
            self.bank1_id45_voice = semantic_bank1_allocator_voice(route, commands)
                .or_else(|| semantic_sfx_source_voice(route, 15))
                .unwrap_or(self.bank1_id45_voice.max(6));
        }
        let bank1_id95_command = commands
            .sfx(AudioSfxBank::Effect1)
            .is_some_and(|command| command.legacy_value() == 0x5f);
        if bank1_id95_command {
            self.bank1_id95_voice = semantic_bank1_allocator_voice(route, commands)
                .or_else(|| semantic_sfx_source_voice(route, 20))
                .unwrap_or(self.bank1_id95_voice.max(5));
        }
        let bank1_id60_command = commands
            .sfx(AudioSfxBank::Effect1)
            .is_some_and(|command| command.legacy_value() == 0x3c);
        let bank2_id28_command = commands
            .sfx(AudioSfxBank::Effect2)
            .is_some_and(|command| command.legacy_value() == 0x1c);
        let bank2_id9_command = commands
            .sfx(AudioSfxBank::Effect2)
            .is_some_and(|command| command.legacy_value() == 0x09);
        let bank2_id14_command = commands
            .sfx(AudioSfxBank::Effect2)
            .is_some_and(|command| command.legacy_value() == 0x0e);
        let bank2_id11_command = commands
            .sfx(AudioSfxBank::Effect2)
            .is_some_and(|command| command.legacy_value() == 0x0b);
        if bank2_id11_command {
            self.bank2_id11_voice = (0..=7u8)
                .rev()
                .find(|voice| self.sfx_voice_mask & (1 << voice) == 0)
                .unwrap_or(7);
        }
        self.semantic_bank1_command_voice = semantic_bank1_allocator_voice(route, commands);
        self.semantic_bank2_command_voice = semantic_bank2_allocator_voice(route, commands);
        if let Some(spc) = route.spc {
            self.engine_receipt_mode |= spc.sfx_kon_count != 0
                || spc.sfx_kof_count != 0
                || spc.raw_kof_count != 0
                || spc.raw_pitch_count != 0
                || spc.raw_volume_count != 0
                || spc.raw_envelope_count != 0;
            self.semantic_pitch_latch_mask &= spc.port2_active;
        }
        self.synchronize_sfx_ownership_from_engine(route.spc);
        self.initialize_timer(route);
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(route, writes);
        self.frame_warble_pitch_events.clear();
        self.frame_warble_volume_events.clear();
        self.frame_port23_id36_voice7_retrigger = false;
        frame.sequenced = true;
        let mut stats = ModernAudioSequenceStats::default();
        stats.sfx_voice_mask_start = self.sfx_voice_mask;
        self.release_finished_sfx_ownership();
        self.advance_sfx_release_overflows();
        let mut processed_semantic_keyons = [0u8; 8];
        let mut semantic_pitch_latch_release = [None; 8];
        self.advance_music_keyoffs(&mut frame, &mut stats);
        self.advance_sfx_keyoffs(&mut frame, &mut stats);
        self.advance_rising_warble_retriggers(&mut frame, &mut stats);
        self.advance_port23_id36_cluster(&mut frame, &mut stats);
        self.advance_port3_id15_actions(&mut frame, &mut stats);
        self.advance_bank1_id1_actions(&mut frame, &mut stats);
        self.advance_bank1_id45_actions(&mut frame, &mut stats);
        self.advance_bank1_id95_actions(&mut frame, &mut stats);
        self.advance_bank1_id41_actions(&mut frame, &mut stats);
        self.advance_bank1_id60_actions(&mut frame, &mut stats);
        self.advance_bank2_id28_actions(&mut frame, &mut stats);
        self.advance_bank2_id9_actions(&mut frame, &mut stats);
        self.advance_bank2_id14_actions(&mut frame, &mut stats);
        self.advance_bank2_id11_actions(&mut frame, &mut stats);
        self.advance_secondary_sfx_keyoffs(&mut frame, &mut stats);
        self.advance_sfx_ownership(&mut frame, &mut stats);
        self.advance_sfx_pitch_changes(&mut frame);
        self.advance_sfx_volume_changes(&mut frame);
        self.advance_persistent_sfx_pitch_refreshes(&mut frame);
        self.advance_looping_sfx(&mut frame, &mut stats);
        self.emit_semantic_sfx_echo_changes(route.spc, &mut frame);
        self.emit_port3_allocator_keyoffs(route.spc, &mut frame, &mut stats);
        self.emit_semantic_sfx_keyons(
            route.spc,
            &mut processed_semantic_keyons,
            &mut semantic_pitch_latch_release,
            &mut frame,
            &mut stats,
        );
        self.advance_voice_lifetimes(&mut frame, &mut stats);

        self.sequence_music(
            route.music,
            commands.music(),
            !writes.is_empty(),
            &mut frame,
            &mut stats,
        );
        self.emit_ambient_music_reset(route, commands, &mut frame, &mut stats);
        self.sequence_sfx(
            route.music,
            commands,
            include_legacy_music_sfx,
            route.spc.map_or(0, |spc| spc.is_chan_on),
            route
                .spc
                .map(|spc| (spc.timer_cycles, spc.sfx_timer_accum))
                .or_else(|| Some(self.sfx_clock_after_current_frame())),
            &mut frame,
            &mut stats,
        );
        self.emit_semantic_sfx_keyons(
            route.spc,
            &mut processed_semantic_keyons,
            &mut semantic_pitch_latch_release,
            &mut frame,
            &mut stats,
        );
        self.emit_engine_music_keyons(route.spc, &mut frame, &mut stats);
        self.emit_semantic_sfx_pitch_changes(route.spc, &mut frame);
        self.emit_raw_pitch_changes(route.spc, semantic_pitch_latch_release, &mut frame);
        self.emit_raw_echo_enable_changes(route.spc, &mut frame);
        self.emit_raw_echo_volume_changes(route.spc, &mut frame);
        self.emit_raw_global_changes(route.spc, &mut frame);
        self.emit_semantic_sfx_volume_changes(route.spc, &mut frame);
        self.emit_raw_volume_changes(route.spc, &mut frame);
        self.emit_raw_envelope_changes(route.spc, &mut frame);
        self.emit_raw_music_keyoffs(route.spc, &mut frame, &mut stats);
        self.reconcile_semantic_sfx_keyoffs(route.spc, &mut frame);
        self.synchronize_sfx_ownership_from_engine(route.spc);
        if port2_id30_command {
            let offsets = frame
                .events
                .iter()
                .filter_map(|event| {
                    matches!(event.kind, AudioEventKind::NoteOff { voice: 1 })
                        .then_some(event.sample_offset)
                })
                .collect::<Vec<_>>();
            for offset in offsets {
                mark_frame_note_off_at_as_sfx(&mut frame, 1, offset);
            }
            self.sfx_voice_mask |= 1 << 1;
            self.active_voice_mask |= 1 << 1;
        }
        self.schedule_port2_id30_automation_from_keyons(&mut frame);
        self.reconcile_port23_id36_cluster(&mut frame, &mut stats);
        self.reconcile_port3_id15_command(port3_id15_command, &mut frame, &mut stats);
        self.reconcile_bank1_id1_command(bank1_id1_command, &mut frame, &mut stats);
        self.reconcile_bank1_id45_command(bank1_id45_command, &mut frame, &mut stats);
        self.reconcile_bank1_id95_command(bank1_id95_command, &mut frame, &mut stats);
        self.reconcile_bank1_id41_command(bank1_id41_command, &mut frame, &mut stats);
        self.reconcile_bank1_id60_command(bank1_id60_command, &mut frame, &mut stats);
        self.reconcile_bank2_id28_command(bank2_id28_command, &mut frame, &mut stats);
        self.reconcile_bank2_id9_command(bank2_id9_command, &mut frame, &mut stats);
        self.reconcile_bank2_id14_command(bank2_id14_command, &mut frame, &mut stats);
        self.reconcile_bank2_id11_command(bank2_id11_command, &mut frame, &mut stats);
        self.emit_music_latch_side_effects(&mut frame, &mut stats);
        self.ensure_keyons_have_echo_state(&mut frame);

        for target in 0..8u8 {
            if !self.rising_warble_long_pattern[usize::from(target)] {
                continue;
            }
            let Some(sample_offset) = frame.events.iter().find_map(|event| {
                matches!(
                    event.kind,
                    AudioEventKind::KeyOnVoice {
                        voice,
                        source: 21,
                        ..
                    } if voice == target
                )
                .then_some(event.sample_offset)
            }) else {
                continue;
            };
            let mut displaced_mask = 0u8;
            for event in &frame.events {
                if event.sample_offset == sample_offset {
                    let displaced_voice = match event.kind {
                        AudioEventKind::KeyOnVoice {
                            voice, source: 21, ..
                        }
                        | AudioEventKind::NoteOn {
                            voice,
                            instrument: 21,
                            ..
                        } => Some(voice),
                        _ => None,
                    };
                    if let Some(voice) = displaced_voice.filter(|voice| *voice != target) {
                        displaced_mask |= 1 << voice;
                    }
                }
            }
            frame.events.retain(|event| {
                !(event.sample_offset == sample_offset
                    && audio_event_voice(&event.kind)
                        .is_some_and(|voice| displaced_mask & (1 << voice) != 0))
            });
            for displaced_voice in 0..8u8 {
                if displaced_mask & (1 << displaced_voice) == 0 {
                    continue;
                }
                self.cancel_sfx_schedules(displaced_voice);
                let displaced_voice = usize::from(displaced_voice);
                self.rising_warble_long_pattern[displaced_voice] = false;
                self.rising_warble_retrigger_samples_remaining[displaced_voice] = 0;
                self.secondary_sfx_keyoff_samples_remaining[displaced_voice] = 0;
                self.sfx_voice_activation_count[displaced_voice] = 0;
                self.active_voice_mask &= !(1 << displaced_voice);
            }
        }
        for target in 0..8u8 {
            let has_warble_events = self
                .frame_warble_pitch_events
                .iter()
                .any(|(voice, ..)| *voice == target)
                || self
                    .frame_warble_volume_events
                    .iter()
                    .any(|(voice, ..)| *voice == target);
            if !has_warble_events {
                continue;
            }
            frame.events.retain(|event| {
                !matches!(
                    event.kind,
                    AudioEventKind::SetPitchWord { voice, .. }
                        | AudioEventKind::SetPitchRegisterWord { voice, .. }
                        | AudioEventKind::SetStereoVolume { voice, .. }
                        if voice == target
                )
            });
            for &(voice, sample_offset, pitch_word) in &self.frame_warble_pitch_events {
                if voice == target {
                    push_event_at(
                        &mut frame,
                        sample_offset,
                        AudioEventKind::SetPitchWord { voice, pitch_word },
                    );
                }
            }
            for &(voice, sample_offset, left, right) in &self.frame_warble_volume_events {
                if voice == target {
                    push_event_at(
                        &mut frame,
                        sample_offset,
                        AudioEventKind::SetStereoVolume { voice, left, right },
                    );
                }
            }
        }

        // Retain the DSP register context independently of voice ownership.
        // A later KON may intentionally reuse SRCN/ADSR left by music or a
        // preceding effect instead of issuing another instrument setup.
        let mut latest_source_event = [None::<(i32, u8)>; 8];
        for event in &frame.events {
            let source = match event.kind {
                AudioEventKind::NoteOn {
                    voice, instrument, ..
                } => Some((voice, instrument)),
                AudioEventKind::KeyOnVoice { voice, source, .. } => Some((voice, source)),
                AudioEventKind::VoiceParameter {
                    voice,
                    parameter: VoiceParameterKind::Source,
                    value,
                } => Some((voice, value)),
                _ => None,
            };
            if let Some((voice, source)) = source {
                let slot = &mut latest_source_event[usize::from(voice)];
                if slot.is_none_or(|(offset, _)| event.sample_offset >= offset) {
                    *slot = Some((event.sample_offset, source));
                }
            }
        }
        for (voice, event) in latest_source_event.into_iter().enumerate() {
            if let Some((_, source)) = event {
                self.voice_dsp_source[voice] = source;
            }
        }

        if self.engine_receipt_mode {
            if let Some(spc) = route.spc {
                self.sfx_voice_mask = spc.is_chan_on;
                self.active_voice_mask |= spc.is_chan_on;
            }
        }
        if let Some(voice) = self.semantic_bank1_command_voice {
            self.sfx_voice_mask |= 1 << voice;
            self.active_voice_mask |= 1 << voice;
        }
        if let Some(voice) = self.semantic_bank2_command_voice {
            self.sfx_voice_mask |= 1 << voice;
            self.active_voice_mask |= 1 << voice;
        }
        for voice in 0..8 {
            if self.sfx_voice_program[voice] == 0x011d
                && self.rising_warble_long_pattern[voice]
                && self.secondary_sfx_keyoff_samples_remaining[voice] == 0
                && self.rising_warble_retrigger_samples_remaining[voice] == 0
                && (!self.pending_sfx_pitch_changes[voice].is_empty()
                    || self
                        .frame_warble_pitch_events
                        .iter()
                        .any(|(event_voice, ..)| usize::from(*event_voice) == voice))
            {
                self.sfx_voice_mask |= 1 << voice;
                self.active_voice_mask |= 1 << voice;
            }
        }
        if self.bank1_id29_voice7_hold_frames != 0 {
            self.sfx_voice_mask |= 1 << 7;
            self.bank1_id29_voice7_hold_frames -= 1;
        }

        stats.active_voice_mask = self.active_voice_mask;
        stats.sfx_voice_mask = self.sfx_voice_mask;
        self.last_stats = stats;
        self.advance_dsp_timer();
        if let Some(spc) = route.spc {
            self.dsp_timer_cycles = spc.timer_cycles.min(64);
            self.sfx_timer_accum = spc.sfx_timer_accum;
        }
        self.previous_sfx_clock = Some((self.dsp_timer_cycles, self.sfx_timer_accum));
        frame
    }

    pub fn last_stats(&self) -> ModernAudioSequenceStats {
        self.last_stats
    }

    pub fn sfx_clock_phase(&self) -> (u8, u8) {
        (self.dsp_timer_cycles, self.sfx_timer_accum)
    }

    fn initialize_timer(&mut self, route: AudioRouteState) {
        if self.timer_initialized {
            return;
        }
        if let Some(spc) = route.spc {
            self.dsp_timer_cycles = spc.timer_cycles.wrapping_sub((534 & 0x3f) as u8) & 0x3f;
            let tick_count = sfx_timer_tick_count(self.dsp_timer_cycles);
            self.sfx_timer_accum = spc
                .sfx_timer_accum
                .wrapping_sub(tick_count.wrapping_mul(0x38));
        }
        self.previous_sfx_clock = Some((self.dsp_timer_cycles, self.sfx_timer_accum));
        self.timer_initialized = true;
    }

    fn advance_dsp_timer(&mut self) {
        let tick_count = sfx_timer_tick_count(self.dsp_timer_cycles);
        self.sfx_timer_accum = self
            .sfx_timer_accum
            .wrapping_add(tick_count.wrapping_mul(0x38));
        self.dsp_timer_cycles = sfx_timer_cycles_after_frame(self.dsp_timer_cycles);
    }

    fn sfx_clock_after_current_frame(&self) -> (u8, u8) {
        let tick_count = sfx_timer_tick_count(self.dsp_timer_cycles);
        (
            sfx_timer_cycles_after_frame(self.dsp_timer_cycles),
            self.sfx_timer_accum
                .wrapping_add(tick_count.wrapping_mul(0x38)),
        )
    }

    fn sfx_sample_offset(&self, exact: Option<ExactSfxDspStep>) -> i32 {
        let Some(exact) = exact else {
            return 0;
        };
        let first_boundary = sfx_first_boundary(self.dsp_timer_cycles);
        i32::from(first_boundary)
            + i32::from(exact.scheduler_tick_index) * 64
            + ENGINE_AUDIO_COMMAND_PHASE_SAMPLES
    }

    fn sequence_music(
        &mut self,
        music: MusicControlState,
        command: AudioMusicCommand,
        authoritative_dsp_writes: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let command_track = command.legacy_value();
        self.advance_music_master_fade(frame, authoritative_dsp_writes);
        let track = first_nonzero([
            command_track,
            music.apui00,
            music.last_music_control,
            music.queued_music_control,
            music.music_control,
        ]);
        if track == self.last_music_track {
            self.advance_music_sequence(track, frame, stats);
            return;
        }

        if track == 0 || track == 0xf0 {
            if self.last_music_track != 0 {
                push_event(frame, AudioEventKind::StopMusic);
                self.release_music_voices(frame, stats);
                stats.music_commands += 1;
            }
            self.last_music_track = 0;
            self.music_frame_position = 0;
            self.music_sample_position = 0;
            return;
        }

        if track >= 0xf1 {
            let is_new_command =
                command_track == track && command_track != self.last_music_control_command;
            match track {
                0xf1 if is_new_command => {
                    self.music_master_fade_ticks = 0x80;
                    self.music_master_fade_add = -((self.music_master_volume >> 8) as i16 * 2);
                    stats.music_commands += 1;
                }
                0xf2 if is_new_command && self.saved_music_master_volume == 0 => {
                    self.saved_music_master_volume = (self.music_master_volume >> 8) as u8;
                    self.music_master_volume = 0x7000;
                    self.emit_music_master_volume(frame, 0x70, 0, authoritative_dsp_writes);
                    stats.music_commands += 1;
                }
                0xf3 if is_new_command && self.saved_music_master_volume != 0 => {
                    let value = self.saved_music_master_volume;
                    self.saved_music_master_volume = 0;
                    self.music_master_volume = u16::from(value) << 8;
                    self.emit_music_master_volume(frame, value, 0, authoritative_dsp_writes);
                    stats.music_commands += 1;
                }
                0xf1..=0xf3 => {}
                _ => stats.ignored_commands += 1,
            }
            self.last_music_control_command = command_track;
            if self.last_music_track != 0 && self.last_music_track < 0xf0 {
                self.advance_music_sequence(self.last_music_track, frame, stats);
            }
            return;
        }

        push_event(frame, AudioEventKind::PlayMusic { track });
        push_event(
            frame,
            AudioEventKind::SetTempo {
                value: tempo_for_track(track),
            },
        );
        stats.music_commands += 1;
        self.last_music_control_command = command_track;
        self.last_music_track = track;
        self.music_tempo = driver_tempo_for_track(track);
        self.music_master_fade_ticks = 0;
        if self.music_master_volume != default_music_master_volume() {
            self.music_master_volume = default_music_master_volume();
            self.emit_music_master_volume(
                frame,
                (default_music_master_volume() >> 8) as u8,
                0,
                authoritative_dsp_writes,
            );
        }
        if packed_track(track).is_some() {
            self.music_frame_position = 0;
            self.music_sample_position = 0;
            self.emit_music_window(track, frame, stats);
        } else {
            self.emit_music_note(
                frame,
                track,
                ModernMusicNote {
                    voice: 0,
                    pitch: pitch_for_code(track),
                    instrument: instrument_for_code(track),
                    volume: 88,
                    pan: 0,
                    start_frame: 0,
                    duration_frames: 0,
                    dsp_pitch: 0,
                    sample_offset: 0,
                    volume_left: 0,
                    volume_right: 0,
                    adsr1: 0,
                    adsr2: 0,
                    gain: 0,
                    echo_send: false,
                    keyoff_sample_offset: 0,
                },
                stats,
            );
        }
    }

    fn advance_music_master_fade(
        &mut self,
        frame: &mut AudioEventFrame,
        authoritative_dsp_writes: bool,
    ) {
        let first_boundary = i32::from(sfx_first_boundary(self.dsp_timer_cycles));
        let tick_count = sfx_timer_tick_count(self.dsp_timer_cycles);
        for tick in 0..tick_count {
            let total = u16::from(self.music_tempo_accum) + u16::from(self.music_tempo);
            self.music_tempo_accum = total as u8;
            if total < 0x100 || self.music_master_fade_ticks == 0 {
                continue;
            }
            self.music_master_fade_ticks -= 1;
            self.music_master_volume = if self.music_master_fade_ticks == 0 {
                0
            } else {
                self.music_master_volume
                    .wrapping_add(self.music_master_fade_add as u16)
            };
            self.emit_music_master_volume(
                frame,
                (self.music_master_volume >> 8) as u8,
                first_boundary + i32::from(tick) * 64,
                authoritative_dsp_writes,
            );
        }
    }

    fn emit_music_master_volume(
        &self,
        frame: &mut AudioEventFrame,
        value: u8,
        sample_offset: i32,
        authoritative_dsp_writes: bool,
    ) {
        if !self.engine_receipt_mode && !authoritative_dsp_writes {
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetMusicVolume { value },
            );
        }
    }

    fn advance_music_sequence(
        &mut self,
        track: u8,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if self.music_sample_position == 0 && self.music_frame_position != 0 {
            // Older audio snapshots predate the sample-domain cursor.
            self.music_sample_position =
                u64::from(self.music_frame_position) * MUSIC_NATIVE_FRAME_SAMPLES;
        } else {
            self.music_sample_position = self
                .music_sample_position
                .saturating_add(MUSIC_NATIVE_FRAME_SAMPLES);
        }
        self.music_frame_position = self
            .music_source_sample_at(self.music_sample_position)
            .checked_div(MUSIC_NATIVE_FRAME_SAMPLES)
            .unwrap_or_default()
            .min(u64::from(u16::MAX)) as u16;
        self.emit_music_window(track, frame, stats);
    }

    fn music_source_sample_at(&self, elapsed_sample: u64) -> u64 {
        let Some(track) = packed_track(self.last_music_track) else {
            return elapsed_sample;
        };
        let Some(loop_range) = track.loop_range() else {
            return elapsed_sample;
        };
        let loop_start = u64::from(loop_range.start);
        let loop_end = u64::from(loop_range.end);
        if elapsed_sample < loop_end {
            elapsed_sample
        } else {
            loop_start + (elapsed_sample - loop_start) % (loop_end - loop_start)
        }
    }

    fn music_window_segments(&self) -> Vec<(u64, u64, i32)> {
        let mut source = self.music_source_sample_at(self.music_sample_position);
        let mut remaining = MUSIC_NATIVE_FRAME_SAMPLES;
        let mut output_offset = 0i32;
        let mut segments = Vec::with_capacity(2);
        let loop_range = packed_track(self.last_music_track).and_then(|track| track.loop_range());
        while remaining != 0 {
            let available = loop_range.as_ref().map_or(remaining, |range| {
                if source < u64::from(range.end) {
                    u64::from(range.end) - source
                } else {
                    remaining
                }
            });
            let length = remaining.min(available);
            segments.push((source, length, output_offset));
            source += length;
            remaining -= length;
            output_offset += length as i32;
            if remaining != 0 {
                source = loop_range
                    .as_ref()
                    .map_or(source, |range| u64::from(range.start));
            }
        }
        segments
    }

    fn music_global_events_in_current_window(
        &self,
    ) -> Vec<(i32, crate::modern_music_globals::ModernMusicGlobalEvent)> {
        let mut result = Vec::new();
        for (source_start, length, output_start) in self.music_window_segments() {
            let source_end = source_start + length;
            let first_frame = source_start / MUSIC_NATIVE_FRAME_SAMPLES;
            let last_frame = (source_end.saturating_sub(1)) / MUSIC_NATIVE_FRAME_SAMPLES;
            for source_frame in first_frame..=last_frame {
                let Ok(source_frame) = u16::try_from(source_frame) else {
                    continue;
                };
                for event in music_global_events_at(self.last_music_track, source_frame) {
                    let absolute = u64::from(source_frame) * MUSIC_NATIVE_FRAME_SAMPLES
                        + u64::from(event.sample_offset);
                    if absolute >= source_start && absolute < source_end {
                        result.push((output_start + (absolute - source_start) as i32, event));
                    }
                }
            }
        }
        result.sort_by_key(|(sample_offset, _)| *sample_offset);
        result
    }

    fn music_notes_in_current_window(&self, track: u8) -> Vec<(i32, ModernMusicNote)> {
        let Some(track_data) = packed_track(track) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for (source_start, length, output_start) in self.music_window_segments() {
            let source_end = source_start + length;
            let first_source_frame = source_start / MUSIC_NATIVE_FRAME_SAMPLES;
            let last_source_frame = source_end.saturating_sub(1) / MUSIC_NATIVE_FRAME_SAMPLES;
            let lead = u64::from(track_data.lead_in_frames);
            if last_source_frame < lead {
                continue;
            }
            let first_note_frame = first_source_frame
                .saturating_sub(lead)
                .min(u64::from(u16::MAX));
            let last_note_frame = (last_source_frame - lead).min(u64::from(u16::MAX));
            for note in
                notes_starting_in(track_data, first_note_frame as u16, last_note_frame as u16)
            {
                let absolute = u64::from(note.start_frame + track_data.lead_in_frames)
                    * MUSIC_NATIVE_FRAME_SAMPLES
                    + u64::from(note.sample_offset);
                if absolute >= source_start && absolute < source_start + length {
                    result.push((output_start + (absolute - source_start) as i32, note));
                }
            }
        }
        result.sort_by_key(|(sample_offset, _)| *sample_offset);
        result
    }

    fn emit_music_window(
        &mut self,
        track: u8,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if !self.engine_receipt_mode {
            let mut events = self.music_global_events_in_current_window();
            events.sort_by_key(|(sample_offset, event)| {
                (
                    *sample_offset,
                    event.register,
                    if event.register == 0x4d {
                        std::cmp::Reverse((event.value.count_ones(), event.value))
                    } else {
                        std::cmp::Reverse((0, 0))
                    },
                )
            });
            for (sample_offset, event) in events {
                if matches!(event.register, 0x4c | 0x5c) {
                    continue;
                }
                self.emit_music_global_event(frame, sample_offset, event.register, event.value);
            }
        }
        for (sample_offset, mut note) in self.music_notes_in_current_window(track) {
            let keyoff_delta = (u64::from(note.duration_frames) * MUSIC_NATIVE_FRAME_SAMPLES
                + u64::from(note.keyoff_sample_offset))
            .saturating_sub(u64::from(note.sample_offset));
            note.sample_offset = sample_offset as u16;
            if note.duration_frames != 0 {
                let adjusted_keyoff = u64::from(note.sample_offset) + keyoff_delta;
                note.duration_frames = (adjusted_keyoff / MUSIC_NATIVE_FRAME_SAMPLES) as u16;
                note.keyoff_sample_offset = (adjusted_keyoff % MUSIC_NATIVE_FRAME_SAMPLES) as u16;
            } else if note.keyoff_sample_offset > 0 {
                note.keyoff_sample_offset = (u64::from(note.sample_offset) + keyoff_delta) as u16;
            }
            self.emit_music_note(frame, track, note, stats);
        }
    }

    fn emit_music_global_event(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        register: u8,
        value: u8,
    ) {
        if register == 0x4d {
            self.music_echo_mask = value;
        }
        let kind = match register & 0x0f {
            0x00 => Some(VoiceParameterKind::VolumeLeft),
            0x01 => Some(VoiceParameterKind::VolumeRight),
            0x02 => Some(VoiceParameterKind::PitchLow),
            0x03 => Some(VoiceParameterKind::PitchHigh),
            0x04 => Some(VoiceParameterKind::Source),
            0x05 => Some(VoiceParameterKind::Adsr1),
            0x06 => Some(VoiceParameterKind::Adsr2),
            0x07 => Some(VoiceParameterKind::Gain),
            _ => None,
        };
        push_event_at(
            frame,
            sample_offset,
            if let Some(parameter) = kind {
                AudioEventKind::VoiceParameter {
                    voice: register >> 4,
                    parameter,
                    value,
                }
            } else {
                AudioEventKind::GlobalParameter { register, value }
            },
        );
    }

    fn emit_music_globals_at_position(
        &mut self,
        track: u8,
        music_frame_position: u16,
        frame: &mut AudioEventFrame,
    ) {
        if self.engine_receipt_mode {
            // Route-derived music register catalogs are a standalone fallback.
            // Once the engine is providing timed DSP receipts, replaying these
            // observations can mutate active envelopes on frames where the
            // engine made no register write at all.
            return;
        }
        let mut events = music_global_events_at(track, music_frame_position).collect::<Vec<_>>();
        // The catalog was authored from coalesced EON snapshots. That preserves
        // every mask written at a sample, but not the write order. The driver
        // removes echo-enabled voices one at a time, so reconstruct the chain
        // from the widest transient mask to the narrowest final mask. The last
        // write is observable DSP state and must match exactly.
        let mut index = 0;
        while index < events.len() {
            if events[index].register != 0x4d {
                index += 1;
                continue;
            }
            let sample_offset = events[index].sample_offset;
            let mut end = index + 1;
            while end < events.len()
                && events[end].register == 0x4d
                && events[end].sample_offset == sample_offset
            {
                end += 1;
            }
            events[index..end].sort_by_key(|event| {
                (
                    std::cmp::Reverse(event.value.count_ones()),
                    std::cmp::Reverse(event.value),
                )
            });
            index = end;
        }
        for event in events {
            if matches!(event.register, 0x4c | 0x5c) {
                continue;
            }
            self.emit_music_global_event(
                frame,
                i32::from(event.sample_offset),
                event.register,
                event.value,
            );
        }
    }

    fn emit_music_latch_side_effects(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if self.engine_receipt_mode || self.last_music_track == 0 {
            return;
        }
        for (sample_offset, event) in self.music_global_events_in_current_window() {
            if event.register != 0x4c {
                continue;
            }
            for voice in 0..8u8 {
                if event.value & (1 << voice) == 0
                    || frame.events.iter().any(|existing| {
                        existing.sample_offset == sample_offset
                            && matches!(
                                existing.kind,
                                AudioEventKind::NoteOn {
                                    voice: event_voice,
                                    ..
                                } | AudioEventKind::KeyOnVoice {
                                    voice: event_voice,
                                    ..
                                } if event_voice == voice
                            )
                    })
                {
                    continue;
                }
                let residual_note =
                    self.music_note_at_current_position(voice, sample_offset as u16);
                if let Some(note) = residual_note {
                    self.emit_music_note_unchecked(frame, self.last_music_track, note, stats);
                } else {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::RetriggerVoice { voice },
                    );
                }
            }
        }
    }

    fn music_note_at_current_position(
        &self,
        voice: u8,
        sample_offset: u16,
    ) -> Option<ModernMusicNote> {
        self.music_notes_in_current_window(self.last_music_track)
            .into_iter()
            .find_map(|(adjusted_offset, mut note)| {
                (note.voice == voice && adjusted_offset == i32::from(sample_offset)).then(|| {
                    note.sample_offset = sample_offset;
                    note
                })
            })
    }

    fn ensure_keyons_have_echo_state(&self, frame: &mut AudioEventFrame) {
        if self.engine_receipt_mode {
            return;
        }
        let keyons = frame
            .events
            .iter()
            .filter_map(|event| match event.kind {
                AudioEventKind::NoteOn { voice, .. } | AudioEventKind::KeyOnVoice { voice, .. } => {
                    Some((voice, event.sample_offset))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        for (voice, sample_offset) in keyons {
            if frame.events.iter().any(|event| {
                event.sample_offset == sample_offset
                    && matches!(
                        event.kind,
                        AudioEventKind::SetEchoSend {
                            voice: echo_voice,
                            ..
                        } if echo_voice == voice
                    )
            }) {
                continue;
            }
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetEchoSend {
                    voice,
                    enabled: self.music_echo_mask & (1 << voice) != 0,
                },
            );
        }
    }

    fn emit_music_notes_at_position(
        &mut self,
        track: u8,
        music_frame_position: u16,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(track_data) = packed_track(track) else {
            return;
        };
        for note in track_data
            .notes
            .chunks_exact(PACKED_NOTE_BYTES)
            .filter_map(decode_note)
            .filter(|note| note.start_frame + track_data.lead_in_frames == music_frame_position)
        {
            self.emit_music_note(frame, track, note, stats);
        }
    }

    fn emit_music_note(
        &mut self,
        frame: &mut AudioEventFrame,
        track: u8,
        note: ModernMusicNote,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if self.sfx_voice_mask & (1 << note.voice) != 0 {
            // The original sound driver temporarily owns music voices while an
            // SFX program is active. Keep advancing the music timeline, but do
            // not let an overlapping music note steal the DSP voice before the
            // SFX key-off boundary.
            stats.note_events += 1;
            return;
        }
        if self.engine_receipt_mode {
            self.mark_music_voice_active(note.voice);
            stats.note_events += 1;
            return;
        }
        self.emit_music_note_unchecked(frame, track, note, stats);
    }

    fn emit_music_note_unchecked(
        &mut self,
        frame: &mut AudioEventFrame,
        track: u8,
        note: ModernMusicNote,
        stats: &mut ModernAudioSequenceStats,
    ) {
        push_event(
            frame,
            AudioEventKind::SetNoteOrigin {
                voice: note.voice,
                origin: AudioNoteOrigin::Music,
            },
        );
        push_event(
            frame,
            AudioEventKind::SetEnvelope {
                voice: note.voice,
                attack: 2,
                decay: 4 + (track & 3),
                sustain: 10,
                release: 4,
            },
        );
        if note.dsp_pitch != 0 {
            let sample_offset = i32::from(note.sample_offset);
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice: note.voice,
                    pitch_word: note.dsp_pitch,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetStereoVolume {
                    voice: note.voice,
                    left: note.volume_left,
                    right: note.volume_right,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetDspEnvelope {
                    voice: note.voice,
                    adsr1: note.adsr1,
                    adsr2: note.adsr2,
                    gain: note.gain,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetEchoSend {
                    voice: note.voice,
                    enabled: note.echo_send,
                },
            );
        }
        push_event_at(
            frame,
            i32::from(note.sample_offset),
            AudioEventKind::NoteOn {
                voice: note.voice,
                pitch: note.pitch,
                instrument: note.instrument,
                volume: note.volume,
            },
        );
        push_event(
            frame,
            AudioEventKind::SetPan {
                voice: note.voice,
                pan: note.pan,
            },
        );
        if note.duration_frames != 0 && note.dsp_pitch == 0 {
            push_event(
                frame,
                AudioEventKind::SetDuration {
                    voice: note.voice,
                    frames: note.duration_frames,
                },
            );
        }
        self.mark_music_voice_active(note.voice);
        if note.dsp_pitch != 0
            && note.duration_frames == 0
            && note.keyoff_sample_offset > note.sample_offset
        {
            push_event_at(
                frame,
                i32::from(note.keyoff_sample_offset),
                AudioEventKind::NoteOff { voice: note.voice },
            );
            self.mark_music_voice_inactive(note.voice);
            stats.note_events += 1;
        } else if note.dsp_pitch != 0 && note.duration_frames != 0 {
            let voice = usize::from(note.voice);
            self.music_keyoff_frames_remaining[voice] = note.duration_frames;
            self.music_keyoff_sample_offset[voice] = note.keyoff_sample_offset;
        }
        stats.note_events += 1;
        stats.envelope_events += 1;
    }

    fn sequence_sfx(
        &mut self,
        music: MusicControlState,
        commands: EngineAudioCommandBatch,
        include_legacy_music_sfx: bool,
        sfx_voice_mask: u8,
        sfx_clock: Option<(u8, u8)>,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let engine_sfx = AudioSfxBank::ALL.map(|bank| {
            commands
                .sfx(bank)
                .map_or(0, |command| command.legacy_value())
        });
        let legacy_sfx = if include_legacy_music_sfx {
            [
                music.sound_effect_ambient,
                music.sound_effect_1,
                music.sound_effect_2,
            ]
        } else {
            [0; 3]
        };
        let candidates = [
            engine_sfx[0],
            engine_sfx[1],
            engine_sfx[2],
            legacy_sfx[0],
            legacy_sfx[1],
            legacy_sfx[2],
            // APUI00 is music port 0 and is consumed by sequence_music above.
            // Keep the seventh slot zeroed for checkpoint compatibility.
            0,
        ];

        for (slot, code) in candidates.into_iter().enumerate() {
            if code == self.last_sfx[slot] {
                continue;
            }
            let voice = (slot + 1).min(7) as u8;
            if code == 0 {
                // A command slot is not a DSP voice. Exact programs own their
                // actual voice lifetimes, and fallback notes carry bounded
                // durations. Mapping a cleared port back to `slot + 1` can key
                // off an unrelated music channel.
                self.last_sfx[slot] = 0;
                continue;
            }

            push_event(
                frame,
                AudioEventKind::PlaySfx {
                    bank: slot as u8,
                    id: code,
                },
            );

            if slot == 0 && code == 0x05 && self.sfx_voice_mask & 0xc0 != 0 {
                let stop_mask = self.sfx_voice_mask & 0xc0;
                if let Some(clock) = self.previous_sfx_clock {
                    let stop_offset = sfx_overflow_positions_after(clock, -1, 2)[1] as i32;
                    for voice in 6..8u8 {
                        if stop_mask & (1 << voice) == 0 {
                            continue;
                        }
                        push_event_at(
                            frame,
                            stop_offset,
                            AudioEventKind::SetNoteOrigin {
                                voice,
                                origin: AudioNoteOrigin::Sfx,
                            },
                        );
                        push_event_at(frame, stop_offset, AudioEventKind::NoteOff { voice });
                        let voice_index = usize::from(voice);
                        self.persistent_sfx_pitch_words[voice_index] = 0;
                        self.pending_sfx_pitch_changes[voice_index].clear();
                        self.pending_sfx_volume_changes[voice_index].clear();
                        self.looping_sfx_voices[voice_index] = None;
                        self.semantic_sfx_pending_steps[voice_index].clear();
                        self.semantic_sfx_repeat_steps[voice_index] = None;
                        self.sfx_release_overflows_remaining[usize::from(voice)] = 3;
                        stats.note_events += 1;
                    }
                }
                stats.known_sfx_commands += 1;
                record_sfx_program(
                    &mut stats.known_sfx_programs,
                    &mut stats.known_sfx_program_count,
                    slot as u8,
                    code,
                );
                stats.sfx_commands += 1;
                self.last_sfx[slot] = code;
                continue;
            }
            if slot == 0 && code == 0x05 {
                for voice in 0..8 {
                    self.cancel_sfx_schedules(voice);
                }
            }
            if let Some(program) = lookup_sfx_program_for_context(
                slot as u8,
                code,
                ModernSfxRuntimeContext {
                    source_slot: slot as u8,
                    active_voice_mask: self.active_voice_mask & !sfx_voice_mask,
                },
            ) {
                if !(program.bank == 1 && matches!(program.id, 0x2d | 0x5f)) {
                    self.expand_sfx_program(frame, program, sfx_clock, stats);
                }
                stats.known_sfx_commands += 1;
                record_sfx_program(
                    &mut stats.known_sfx_programs,
                    &mut stats.known_sfx_program_count,
                    slot as u8,
                    code,
                );
                stats.program_hash =
                    fold_program_hash(stats.program_hash, sfx_program_hash(program));
            } else {
                self.expand_fallback_sfx(frame, voice, code, slot as u8, stats);
                stats.unknown_sfx_commands += 1;
                record_sfx_program(
                    &mut stats.unknown_sfx_programs,
                    &mut stats.unknown_sfx_program_count,
                    slot as u8,
                    code,
                );
                stats.fallback_sfx_commands += 1;
            }
            stats.sfx_commands += 1;
            self.last_sfx[slot] = code;
        }
    }

    fn emit_ambient_music_reset(
        &mut self,
        route: AudioRouteState,
        commands: EngineAudioCommandBatch,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let ambient_command = commands
            .sfx(AudioSfxBank::Ambient)
            .map_or(0, |command| command.legacy_value());
        let should_reset = ambient_command == 0x03
            && route
                .spc
                .is_some_and(|spc| spc.spc_out[0] == 0 && spc.block_count == 0xff);
        if !should_reset {
            return;
        }
        let restore_offset = route.spc.map_or(0, |spc| {
            if spc.echo_enable_count != 0 {
                spc.echo_enable_offsets[0]
            } else {
                u16::from(sfx_first_boundary(spc.timer_cycles))
            }
        });
        push_event_at(frame, 0, AudioEventKind::ResetEchoVolume { restore_offset });
        let active_voices = self.active_voice_mask | route.spc.map_or(0, |spc| spc.is_chan_on);
        for voice in 0..8 {
            if active_voices & (1 << voice) == 0 {
                continue;
            }
            push_event_at(frame, 0, AudioEventKind::NoteOff { voice: voice as u8 });
            self.active_voice_mask &= !(1 << voice);
            self.music_voice_mask &= !(1 << voice);
            self.music_keyoff_frames_remaining[voice] = 0;
            stats.note_events += 1;
        }
    }

    fn expand_sfx_program(
        &mut self,
        frame: &mut AudioEventFrame,
        program: &ModernSfxProgram,
        sfx_clock: Option<(u8, u8)>,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if self.engine_receipt_mode {
            // Catalog voices are harvested routing observations.
            // Runtime setup/KON/KOF and parameter receipts contain the
            // authoritative allocated voice, envelope, pitch, volume, echo,
            // and timing, so these catalogs remain metadata-only at runtime.
            return;
        }
        if program.bank == 2 && program.id == 0x1b && program.variant_hash == 0xa44764fc {
            for voice in 0..8 {
                let has_warble_repeat = self.semantic_sfx_repeat_steps[voice]
                    .and_then(|repeat| repeat.exact)
                    .is_some_and(|exact| {
                        exact.bank == 1 && exact.id == 0x1d && exact.variant_hash == 0xf31c8f91
                    });
                if has_warble_repeat {
                    self.rising_warble_long_pattern[voice] = true;
                }
            }
        }
        let mut interrupted_voices = 0u8;
        let mut voice_timeline_end = [0u16; 8];
        // Both SFX command ports use the engine's descending single-voice
        // allocator. Catalog voices describe the capture that produced the
        // asset; they are not stable when another effect overlaps it.
        let single_catalog_voice = program
            .steps
            .first()
            .is_some_and(|first| program.steps.iter().all(|step| step.voice == first.voice));
        let allocated_voice = (matches!(program.bank, 1 | 2)
            && program.context.voice_mask.count_ones() == 1
            && single_catalog_voice)
            .then(|| {
                if self.port23_id36_cluster_command && program.id == 0x24 {
                    return if program.bank == 1 { 7 } else { 5 };
                }
                let program_key = (u16::from(program.bank) << 8) | u16::from(program.id);
                let long_pattern_voice = (program.bank == 1 && program.id == 0x1d)
                    .then(|| {
                        self.rising_warble_long_pattern
                            .iter()
                            .enumerate()
                            .position(|(voice, active)| {
                                *active
                                    && (self.sfx_voice_mask & (1 << voice) != 0
                                        || self.rising_warble_retrigger_samples_remaining[voice]
                                            != 0
                                        || self.secondary_sfx_keyoff_samples_remaining[voice] != 0
                                        || !self.pending_sfx_pitch_changes[voice].is_empty())
                            })
                            .map(|voice| voice as u8)
                    })
                    .flatten();
                long_pattern_voice
                    .or_else(|| {
                        (0..=7)
                            .find(|voice| {
                                self.sfx_voice_mask & (1 << voice) != 0
                                    && self.sfx_voice_program[usize::from(*voice)] == program_key
                            })
                            .or_else(|| {
                                (0..=7)
                                    .rev()
                                    .find(|voice| self.sfx_voice_mask & (1 << voice) == 0)
                            })
                    })
                    .unwrap_or_else(|| program.context.voice_mask.trailing_zeros() as u8)
            });
        for (step_index, catalog_step) in program.steps.iter().enumerate() {
            if program.bank == 1
                && program.id == 0x1d
                && program.variant_hash == 0xf31c8f91
                && step_index != 0
            {
                continue;
            }
            if program.bank == 2
                && program.id == 0x13
                && program.variant_hash == 0x8212a1e6
                && step_index != 0
            {
                continue;
            }
            let mut step = *catalog_step;
            if let Some(voice) = allocated_voice {
                step.voice = voice;
            }
            if program.bank == 1 && matches!(program.id, 0x1c | 0x5e) {
                step.voice = self.semantic_bank1_command_voice.unwrap_or(step.voice);
            }
            if step.voice >= 8 {
                continue;
            }
            step.echo = catalog_echo_send(step.echo, step.voice, self.music_echo_mask);
            let voice_bit = 1 << step.voice;
            let voice_was_sfx_owned = self.sfx_voice_mask & voice_bit != 0;
            let first_for_voice = interrupted_voices & voice_bit == 0;
            if first_for_voice
                && program.bank == 1
                && program.id == 0x1d
                && program.variant_hash == 0xf31c8f91
                && !self.rising_warble_long_pattern[usize::from(step.voice)]
            {
                let voice = usize::from(step.voice);
                self.sfx_voice_activation_count[voice] = 0;
                self.rising_warble_long_pattern[voice] = false;
            }
            let mut interrupted_pitch_changes = None;
            if first_for_voice {
                if !self.engine_receipt_mode
                    || !uses_semantic_sfx_keyons(program)
                    || (program.bank == 0
                        && program.id == 0x01
                        && program.variant_hash == 0x6f23aa01)
                {
                    if program.bank == 2 && program.id == 0x0c {
                        interrupted_pitch_changes = Some(std::mem::take(
                            &mut self.pending_sfx_pitch_changes[usize::from(step.voice)],
                        ));
                    }
                    self.interrupt_voice(step.voice);
                    if let Some(changes) = interrupted_pitch_changes.as_ref() {
                        self.pending_sfx_pitch_changes[usize::from(step.voice)] = changes.clone();
                    }
                }
                interrupted_voices |= voice_bit;
            }
            if program.bank == 1 && program.id == 0x5e {
                self.engine_automated_sfx_volume_mask |= voice_bit;
            }
            if program.bank == 2 && program.id == 0x1f {
                self.engine_automated_sfx_volume_mask |= voice_bit;
            }
            if program.bank == 1 && program.id == 0x1e {
                self.engine_automated_sfx_volume_mask |= voice_bit;
            }
            if program.bank == 0 && program.id == 0x05 && program.variant_hash == 0x5c065005 {
                self.engine_automated_sfx_volume_mask |= voice_bit;
            }
            let voice = usize::from(step.voice);
            let exact_shape = if program.bank == 1 && matches!(program.id, 0x1c | 0x5e) {
                step
            } else {
                *catalog_step
            };
            let mut exact = exact_sfx_dsp_step(
                program.bank,
                program.id,
                program.variant_hash,
                step_index,
                exact_shape,
            );
            if let Some(exact) = exact.as_mut() {
                apply_engine_sfx_timing(program, step_index, exact);
            }
            if program.bank == 2
                && program.id == 0x4f
                && program.variant_hash == 0xede5b411
                && matches!(step_index, 0 | 4 | 6 | 10 | 14)
            {
                if let Some(exact) = exact.as_mut() {
                    exact.interrupt_voice = true;
                }
            }
            if program.bank == 1 && program.id == 0x20 && step_index == 0 {
                if let Some(exact) = exact.as_mut() {
                    exact.interrupt_voice = true;
                }
            }
            if program.bank == 2
                && program.id == 0x1b
                && program.variant_hash == 0xa44764fc
                && matches!(step_index, 0 | 1)
            {
                if let Some(exact) = exact.as_mut() {
                    exact.interrupt_voice = true;
                }
            }
            if program.bank == 1
                && program.id == 0x36
                && program.variant_hash == 0x102e5506
                && step_index == 0
                && self.voice_dsp_source[voice] == 0
            {
                // This bytecode note deliberately omits E0 instrument setup;
                // KON therefore reuses the voice's current DSP register set.
                step.instrument = 0;
                if let Some(exact) = exact.as_mut() {
                    exact.instrument = 0;
                    exact.dsp_pitch = 2995;
                    exact.volume = 0;
                    exact.volume_left = 0;
                    exact.volume_right = 0;
                    exact.adsr1 = 255;
                    exact.adsr2 = 224;
                    exact.gain = 184;
                }
            }
            if program.bank == 0 && program.id == 0x01 && program.variant_hash == 0x6f23aa01 {
                if let Some(exact) = exact.as_mut() {
                    exact.interrupt_voice = true;
                }
            }
            if program.bank == 1
                && program.id == 0x36
                && program.variant_hash == 0x102e5506
                && step_index == 0
            {
                if let Some(exact) = exact.as_mut() {
                    exact.interrupt_voice = true;
                }
            }
            if program.bank == 1
                && program.id == 0x1d
                && program.variant_hash == 0xf31c8f91
                && step_index == 0
            {
                if let Some(exact) = exact.as_mut() {
                    exact.interrupt_voice = true;
                }
            }
            if program.bank == 2
                && program.id == 0x13
                && program.variant_hash == 0x8212a1e6
                && step_index == 0
            {
                if let Some(exact) = exact.as_mut() {
                    exact.interrupt_voice = true;
                }
            }
            if let (
                Some(exact),
                Some((timer_cycles, sfx_timer_accum)),
                Some((overflow_index, keyoff_overflows)),
            ) = (
                exact.as_mut(),
                sfx_clock,
                exact_sfx_clock_timing(program, step_index),
            ) {
                let dispatch_overflow_delay = if uses_second_overflow_dispatch(program) {
                    usize::from(2u8.saturating_sub(sfx_overflow_count_in_frame(
                        self.dsp_timer_cycles,
                        self.sfx_timer_accum,
                    )))
                } else {
                    0
                };
                let overflow_index = overflow_index + dispatch_overflow_delay;
                let (frame_delta, scheduler_tick_index) =
                    sfx_clock_target(timer_cycles, sfx_timer_accum, overflow_index);
                exact.command_delay_frames = frame_delta;
                exact.scheduler_tick_index = scheduler_tick_index;
                let (key_frame, _, key_offset) =
                    sfx_clock_target_position(timer_cycles, sfx_timer_accum, overflow_index);
                let (keyoff_frame, _, keyoff_offset) = sfx_clock_target_position(
                    timer_cycles,
                    sfx_timer_accum,
                    overflow_index + keyoff_overflows,
                );
                exact.duration_samples = (u32::from(keyoff_frame - key_frame) * 534)
                    .saturating_add(u32::from(keyoff_offset))
                    .saturating_sub(u32::from(key_offset));
                if exact.interrupt_voice {
                    let interrupt_position = sfx_overflow_positions_after(
                        (self.dsp_timer_cycles, self.sfx_timer_accum),
                        -1,
                        2,
                    )[1];
                    let interrupt_delay = (interrupt_position / 534) as u8;
                    let interrupt_offset = (interrupt_position % 534) as u16;
                    let interrupt_cycles =
                        sfx_timer_cycles_after_frames(self.dsp_timer_cycles, interrupt_delay);
                    let first_boundary = u16::from(sfx_first_boundary(interrupt_cycles));
                    exact.interrupt_delay_frames = interrupt_delay;
                    exact.interrupt_scheduler_tick_index =
                        ((interrupt_offset - first_boundary) / 64) as u8;
                }
            }
            if first_for_voice {
                if let Some(exact) = exact.filter(|exact| exact.interrupt_voice) {
                    let interrupt_delay = u32::from(exact.interrupt_delay_frames);
                    let interrupt_cycles = sfx_timer_cycles_after_frames(
                        self.dsp_timer_cycles,
                        exact.interrupt_delay_frames,
                    );
                    let interrupt_boundary = sfx_first_boundary(interrupt_cycles);
                    let interrupt_samples = interrupt_delay * 534
                        + u32::from(interrupt_boundary)
                        + u32::from(exact.interrupt_scheduler_tick_index) * 64;
                    if program.bank == 1 && program.id == 0x36 {
                        if let Some(offset) =
                            latest_music_note_offset_before(frame, interrupt_samples as i32)
                        {
                            let already_retriggered = frame.events.iter().any(|event| {
                                event.sample_offset == offset
                                    && matches!(
                                        event.kind,
                                        AudioEventKind::NoteOn { voice, .. }
                                            | AudioEventKind::RetriggerVoice { voice }
                                            | AudioEventKind::KeyOnVoice { voice, .. }
                                            if voice == step.voice
                                    )
                            });
                            if !already_retriggered {
                                if let Some(note) =
                                    self.music_note_at_current_position(step.voice, offset as u16)
                                {
                                    self.emit_music_note_unchecked(
                                        frame,
                                        self.last_music_track,
                                        note,
                                        stats,
                                    );
                                } else {
                                    push_event_at(
                                        frame,
                                        offset,
                                        AudioEventKind::SetNoteOrigin {
                                            voice: step.voice,
                                            origin: AudioNoteOrigin::Music,
                                        },
                                    );
                                    push_event_at(
                                        frame,
                                        offset,
                                        AudioEventKind::RetriggerVoice { voice: step.voice },
                                    );
                                }
                            }
                        }
                    }
                    if !voice_was_sfx_owned {
                        mark_frame_note_offs_for_voice_before_as_music(
                            frame,
                            step.voice,
                            interrupt_samples as i32,
                        );
                    }
                    discard_frame_music_events_for_voice_at_or_after(
                        frame,
                        step.voice,
                        interrupt_samples as i32,
                    );
                    if let Some(changes) = interrupted_pitch_changes.take() {
                        self.pending_sfx_pitch_changes[voice] = changes
                            .into_iter()
                            .filter(|change| change.samples_remaining <= interrupt_samples)
                            .collect();
                    }
                    if interrupt_samples < 534 {
                        if !frame.events.iter().any(|event| {
                            event.sample_offset == interrupt_samples as i32
                                && matches!(
                                    event.kind,
                                    AudioEventKind::SetEchoSend { voice, .. }
                                        if voice == step.voice
                                )
                        }) {
                            push_event_at(
                                frame,
                                interrupt_samples as i32,
                                AudioEventKind::SetEchoSend {
                                    voice: step.voice,
                                    enabled: step.echo,
                                },
                            );
                        }
                        let already_scheduled = frame.events.iter().any(|event| {
                            event.sample_offset == interrupt_samples as i32
                                && matches!(
                                    event.kind,
                                    AudioEventKind::NoteOff { voice } if voice == step.voice
                                )
                        });
                        if !already_scheduled {
                            push_event_at(
                                frame,
                                interrupt_samples as i32,
                                AudioEventKind::SetNoteOrigin {
                                    voice: step.voice,
                                    origin: AudioNoteOrigin::Sfx,
                                },
                            );
                            push_event_at(
                                frame,
                                interrupt_samples as i32,
                                AudioEventKind::NoteOff { voice: step.voice },
                            );
                            stats.note_events += 1;
                        }
                    } else {
                        self.sfx_keyoff_samples_remaining[voice] = interrupt_samples;
                        self.sfx_keyoff_starts_ownership_mask |= voice_bit;
                        if voice_was_sfx_owned {
                            self.sfx_voice_mask |= voice_bit;
                            self.active_voice_mask |= voice_bit;
                        }
                    }
                    if interrupt_samples < 534 {
                        self.mark_voice_active(step.voice);
                    }
                    if exact.ownership_duration_samples != 0 {
                        let command_delay = u32::from(exact.command_delay_frames);
                        let keyon_cycles = sfx_timer_cycles_after_frames(
                            self.dsp_timer_cycles,
                            exact.command_delay_frames,
                        );
                        let keyon_boundary = sfx_first_boundary(keyon_cycles);
                        let keyon_samples = command_delay * 534
                            + u32::from(keyon_boundary)
                            + u32::from(exact.scheduler_tick_index) * 64;
                        self.sfx_ownership_samples_remaining[voice] =
                            keyon_samples + exact.ownership_duration_samples;
                    }
                }
            }
            let start = exact.map_or(voice_timeline_end[voice], |exact| {
                u16::from(exact.command_delay_frames)
            });
            let delay_after_previous = start.saturating_sub(voice_timeline_end[voice]) as u8;
            voice_timeline_end[voice] = start.saturating_add(u16::from(step.duration_frames));
            let mut pending = PendingSfxStep {
                step,
                exact,
                engine_dsp_envelope: None,
                delay_after_previous,
                preserve_existing_volume: false,
                volume_via_parameters: exact.is_some_and(|exact| exact.volume_via_parameters)
                    || (program.bank == 0
                        && program.id == 0x01
                        && program.variant_hash == 0x6f23aa01)
                    || (program.bank == 2
                        && program.id == 0x4f
                        && program.variant_hash == 0xede5b411
                        && matches!(
                            step_index,
                            1..=3
                                | 7..=9
                                | 11..=13
                                | 15..=18
                                | 20
                                | 22
                                | 24..=28
                        )),
                refresh_repeat_on_keyon: program.bank == 0
                    && program.id == 0x05
                    && program.variant_hash == 0x5c065005,
                preserve_inactive_pitch_latch: program.bank == 1 && program.id == 0x1d,
                engine_keyoff_owned: self.engine_receipt_mode
                    && program.bank == 1
                    && program.id == 0x2b
                    && program.variant_hash == 0x4b866332,
            };
            if program.bank == 2
                && program.id == 0x1b
                && program.variant_hash == 0xa44764fc
                && matches!(step_index, 0 | 1 | 2)
            {
                if let Some(exact) = pending.exact.as_mut() {
                    // The bytecode pattern owns these KOFs; the catalog step
                    // only describes the initial DSP register setup.
                    exact.duration_samples = 0;
                }
            }
            if program.bank == 1
                && program.id == 0x1d
                && program.variant_hash == 0xf31c8f91
                && step_index == 0
            {
                self.semantic_sfx_repeat_steps[voice] = Some(pending);
            }
            if program.bank == 1
                && program.id == 0x36
                && program.variant_hash == 0x102e5506
                && step_index == 0
            {
                self.semantic_sfx_repeat_steps[voice] = Some(pending);
            }
            if program.bank == 0 && program.id == 0x01 && program.variant_hash == 0x6f23aa01 {
                self.semantic_sfx_repeat_steps[voice] = Some(pending);
            }
            if program.bank == 0 && program.id == 0x05 && program.variant_hash == 0x5c065005 {
                self.semantic_sfx_repeat_steps[voice] = Some(pending);
            }
            if let (Some(exact), Some((initial_overflow, active_overflows, gap_overflows))) =
                (exact, looping_sfx_timing(program, step_index))
            {
                self.looping_sfx_voices[voice] = Some(LoopingSfxVoice {
                    step,
                    exact: ExactSfxDspStep {
                        duration_samples: 0,
                        ..exact
                    },
                    overflows_remaining: if program.bank == 2
                        && program.id == 0x1b
                        && program.variant_hash == 0xa44764fc
                    {
                        // These are absolute command-to-first-KOF counts. The
                        // later active notes use `active_overflows` below.
                        if step_index == 1 {
                            30
                        } else {
                            16
                        }
                    } else {
                        initial_overflow + active_overflows
                    },
                    active: true,
                    active_overflows,
                    gap_overflows,
                    retriggers_remaining: looping_sfx_retrigger_count(program, step_index),
                    retrigger_index: 0,
                    staggered_chime: program.bank == 2
                        && program.id == 0x1b
                        && program.variant_hash == 0xa44764fc,
                });
            }
            if program.bank == 2
                && program.id == 0x1b
                && program.variant_hash == 0xa44764fc
                && step_index == 2
            {
                // Voice 2's later notes are generated by the clocked pattern
                // above. Leaving this in the semantic receipt queue lets an
                // unrelated KON consume it early.
                continue;
            }
            if self.engine_receipt_mode && uses_semantic_sfx_keyons(program) {
                let allocator_voice = if program.bank == 2
                    && matches!(
                        program.id,
                        0x04 | 0x08
                            | 0x09
                            | 0x0b
                            | 0x0c
                            | 0x0d
                            | 0x0e
                            | 0x0f
                            | 0x11
                            | 0x14
                            | 0x15
                            | 0x16
                            | 0x17
                            | 0x1c
                            | 0x24
                            | 0x31
                            | 0x44
                            | 0x49
                            | 0x4b
                            | 0x57
                            | 0x5c
                            | 0x89
                            | 0x8b
                            | 0x97
                    ) {
                    self.semantic_bank2_command_voice
                } else if program.bank == 1
                    && (matches!(
                        program.id,
                        0x01 | 0x05
                            | 0x16
                            | 0x17
                            | 0x18
                            | 0x19
                            | 0x1d
                            | 0x20
                            | 0x22
                            | 0x26
                            | 0x29
                            | 0x2a
                            | 0x2d
                            | 0x3c
                            | 0x41
                            | 0x45
                            | 0x56
                            | 0x57
                            | 0x5f
                            | 0x6a
                            | 0x81
                            | 0x85
                            | 0x96
                            | 0x97
                            | 0x9e
                            | 0x9f
                    ) || (program.id == 0x2b && program.variant_hash == 0x4b866332))
                {
                    self.semantic_bank1_command_voice
                } else {
                    None
                };
                if let Some(allocator_voice) = allocator_voice {
                    let target_voice = usize::from(allocator_voice);
                    let mut pending = PendingSfxStep {
                        delay_after_previous: 0,
                        ..pending
                    };
                    pending.step.voice = target_voice as u8;
                    if pending.preserve_inactive_pitch_latch {
                        self.semantic_pitch_latch_mask |= 1 << target_voice;
                    }
                    if first_for_voice {
                        self.semantic_sfx_pending_steps[target_voice].clear();
                    }
                    self.semantic_sfx_pending_steps[target_voice].push(pending);
                } else {
                    if pending.preserve_inactive_pitch_latch {
                        self.semantic_pitch_latch_mask |= 1 << voice;
                    }
                    if first_for_voice {
                        self.semantic_sfx_pending_steps[voice].clear();
                    }
                    self.semantic_sfx_pending_steps[voice].push(PendingSfxStep {
                        delay_after_previous: 0,
                        ..pending
                    });
                }
                continue;
            }
            if first_for_voice && pending.delay_after_previous != 0 {
                self.voice_frames_remaining[voice] = u16::from(pending.delay_after_previous);
                let mut pending = pending;
                pending.delay_after_previous = 0;
                self.pending_voice_steps[voice].push(pending);
            } else if self.voice_frames_remaining[voice] == 0
                && self.pending_voice_steps[voice].is_empty()
            {
                self.emit_sfx_step(frame, pending, stats);
            } else {
                self.pending_voice_steps[voice].push(pending);
            }
        }
    }

    fn expand_fallback_sfx(
        &mut self,
        frame: &mut AudioEventFrame,
        voice: u8,
        code: u8,
        slot: u8,
        stats: &mut ModernAudioSequenceStats,
    ) {
        self.interrupt_voice(voice);
        push_event(
            frame,
            AudioEventKind::SetNoteOrigin {
                voice,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        push_event(
            frame,
            AudioEventKind::SetEnvelope {
                voice,
                attack: 1,
                decay: 2 + (code & 7),
                sustain: 8,
                release: 2,
            },
        );
        push_event(
            frame,
            AudioEventKind::NoteOn {
                voice,
                pitch: pitch_for_code(code).saturating_add(slot),
                instrument: instrument_for_code(code),
                volume: 112,
            },
        );
        self.mark_voice_active(voice);
        push_event(frame, AudioEventKind::SetDuration { voice, frames: 6 });
        self.extend_voice_lifetime(voice, 6);
        stats.note_events += 1;
        stats.envelope_events += 1;
    }

    fn mark_voice_active(&mut self, voice: u8) {
        if voice < 8 {
            self.sfx_release_pending_mask &= !(1 << voice);
            self.sfx_release_overflows_remaining[usize::from(voice)] = 0;
            self.sfx_keyoff_starts_ownership_mask &= !(1 << voice);
            self.sfx_voice_mask |= 1 << voice;
            self.active_voice_mask |= 1 << voice;
        }
    }

    fn synchronize_sfx_ownership_from_engine(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
    ) {
        if !self.engine_receipt_mode {
            return;
        }
        let Some(spc) = spc else {
            return;
        };

        let engine_mask = spc.is_chan_on;
        let released_mask = self.sfx_voice_mask & !engine_mask;
        for voice in 0..8 {
            let voice_bit = 1 << voice;
            if released_mask & voice_bit == 0 {
                continue;
            }
            self.voice_frames_remaining[voice] = 0;
            self.pending_voice_steps[voice].clear();
            self.sfx_keyoff_samples_remaining[voice] = 0;
            self.sfx_keyoff_starts_ownership_mask &= !voice_bit;
            self.sfx_ownership_samples_remaining[voice] = 0;
            self.pending_sfx_pitch_changes[voice].clear();
            self.looping_sfx_voices[voice] = None;
            if self.music_voice_mask & voice_bit == 0 {
                self.active_voice_mask &= !voice_bit;
            }
        }
        self.sfx_voice_mask = engine_mask;
        self.active_voice_mask |= engine_mask;
    }

    fn mark_music_voice_active(&mut self, voice: u8) {
        if voice < 8 {
            self.music_voice_mask |= 1 << voice;
            self.active_voice_mask |= 1 << voice;
        }
    }

    fn mark_voice_inactive(&mut self, voice: u8) {
        if voice < 8 {
            self.sfx_voice_mask &= !(1 << voice);
            self.active_voice_mask &= !(1 << voice);
            self.voice_frames_remaining[usize::from(voice)] = 0;
            self.pending_voice_steps[usize::from(voice)].clear();
            self.sfx_keyoff_samples_remaining[usize::from(voice)] = 0;
            self.sfx_keyoff_starts_ownership_mask &= !(1 << voice);
            self.sfx_ownership_samples_remaining[usize::from(voice)] = 0;
            self.sfx_release_pending_mask &= !(1 << voice);
            self.sfx_release_overflows_remaining[usize::from(voice)] = 0;
            self.sfx_release_pending_mask &= !(1 << voice);
            self.sfx_ownership_release_overflows[usize::from(voice)] = 0;
            self.sfx_release_overflows_remaining[usize::from(voice)] = 0;
            self.pending_sfx_pitch_changes[usize::from(voice)].clear();
            self.pending_sfx_volume_changes[usize::from(voice)].clear();
        }
    }

    fn mark_music_voice_inactive(&mut self, voice: u8) {
        if voice < 8 {
            let voice = usize::from(voice);
            self.music_voice_mask &= !(1 << voice);
            self.music_keyoff_frames_remaining[voice] = 0;
            if self.voice_frames_remaining[voice] == 0
                && self.pending_voice_steps[voice].is_empty()
                && self.sfx_keyoff_samples_remaining[voice] == 0
                && self.sfx_ownership_samples_remaining[voice] == 0
            {
                self.active_voice_mask &= !(1 << voice);
            }
        }
    }

    fn release_music_voices(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let music_voices = self.music_voice_mask;
        for voice in 0..8 {
            if music_voices & (1 << voice) == 0 {
                continue;
            }
            push_event(frame, AudioEventKind::NoteOff { voice });
            self.mark_music_voice_inactive(voice);
            stats.note_events += 1;
        }
    }

    fn interrupt_voice(&mut self, voice: u8) {
        if voice < 8 {
            self.sfx_voice_mask &= !(1 << voice);
            self.active_voice_mask &= !(1 << voice);
            self.voice_frames_remaining[usize::from(voice)] = 0;
            self.pending_voice_steps[usize::from(voice)].clear();
            self.sfx_keyoff_samples_remaining[usize::from(voice)] = 0;
            self.sfx_keyoff_starts_ownership_mask &= !(1 << voice);
            self.sfx_ownership_samples_remaining[usize::from(voice)] = 0;
            self.pending_sfx_pitch_changes[usize::from(voice)].clear();
            self.persistent_sfx_pitch_words[usize::from(voice)] = 0;
            self.looping_sfx_voices[usize::from(voice)] = None;
            self.semantic_sfx_pending_steps[usize::from(voice)].clear();
            self.engine_automated_sfx_volume_mask &= !(1 << voice);
        }
    }

    fn cancel_sfx_schedules(&mut self, voice: u8) {
        if voice >= 8 {
            return;
        }
        let voice = usize::from(voice);
        self.sfx_voice_mask &= !(1 << voice);
        self.voice_frames_remaining[voice] = 0;
        self.pending_voice_steps[voice].clear();
        self.sfx_keyoff_samples_remaining[voice] = 0;
        self.sfx_keyoff_starts_ownership_mask &= !(1 << voice);
        self.sfx_ownership_samples_remaining[voice] = 0;
        self.sfx_ownership_release_overflows[voice] = 0;
        self.sfx_release_overflows_remaining[voice] = 0;
        self.pending_sfx_pitch_changes[voice].clear();
        self.pending_sfx_volume_changes[voice].clear();
        self.looping_sfx_voices[voice] = None;
        self.semantic_sfx_pending_steps[voice].clear();
        if voice < 6 {
            self.semantic_sfx_repeat_steps[voice] = None;
        }
        self.engine_automated_sfx_volume_mask &= !(1 << voice);
    }

    fn extend_voice_lifetime(&mut self, voice: u8, frames: u16) {
        if voice < 8 {
            let remaining = &mut self.voice_frames_remaining[usize::from(voice)];
            *remaining = remaining.saturating_add(frames);
        }
    }

    fn advance_voice_lifetimes(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for voice in 0..self.voice_frames_remaining.len() {
            let remaining = &mut self.voice_frames_remaining[voice];
            if *remaining == 0 {
                continue;
            }
            *remaining -= 1;
            if *remaining == 0 {
                if self.pending_voice_steps[voice].is_empty() {
                    if self.sfx_keyoff_samples_remaining[voice] == 0
                        && self.sfx_ownership_samples_remaining[voice] == 0
                        && self.sfx_release_pending_mask & (1 << voice) == 0
                        && self.sfx_release_overflows_remaining[voice] == 0
                    {
                        self.sfx_voice_mask &= !(1 << voice);
                        if self.music_voice_mask & (1 << voice) == 0 {
                            self.active_voice_mask &= !(1 << voice);
                        }
                    }
                } else {
                    let delay = self.pending_voice_steps[voice][0].delay_after_previous;
                    if delay != 0 {
                        self.voice_frames_remaining[voice] = u16::from(delay);
                        self.pending_voice_steps[voice][0].delay_after_previous = 0;
                    } else {
                        let pending = self.pending_voice_steps[voice].remove(0);
                        self.emit_sfx_step(frame, pending, stats);
                        while self.voice_frames_remaining[voice] == 0
                            && !self.pending_voice_steps[voice].is_empty()
                        {
                            let delay = self.pending_voice_steps[voice][0].delay_after_previous;
                            if delay != 0 {
                                self.voice_frames_remaining[voice] = u16::from(delay);
                                self.pending_voice_steps[voice][0].delay_after_previous = 0;
                                break;
                            }
                            let pending = self.pending_voice_steps[voice].remove(0);
                            self.emit_sfx_step(frame, pending, stats);
                        }
                    }
                }
            }
        }
    }

    fn advance_sfx_keyoffs(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let sfx_clock = self.previous_sfx_clock;
        for voice in 0..self.sfx_keyoff_samples_remaining.len() {
            let remaining = &mut self.sfx_keyoff_samples_remaining[voice];
            if *remaining == 0 {
                continue;
            }
            *remaining = remaining.saturating_sub(534);
            // Offset 534 is the first sample of the following native DSP
            // frame, not the final sample of the current one.
            if *remaining < 534 {
                let keyoff_offset = *remaining;
                push_event_at(
                    frame,
                    keyoff_offset as i32,
                    AudioEventKind::SetNoteOrigin {
                        voice: voice as u8,
                        origin: AudioNoteOrigin::Sfx,
                    },
                );
                push_event_at(
                    frame,
                    keyoff_offset as i32,
                    AudioEventKind::NoteOff { voice: voice as u8 },
                );
                *remaining = 0;
                let voice_bit = 1 << voice;
                let starts_ownership = self.sfx_keyoff_starts_ownership_mask & voice_bit != 0;
                self.sfx_keyoff_starts_ownership_mask &= !voice_bit;
                if starts_ownership {
                    self.sfx_voice_mask |= voice_bit;
                    self.active_voice_mask |= voice_bit;
                } else if self.sfx_ownership_samples_remaining[voice] == 0
                    && self.pending_voice_steps[voice].is_empty()
                {
                    let overflows = self.sfx_ownership_release_overflows[voice];
                    self.sfx_ownership_release_overflows[voice] = 0;
                    if overflows == 0 {
                        self.sfx_release_pending_mask |= 1 << voice;
                    } else {
                        let elapsed_after_keyoff = sfx_clock.map_or(0, |clock| {
                            sfx_overflow_positions_after(
                                clock,
                                keyoff_offset as i32,
                                usize::from(overflows),
                            )
                            .into_iter()
                            .take_while(|position| *position < 534)
                            .count() as u8
                        });
                        let remaining_overflows = overflows.saturating_sub(elapsed_after_keyoff);
                        if remaining_overflows == 0 {
                            self.sfx_release_pending_mask |= 1 << voice;
                        } else {
                            self.sfx_release_overflows_remaining[voice] = remaining_overflows;
                        }
                    }
                } else {
                    self.sfx_voice_mask |= voice_bit;
                    self.active_voice_mask |= voice_bit;
                }
                if self.sfx_voice_mask & (1 << voice) == 0
                    && self.music_voice_mask & (1 << voice) == 0
                    && self.voice_frames_remaining[voice] == 0
                    && self.pending_voice_steps[voice].is_empty()
                {
                    self.active_voice_mask &= !(1 << voice);
                }
                stats.note_events += 1;
            }
        }
    }

    fn advance_rising_warble_retriggers(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for voice in 0..8 {
            let remaining = self.rising_warble_retrigger_samples_remaining[voice];
            if remaining == 0 {
                continue;
            }
            let remaining = remaining.saturating_sub(534);
            self.rising_warble_retrigger_samples_remaining[voice] = remaining;
            if remaining < 534 {
                let sample_offset = remaining as i32;
                self.rising_warble_retrigger_samples_remaining[voice] = 0;
                for displaced_voice in 0..8 {
                    if displaced_voice == voice || !self.rising_warble_long_pattern[displaced_voice]
                    {
                        continue;
                    }
                    self.cancel_sfx_schedules(displaced_voice as u8);
                    self.rising_warble_long_pattern[displaced_voice] = false;
                    self.rising_warble_retrigger_samples_remaining[displaced_voice] = 0;
                    self.secondary_sfx_keyoff_samples_remaining[displaced_voice] = 0;
                    self.sfx_voice_activation_count[displaced_voice] = 0;
                    self.active_voice_mask &= !(1 << displaced_voice);
                }
                self.frame_warble_pitch_events
                    .push((voice as u8, sample_offset, 4_808));
                self.frame_warble_volume_events
                    .push((voice as u8, sample_offset, 63, 63));
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetNoteOrigin {
                        voice: voice as u8,
                        origin: AudioNoteOrigin::Sfx,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchWord {
                        voice: voice as u8,
                        pitch_word: 4_808,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: voice as u8,
                        left: 63,
                        right: 63,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice: voice as u8,
                        source: 21,
                        adsr1: 255,
                        adsr2: 224,
                        gain: 184,
                        volume_left: 63,
                        volume_right: 63,
                        rate_counter: 0,
                    },
                );
                self.schedule_rising_warble_second_activation(voice as u8, sample_offset, frame);
                self.sfx_voice_activation_count[voice] = 2;
                self.sfx_voice_program[voice] = 0x011d;
                self.sfx_voice_mask |= 1 << voice;
                self.active_voice_mask |= 1 << voice;
                stats.note_events += 1;
            }
        }
    }

    fn advance_secondary_sfx_keyoffs(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for voice in 0..8 {
            let remaining = &mut self.secondary_sfx_keyoff_samples_remaining[voice];
            if *remaining == 0 {
                continue;
            }
            *remaining = remaining.saturating_sub(534);
            if *remaining < 534 {
                let sample_offset = *remaining as i32;
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetNoteOrigin {
                        voice: voice as u8,
                        origin: AudioNoteOrigin::Sfx,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::NoteOff { voice: voice as u8 },
                );
                *remaining = 0;
                self.sfx_voice_mask |= 1 << voice;
                self.active_voice_mask |= 1 << voice;
                if self.rising_warble_long_pattern[voice]
                    && self.sfx_voice_activation_count[voice] == 1
                {
                    self.sfx_voice_activation_count[voice] = 3;
                    if let Some(clock) = self.previous_sfx_clock {
                        self.rising_warble_retrigger_samples_remaining[voice] =
                            sfx_overflow_positions_after(clock, sample_offset, 2)[1] + 320;
                    }
                }
                stats.note_events += 1;
            }
        }
    }

    fn advance_port23_id36_cluster(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for remaining in &mut self.port23_id36_voice7_keyoffs {
            if *remaining == 0 {
                continue;
            }
            *remaining = remaining.saturating_sub(534);
            if *remaining < 534 {
                let sample_offset = *remaining as i32;
                *remaining = 0;
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetNoteOrigin {
                        voice: 7,
                        origin: AudioNoteOrigin::Sfx,
                    },
                );
                push_event_at(frame, sample_offset, AudioEventKind::NoteOff { voice: 7 });
                stats.note_events += 1;
            }
        }
        if self.port23_id36_voice7_retrigger != 0 {
            self.port23_id36_voice7_retrigger =
                self.port23_id36_voice7_retrigger.saturating_sub(534);
            if self.port23_id36_voice7_retrigger < 534 {
                let sample_offset = self.port23_id36_voice7_retrigger as i32;
                self.port23_id36_voice7_retrigger = 0;
                self.emit_port23_id36_voice7_keyon(frame, sample_offset, stats);
            }
        }
        if self.port23_id36_voice6_final_keyoff != 0 {
            self.port23_id36_voice6_final_keyoff =
                self.port23_id36_voice6_final_keyoff.saturating_sub(534);
            if self.port23_id36_voice6_final_keyoff < 534 {
                let sample_offset = self.port23_id36_voice6_final_keyoff as i32;
                self.port23_id36_voice6_final_keyoff = 0;
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetNoteOrigin {
                        voice: 6,
                        origin: AudioNoteOrigin::Sfx,
                    },
                );
                push_event_at(frame, sample_offset, AudioEventKind::NoteOff { voice: 6 });
                self.port23_id36_voice6_owned = false;
                self.sfx_release_pending_mask |= 1 << 6;
                stats.note_events += 1;
            }
        }
        for index in 0..self.port23_id36_voice7_late_keyoffs.len() {
            let remaining = &mut self.port23_id36_voice7_late_keyoffs[index];
            if *remaining == 0 {
                continue;
            }
            *remaining = remaining.saturating_sub(534);
            if *remaining >= 534 {
                continue;
            }
            let sample_offset = *remaining as i32;
            *remaining = 0;
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetNoteOrigin {
                    voice: 7,
                    origin: AudioNoteOrigin::Sfx,
                },
            );
            push_event_at(frame, sample_offset, AudioEventKind::NoteOff { voice: 7 });
            if index == 1 {
                self.port23_id36_voice7_owned = true;
                self.sfx_voice_mask |= 1 << 7;
                self.active_voice_mask |= 1 << 7;
            } else {
                self.port23_id36_voice7_owned = false;
                self.sfx_release_pending_mask |= 1 << 7;
            }
            stats.note_events += 1;
        }
        if self.port23_id36_voice7_late_retrigger != 0 {
            self.port23_id36_voice7_late_retrigger =
                self.port23_id36_voice7_late_retrigger.saturating_sub(534);
            if self.port23_id36_voice7_late_retrigger < 534 {
                let sample_offset = self.port23_id36_voice7_late_retrigger as i32;
                self.port23_id36_voice7_late_retrigger = 0;
                self.emit_port23_id36_voice7_keyon(frame, sample_offset, stats);
            }
        }
    }

    fn emit_port23_id36_voice7_keyon(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        stats: &mut ModernAudioSequenceStats,
    ) {
        self.frame_port23_id36_voice7_retrigger = true;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice: 7,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::KeyOnVoice {
                voice: 7,
                source: 16,
                adsr1: 142,
                adsr2: 224,
                gain: 184,
                volume_left: 63,
                volume_right: 63,
                rate_counter: 0,
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetPitchWord {
                voice: 7,
                pitch_word: 3_606,
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetStereoVolume {
                voice: 7,
                left: 63,
                right: 63,
            },
        );
        self.sfx_voice_mask |= 1 << 7;
        self.active_voice_mask |= 1 << 7;
        self.port23_id36_voice7_owned = true;
        stats.note_events += 1;
    }

    fn advance_port3_id15_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let mut ready = Vec::new();
        for action in &mut self.pending_port3_id15_actions {
            action.samples_remaining = action.samples_remaining.saturating_sub(534);
        }
        let mut index = 0;
        while index < self.pending_port3_id15_actions.len() {
            if self.pending_port3_id15_actions[index].samples_remaining < 534 {
                ready.push(self.pending_port3_id15_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            let sample_offset = pending.samples_remaining as i32;
            match pending.action {
                Port3Id15Action::KeyOff { mask, release } => {
                    for voice in 0..8u8 {
                        if mask & (1 << voice) == 0 {
                            continue;
                        }
                        push_event_at(
                            frame,
                            sample_offset,
                            AudioEventKind::SetNoteOrigin {
                                voice,
                                origin: AudioNoteOrigin::Sfx,
                            },
                        );
                        push_event_at(frame, sample_offset, AudioEventKind::NoteOff { voice });
                        stats.note_events += 1;
                    }
                    if release {
                        self.port3_id15_owned_mask &= !mask;
                        self.sfx_release_pending_mask |= mask;
                    }
                }
                Port3Id15Action::Chord { index } => {
                    self.emit_port3_id15_chord(frame, sample_offset, index, stats);
                }
                Port3Id15Action::Voice3Note {
                    pitch_word,
                    volume,
                    rate_counter,
                } => {
                    let voice = self.port3_id15_voices[1];
                    let write_volume = volume >= 0;
                    if write_volume {
                        self.port3_id15_voice3_volume = volume;
                    }
                    let volume = self.port3_id15_voice3_volume;
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetNoteOrigin {
                            voice,
                            origin: AudioNoteOrigin::Sfx,
                        },
                    );
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::KeyOnVoice {
                            voice,
                            source: 2,
                            adsr1: 254,
                            adsr2: 106,
                            gain: 127,
                            volume_left: volume,
                            volume_right: volume,
                            rate_counter,
                        },
                    );
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetPitchWord { voice, pitch_word },
                    );
                    if write_volume {
                        push_event_at(
                            frame,
                            sample_offset,
                            AudioEventKind::SetStereoVolume {
                                voice,
                                left: volume,
                                right: volume,
                            },
                        );
                    }
                    self.port3_id15_owned_mask |= 1 << voice;
                    stats.note_events += 1;
                }
                Port3Id15Action::AuxVoice6Note {
                    pitch_word,
                    rate_counter,
                    write_volume,
                } => {
                    let voice = 6;
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetNoteOrigin {
                            voice,
                            origin: AudioNoteOrigin::Sfx,
                        },
                    );
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::KeyOnVoice {
                            voice,
                            source: 5,
                            adsr1: 254,
                            adsr2: 106,
                            gain: 112,
                            volume_left: 25,
                            volume_right: 25,
                            rate_counter,
                        },
                    );
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetPitchWord { voice, pitch_word },
                    );
                    if write_volume {
                        push_event_at(
                            frame,
                            sample_offset,
                            AudioEventKind::SetStereoVolume {
                                voice,
                                left: 25,
                                right: 25,
                            },
                        );
                    }
                    self.port3_id15_owned_mask |= 1 << voice;
                    stats.note_events += 1;
                }
            }
        }
    }

    fn advance_bank1_id1_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for pending in &mut self.pending_bank1_id1_actions {
            pending.samples_remaining = pending.samples_remaining.saturating_sub(534);
        }
        let mut ready = Vec::new();
        let mut index = 0;
        while index < self.pending_bank1_id1_actions.len() {
            if self.pending_bank1_id1_actions[index].samples_remaining < 534 {
                ready.push(self.pending_bank1_id1_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            self.emit_bank1_id1_action(
                frame,
                pending.samples_remaining as i32,
                pending.action,
                stats,
            );
        }
    }

    fn emit_bank1_id1_action(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        action: Bank1Id1Action,
        stats: &mut ModernAudioSequenceStats,
    ) {
        const VOICE: u8 = 7;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice: VOICE,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        match action {
            Bank1Id1Action::KeyOff { release } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::NoteOff { voice: VOICE },
                );
                if release {
                    self.bank1_id1_active = false;
                    self.sfx_release_pending_mask |= 1 << VOICE;
                } else {
                    self.bank1_id1_active = true;
                }
                stats.note_events += 1;
            }
            Bank1Id1Action::KeyOn { pitch_word, volume } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice: VOICE,
                        enabled: false,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice: VOICE,
                        source: 1,
                        adsr1: 142,
                        adsr2: 224,
                        gain: 184,
                        volume_left: volume,
                        volume_right: volume,
                        rate_counter: u16::MAX,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchWord {
                        voice: VOICE,
                        pitch_word: 802,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchWord {
                        voice: VOICE,
                        pitch_word,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: VOICE,
                        left: volume,
                        right: volume,
                    },
                );
                self.bank1_id1_active = true;
                stats.note_events += 1;
            }
            Bank1Id1Action::Pitch { pitch_word } => push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice: VOICE,
                    pitch_word,
                },
            ),
        }
    }

    fn advance_bank1_id45_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for pending in &mut self.pending_bank1_id45_actions {
            pending.samples_remaining = pending.samples_remaining.saturating_sub(534);
        }
        let mut ready = Vec::new();
        let mut index = 0;
        while index < self.pending_bank1_id45_actions.len() {
            if self.pending_bank1_id45_actions[index].samples_remaining < 534 {
                ready.push(self.pending_bank1_id45_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            self.emit_bank1_id45_action(
                frame,
                pending.samples_remaining as i32,
                pending.action,
                stats,
            );
        }
    }

    fn emit_bank1_id45_action(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        action: Bank1Id45Action,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let voice = self.bank1_id45_voice;
        let voice_bit = 1 << voice;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        match action {
            Bank1Id45Action::KeyOff { release } => {
                push_event_at(frame, sample_offset, AudioEventKind::NoteOff { voice });
                if release {
                    self.bank1_id45_active = false;
                    self.sfx_release_pending_mask |= voice_bit;
                } else {
                    self.bank1_id45_active = true;
                }
                stats.note_events += 1;
            }
            Bank1Id45Action::KeyOn => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice,
                        enabled: false,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice,
                        source: 15,
                        adsr1: 254,
                        adsr2: 245,
                        gain: 184,
                        volume_left: 31,
                        volume_right: 31,
                        rate_counter: 2,
                    },
                );
                for pitch_word in [4_812, 4_794] {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetPitchWord { voice, pitch_word },
                    );
                }
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice,
                        left: 31,
                        right: 31,
                    },
                );
                self.bank1_id45_active = true;
                stats.note_events += 1;
            }
            Bank1Id45Action::Pitch { pitch_word } => push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord { voice, pitch_word },
            ),
            Bank1Id45Action::PitchAndVolume { pitch_word, volume } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchWord { voice, pitch_word },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice,
                        left: volume,
                        right: volume,
                    },
                );
            }
        }
    }

    fn advance_bank1_id95_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for pending in &mut self.pending_bank1_id95_actions {
            pending.samples_remaining = pending.samples_remaining.saturating_sub(534);
        }
        let mut ready = Vec::new();
        let mut index = 0;
        while index < self.pending_bank1_id95_actions.len() {
            if self.pending_bank1_id95_actions[index].samples_remaining < 534 {
                ready.push(self.pending_bank1_id95_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            self.emit_bank1_id95_action(
                frame,
                pending.samples_remaining as i32,
                pending.action,
                stats,
            );
        }
    }

    fn emit_bank1_id95_action(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        action: Bank1Id95Action,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let voice = self.bank1_id95_voice;
        let voice_bit = 1 << voice;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        match action {
            Bank1Id95Action::KeyOff { release } => {
                push_event_at(frame, sample_offset, AudioEventKind::NoteOff { voice });
                if release {
                    self.bank1_id95_active = false;
                    self.sfx_release_pending_mask |= voice_bit;
                } else {
                    self.bank1_id95_active = true;
                }
                stats.note_events += 1;
            }
            Bank1Id95Action::KeyOn {
                source,
                adsr1,
                adsr2,
                pitch_word,
                left,
                right,
                write_volume,
            } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice,
                        enabled: false,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice,
                        source,
                        adsr1,
                        adsr2,
                        gain: 184,
                        volume_left: left,
                        volume_right: right,
                        rate_counter: u16::MAX,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchWord { voice, pitch_word },
                );
                if write_volume {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetStereoVolume { voice, left, right },
                    );
                }
                self.bank1_id95_active = true;
                stats.note_events += 1;
            }
        }
    }

    fn advance_bank1_id41_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for pending in &mut self.pending_bank1_id41_actions {
            pending.samples_remaining = pending.samples_remaining.saturating_sub(534);
        }
        let mut ready = Vec::new();
        let mut index = 0;
        while index < self.pending_bank1_id41_actions.len() {
            if self.pending_bank1_id41_actions[index].samples_remaining < 534 {
                ready.push(self.pending_bank1_id41_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            self.emit_bank1_id41_action(
                frame,
                pending.samples_remaining as i32,
                pending.action,
                stats,
            );
        }
    }

    fn emit_bank1_id41_action(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        action: Bank1Id41Action,
        stats: &mut ModernAudioSequenceStats,
    ) {
        const VOICE: u8 = 7;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice: VOICE,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        match action {
            Bank1Id41Action::KeyOff { release } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::NoteOff { voice: VOICE },
                );
                if release {
                    self.bank1_id41_active = false;
                    self.sfx_release_pending_mask |= 1 << VOICE;
                } else {
                    self.bank1_id41_active = true;
                }
                stats.note_events += 1;
            }
            Bank1Id41Action::KeyOn {
                initial_pitch,
                pitch_word,
                volume,
                rate_counter,
            } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice: VOICE,
                        enabled: false,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice: VOICE,
                        source: 1,
                        adsr1: 142,
                        adsr2: 224,
                        gain: 184,
                        volume_left: volume,
                        volume_right: volume,
                        rate_counter,
                    },
                );
                for pitch_word in [initial_pitch, pitch_word] {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetPitchWord {
                            voice: VOICE,
                            pitch_word,
                        },
                    );
                }
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: VOICE,
                        left: volume,
                        right: volume,
                    },
                );
                self.bank1_id41_active = true;
                stats.note_events += 1;
            }
            Bank1Id41Action::Pitch { pitch_word } => push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice: VOICE,
                    pitch_word,
                },
            ),
            Bank1Id41Action::Volume { volume } => push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetStereoVolume {
                    voice: VOICE,
                    left: volume,
                    right: volume,
                },
            ),
        }
    }

    fn advance_bank1_id60_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for pending in &mut self.pending_bank1_id60_actions {
            pending.samples_remaining = pending.samples_remaining.saturating_sub(534);
        }
        let mut ready = Vec::new();
        let mut index = 0;
        while index < self.pending_bank1_id60_actions.len() {
            if self.pending_bank1_id60_actions[index].samples_remaining < 534 {
                ready.push(self.pending_bank1_id60_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            self.emit_bank1_id60_action(
                frame,
                pending.samples_remaining as i32,
                pending.action,
                stats,
            );
        }
    }

    fn emit_bank1_id60_action(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        action: Bank1Id60Action,
        stats: &mut ModernAudioSequenceStats,
    ) {
        const VOICE: u8 = 7;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice: VOICE,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        match action {
            Bank1Id60Action::KeyOff { release } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::NoteOff { voice: VOICE },
                );
                if release {
                    self.bank1_id60_active = false;
                    self.sfx_release_overflows_remaining[usize::from(VOICE)] = 3;
                } else {
                    self.bank1_id60_active = true;
                }
                stats.note_events += 1;
            }
            Bank1Id60Action::KeyOn {
                pitch_word,
                volume,
                rate_counter,
            } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice: VOICE,
                        enabled: false,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice: VOICE,
                        source: 4,
                        adsr1: 254,
                        adsr2: 106,
                        gain: 127,
                        volume_left: volume,
                        volume_right: volume,
                        rate_counter,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchWord {
                        voice: VOICE,
                        pitch_word,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: VOICE,
                        left: volume,
                        right: volume,
                    },
                );
                self.bank1_id60_active = true;
                stats.note_events += 1;
            }
        }
    }

    fn advance_bank2_id28_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for pending in &mut self.pending_bank2_id28_actions {
            pending.samples_remaining = pending.samples_remaining.saturating_sub(534);
        }
        let mut ready = Vec::new();
        let mut index = 0;
        while index < self.pending_bank2_id28_actions.len() {
            if self.pending_bank2_id28_actions[index].samples_remaining < 534 {
                ready.push(self.pending_bank2_id28_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            self.emit_bank2_id28_action(
                frame,
                pending.samples_remaining as i32,
                pending.action,
                stats,
            );
        }
    }

    fn emit_bank2_id28_action(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        action: Bank2Id28Action,
        stats: &mut ModernAudioSequenceStats,
    ) {
        const VOICE: u8 = 6;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice: VOICE,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        match action {
            Bank2Id28Action::KeyOff { release } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::NoteOff { voice: VOICE },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice: VOICE,
                        enabled: false,
                    },
                );
                if release {
                    self.bank2_id28_active = false;
                    self.sfx_release_pending_mask |= 1 << VOICE;
                }
                stats.note_events += 1;
            }
            Bank2Id28Action::KeyOn {
                pitch_word,
                rate_counter,
            } => {
                let initial_pitch = if pitch_word == 2_150 { 2_250 } else { 2_525 };
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice: VOICE,
                        source: 7,
                        adsr1: 254,
                        adsr2: 106,
                        gain: 112,
                        volume_left: 79,
                        volume_right: 79,
                        rate_counter,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchWord {
                        voice: VOICE,
                        pitch_word: initial_pitch,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchWord {
                        voice: VOICE,
                        pitch_word,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: VOICE,
                        left: 79,
                        right: 79,
                    },
                );
                self.bank2_id28_active = true;
                stats.note_events += 1;
            }
            Bank2Id28Action::Pitch { pitch_word } => push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice: VOICE,
                    pitch_word,
                },
            ),
        }
    }

    fn advance_bank2_id9_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for pending in &mut self.pending_bank2_id9_actions {
            pending.samples_remaining = pending.samples_remaining.saturating_sub(534);
        }
        let mut ready = Vec::new();
        let mut index = 0;
        while index < self.pending_bank2_id9_actions.len() {
            if self.pending_bank2_id9_actions[index].samples_remaining < 534 {
                ready.push(self.pending_bank2_id9_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            self.emit_bank2_id9_action(
                frame,
                pending.samples_remaining as i32,
                pending.action,
                stats,
            );
        }
    }

    fn emit_bank2_id9_action(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        action: Bank2Id9Action,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let voice = self.bank2_id9_voice;
        let voice_bit = 1 << voice;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        match action {
            Bank2Id9Action::KeyOff { release } => {
                push_event_at(frame, sample_offset, AudioEventKind::NoteOff { voice });
                if release {
                    self.bank2_id9_active = false;
                    self.sfx_release_pending_mask |= voice_bit;
                } else {
                    self.bank2_id9_active = true;
                }
                stats.note_events += 1;
            }
            Bank2Id9Action::KeyOn {
                initial_pitch,
                pitch_word,
                volume,
            } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice,
                        enabled: false,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice,
                        source: 12,
                        adsr1: 254,
                        adsr2: 224,
                        gain: 184,
                        volume_left: volume,
                        volume_right: volume,
                        // KON does not reset the DSP's global envelope-rate
                        // phase. Preserve the phase carried by the voice.
                        rate_counter: u16::MAX,
                    },
                );
                for pitch_word in [initial_pitch, pitch_word] {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetPitchWord { voice, pitch_word },
                    );
                }
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice,
                        left: volume,
                        right: volume,
                    },
                );
                self.bank2_id9_active = true;
                stats.note_events += 1;
            }
            Bank2Id9Action::Pitch { pitch_word } => push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord { voice, pitch_word },
            ),
        }
    }

    fn advance_bank2_id14_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for pending in &mut self.pending_bank2_id14_actions {
            pending.samples_remaining = pending.samples_remaining.saturating_sub(534);
        }
        let mut ready = Vec::new();
        let mut index = 0;
        while index < self.pending_bank2_id14_actions.len() {
            if self.pending_bank2_id14_actions[index].samples_remaining < 534 {
                ready.push(self.pending_bank2_id14_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            self.emit_bank2_id14_action(
                frame,
                pending.samples_remaining as i32,
                pending.action,
                stats,
            );
        }
    }

    fn emit_bank2_id14_action(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        action: Bank2Id14Action,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let voice = self.bank2_id14_voice;
        let voice_bit = 1 << voice;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        match action {
            Bank2Id14Action::KeyOff { release } => {
                if !self.bank2_id14_active && voice == 5 {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetEnvelopeRateCounter {
                            voice,
                            rate_counter: 19,
                        },
                    );
                }
                push_event_at(frame, sample_offset, AudioEventKind::NoteOff { voice });
                if release {
                    self.bank2_id14_active = false;
                    self.sfx_release_pending_mask |= voice_bit;
                } else {
                    self.bank2_id14_active = true;
                }
                stats.note_events += 1;
            }
            Bank2Id14Action::KeyOn { pitch_word, volume } => {
                let rate_counter = match (voice, pitch_word) {
                    (5, 3_608) => 19,
                    (5, 1_802) => 71,
                    (5, 4_292) => 135,
                    (7, 1_802) => 69,
                    (7, 4_292) => 71,
                    _ => 0,
                };
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice,
                        enabled: false,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice,
                        source: 20,
                        adsr1: 254,
                        adsr2: 106,
                        gain: 184,
                        volume_left: volume,
                        volume_right: volume,
                        rate_counter,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchWord { voice, pitch_word },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice,
                        left: volume,
                        right: volume,
                    },
                );
                self.bank2_id14_active = true;
                stats.note_events += 1;
            }
        }
    }

    fn advance_bank2_id11_actions(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for pending in &mut self.pending_bank2_id11_actions {
            pending.samples_remaining = pending.samples_remaining.saturating_sub(534);
        }
        let mut ready = Vec::new();
        let mut index = 0;
        while index < self.pending_bank2_id11_actions.len() {
            if self.pending_bank2_id11_actions[index].samples_remaining < 534 {
                ready.push(self.pending_bank2_id11_actions.remove(index));
            } else {
                index += 1;
            }
        }
        for pending in ready {
            self.emit_bank2_id11_action(
                frame,
                pending.samples_remaining as i32,
                pending.action,
                stats,
            );
        }
    }

    fn emit_bank2_id11_action(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        action: Bank2Id11Action,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let voice = self.bank2_id11_voice;
        let voice_bit = 1 << voice;
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        match action {
            Bank2Id11Action::KeyOff { release } => {
                push_event_at(frame, sample_offset, AudioEventKind::NoteOff { voice });
                if release {
                    self.bank2_id11_active = false;
                    self.sfx_release_pending_mask |= voice_bit;
                } else {
                    self.bank2_id11_active = true;
                }
                stats.note_events += 1;
            }
            Bank2Id11Action::KeyOn {
                initial_pitch,
                pitch_word,
                rate_counter,
            } => {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice,
                        // This program does not write EON; inherit the voice's
                        // current music/driver echo routing.
                        enabled: self.music_echo_mask & voice_bit != 0,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice,
                        source: 11,
                        adsr1: 254,
                        adsr2: 106,
                        gain: 184,
                        volume_left: 63,
                        volume_right: 63,
                        rate_counter,
                    },
                );
                for pitch_word in [initial_pitch, pitch_word] {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetPitchWord { voice, pitch_word },
                    );
                }
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice,
                        left: 63,
                        right: 63,
                    },
                );
                self.bank2_id11_active = true;
                stats.note_events += 1;
            }
            Bank2Id11Action::Pitch { pitch_word } => push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord { voice, pitch_word },
            ),
        }
    }

    fn emit_port3_id15_chord(
        &mut self,
        frame: &mut AudioEventFrame,
        sample_offset: i32,
        index: u8,
        stats: &mut ModernAudioSequenceStats,
    ) {
        const PITCHES: [[u16; 5]; 4] = [
            [1_350, 900, 2_145, 3_213, 5_412],
            [1_431, 0, 2_271, 3_405, 5_733],
            [1_515, 0, 2_406, 3_606, 6_075],
            [1_605, 1_071, 2_550, 3_822, 6_438],
        ];
        const RATE_COUNTERS: [[u16; 5]; 4] = [
            [167, 0, 295, 295, 0],
            [7, 0, 7, 7, 5],
            [7, 0, 7, 7, 7],
            [7, 133, 7, 7, 7],
        ];
        const SHIFTED_RATE_COUNTERS: [[u16; 5]; 4] = [
            [0, 39, 0, 167, 167],
            [5, 0, 5, 7, 7],
            [7, 0, 7, 7, 7],
            [7, 199, 7, 7, 7],
        ];
        let pitches = PITCHES[usize::from(index)];
        for (slot, voice) in self.port3_id15_voices.into_iter().enumerate() {
            let pitch_word = pitches[slot];
            if pitch_word == 0 {
                continue;
            }
            let is_voice3 = slot == 1;
            let volume = if slot == 4 {
                75
            } else if is_voice3 && index == 3 {
                6
            } else {
                63
            };
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetNoteOrigin {
                    voice,
                    origin: AudioNoteOrigin::Sfx,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::KeyOnVoice {
                    voice,
                    source: if is_voice3 { 2 } else { 11 },
                    adsr1: 254,
                    adsr2: 106,
                    gain: if is_voice3 { 127 } else { 184 },
                    volume_left: volume,
                    volume_right: volume,
                    rate_counter: if self.port3_id15_voices[0] == 1 {
                        SHIFTED_RATE_COUNTERS[usize::from(index)][slot]
                    } else {
                        RATE_COUNTERS[usize::from(index)][slot]
                    },
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord { voice, pitch_word },
            );
            if index == 0 || (is_voice3 && index == 3) {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice,
                        left: volume,
                        right: volume,
                    },
                );
            }
            if is_voice3 {
                self.port3_id15_voice3_volume = volume;
            }
            self.port3_id15_owned_mask |= 1 << voice;
            stats.note_events += 1;
        }
    }

    fn advance_sfx_ownership(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for voice in 0..self.sfx_ownership_samples_remaining.len() {
            let remaining = &mut self.sfx_ownership_samples_remaining[voice];
            if *remaining == 0 {
                continue;
            }
            *remaining = remaining.saturating_sub(534);
            if *remaining < 534 {
                push_event_at(
                    frame,
                    *remaining as i32,
                    AudioEventKind::NoteOff { voice: voice as u8 },
                );
                *remaining = 0;
                self.sfx_release_pending_mask |= 1 << voice;
                stats.note_events += 1;
            }
        }
    }

    fn release_finished_sfx_ownership(&mut self) {
        let finished = std::mem::take(&mut self.sfx_release_pending_mask);
        self.sfx_voice_mask &= !finished;
        for voice in 0..8 {
            if finished & (1 << voice) != 0 && self.music_voice_mask & (1 << voice) == 0 {
                self.active_voice_mask &= !(1 << voice);
            }
        }
    }

    fn advance_sfx_release_overflows(&mut self) {
        let elapsed = sfx_overflow_count_in_frame(self.dsp_timer_cycles, self.sfx_timer_accum);
        for voice in 0..self.sfx_release_overflows_remaining.len() {
            let remaining = &mut self.sfx_release_overflows_remaining[voice];
            if *remaining == 0 {
                continue;
            }
            *remaining = remaining.saturating_sub(elapsed);
            if *remaining == 0 {
                let voice_bit = 1 << voice;
                self.sfx_voice_mask &= !voice_bit;
                if self.music_voice_mask & voice_bit == 0 {
                    self.active_voice_mask &= !voice_bit;
                }
            }
        }
    }

    fn advance_music_keyoffs(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        for voice in 0..self.music_keyoff_frames_remaining.len() {
            let remaining = &mut self.music_keyoff_frames_remaining[voice];
            if *remaining == 0 {
                continue;
            }
            *remaining -= 1;
            if *remaining == 0 {
                push_event_at(
                    frame,
                    i32::from(self.music_keyoff_sample_offset[voice]),
                    AudioEventKind::NoteOff { voice: voice as u8 },
                );
                // Keep ownership latched for the track. A KOF and the next
                // music KON can share a DSP sample; clearing this mask after
                // sequencing would lose the following note's later KOF.
                stats.note_events += 1;
            }
        }
    }

    fn advance_sfx_pitch_changes(&mut self, frame: &mut AudioEventFrame) {
        for voice in 0..self.pending_sfx_pitch_changes.len() {
            let changes = &mut self.pending_sfx_pitch_changes[voice];
            let mut index = 0;
            while index < changes.len() {
                changes[index].samples_remaining =
                    changes[index].samples_remaining.saturating_sub(534);
                // Offset 534 is the first sample of the following native DSP
                // frame, not the final sample of the current one.
                if changes[index].samples_remaining < 534 {
                    let change = changes.remove(index);
                    if self.rising_warble_long_pattern[voice]
                        && self.sfx_voice_activation_count[voice] == 2
                    {
                        self.frame_warble_pitch_events.push((
                            voice as u8,
                            change.samples_remaining as i32,
                            change.pitch_word,
                        ));
                    }
                    push_event_at(
                        frame,
                        change.samples_remaining as i32,
                        AudioEventKind::SetPitchRegisterWord {
                            voice: voice as u8,
                            pitch_word: change.pitch_word,
                        },
                    );
                } else {
                    index += 1;
                }
            }
        }
    }

    fn advance_sfx_volume_changes(&mut self, frame: &mut AudioEventFrame) {
        for voice in 0..self.pending_sfx_volume_changes.len() {
            let changes = &mut self.pending_sfx_volume_changes[voice];
            let mut index = 0;
            while index < changes.len() {
                changes[index].samples_remaining =
                    changes[index].samples_remaining.saturating_sub(534);
                if changes[index].samples_remaining < 534 {
                    let change = changes.remove(index);
                    if self.rising_warble_long_pattern[voice]
                        && self.sfx_voice_activation_count[voice] == 2
                    {
                        self.frame_warble_volume_events.push((
                            voice as u8,
                            change.samples_remaining as i32,
                            change.left,
                            change.right,
                        ));
                    }
                    push_event_at(
                        frame,
                        change.samples_remaining as i32,
                        AudioEventKind::SetStereoVolume {
                            voice: voice as u8,
                            left: change.left,
                            right: change.right,
                        },
                    );
                } else {
                    index += 1;
                }
            }
        }
    }

    fn advance_persistent_sfx_pitch_refreshes(&self, frame: &mut AudioEventFrame) {
        let Some(clock) = self.previous_sfx_clock else {
            return;
        };
        let positions = sfx_overflow_positions_after(clock, -1, 3);
        for voice in 0..8 {
            let pitch_word = self.persistent_sfx_pitch_words[voice];
            if pitch_word == 0 || self.sfx_voice_mask & (1 << voice) == 0 {
                continue;
            }
            for &position in positions.iter().take_while(|position| **position < 534) {
                push_event_at(
                    frame,
                    position as i32,
                    AudioEventKind::SetPitchRegisterWord {
                        voice: voice as u8,
                        pitch_word,
                    },
                );
            }
        }
    }

    fn advance_looping_sfx(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some((timer_cycles, mut sfx_timer_accum)) = self.previous_sfx_clock else {
            return;
        };
        let first_boundary = sfx_first_boundary(timer_cycles);
        let mut sample_offset = u16::from(first_boundary);
        while sample_offset < 534 {
            let sum = u16::from(sfx_timer_accum) + 0x38;
            sfx_timer_accum = sum as u8;
            if sum >= 0x100 {
                let mut retriggers = Vec::new();
                let mut finished_mask = 0u8;
                for voice in 0..self.looping_sfx_voices.len() {
                    let Some(looping) = self.looping_sfx_voices[voice].as_mut() else {
                        continue;
                    };
                    looping.overflows_remaining = looping.overflows_remaining.saturating_sub(1);
                    if looping.overflows_remaining != 0 {
                        continue;
                    }
                    if looping.active {
                        self.pending_sfx_pitch_changes[voice].clear();
                        push_event_at(
                            frame,
                            i32::from(sample_offset),
                            AudioEventKind::NoteOff { voice: voice as u8 },
                        );
                        if looping.retriggers_remaining == 0 {
                            finished_mask |= 1 << voice;
                        } else {
                            looping.active = false;
                            looping.overflows_remaining = looping.gap_overflows;
                        }
                        stats.note_events += 1;
                    } else {
                        let mut exact = looping.exact;
                        if let Some(pitch_word) = looping
                            .staggered_chime
                            .then(|| {
                                staggered_chime_retrigger_pitch(
                                    looping.step.voice,
                                    looping.retrigger_index,
                                )
                            })
                            .flatten()
                        {
                            exact.dsp_pitch = pitch_word;
                        }
                        retriggers.push(PendingSfxStep {
                            step: looping.step,
                            exact: Some(exact),
                            engine_dsp_envelope: None,
                            delay_after_previous: 0,
                            preserve_existing_volume: false,
                            // A loop KON reuses the already-programmed DSP
                            // volume. Parameter writes preserve that behavior
                            // while still making the NoteOn self-describing.
                            volume_via_parameters: !(looping.staggered_chime
                                && looping.step.voice == 2
                                && matches!(looping.retrigger_index, 0 | 3)),
                            refresh_repeat_on_keyon: false,
                            preserve_inactive_pitch_latch: false,
                            engine_keyoff_owned: false,
                        });
                        looping.retriggers_remaining =
                            looping.retriggers_remaining.saturating_sub(1);
                        looping.retrigger_index = looping.retrigger_index.saturating_add(1);
                        looping.active = true;
                        looping.overflows_remaining = if looping.staggered_chime
                            && staggered_chime_retrigger_pitch(
                                looping.step.voice,
                                looping.retrigger_index,
                            )
                            .is_none()
                            && looping.retriggers_remaining == 0
                        {
                            34
                        } else {
                            looping.active_overflows
                        };
                    }
                }
                for pending in retriggers {
                    self.emit_sfx_step_at(frame, pending, i32::from(sample_offset), stats);
                }
                for voice in 0..8 {
                    if finished_mask & (1 << voice) != 0 {
                        self.looping_sfx_voices[voice] = None;
                        self.sfx_release_pending_mask |= 1 << voice;
                    }
                }
            }
            sample_offset += 64;
        }
    }

    fn emit_semantic_sfx_echo_changes(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_echo_count.min(8)) {
            let mask = spc.sfx_echo_masks[event_index];
            let enabled = spc.sfx_echo_enabled[event_index];
            let sample_offset = i32::from(spc.sfx_echo_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) != 0 {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetEchoSend {
                            voice: voice as u8,
                            enabled,
                        },
                    );
                }
            }
        }
    }

    fn emit_semantic_sfx_pitch_changes(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_pitch_count.min(32)) {
            let mask = spc.sfx_pitch_masks[event_index];
            let pitch_word = spc.sfx_pitch_words[event_index];
            let sample_offset = i32::from(spc.sfx_pitch_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) != 0 {
                    if self.rising_warble_long_pattern[voice]
                        && self.sfx_voice_activation_count[voice] >= 2
                    {
                        continue;
                    }
                    frame.events.retain(|event| {
                        !(event.sample_offset >= sample_offset
                            && matches!(
                                event.kind,
                                AudioEventKind::SetPitchWord { voice: event_voice, .. }
                                    | AudioEventKind::SetPitchRegisterWord { voice: event_voice, .. }
                                    if usize::from(event_voice) == voice
                            ))
                    });
                    self.pending_sfx_pitch_changes[voice].clear();
                    let has_key_on = frame.events.iter().any(|event| {
                        matches!(
                            event.kind,
                            AudioEventKind::NoteOn {
                                voice: event_voice,
                                ..
                            } | AudioEventKind::KeyOnVoice {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        )
                    });
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetNoteOrigin {
                            voice: voice as u8,
                            origin: AudioNoteOrigin::Sfx,
                        },
                    );
                    push_event_at(
                        frame,
                        sample_offset,
                        if !has_key_on {
                            AudioEventKind::SetPitchRegisterWord {
                                voice: voice as u8,
                                pitch_word,
                            }
                        } else {
                            AudioEventKind::SetPitchWord {
                                voice: voice as u8,
                                pitch_word,
                            }
                        },
                    );
                }
            }
        }
    }

    fn emit_semantic_sfx_volume_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_volume_count.min(32)) {
            let semantic_volume_mask = if spc.spc_in[1] == 0x05 {
                u8::MAX
            } else {
                self.engine_automated_sfx_volume_mask | 0x80
            };
            let mask = spc.sfx_volume_masks[event_index] & semantic_volume_mask;
            let sample_offset = i32::from(spc.sfx_volume_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                if self.rising_warble_long_pattern[voice]
                    && self.sfx_voice_activation_count[voice] >= 2
                {
                    continue;
                }
                frame.events.retain(|event| {
                    !(event.sample_offset >= sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::SetStereoVolume {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: voice as u8,
                        left: spc.sfx_volume_left[event_index],
                        right: spc.sfx_volume_right[event_index],
                    },
                );
            }
        }
    }

    fn emit_raw_volume_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.raw_volume_count.min(32)) {
            let mask = spc.raw_volume_masks[index];
            let sample_offset = i32::from(spc.raw_volume_offsets[index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                if self.rising_warble_long_pattern[voice]
                    && self.sfx_voice_activation_count[voice] >= 2
                {
                    continue;
                }
                frame.events.retain(|event| {
                    !(event.sample_offset == sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::SetStereoVolume {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: voice as u8,
                        left: spc.raw_volume_left[index],
                        right: spc.raw_volume_right[index],
                    },
                );
            }
        }
    }

    fn emit_raw_envelope_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.raw_envelope_count.min(32)) {
            let parameter = match spc.raw_envelope_registers[index] {
                5 => VoiceParameterKind::Adsr1,
                6 => VoiceParameterKind::Adsr2,
                7 => VoiceParameterKind::Gain,
                _ => continue,
            };
            let mask = spc.raw_envelope_masks[index];
            let sample_offset = i32::from(spc.raw_envelope_offsets[index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::VoiceParameter {
                        voice: voice as u8,
                        parameter,
                        value: spc.raw_envelope_values[index],
                    },
                );
            }
        }
    }

    fn emit_raw_pitch_changes(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        latch_release_offsets: [Option<i32>; 8],
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.raw_pitch_count.min(128)) {
            let (event_mask, pitch_word, raw_offset) = raw_pitch_event(&spc, index);
            let sample_offset = i32::from(raw_offset);
            for voice in 0..8 {
                if event_mask & (1 << voice) == 0 {
                    continue;
                }
                if self.rising_warble_long_pattern[voice]
                    && self.sfx_voice_activation_count[voice] >= 2
                {
                    continue;
                }
                frame.events.retain(|event| {
                    !(event.sample_offset >= sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::SetPitchWord { voice: event_voice, .. }
                                | AudioEventKind::SetPitchRegisterWord { voice: event_voice, .. }
                                if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    if self.semantic_pitch_latch_mask & (1 << voice) != 0
                        && latch_release_offsets[voice]
                            .is_none_or(|release_offset| sample_offset <= release_offset)
                    {
                        AudioEventKind::SetPitchRegisterWord {
                            voice: voice as u8,
                            pitch_word,
                        }
                    } else {
                        AudioEventKind::SetPitchWord {
                            voice: voice as u8,
                            pitch_word,
                        }
                    },
                );
            }
        }
    }

    fn emit_raw_echo_enable_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.echo_enable_count.min(16)) {
            let sample_offset = i32::from(spc.echo_enable_offsets[index]);
            let mask = spc.echo_enable_values[index];
            for voice in 0..8 {
                frame.events.retain(|event| {
                    !(event.sample_offset >= sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::SetEchoSend {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetEchoSend {
                        voice: voice as u8,
                        enabled: mask & (1 << voice) != 0,
                    },
                );
            }
        }
    }

    fn emit_raw_music_keyoffs(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.raw_kof_count.min(32)) {
            let mask = spc.raw_kof_masks[index];
            let sample_offset = i32::from(spc.raw_kof_offsets[index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                if self.rising_warble_long_pattern[voice]
                    && self.sfx_voice_activation_count[voice] >= 2
                {
                    continue;
                }
                if !frame.events.iter().any(|event| {
                    event.sample_offset == sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOff { voice: event_voice }
                                if usize::from(event_voice) == voice
                        )
                }) {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::NoteOff { voice: voice as u8 },
                    );
                    stats.note_events += 1;
                }
                self.mark_music_voice_inactive(voice as u8);
            }
        }
    }

    fn emit_raw_echo_volume_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.echo_volume_count.min(32)) {
            push_event_at(
                frame,
                i32::from(spc.echo_volume_offsets[index]),
                AudioEventKind::GlobalParameter {
                    register: spc.echo_volume_registers[index],
                    value: spc.echo_volume_values[index],
                },
            );
        }
    }

    fn emit_raw_global_changes(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.global_count.min(32)) {
            let register = spc.global_registers[index];
            let value = spc.global_values[index];
            let sample_offset = i32::from(spc.global_offsets[index]);
            frame.events.retain(|event| {
                !(event.sample_offset == sample_offset
                    && matches!(
                        event.kind,
                        AudioEventKind::GlobalParameter {
                            register: event_register,
                            ..
                        } if event_register == register
                    ))
            });
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::GlobalParameter { register, value },
            );
        }
    }

    fn emit_port3_allocator_keyoffs(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_kof_count.min(8)) {
            let mask = spc.sfx_kof_masks[event_index];
            let sample_offset = i32::from(spc.sfx_kof_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) != 0 {
                    frame.events.retain(|event| {
                        !(event.sample_offset > sample_offset
                            && matches!(
                                event.kind,
                                AudioEventKind::SetPitchWord { voice: event_voice, .. }
                                    | AudioEventKind::SetPitchRegisterWord { voice: event_voice, .. }
                                    if usize::from(event_voice) == voice
                            ))
                    });
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetNoteOrigin {
                            voice: voice as u8,
                            origin: AudioNoteOrigin::Sfx,
                        },
                    );
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::NoteOff { voice: voice as u8 },
                    );
                    stats.note_events += 1;
                    self.pending_sfx_pitch_changes[voice].clear();
                    if spc.is_chan_on & (1 << voice) != 0 {
                        self.sfx_voice_mask |= 1 << voice;
                        self.active_voice_mask |= 1 << voice;
                        self.sfx_release_pending_mask &= !(1 << voice);
                        self.sfx_release_overflows_remaining[voice] = 0;
                    }
                }
            }
        }
    }

    fn emit_semantic_sfx_keyons(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        processed_masks: &mut [u8; 8],
        pitch_latch_release_offsets: &mut [Option<i32>; 8],
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for event_index in 0..usize::from(spc.sfx_kon_count.min(8)) {
            let receipt_mask = spc.sfx_kon_owned_masks[event_index];
            let receipt_offset = i32::from(spc.sfx_kon_offsets[event_index]);
            for voice in 0..8 {
                if receipt_mask & processed_masks[event_index] & (1 << voice) == 0 {
                    continue;
                }
                let has_semantic_key_on = frame.events.iter().any(|event| {
                    event.sample_offset == receipt_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::KeyOnVoice {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        )
                });
                // The pre-sequence receipt already transferred this voice to
                // the engine. `sequence_sfx` may nevertheless have expanded
                // the just-received command and queued catalog pitch changes
                // before this post-sequence cleanup pass. Drop those future
                // synthetic writes as well as the frame-local events below;
                // otherwise they leak into the next frame and retune the
                // engine-owned voice after its KON.
                self.pending_sfx_pitch_changes[voice].clear();
                // A semantic KON already emitted on the pre-sequence pass owns
                // this voice for the remainder of the frame. Discard catalog
                // automation queued later in the same frame without a
                // corresponding engine receipt.
                let ownership_start = (0..event_index)
                    .filter(|&index| spc.sfx_kon_masks[index] & (1 << voice) != 0)
                    .map(|index| i32::from(spc.sfx_kon_offsets[index]) + 1)
                    .max()
                    .unwrap_or(0);
                frame.events.retain(|event| {
                    let conflicting_note_on = event.sample_offset >= ownership_start
                        && ((has_semantic_key_on && event.sample_offset == receipt_offset)
                            || event.sample_offset != receipt_offset)
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOn {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        );
                    let conflicting_automation = (event.sample_offset > receipt_offset
                        || (has_semantic_key_on && event.sample_offset == receipt_offset)
                        || (event.sample_offset >= ownership_start
                            && event.sample_offset < receipt_offset))
                        && matches!(
                            event.kind,
                            AudioEventKind::SetNoteOrigin { voice: event_voice, .. }
                                | AudioEventKind::SetPitchWord { voice: event_voice, .. }
                                | AudioEventKind::SetPitchRegisterWord { voice: event_voice, .. }
                                | AudioEventKind::SetNoise { voice: event_voice, .. }
                                | AudioEventKind::SetPan { voice: event_voice, .. }
                                | AudioEventKind::SetEchoSend { voice: event_voice, .. }
                                | AudioEventKind::SetEnvelope { voice: event_voice, .. }
                                | AudioEventKind::SetStereoVolume { voice: event_voice, .. }
                                | AudioEventKind::SetDspEnvelope { voice: event_voice, .. }
                                | AudioEventKind::PitchSlide { voice: event_voice, .. }
                                | AudioEventKind::SetDuration { voice: event_voice, .. }
                                if usize::from(event_voice) == voice
                        );
                    !conflicting_note_on && !conflicting_automation
                });
            }
            let mask = spc.sfx_kon_owned_masks[event_index] & !processed_masks[event_index];
            let sample_offset = i32::from(spc.sfx_kon_offsets[event_index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                let receipt_source = spc.sfx_kon_sources[event_index][voice];
                if self.bank1_id95_active
                    && voice == usize::from(self.bank1_id95_voice)
                    && matches!(receipt_source, 1 | 20)
                {
                    processed_masks[event_index] |= 1 << voice;
                    continue;
                }
                if spc.sfx_kon_sources[event_index][voice] == 21
                    && frame.events.iter().any(|event| {
                        event.sample_offset == sample_offset
                            && matches!(
                                event.kind,
                                AudioEventKind::KeyOnVoice {
                                    voice: target,
                                    source: 21,
                                    ..
                                } if usize::from(target) != voice
                                    && self.rising_warble_long_pattern[usize::from(target)]
                            )
                    })
                {
                    processed_masks[event_index] |= 1 << voice;
                    continue;
                }
                if self.rising_warble_long_pattern[voice]
                    && self.sfx_voice_activation_count[voice] >= 2
                {
                    continue;
                }
                if self.semantic_pitch_latch_mask & (1 << voice) != 0 {
                    pitch_latch_release_offsets[voice] = Some(sample_offset);
                }
                let receipt_echo_mask = semantic_echo_mask_at(&spc, sample_offset);
                let receipt_pitch = (0..usize::from(spc.raw_pitch_count.min(128)))
                    .rev()
                    .find(|&index| {
                        let (mask, _, offset) = raw_pitch_event(&spc, index);
                        mask & (1 << voice) != 0 && i32::from(offset) == sample_offset
                    })
                    .map(|index| raw_pitch_event(&spc, index).1)
                    .or_else(|| {
                        (0..usize::from(spc.sfx_pitch_count.min(32)))
                            .rev()
                            .find(|&index| {
                                spc.sfx_pitch_masks[index] & (1 << voice) != 0
                                    && i32::from(spc.sfx_pitch_offsets[index]) == sample_offset
                            })
                            .map(|index| spc.sfx_pitch_words[index])
                    });
                if !self.semantic_sfx_pending_steps[voice].is_empty()
                    || self.semantic_sfx_repeat_steps[voice].is_some()
                {
                    // Loop automation runs before semantic receipts. Replace a
                    // coincident synthetic trigger with the engine-confirmed
                    // pending/repeat definition instead of layering both.
                    frame.events.retain(|event| {
                        !(event.sample_offset >= sample_offset
                            && matches!(
                                    event.kind,
                                    AudioEventKind::NoteOn {
                                        voice: event_voice,
                                        ..
                                    } if usize::from(event_voice) == voice
                            ))
                    });
                }
                let setup_index =
                    (0..usize::from(spc.sfx_setup_count.min(8)))
                        .rev()
                        .find(|&index| {
                            spc.sfx_setup_masks[index] & (1 << voice) != 0
                                && i32::from(spc.sfx_setup_offsets[index]) <= sample_offset
                        });
                let setup_index = setup_index.filter(|&index| {
                    if i32::from(spc.sfx_setup_offsets[index]) < sample_offset {
                        return true;
                    }
                    if frame.events.iter().any(|event| {
                        event.sample_offset < i32::from(spc.sfx_setup_offsets[index])
                            && matches!(
                                event.kind,
                                AudioEventKind::NoteOn {
                                    voice: event_voice,
                                    ..
                                } if usize::from(event_voice) == voice
                            )
                    }) {
                        return true;
                    }
                    let source = spc.sfx_setup_sources[index];
                    let pending_matches =
                        self.semantic_sfx_pending_steps[voice]
                            .iter()
                            .any(|pending| {
                                pending.step.instrument == source
                                    && receipt_pitch.is_none_or(|pitch| {
                                        pending.exact.is_some_and(|exact| exact.dsp_pitch == pitch)
                                    })
                            });
                    let repeat_matches =
                        self.semantic_sfx_repeat_steps[voice].is_some_and(|repeat| {
                            repeat.step.instrument == source
                                && receipt_pitch.is_none_or(|pitch| {
                                    repeat.exact.is_some_and(|exact| exact.dsp_pitch == pitch)
                                })
                        });
                    !pending_matches && !repeat_matches
                });
                if let Some(setup_index) = setup_index {
                    self.pending_sfx_pitch_changes[voice].clear();
                    self.engine_automated_sfx_volume_mask |= 1 << voice;
                    let setup_offset = i32::from(spc.sfx_setup_offsets[setup_index]);
                    let ownership_start = (0..event_index)
                        .filter(|&index| spc.sfx_kon_masks[index] & (1 << voice) != 0)
                        .map(|index| i32::from(spc.sfx_kon_offsets[index]) + 1)
                        .max()
                        .unwrap_or(0);
                    frame.events.retain(|event| {
                        !(event.sample_offset >= ownership_start
                            && matches!(
                                event.kind,
                                AudioEventKind::NoteOn {
                                    voice: event_voice,
                                    ..
                                } if usize::from(event_voice) == voice
                            ))
                    });
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetNoteOrigin {
                            voice: voice as u8,
                            origin: AudioNoteOrigin::Sfx,
                        },
                    );
                    if let Some(pitch_word) = receipt_pitch {
                        push_event_at(
                            frame,
                            sample_offset,
                            AudioEventKind::SetPitchWord {
                                voice: voice as u8,
                                pitch_word,
                            },
                        );
                    }
                    let setup_volume_index = (0..usize::from(spc.sfx_volume_count.min(32)))
                        .rev()
                        .find(|&index| {
                            spc.sfx_volume_masks[index] & (1 << voice) != 0
                                && i32::from(spc.sfx_volume_offsets[index]) <= sample_offset
                                && i32::from(spc.sfx_volume_offsets[index]) >= setup_offset
                        });
                    if let Some(volume_index) = setup_volume_index {
                        push_event_at(
                            frame,
                            i32::from(spc.sfx_volume_offsets[volume_index]),
                            AudioEventKind::SetStereoVolume {
                                voice: voice as u8,
                                left: spc.sfx_volume_left[volume_index],
                                right: spc.sfx_volume_right[volume_index],
                            },
                        );
                    }
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::KeyOnVoice {
                            voice: voice as u8,
                            source: spc.sfx_kon_sources[event_index][voice],
                            adsr1: spc.sfx_kon_adsr1[event_index][voice],
                            adsr2: spc.sfx_kon_adsr2[event_index][voice],
                            gain: spc.sfx_kon_gain[event_index][voice],
                            volume_left: spc.sfx_kon_volume_left[event_index][voice],
                            volume_right: spc.sfx_kon_volume_right[event_index][voice],
                            rate_counter: spc.sfx_kon_rate_counters[event_index][voice],
                        },
                    );
                    if let Some(mut repeat) = self.semantic_sfx_repeat_steps[voice] {
                        repeat.step.instrument = spc.sfx_setup_sources[setup_index];
                        repeat.step.echo = receipt_echo_mask & (1 << voice) != 0;
                        if let Some(mut exact) = repeat.exact {
                            exact.instrument = spc.sfx_setup_sources[setup_index];
                            exact.adsr1 = spc.sfx_setup_adsr1[setup_index];
                            exact.adsr2 = spc.sfx_setup_adsr2[setup_index];
                            exact.gain = spc.sfx_setup_gain[setup_index];
                            if let Some(pitch_word) = receipt_pitch {
                                exact.dsp_pitch = pitch_word;
                            }
                            exact.echo = repeat.step.echo;
                            if let Some(volume_index) = setup_volume_index {
                                repeat.step.volume = spc.sfx_volume_left[volume_index]
                                    .unsigned_abs()
                                    .max(spc.sfx_volume_right[volume_index].unsigned_abs());
                                exact.volume = repeat.step.volume;
                                exact.volume_left = spc.sfx_volume_left[volume_index];
                                exact.volume_right = spc.sfx_volume_right[volume_index];
                            }
                            repeat.exact = Some(exact);
                        }
                        self.semantic_sfx_repeat_steps[voice] = Some(repeat);
                    }
                    processed_masks[event_index] |= 1 << voice;
                    stats.note_events += 1;
                    continue;
                }
                if !self.semantic_sfx_pending_steps[voice].is_empty() {
                    let matching_pending = receipt_pitch.and_then(|receipt_pitch| {
                        self.semantic_sfx_pending_steps[voice]
                            .iter()
                            .position(|pending| {
                                pending
                                    .exact
                                    .is_some_and(|exact| exact.dsp_pitch == receipt_pitch)
                            })
                    });
                    let mut pending = if let Some(index) = matching_pending {
                        self.semantic_sfx_pending_steps[voice].remove(index)
                    } else if receipt_pitch.is_some() {
                        self.semantic_sfx_repeat_steps[voice]
                            .unwrap_or_else(|| self.semantic_sfx_pending_steps[voice].remove(0))
                    } else {
                        self.semantic_sfx_pending_steps[voice].remove(0)
                    };
                    apply_semantic_voice_state(&mut pending, &spc, voice);
                    if let (Some(pitch_word), Some(mut exact)) = (receipt_pitch, pending.exact) {
                        exact.dsp_pitch = pitch_word;
                        pending.exact = Some(exact);
                    }
                    pending.step.echo = receipt_echo_mask & (1 << voice) != 0;
                    if let Some(mut exact) = pending.exact {
                        exact.echo = pending.step.echo;
                        pending.exact = Some(exact);
                    }
                    let has_volume_receipt =
                        apply_semantic_volume(&mut pending, &spc, voice, sample_offset);
                    if !has_volume_receipt {
                        pending.preserve_existing_volume = true;
                    }
                    if spc.sfx_kon_masks[event_index].count_ones() > 1
                        && pending.step.voice == 5
                        && pending.step.pitch == 10
                        && pending.step.instrument == 10
                        && pending.step.volume == 0
                    {
                        pending.preserve_existing_volume = true;
                    }
                    if pending.refresh_repeat_on_keyon {
                        self.semantic_sfx_repeat_steps[voice] = Some(pending);
                    }
                    pending.engine_keyoff_owned = true;
                    self.emit_sfx_step_at(frame, pending, sample_offset, stats);
                    processed_masks[event_index] |= 1 << voice;
                } else if let Some(mut pending) = self.semantic_sfx_repeat_steps[voice] {
                    apply_semantic_voice_state(&mut pending, &spc, voice);
                    if let (Some(pitch_word), Some(mut exact)) = (receipt_pitch, pending.exact) {
                        exact.dsp_pitch = pitch_word;
                        pending.exact = Some(exact);
                    }
                    pending.step.echo = receipt_echo_mask & (1 << voice) != 0;
                    if let Some(mut exact) = pending.exact {
                        exact.echo = pending.step.echo;
                        pending.exact = Some(exact);
                    }
                    let has_volume_receipt =
                        apply_semantic_volume(&mut pending, &spc, voice, sample_offset);
                    if !has_volume_receipt {
                        pending.preserve_existing_volume = true;
                    }
                    if spc.sfx_kon_masks[event_index].count_ones() > 1
                        && pending.step.voice == 5
                        && pending.step.pitch == 10
                        && pending.step.instrument == 10
                        && pending.step.volume == 0
                    {
                        pending.preserve_existing_volume = true;
                    }
                    pending.engine_keyoff_owned = true;
                    self.emit_sfx_step_at(frame, pending, sample_offset, stats);
                    processed_masks[event_index] |= 1 << voice;
                } else {
                    self.pending_sfx_pitch_changes[voice].clear();
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetNoteOrigin {
                            voice: voice as u8,
                            origin: AudioNoteOrigin::Sfx,
                        },
                    );
                    if let Some(pitch_word) = receipt_pitch {
                        push_event_at(
                            frame,
                            sample_offset,
                            AudioEventKind::SetPitchWord {
                                voice: voice as u8,
                                pitch_word,
                            },
                        );
                    }
                    let (left, right) = semantic_volume_at(&spc, voice, sample_offset)
                        .unwrap_or((spc.voice_volume_left[voice], spc.voice_volume_right[voice]));
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::SetStereoVolume {
                            voice: voice as u8,
                            left,
                            right,
                        },
                    );
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::KeyOnVoice {
                            voice: voice as u8,
                            source: spc.sfx_kon_sources[event_index][voice],
                            adsr1: spc.sfx_kon_adsr1[event_index][voice],
                            adsr2: spc.sfx_kon_adsr2[event_index][voice],
                            gain: spc.sfx_kon_gain[event_index][voice],
                            volume_left: spc.sfx_kon_volume_left[event_index][voice],
                            volume_right: spc.sfx_kon_volume_right[event_index][voice],
                            rate_counter: spc.sfx_kon_rate_counters[event_index][voice],
                        },
                    );
                    processed_masks[event_index] |= 1 << voice;
                    stats.note_events += 1;
                }
            }
            for voice in 0..8 {
                if receipt_mask & (1 << voice) == 0 {
                    continue;
                }
                push_event_at(
                    frame,
                    receipt_offset,
                    AudioEventKind::SetEchoSend {
                        voice: voice as u8,
                        enabled: semantic_echo_mask_at(&spc, receipt_offset) & (1 << voice) != 0,
                    },
                );
            }
        }
    }

    fn emit_engine_music_keyons(
        &mut self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let Some(spc) = spc else {
            return;
        };
        for index in 0..usize::from(spc.sfx_kon_count.min(8)) {
            let mask = spc.sfx_kon_masks[index] & !spc.sfx_kon_owned_masks[index];
            let sample_offset = i32::from(spc.sfx_kon_offsets[index]);
            for voice in 0..8 {
                if mask & (1 << voice) == 0 {
                    continue;
                }
                if spc.sfx_kon_sources[index][voice] == 21
                    && frame.events.iter().any(|event| {
                        event.sample_offset == sample_offset
                            && matches!(
                                event.kind,
                                AudioEventKind::KeyOnVoice {
                                    voice: target,
                                    source: 21,
                                    ..
                                } if usize::from(target) != voice
                                    && self.rising_warble_long_pattern[usize::from(target)]
                            )
                    })
                {
                    continue;
                }
                frame.events.retain(|event| {
                    !(event.sample_offset == sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOn {
                                voice: event_voice,
                                ..
                            } | AudioEventKind::KeyOnVoice {
                                voice: event_voice,
                                ..
                            } if usize::from(event_voice) == voice
                        ))
                });
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetNoteOrigin {
                        voice: voice as u8,
                        origin: AudioNoteOrigin::Music,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::KeyOnVoice {
                        voice: voice as u8,
                        source: spc.sfx_kon_sources[index][voice],
                        adsr1: spc.sfx_kon_adsr1[index][voice],
                        adsr2: spc.sfx_kon_adsr2[index][voice],
                        gain: spc.sfx_kon_gain[index][voice],
                        volume_left: spc.sfx_kon_volume_left[index][voice],
                        volume_right: spc.sfx_kon_volume_right[index][voice],
                        rate_counter: spc.sfx_kon_rate_counters[index][voice],
                    },
                );
                self.mark_music_voice_active(voice as u8);
                stats.note_events += 1;
            }
        }
    }

    fn reconcile_semantic_sfx_keyoffs(
        &self,
        spc: Option<crate::game_output::SpcSequencerState>,
        frame: &mut AudioEventFrame,
    ) {
        let Some(spc) = spc else {
            return;
        };
        let ambient_reset_owns_keyoffs = spc.spc_out[0] == 0
            && spc.block_count == 0xff
            && frame
                .events
                .iter()
                .any(|event| matches!(event.kind, AudioEventKind::PlaySfx { bank: 0, id: 0x03 }));
        if ambient_reset_owns_keyoffs {
            return;
        }
        for voice in 0..8 {
            // Channel 7 has an independent delayed-keyoff scheduler whose
            // synthetic deadline can conflict with the engine's raw KOF.
            // Allocated voices 0-6 may legitimately receive both music and
            // SFX KOF writes in one frame, so their schedules are left intact.
            let has_sfx_receipt = (0..usize::from(spc.sfx_kof_count.min(8)))
                .any(|index| spc.sfx_kof_masks[index] & (1 << voice) != 0);
            if !has_sfx_receipt {
                continue;
            }
            let mut offsets = (0..usize::from(spc.raw_kof_count.min(32)))
                .filter(|&index| spc.raw_kof_masks[index] & (1 << voice) != 0)
                .map(|index| i32::from(spc.raw_kof_offsets[index]))
                .collect::<Vec<_>>();
            if voice != 7 {
                // Allocated SFX voices can receive KON followed by KOF at the
                // same DSP sample. The receipt passes run KOF before KON, so
                // re-append only these coincident raw KOFs after the semantic
                // key-on to preserve the engine's final write order. Do not
                // broadly adopt raw KOF for voices shared with music.
                offsets.retain(|&offset| {
                    (0..usize::from(spc.sfx_kon_count.min(8))).any(|index| {
                        spc.sfx_kon_masks[index] & (1 << voice) != 0
                            && i32::from(spc.sfx_kon_offsets[index]) == offset
                    })
                });
            }
            if offsets.is_empty() {
                continue;
            }
            if voice != 7 {
                frame.events.retain(|event| {
                    !(offsets.contains(&event.sample_offset)
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOff { voice: event_voice }
                                if usize::from(event_voice) == voice
                        ))
                });
                for sample_offset in offsets {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::NoteOff { voice: voice as u8 },
                    );
                }
                continue;
            }
            frame.events.retain(|event| {
                !matches!(
                    event.kind,
                    AudioEventKind::NoteOff { voice: event_voice }
                        if usize::from(event_voice) == voice
                            && !offsets.contains(&event.sample_offset)
                )
            });
            for sample_offset in offsets {
                if !frame.events.iter().any(|event| {
                    event.sample_offset == sample_offset
                        && matches!(
                            event.kind,
                            AudioEventKind::NoteOff { voice: event_voice }
                                if usize::from(event_voice) == voice
                        )
                }) {
                    push_event_at(
                        frame,
                        sample_offset,
                        AudioEventKind::NoteOff { voice: voice as u8 },
                    );
                }
            }
        }
        for index in 0..usize::from(spc.sfx_kof_count.min(8)) {
            let offset = i32::from(spc.sfx_kof_offsets[index]);
            let mask = spc.sfx_kof_masks[index];
            for voice in 0..8u8 {
                if mask & (1 << voice) != 0 {
                    mark_frame_note_off_at_as_sfx(frame, voice, offset);
                }
            }
        }
    }

    fn schedule_rising_warble_second_activation(
        &mut self,
        voice: u8,
        sample_offset: i32,
        frame: &mut AudioEventFrame,
    ) {
        let Some(clock) = self.previous_sfx_clock else {
            return;
        };
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetPitchWord {
                voice,
                pitch_word: RISING_WARBLE_SECOND_PITCHES[0],
            },
        );
        self.frame_warble_pitch_events.push((
            voice,
            sample_offset,
            RISING_WARBLE_SECOND_PITCHES[0],
        ));
        let positions =
            sfx_overflow_positions_after(clock, sample_offset, RISING_WARBLE_SECOND_PITCHES.len());
        for (&pitch_word, &samples_remaining) in
            RISING_WARBLE_SECOND_PITCHES[1..].iter().zip(&positions)
        {
            if samples_remaining < 534 {
                self.frame_warble_pitch_events
                    .push((voice, samples_remaining as i32, pitch_word));
                push_event_at(
                    frame,
                    samples_remaining as i32,
                    AudioEventKind::SetPitchWord { voice, pitch_word },
                );
            } else {
                self.pending_sfx_pitch_changes[usize::from(voice)].push(PendingSfxPitchChange {
                    // This activation is scheduled before this frame's
                    // pending-change pass. Compensate for that pass so the
                    // absolute APU-clock position is not advanced one frame.
                    samples_remaining: samples_remaining + 534,
                    pitch_word,
                });
            }
        }
        for (&overflow_index, &volume) in [19usize, 27, 35, 43, 51, 59, 67, 75, 83, 91]
            .iter()
            .zip(&[56i8, 50, 44, 37, 31, 25, 18, 12, 6, 3])
        {
            let samples_remaining = positions[overflow_index - 2];
            if samples_remaining < 534 {
                self.frame_warble_volume_events.push((
                    voice,
                    samples_remaining as i32,
                    volume,
                    volume,
                ));
                push_event_at(
                    frame,
                    samples_remaining as i32,
                    AudioEventKind::SetStereoVolume {
                        voice,
                        left: volume,
                        right: volume,
                    },
                );
            } else {
                self.pending_sfx_volume_changes[usize::from(voice)].push(PendingSfxVolumeChange {
                    // See the pitch queue above: the queue is decremented
                    // later in this same native DSP frame.
                    samples_remaining: samples_remaining + 534,
                    left: volume,
                    right: volume,
                });
            }
        }
        self.sfx_keyoff_samples_remaining[usize::from(voice)] =
            positions[RISING_WARBLE_SECOND_PITCHES.len() - 1];
        self.sfx_keyoff_starts_ownership_mask &= !(1 << voice);
    }

    fn schedule_port2_id30_automation_from_keyons(&mut self, frame: &mut AudioEventFrame) {
        let Some(clock) = self.previous_sfx_clock else {
            return;
        };
        let activations = frame
            .events
            .iter()
            .filter_map(|event| {
                let voice = match event.kind {
                    AudioEventKind::KeyOnVoice {
                        voice, source: 1, ..
                    }
                    | AudioEventKind::NoteOn {
                        voice,
                        instrument: 1,
                        ..
                    } => voice,
                    _ => return None,
                };
                (self.sfx_voice_program[usize::from(voice)] == 0x011e)
                    .then_some((voice, event.sample_offset))
            })
            .collect::<Vec<_>>();
        for (voice, sample_offset) in activations {
            let voice_index = usize::from(voice);
            let activation = self.sfx_voice_activation_count[voice_index];
            let (pitches, volume_changes): (&[u16], &[(usize, i8)]) = if activation == 0 {
                (&PORT2_ID30_FIRST_PITCHES, &[])
            } else {
                (&PORT2_ID30_SECOND_PITCHES, &PORT2_ID30_SECOND_VOLUMES)
            };
            self.sfx_voice_activation_count[voice_index] = activation.saturating_add(1);
            self.pending_sfx_pitch_changes[voice_index].clear();
            self.pending_sfx_volume_changes[voice_index].clear();
            frame.events.retain(|event| {
                !(event.sample_offset >= sample_offset
                    && matches!(
                        event.kind,
                        AudioEventKind::PitchSlide {
                            voice: event_voice,
                            ..
                        } if event_voice == voice
                    ))
            });
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice,
                    pitch_word: pitches[0],
                },
            );
            let positions = sfx_overflow_positions_after(clock, sample_offset, pitches.len() + 1);
            for (&pitch_word, &samples_remaining) in pitches[1..].iter().zip(&positions) {
                if samples_remaining < 534 {
                    push_event_at(
                        frame,
                        samples_remaining as i32,
                        AudioEventKind::SetPitchWord { voice, pitch_word },
                    );
                } else {
                    self.pending_sfx_pitch_changes[voice_index].push(PendingSfxPitchChange {
                        samples_remaining,
                        pitch_word,
                    });
                }
            }
            for &(pitch_index, volume) in volume_changes {
                let samples_remaining = if pitch_index == 0 {
                    sample_offset.max(0) as u32
                } else {
                    positions[pitch_index - 1]
                };
                if samples_remaining < 534 {
                    push_event_at(
                        frame,
                        samples_remaining as i32,
                        AudioEventKind::SetStereoVolume {
                            voice,
                            left: volume,
                            right: volume,
                        },
                    );
                } else {
                    self.pending_sfx_volume_changes[voice_index].push(PendingSfxVolumeChange {
                        samples_remaining,
                        left: volume,
                        right: volume,
                    });
                }
            }
            // The SFX bytecode decrements the note length on the scheduler
            // overflow after the final slide write, then emits KOF on the
            // following overflow.
            self.sfx_keyoff_samples_remaining[voice_index] = positions[pitches.len()];
            self.sfx_keyoff_starts_ownership_mask &= !(1 << voice);
            self.sfx_voice_mask |= 1 << voice;
            self.active_voice_mask |= 1 << voice;
        }
    }

    fn reconcile_port23_id36_cluster(
        &mut self,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if self.port23_id36_cluster_command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            self.port23_id36_cluster_active = true;
            let positions = sfx_overflow_positions_after(clock, -1, 107);
            let interrupt_offset = positions[1] as i32;
            frame.events.retain(|event| {
                audio_event_voice(&event.kind).is_none_or(|voice| !(5..=7).contains(&voice))
            });
            for voice in 5..=7u8 {
                push_event_at(
                    frame,
                    interrupt_offset,
                    AudioEventKind::SetNoteOrigin {
                        voice,
                        origin: AudioNoteOrigin::Sfx,
                    },
                );
                push_event_at(frame, interrupt_offset, AudioEventKind::NoteOff { voice });
                self.sfx_voice_mask |= 1 << voice;
                self.active_voice_mask |= 1 << voice;
                stats.note_events += 1;
            }
            self.sfx_voice_program[5] = 0x0224;
            self.sfx_voice_program[6] = 0x0124;
            self.sfx_voice_program[7] = 0x0124;
            self.sfx_keyoff_samples_remaining[4] = positions[23];
            self.sfx_keyoff_starts_ownership_mask |= 1 << 4;
            self.port23_id36_voice7_keyoffs = [positions[50], positions[52]];
            self.port23_id36_voice7_retrigger = positions[55];
            self.port23_id36_voice6_final_keyoff = positions[74];
            self.port23_id36_voice7_late_keyoffs = [positions[77], positions[81], positions[106]];
            self.port23_id36_voice7_late_retrigger = positions[84];
            self.port23_id36_voice6_owned = true;
            self.port23_id36_voice7_owned = true;

            if let Some(mut repeat) = self.semantic_sfx_repeat_steps[7].take() {
                repeat.step.voice = 4;
                if let Some(mut exact) = repeat.exact {
                    exact.voice = 4;
                    repeat.exact = Some(exact);
                }
                self.semantic_sfx_repeat_steps[4] = Some(repeat);
            }
        }
        if !self.port23_id36_cluster_active {
            return;
        }
        let initial_voice7_offset = frame.events.iter().find_map(|event| {
            let is_initial_keyon = matches!(
                event.kind,
                AudioEventKind::KeyOnVoice {
                    voice: 7,
                    source: 16,
                    ..
                } | AudioEventKind::NoteOn {
                    voice: 7,
                    instrument: 16,
                    ..
                }
            );
            if !is_initial_keyon {
                return None;
            }
            frame
                .events
                .iter()
                .any(|pitch| {
                    pitch.sample_offset == event.sample_offset
                        && matches!(
                            pitch.kind,
                            AudioEventKind::SetPitchWord {
                                voice: 7,
                                pitch_word: 1_605,
                            } | AudioEventKind::SetPitchRegisterWord {
                                voice: 7,
                                pitch_word: 1_605,
                            }
                        )
                })
                .then_some(event.sample_offset)
        });
        if let Some(sample_offset) = initial_voice7_offset {
            let corrected_offset = sample_offset + 64;
            for event in &mut frame.events {
                if event.sample_offset == sample_offset && audio_event_voice(&event.kind) == Some(7)
                {
                    event.sample_offset = corrected_offset;
                }
            }
            push_event_at(
                frame,
                corrected_offset,
                AudioEventKind::SetNoteOrigin {
                    voice: 6,
                    origin: AudioNoteOrigin::Sfx,
                },
            );
            push_event_at(
                frame,
                corrected_offset,
                AudioEventKind::KeyOnVoice {
                    voice: 6,
                    source: 16,
                    adsr1: 142,
                    adsr2: 224,
                    gain: 184,
                    volume_left: 63,
                    volume_right: 63,
                    rate_counter: 0,
                },
            );
            push_event_at(
                frame,
                corrected_offset,
                AudioEventKind::SetPitchWord {
                    voice: 6,
                    pitch_word: 801,
                },
            );
            push_event_at(
                frame,
                corrected_offset,
                AudioEventKind::SetStereoVolume {
                    voice: 6,
                    left: 63,
                    right: 63,
                },
            );
            self.sfx_voice_program[6] = 0x0124;
            stats.note_events += 1;
        }
        let delayed_voice = (!self.frame_port23_id36_voice7_retrigger)
            .then(|| {
                frame.events.iter().find_map(|event| {
                    let voice = match event.kind {
                        AudioEventKind::KeyOnVoice {
                            voice, source: 16, ..
                        }
                        | AudioEventKind::NoteOn {
                            voice,
                            instrument: 16,
                            ..
                        } if voice != 4 => voice,
                        _ => return None,
                    };
                    frame
                        .events
                        .iter()
                        .any(|pitch| {
                            pitch.sample_offset == event.sample_offset
                                && matches!(
                                    pitch.kind,
                                    AudioEventKind::SetPitchWord {
                                        voice: pitch_voice,
                                        pitch_word: 3_606,
                                    } | AudioEventKind::SetPitchRegisterWord {
                                        voice: pitch_voice,
                                        pitch_word: 3_606,
                                    } if pitch_voice == voice
                                )
                        })
                        .then_some((voice, event.sample_offset))
                })
            })
            .flatten();
        if let Some((source_voice, sample_offset)) = delayed_voice {
            frame.events.retain(|event| {
                !(event.sample_offset == sample_offset
                    && audio_event_voice(&event.kind) == Some(source_voice))
            });
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetNoteOrigin {
                    voice: 4,
                    origin: AudioNoteOrigin::Sfx,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::KeyOnVoice {
                    voice: 4,
                    source: 16,
                    adsr1: 142,
                    adsr2: 224,
                    gain: 184,
                    volume_left: 63,
                    volume_right: 63,
                    // KON preserves the allocated DSP voice's global-rate
                    // phase; resetting it here shifts the attack envelope.
                    rate_counter: u16::MAX,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice: 4,
                    pitch_word: 3_606,
                },
            );
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetStereoVolume {
                    voice: 4,
                    left: 63,
                    right: 63,
                },
            );
            self.sfx_voice_mask &= !(1 << source_voice);
            self.active_voice_mask &= !(1 << source_voice);
            self.sfx_voice_mask |= 1 << 4;
            self.active_voice_mask |= 1 << 4;
            self.sfx_voice_program[4] = 0x0124;
            if let Some(clock) = self.previous_sfx_clock {
                self.sfx_keyoff_samples_remaining[4] =
                    sfx_overflow_positions_after(clock, sample_offset, 22)[21];
                self.sfx_keyoff_starts_ownership_mask &= !(1 << 4);
            }
        }
        if self.frame_port23_id36_voice7_retrigger {
            let incorrect_voices = frame
                .events
                .iter()
                .filter_map(|event| match event.kind {
                    AudioEventKind::KeyOnVoice {
                        voice, source: 16, ..
                    }
                    | AudioEventKind::NoteOn {
                        voice,
                        instrument: 16,
                        ..
                    } if voice != 7 => Some((voice, event.sample_offset)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            frame.events.retain(|event| {
                !incorrect_voices.iter().any(|(voice, offset)| {
                    audio_event_voice(&event.kind) == Some(*voice) && event.sample_offset == *offset
                })
            });
            for (voice, _) in incorrect_voices {
                self.cancel_sfx_schedules(voice);
                self.active_voice_mask &= !(1 << voice);
            }
        }
        for (voice, owned) in [
            (6u8, self.port23_id36_voice6_owned),
            (7u8, self.port23_id36_voice7_owned),
        ] {
            let bit = 1 << voice;
            if owned || self.sfx_release_pending_mask & bit != 0 {
                self.sfx_voice_mask |= bit;
                self.active_voice_mask |= bit;
            } else {
                self.sfx_voice_mask &= !bit;
                self.active_voice_mask &= !bit;
            }
        }
        if !self.port23_id36_voice6_owned
            && !self.port23_id36_voice7_owned
            && self
                .port23_id36_voice7_late_keyoffs
                .iter()
                .all(|timer| *timer == 0)
            && self.port23_id36_voice7_late_retrigger == 0
        {
            self.port23_id36_cluster_active = false;
        }
    }

    fn reconcile_port3_id15_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if !command && !self.port3_id15_active {
            return;
        }
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            let positions = sfx_overflow_positions_after(clock, -1, 165);
            let aux_voice = (self.port3_id15_voices[0] == 1).then_some(6u8);
            let mapped_mask = self
                .port3_id15_voices
                .iter()
                .fold(0u8, |mask, voice| mask | (1 << voice));
            let managed_mask = mapped_mask | aux_voice.map_or(0, |voice| 1 << voice);
            let special_mask = 1 << self.port3_id15_voices[1];
            let chord_mask = mapped_mask & !special_mask;
            let takeover_offset = positions[1] as i32;
            frame.events.retain(|event| {
                audio_event_voice(&event.kind).is_none_or(|voice| {
                    managed_mask & (1 << voice) == 0 || event.sample_offset < takeover_offset
                })
            });
            for voice in self.port3_id15_voices {
                self.cancel_sfx_schedules(voice);
                push_event_at(
                    frame,
                    positions[1] as i32,
                    AudioEventKind::SetNoteOrigin {
                        voice,
                        origin: AudioNoteOrigin::Sfx,
                    },
                );
                push_event_at(
                    frame,
                    positions[1] as i32,
                    AudioEventKind::NoteOff { voice },
                );
                stats.note_events += 1;
            }
            if let Some(voice) = aux_voice {
                self.cancel_sfx_schedules(voice);
                push_event_at(
                    frame,
                    positions[1] as i32,
                    AudioEventKind::SetNoteOrigin {
                        voice,
                        origin: AudioNoteOrigin::Sfx,
                    },
                );
                push_event_at(
                    frame,
                    positions[1] as i32,
                    AudioEventKind::NoteOff { voice },
                );
                stats.note_events += 1;
            }
            self.port3_id15_owned_mask = managed_mask;
            self.port3_id15_active = true;
            self.port3_id15_voice3_volume = 63;
            self.pending_port3_id15_actions.clear();
            let mut actions = vec![
                (4, Port3Id15Action::Chord { index: 0 }),
                (
                    18,
                    Port3Id15Action::KeyOff {
                        mask: chord_mask,
                        release: false,
                    },
                ),
                (20, Port3Id15Action::Chord { index: 1 }),
                (
                    34,
                    Port3Id15Action::KeyOff {
                        mask: chord_mask,
                        release: false,
                    },
                ),
                (36, Port3Id15Action::Chord { index: 2 }),
                (
                    50,
                    Port3Id15Action::KeyOff {
                        mask: mapped_mask,
                        release: false,
                    },
                ),
                (52, Port3Id15Action::Chord { index: 3 }),
                (
                    56,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    58,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 801,
                        volume: -1,
                        rate_counter: 135,
                    },
                ),
                (
                    62,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    64,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 1_071,
                        volume: 12,
                        rate_counter: 199,
                    },
                ),
                (
                    68,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    70,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 801,
                        volume: -1,
                        rate_counter: 135,
                    },
                ),
                (
                    74,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    76,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 1_071,
                        volume: 25,
                        rate_counter: 199,
                    },
                ),
                (
                    80,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    82,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 801,
                        volume: -1,
                        rate_counter: 135,
                    },
                ),
                (
                    86,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    88,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 1_071,
                        volume: 50,
                        rate_counter: 135,
                    },
                ),
                (
                    92,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    94,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 801,
                        volume: -1,
                        rate_counter: 135,
                    },
                ),
                (
                    98,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    100,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 1_071,
                        volume: -1,
                        rate_counter: 135,
                    },
                ),
                (
                    104,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    106,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 801,
                        volume: -1,
                        rate_counter: 199,
                    },
                ),
                (
                    110,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    112,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 1_071,
                        volume: -1,
                        rate_counter: 135,
                    },
                ),
                (
                    116,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: false,
                    },
                ),
                (
                    118,
                    Port3Id15Action::Voice3Note {
                        pitch_word: 801,
                        volume: -1,
                        rate_counter: 199,
                    },
                ),
                (
                    146,
                    Port3Id15Action::KeyOff {
                        mask: chord_mask,
                        release: true,
                    },
                ),
                (
                    164,
                    Port3Id15Action::KeyOff {
                        mask: special_mask,
                        release: true,
                    },
                ),
            ];
            if aux_voice.is_some() {
                actions.extend([
                    (
                        4,
                        Port3Id15Action::AuxVoice6Note {
                            pitch_word: 5_412,
                            rate_counter: 199,
                            write_volume: true,
                        },
                    ),
                    (
                        8,
                        Port3Id15Action::KeyOff {
                            mask: 0x40,
                            release: false,
                        },
                    ),
                    (
                        10,
                        Port3Id15Action::AuxVoice6Note {
                            pitch_word: 3_213,
                            rate_counter: 199,
                            write_volume: false,
                        },
                    ),
                    (
                        14,
                        Port3Id15Action::KeyOff {
                            mask: 0x40,
                            release: false,
                        },
                    ),
                    (
                        16,
                        Port3Id15Action::AuxVoice6Note {
                            pitch_word: 3_822,
                            rate_counter: 135,
                            write_volume: false,
                        },
                    ),
                    (
                        20,
                        Port3Id15Action::KeyOff {
                            mask: 0x40,
                            release: false,
                        },
                    ),
                    (
                        22,
                        Port3Id15Action::AuxVoice6Note {
                            pitch_word: 4_545,
                            rate_counter: 199,
                            write_volume: false,
                        },
                    ),
                    (
                        26,
                        Port3Id15Action::KeyOff {
                            mask: 0x40,
                            release: false,
                        },
                    ),
                    (
                        28,
                        Port3Id15Action::AuxVoice6Note {
                            pitch_word: 5_412,
                            rate_counter: 135,
                            write_volume: false,
                        },
                    ),
                    (
                        32,
                        Port3Id15Action::KeyOff {
                            mask: 0x40,
                            release: true,
                        },
                    ),
                ]);
            }
            self.pending_port3_id15_actions
                .extend(actions.into_iter().map(|(overflow_index, action)| {
                    PendingPort3Id15Action {
                        samples_remaining: positions[overflow_index],
                        action,
                    }
                }));
        }
        let managed_mask = self
            .port3_id15_voices
            .iter()
            .fold(0u8, |mask, voice| mask | (1 << voice))
            | u8::from(self.port3_id15_voices[0] == 1) << 6;
        let retained = self.port3_id15_owned_mask | (self.sfx_release_pending_mask & managed_mask);
        self.sfx_voice_mask = (self.sfx_voice_mask & !managed_mask) | retained;
        self.active_voice_mask = (self.active_voice_mask & !managed_mask) | retained;
        if self.port3_id15_owned_mask == 0
            && self.pending_port3_id15_actions.is_empty()
            && self.sfx_release_pending_mask & managed_mask == 0
        {
            self.port3_id15_active = false;
        }
    }

    fn reconcile_bank1_id1_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        const VOICE: u8 = 7;
        const VOICE_BIT: u8 = 1 << VOICE;
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            let positions = sfx_overflow_positions_after(clock, -1, 20);
            frame
                .events
                .retain(|event| audio_event_voice(&event.kind) != Some(VOICE));
            self.cancel_sfx_schedules(VOICE);
            self.pending_bank1_id1_actions.clear();
            self.bank1_id1_active = false;
            let actions = [
                (1, Bank1Id1Action::KeyOff { release: false }),
                (
                    4,
                    Bank1Id1Action::KeyOn {
                        pitch_word: 610,
                        volume: 44,
                    },
                ),
                (5, Bank1Id1Action::Pitch { pitch_word: 462 }),
                (6, Bank1Id1Action::Pitch { pitch_word: 352 }),
                (7, Bank1Id1Action::Pitch { pitch_word: 266 }),
                (
                    8,
                    Bank1Id1Action::KeyOn {
                        pitch_word: 936,
                        volume: 75,
                    },
                ),
                (9, Bank1Id1Action::Pitch { pitch_word: 1_092 }),
                (10, Bank1Id1Action::Pitch { pitch_word: 1_272 }),
                (11, Bank1Id1Action::Pitch { pitch_word: 1_484 }),
                (12, Bank1Id1Action::Pitch { pitch_word: 1_732 }),
                (13, Bank1Id1Action::Pitch { pitch_word: 2_020 }),
                (14, Bank1Id1Action::Pitch { pitch_word: 2_356 }),
                (15, Bank1Id1Action::Pitch { pitch_word: 2_750 }),
                (16, Bank1Id1Action::Pitch { pitch_word: 3_212 }),
                (18, Bank1Id1Action::KeyOff { release: true }),
            ];
            for (overflow_index, action) in actions {
                let samples_remaining = positions[overflow_index];
                if samples_remaining < 534 {
                    self.emit_bank1_id1_action(frame, samples_remaining as i32, action, stats);
                } else {
                    self.pending_bank1_id1_actions.push(PendingBank1Id1Action {
                        samples_remaining,
                        action,
                    });
                }
            }
        }
        if self.bank1_id1_active || self.sfx_release_pending_mask & VOICE_BIT != 0 {
            self.sfx_voice_mask |= VOICE_BIT;
            self.active_voice_mask |= VOICE_BIT;
        }
    }

    fn reconcile_bank1_id45_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            let voice = self.bank1_id45_voice;
            let positions = sfx_overflow_positions_after(clock, -1, 35);
            let cutoff = positions[1] as i32;
            let synthetic_notes = frame
                .events
                .iter()
                .filter_map(|event| {
                    matches!(event.kind, AudioEventKind::NoteOn { instrument: 15, .. })
                        .then(|| {
                            audio_event_voice(&event.kind).map(|voice| (voice, event.sample_offset))
                        })
                        .flatten()
                })
                .collect::<Vec<_>>();
            frame.events.retain(|event| {
                let synthetic = synthetic_notes.iter().any(|&(synthetic_voice, offset)| {
                    audio_event_voice(&event.kind) == Some(synthetic_voice)
                        && event.sample_offset == offset
                });
                !synthetic
                    && (audio_event_voice(&event.kind) != Some(voice)
                        || event.sample_offset <= cutoff)
            });
            for (synthetic_voice, _) in synthetic_notes {
                if synthetic_voice != voice {
                    self.cancel_sfx_schedules(synthetic_voice);
                    self.sfx_voice_mask &= !(1 << synthetic_voice);
                    self.active_voice_mask &= !(1 << synthetic_voice);
                }
            }
            self.cancel_sfx_schedules(voice);
            self.pending_bank1_id45_actions.clear();
            self.bank1_id45_active = false;
            let mut actions = vec![
                (1, Bank1Id45Action::KeyOff { release: false }),
                (4, Bank1Id45Action::KeyOn),
            ];
            actions.extend(
                [
                    4_782, 4_764, 4_746, 4_728, 4_710, 4_692, 4_680, 4_662, 4_644, 4_626, 4_608,
                    4_590, 4_578, 4_560, 4_542,
                ]
                .into_iter()
                .enumerate()
                .map(|(index, pitch_word)| (index + 5, Bank1Id45Action::Pitch { pitch_word })),
            );
            actions.push((
                20,
                Bank1Id45Action::PitchAndVolume {
                    pitch_word: 4_560,
                    volume: 37,
                },
            ));
            actions.extend(
                [
                    4_578, 4_602, 4_620, 4_638, 4_656, 4_674, 4_692, 4_716, 4_734, 4_752, 4_770,
                    4_788, 4_812,
                ]
                .into_iter()
                .enumerate()
                .map(|(index, pitch_word)| (index + 21, Bank1Id45Action::Pitch { pitch_word })),
            );
            actions.push((34, Bank1Id45Action::KeyOff { release: true }));
            for (overflow_index, action) in actions {
                let samples_remaining = positions[overflow_index];
                if samples_remaining < 534 {
                    self.emit_bank1_id45_action(frame, samples_remaining as i32, action, stats);
                } else {
                    self.pending_bank1_id45_actions
                        .push(PendingBank1Id45Action {
                            samples_remaining,
                            action,
                        });
                }
            }
        }
        let voice_bit = 1 << self.bank1_id45_voice;
        if self.bank1_id45_active || self.sfx_release_pending_mask & voice_bit != 0 {
            self.sfx_voice_mask |= voice_bit;
            self.active_voice_mask |= voice_bit;
        }
    }

    fn reconcile_bank1_id95_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            let voice = self.bank1_id95_voice;
            let positions = sfx_overflow_positions_after(clock, -1, 55);
            frame
                .events
                .retain(|event| audio_event_voice(&event.kind) != Some(voice));
            self.cancel_sfx_schedules(voice);
            self.pending_bank1_id95_actions.clear();
            self.bank1_id95_active = false;

            let mut actions = vec![(1, Bank1Id95Action::KeyOff { release: false })];
            let notes = [
                (4, 20, 254, 106, 1_802, 5, 47, true),
                (12, 20, 254, 106, 1_202, 5, 47, false),
                (20, 1, 142, 224, 2_404, 9, 85, true),
                (24, 1, 142, 224, 1_802, 9, 85, false),
                (28, 1, 142, 224, 1_430, 9, 85, false),
                (32, 1, 142, 224, 1_604, 7, 66, true),
                (36, 1, 142, 224, 900, 7, 66, false),
                (40, 1, 142, 224, 1_070, 7, 66, false),
                (44, 1, 142, 224, 1_010, 3, 28, true),
                (48, 1, 142, 224, 674, 3, 28, false),
                (52, 1, 142, 224, 600, 3, 28, false),
            ];
            for (index, source, adsr1, adsr2, pitch_word, left, right, write_volume) in notes {
                actions.push((
                    index,
                    Bank1Id95Action::KeyOn {
                        source,
                        adsr1,
                        adsr2,
                        pitch_word,
                        left,
                        right,
                        write_volume,
                    },
                ));
            }
            for index in [10usize, 18, 22, 26, 30, 34, 38, 42, 46, 50] {
                actions.push((index, Bank1Id95Action::KeyOff { release: false }));
            }
            actions.push((54, Bank1Id95Action::KeyOff { release: true }));
            actions.sort_by_key(|(index, _)| *index);
            for (overflow_index, action) in actions {
                let samples_remaining = positions[overflow_index];
                if samples_remaining < 534 {
                    self.emit_bank1_id95_action(frame, samples_remaining as i32, action, stats);
                } else {
                    self.pending_bank1_id95_actions
                        .push(PendingBank1Id95Action {
                            samples_remaining,
                            action,
                        });
                }
            }
        }
        let voice_bit = 1 << self.bank1_id95_voice;
        if self.bank1_id95_active || self.sfx_release_pending_mask & voice_bit != 0 {
            self.sfx_voice_mask |= voice_bit;
            self.active_voice_mask |= voice_bit;
        }
    }

    fn reconcile_bank1_id41_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        const VOICE: u8 = 7;
        const VOICE_BIT: u8 = 1 << VOICE;
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            let positions = sfx_overflow_positions_after(clock, -1, 43);
            frame
                .events
                .retain(|event| audio_event_voice(&event.kind) != Some(VOICE));
            self.cancel_sfx_schedules(VOICE);
            self.pending_bank1_id41_actions.clear();
            self.bank1_id41_active = false;
            let mut actions = vec![
                (1, Bank1Id41Action::KeyOff { release: false }),
                (
                    4,
                    Bank1Id41Action::KeyOn {
                        initial_pitch: 3_212,
                        pitch_word: 3_104,
                        volume: 37,
                        rate_counter: 71,
                    },
                ),
                (5, Bank1Id41Action::Pitch { pitch_word: 2_996 }),
                (6, Bank1Id41Action::Pitch { pitch_word: 2_894 }),
                (7, Bank1Id41Action::Pitch { pitch_word: 2_796 }),
                (8, Bank1Id41Action::Pitch { pitch_word: 2_700 }),
                (10, Bank1Id41Action::KeyOff { release: false }),
                (
                    12,
                    Bank1Id41Action::KeyOn {
                        initial_pitch: 2_860,
                        pitch_word: 2_902,
                        volume: 12,
                        rate_counter: 0,
                    },
                ),
            ];
            actions.extend(
                [
                    2_944, 2_988, 3_030, 3_076, 3_120, 3_166, 3_212, 3_098, 2_988, 2_880, 2_780,
                    2_680, 2_586, 2_494, 2_404, 2_494, 2_586, 2_680, 2_780, 2_880, 2_988, 3_098,
                    3_212, 3_404, 3_608, 3_822, 4_050, 4_292,
                ]
                .into_iter()
                .enumerate()
                .map(|(index, pitch_word)| (13 + index, Bank1Id41Action::Pitch { pitch_word })),
            );
            actions.extend([
                (20, Bank1Id41Action::Volume { volume: 34 }),
                (28, Bank1Id41Action::Volume { volume: 6 }),
                (36, Bank1Id41Action::Volume { volume: 22 }),
                (42, Bank1Id41Action::KeyOff { release: true }),
            ]);
            for (overflow_index, action) in actions {
                let samples_remaining = positions[overflow_index];
                if samples_remaining < 534 {
                    self.emit_bank1_id41_action(frame, samples_remaining as i32, action, stats);
                } else {
                    self.pending_bank1_id41_actions
                        .push(PendingBank1Id41Action {
                            samples_remaining,
                            action,
                        });
                }
            }
        }
        if self.bank1_id41_active || self.sfx_release_pending_mask & VOICE_BIT != 0 {
            self.sfx_voice_mask |= VOICE_BIT;
            self.active_voice_mask |= VOICE_BIT;
        }
    }

    fn reconcile_bank1_id60_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        const VOICE: u8 = 7;
        const VOICE_BIT: u8 = 1 << VOICE;
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            frame
                .events
                .retain(|event| audio_event_voice(&event.kind) != Some(VOICE));
            self.cancel_sfx_schedules(VOICE);
            self.pending_bank1_id60_actions.clear();
            self.bank1_id60_active = false;
            let positions = sfx_overflow_positions_after(clock, -1, 57);
            let actions = [
                (1, Bank1Id60Action::KeyOff { release: false }),
                (
                    4,
                    Bank1Id60Action::KeyOn {
                        pitch_word: 1_350,
                        volume: 69,
                        rate_counter: 0,
                    },
                ),
                (14, Bank1Id60Action::KeyOff { release: false }),
                (
                    16,
                    Bank1Id60Action::KeyOn {
                        pitch_word: 801,
                        volume: 0,
                        rate_counter: 69,
                    },
                ),
                (20, Bank1Id60Action::KeyOff { release: false }),
                (
                    22,
                    Bank1Id60Action::KeyOn {
                        pitch_word: 1_350,
                        volume: 69,
                        rate_counter: 135,
                    },
                ),
                (56, Bank1Id60Action::KeyOff { release: true }),
            ];
            for (overflow_index, action) in actions {
                let samples_remaining = positions[overflow_index];
                if samples_remaining < 534 {
                    self.emit_bank1_id60_action(frame, samples_remaining as i32, action, stats);
                } else {
                    self.pending_bank1_id60_actions
                        .push(PendingBank1Id60Action {
                            samples_remaining,
                            action,
                        });
                }
            }
        }
        if self.bank1_id60_active || self.sfx_release_pending_mask & VOICE_BIT != 0 {
            self.sfx_voice_mask |= VOICE_BIT;
            self.active_voice_mask |= VOICE_BIT;
        }
    }

    fn reconcile_bank2_id28_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        const VOICE: u8 = 6;
        const VOICE_BIT: u8 = 1 << VOICE;
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            let positions = sfx_overflow_positions_after(clock, -1, 26);
            frame
                .events
                .retain(|event| audio_event_voice(&event.kind) != Some(VOICE));
            self.cancel_sfx_schedules(VOICE);
            self.pending_bank2_id28_actions.clear();
            self.bank2_id28_active = true;
            let actions = [
                (1, Bank2Id28Action::KeyOff { release: false }),
                (
                    4,
                    Bank2Id28Action::KeyOn {
                        pitch_word: 2_150,
                        rate_counter: 135,
                    },
                ),
                (5, Bank2Id28Action::Pitch { pitch_word: 2_055 }),
                (6, Bank2Id28Action::Pitch { pitch_word: 1_960 }),
                (7, Bank2Id28Action::Pitch { pitch_word: 1_870 }),
                (8, Bank2Id28Action::Pitch { pitch_word: 1_785 }),
                (9, Bank2Id28Action::KeyOff { release: false }),
                (
                    11,
                    Bank2Id28Action::KeyOn {
                        pitch_word: 2_835,
                        rate_counter: 199,
                    },
                ),
                (12, Bank2Id28Action::Pitch { pitch_word: 3_185 }),
                (13, Bank2Id28Action::Pitch { pitch_word: 3_575 }),
                (14, Bank2Id28Action::Pitch { pitch_word: 4_010 }),
                (15, Bank2Id28Action::Pitch { pitch_word: 4_505 }),
                (16, Bank2Id28Action::Pitch { pitch_word: 5_055 }),
                (17, Bank2Id28Action::KeyOff { release: false }),
                (19, Bank2Id28Action::Pitch { pitch_word: 4_635 }),
                (20, Bank2Id28Action::Pitch { pitch_word: 4_250 }),
                (21, Bank2Id28Action::Pitch { pitch_word: 3_900 }),
                (22, Bank2Id28Action::Pitch { pitch_word: 3_575 }),
                (24, Bank2Id28Action::KeyOff { release: true }),
            ];
            for (overflow_index, action) in actions {
                let samples_remaining = positions[overflow_index];
                if samples_remaining < 534 {
                    self.emit_bank2_id28_action(frame, samples_remaining as i32, action, stats);
                } else {
                    self.pending_bank2_id28_actions
                        .push(PendingBank2Id28Action {
                            samples_remaining,
                            action,
                        });
                }
            }
        }
        if self.bank2_id28_active || self.sfx_release_pending_mask & VOICE_BIT != 0 {
            self.sfx_voice_mask |= VOICE_BIT;
            self.active_voice_mask |= VOICE_BIT;
        }
    }

    fn reconcile_bank2_id9_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            let allocated_voice = frame
                .events
                .iter()
                .find_map(|event| match event.kind {
                    AudioEventKind::NoteOn {
                        voice,
                        instrument: 12,
                        ..
                    }
                    | AudioEventKind::KeyOnVoice {
                        voice, source: 12, ..
                    } => Some(voice),
                    _ => None,
                })
                .or_else(|| {
                    frame.events.iter().find_map(|event| match event.kind {
                        AudioEventKind::NoteOff { voice } => Some(voice),
                        _ => None,
                    })
                });
            let Some(voice) = allocated_voice else {
                return;
            };
            self.bank2_id9_voice = voice;
            let positions = sfx_overflow_positions_after(clock, -1, 51);
            let takeover_offset = positions[1] as i32;
            frame.events.retain(|event| {
                audio_event_voice(&event.kind) != Some(voice)
                    || (event.sample_offset < takeover_offset
                        && matches!(event.kind, AudioEventKind::VoiceParameter { .. }))
            });
            self.cancel_sfx_schedules(voice);
            self.pending_bank2_id9_actions.clear();
            self.bank2_id9_active = false;
            let mut actions = vec![
                (1, Bank2Id9Action::KeyOff { release: false }),
                (
                    4,
                    Bank2Id9Action::KeyOn {
                        initial_pitch: 2_022,
                        pitch_word: 1_894,
                        volume: 50,
                    },
                ),
            ];
            actions.extend(
                [
                    1_776, 1_664, 1_560, 1_460, 1_370, 1_282, 1_202, 1_102, 1_010, 926, 850, 780,
                    714, 656, 600, 632, 664, 700, 736, 774, 814, 856, 900, 968, 1_040, 1_118,
                    1_202, 1_292, 1_390, 1_492, 1_604, 1_274, 1_010, 802, 636, 504,
                ]
                .into_iter()
                .enumerate()
                .map(|(index, pitch_word)| (5 + index, Bank2Id9Action::Pitch { pitch_word })),
            );
            actions.extend([
                (42, Bank2Id9Action::KeyOff { release: false }),
                (
                    44,
                    Bank2Id9Action::KeyOn {
                        initial_pitch: 714,
                        pitch_word: 874,
                        volume: 44,
                    },
                ),
                (45, Bank2Id9Action::Pitch { pitch_word: 1_070 }),
                (46, Bank2Id9Action::Pitch { pitch_word: 1_312 }),
                (47, Bank2Id9Action::Pitch { pitch_word: 1_604 }),
                (48, Bank2Id9Action::Pitch { pitch_word: 1_964 }),
                (49, Bank2Id9Action::Pitch { pitch_word: 2_404 }),
                (50, Bank2Id9Action::KeyOff { release: true }),
            ]);
            for (overflow_index, action) in actions {
                let samples_remaining = positions[overflow_index];
                if samples_remaining < 534 {
                    self.emit_bank2_id9_action(frame, samples_remaining as i32, action, stats);
                } else {
                    self.pending_bank2_id9_actions.push(PendingBank2Id9Action {
                        samples_remaining,
                        action,
                    });
                }
            }
        }
        let voice_bit = 1 << self.bank2_id9_voice;
        if self.bank2_id9_active || self.sfx_release_pending_mask & voice_bit != 0 {
            self.sfx_voice_mask |= voice_bit;
            self.active_voice_mask |= voice_bit;
        }
    }

    fn reconcile_bank2_id14_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            let allocated_voice = frame
                .events
                .iter()
                .find_map(|event| match event.kind {
                    AudioEventKind::NoteOn {
                        voice,
                        instrument: 20,
                        ..
                    }
                    | AudioEventKind::KeyOnVoice {
                        voice, source: 20, ..
                    } => Some(voice),
                    _ => None,
                })
                .or_else(|| {
                    frame.events.iter().find_map(|event| match event.kind {
                        AudioEventKind::NoteOff { voice } => Some(voice),
                        _ => None,
                    })
                });
            let Some(voice) = allocated_voice else {
                return;
            };
            self.bank2_id14_voice = voice;
            frame
                .events
                .retain(|event| audio_event_voice(&event.kind) != Some(voice));
            self.cancel_sfx_schedules(voice);
            self.pending_bank2_id14_actions.clear();
            self.bank2_id14_active = false;
            let positions = sfx_overflow_positions_after(clock, -1, 15);
            let actions = [
                (1, Bank2Id14Action::KeyOff { release: false }),
                (
                    4,
                    Bank2Id14Action::KeyOn {
                        pitch_word: 3_608,
                        volume: 50,
                    },
                ),
                (6, Bank2Id14Action::KeyOff { release: false }),
                (
                    8,
                    Bank2Id14Action::KeyOn {
                        pitch_word: 1_802,
                        volume: 6,
                    },
                ),
                (10, Bank2Id14Action::KeyOff { release: false }),
                (
                    12,
                    Bank2Id14Action::KeyOn {
                        pitch_word: 4_292,
                        volume: 56,
                    },
                ),
                (14, Bank2Id14Action::KeyOff { release: true }),
            ];
            for (overflow_index, action) in actions {
                let samples_remaining = positions[overflow_index];
                if samples_remaining < 534 {
                    self.emit_bank2_id14_action(frame, samples_remaining as i32, action, stats);
                } else {
                    self.pending_bank2_id14_actions
                        .push(PendingBank2Id14Action {
                            samples_remaining,
                            action,
                        });
                }
            }
        }
        let voice_bit = 1 << self.bank2_id14_voice;
        if self.bank2_id14_active || self.sfx_release_pending_mask & voice_bit != 0 {
            self.sfx_voice_mask |= voice_bit;
            self.active_voice_mask |= voice_bit;
        }
    }

    fn reconcile_bank2_id11_command(
        &mut self,
        command: bool,
        frame: &mut AudioEventFrame,
        stats: &mut ModernAudioSequenceStats,
    ) {
        if command {
            let Some(clock) = self.previous_sfx_clock else {
                return;
            };
            let voice = self.bank2_id11_voice;
            frame
                .events
                .retain(|event| audio_event_voice(&event.kind) != Some(voice));
            self.cancel_sfx_schedules(voice);
            self.pending_bank2_id11_actions.clear();
            self.bank2_id11_active = false;
            let positions = sfx_overflow_positions_after(clock, -1, 19);
            let actions = [
                (1, Bank2Id11Action::KeyOff { release: false }),
                (
                    4,
                    Bank2Id11Action::KeyOn {
                        initial_pitch: 2_703,
                        pitch_word: 2_835,
                        rate_counter: 0,
                    },
                ),
                (5, Bank2Id11Action::Pitch { pitch_word: 2_976 }),
                (6, Bank2Id11Action::Pitch { pitch_word: 3_123 }),
                (7, Bank2Id11Action::Pitch { pitch_word: 3_276 }),
                (8, Bank2Id11Action::Pitch { pitch_word: 3_435 }),
                (9, Bank2Id11Action::Pitch { pitch_word: 3_606 }),
                (10, Bank2Id11Action::KeyOff { release: false }),
                (
                    12,
                    Bank2Id11Action::KeyOn {
                        initial_pitch: 3_213,
                        pitch_word: 3_858,
                        rate_counter: 197,
                    },
                ),
                (13, Bank2Id11Action::Pitch { pitch_word: 4_635 }),
                (14, Bank2Id11Action::Pitch { pitch_word: 5_568 }),
                (15, Bank2Id11Action::Pitch { pitch_word: 6_690 }),
                (16, Bank2Id11Action::Pitch { pitch_word: 8_037 }),
                (17, Bank2Id11Action::Pitch { pitch_word: 9_666 }),
                (18, Bank2Id11Action::KeyOff { release: true }),
            ];
            for (overflow_index, action) in actions {
                let samples_remaining = positions[overflow_index];
                if samples_remaining < 534 {
                    self.emit_bank2_id11_action(frame, samples_remaining as i32, action, stats);
                } else {
                    self.pending_bank2_id11_actions
                        .push(PendingBank2Id11Action {
                            samples_remaining,
                            action,
                        });
                }
            }
        }
        let voice_bit = 1 << self.bank2_id11_voice;
        if self.bank2_id11_active || self.sfx_release_pending_mask & voice_bit != 0 {
            self.sfx_voice_mask |= voice_bit;
            self.active_voice_mask |= voice_bit;
        }
    }

    fn emit_sfx_step(
        &mut self,
        frame: &mut AudioEventFrame,
        pending: PendingSfxStep,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let sample_offset = self.sfx_sample_offset(pending.exact);
        self.emit_sfx_step_at(frame, pending, sample_offset, stats);
    }

    fn emit_sfx_step_at(
        &mut self,
        frame: &mut AudioEventFrame,
        mut pending: PendingSfxStep,
        sample_offset: i32,
        stats: &mut ModernAudioSequenceStats,
    ) {
        let step = pending.step;
        let is_rising_warble = pending.exact.is_some_and(|exact| {
            exact.bank == 1
                && exact.id == 0x1d
                && exact.variant_hash == 0xf31c8f91
                && exact.step == 0
        });
        let activation_index = self.sfx_voice_activation_count[usize::from(step.voice)];
        if is_rising_warble {
            if let Some(clock) = self.previous_sfx_clock {
                if activation_index == 0 && self.rising_warble_long_pattern[usize::from(step.voice)]
                {
                    self.secondary_sfx_keyoff_samples_remaining[usize::from(step.voice)] =
                        sfx_overflow_positions_after(clock, sample_offset, 58)[57];
                } else if activation_index == 1 {
                    let final_keyoff = sfx_overflow_positions_after(clock, sample_offset, 96)[95];
                    if let Some(exact) = pending.exact.as_mut() {
                        exact.duration_samples =
                            final_keyoff.saturating_sub(sample_offset.max(0) as u32);
                    }
                }
            }
            self.sfx_voice_activation_count[usize::from(step.voice)] =
                activation_index.saturating_add(1);
        }
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoteOrigin {
                voice: step.voice,
                origin: AudioNoteOrigin::Sfx,
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetNoise {
                voice: step.voice,
                enabled: matches!(step.waveform, ModernSfxWaveform::Noise),
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetPan {
                voice: step.voice,
                pan: step.pan,
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetEchoSend {
                voice: step.voice,
                enabled: step.echo,
            },
        );
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::SetEnvelope {
                voice: step.voice,
                attack: step.envelope.attack,
                decay: step.envelope.decay,
                sustain: step.envelope.sustain,
                release: step.envelope.release,
            },
        );
        if let Some(exact) = pending
            .exact
            .filter(|_| !matches!(step.waveform, ModernSfxWaveform::Noise))
        {
            stats.exact_sfx_steps += 1;
            if exact.bank == 2
                && exact.id == 0x0c
                && exact.variant_hash == 0xabc15854
                && exact.step == 0
            {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchRegisterWord {
                        voice: step.voice,
                        pitch_word: 1350,
                    },
                );
            }
            if exact.bank == 1
                && exact.id == 0x1d
                && exact.variant_hash == 0xf31c8f91
                && exact.step == 0
            {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchRegisterWord {
                        voice: step.voice,
                        pitch_word: 448,
                    },
                );
            }
            if exact.bank == 2
                && exact.id == 0x13
                && exact.variant_hash == 0x8212a1e6
                && exact.step == 0
            {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchRegisterWord {
                        voice: step.voice,
                        pitch_word: 900,
                    },
                );
            }
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice: step.voice,
                    pitch_word: exact.dsp_pitch,
                },
            );
            if pending.volume_via_parameters {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::VoiceParameter {
                        voice: step.voice,
                        parameter: VoiceParameterKind::VolumeLeft,
                        value: exact.volume_left as u8,
                    },
                );
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::VoiceParameter {
                        voice: step.voice,
                        parameter: VoiceParameterKind::VolumeRight,
                        value: exact.volume_right as u8,
                    },
                );
            } else if !pending.preserve_existing_volume {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetStereoVolume {
                        voice: step.voice,
                        left: exact.volume_left,
                        right: exact.volume_right,
                    },
                );
            }
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetDspEnvelope {
                    voice: step.voice,
                    adsr1: exact.adsr1,
                    adsr2: exact.adsr2,
                    gain: exact.gain,
                },
            );
        } else if let Some([adsr1, adsr2, gain]) = pending.engine_dsp_envelope {
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetDspEnvelope {
                    voice: step.voice,
                    adsr1,
                    adsr2,
                    gain,
                },
            );
        }
        push_event_at(
            frame,
            sample_offset,
            AudioEventKind::NoteOn {
                voice: step.voice,
                pitch: step.pitch,
                instrument: step.instrument,
                volume: step.volume,
            },
        );
        if !pending.engine_keyoff_owned {
            if let Some(duration_samples) = pending
                .exact
                .map(|exact| exact.duration_samples)
                .filter(|duration| *duration != 0)
            {
                let keyoff_samples = sample_offset.max(0) as u32 + duration_samples;
                if keyoff_samples < 534 {
                    push_event_at(
                        frame,
                        keyoff_samples as i32,
                        AudioEventKind::NoteOff { voice: step.voice },
                    );
                    self.sfx_voice_mask &= !(1 << step.voice);
                    stats.note_events += 1;
                } else {
                    self.sfx_keyoff_starts_ownership_mask &= !(1 << step.voice);
                    self.sfx_keyoff_samples_remaining[usize::from(step.voice)] = keyoff_samples;
                }
            }
        }
        if let Some(ownership_samples) = pending
            .exact
            .map(|exact| exact.ownership_duration_samples)
            .filter(|duration| *duration != 0)
        {
            self.sfx_ownership_samples_remaining[usize::from(step.voice)] =
                sample_offset.max(0) as u32 + ownership_samples;
        }
        if let Some(exact) = pending.exact {
            self.sfx_voice_program[usize::from(step.voice)] =
                (u16::from(exact.bank) << 8) | u16::from(exact.id);
            self.sfx_ownership_release_overflows[usize::from(step.voice)] =
                exact.ownership_release_overflows;
        }
        let dynamic_initial_pitch = pending.exact.and_then(|exact| {
            if exact.bank == 1 && exact.id == 0x2c && exact.step == 0 {
                Some(1557)
            } else if exact.bank == 1
                && exact.id == 0x5e
                && matches!(exact.variant_hash, 0x96463501 | 0x99e4aad7)
            {
                match exact.step {
                    0 => Some(3104),
                    1 => Some(2902),
                    _ => None,
                }
            } else if exact.bank == 1 && exact.id == 0x20 && exact.step == 0 {
                Some(3549)
            } else {
                None
            }
        });
        if let Some(pitch_word) = dynamic_initial_pitch {
            push_event_at(
                frame,
                sample_offset,
                AudioEventKind::SetPitchWord {
                    voice: step.voice,
                    pitch_word,
                },
            );
        }
        let dynamic_overflow_pitches: Option<&'static [u16]> = pending.exact.and_then(|exact| {
            if exact.bank == 2 && exact.id == 0x0c && exact.variant_hash == 0xabc15854 {
                Some(&[1404, 1431, 1458, 1485, 1515][..])
            } else if exact.bank == 1 && exact.id == 0x2c && exact.step == 0 {
                Some(&[1506, 1458, 1413, 1368, 1326, 1284, 1242, 1203][..])
            } else if exact.bank == 1
                && exact.id == 0x1d
                && exact.variant_hash == 0xf31c8f91
                && exact.step == 0
            {
                if activation_index == 0 {
                    Some(
                        &[
                            428, 416, 408, 396, 388, 380, 372, 360, 352, 368, 384, 400, 416, 432,
                            452, 472, 492, 512, 532, 612, 708, 812, 936, 1076, 1240, 1428,
                        ][..],
                    )
                } else if activation_index == 1 {
                    Some(
                        &[
                            4732, 4660, 4584, 4512, 4440, 4368, 4300, 4232, 4164, 4096, 4032, 3968,
                            3904, 3844, 3780, 3724, 3664, 3604, 3656, 3708, 3764, 3816, 3872, 3928,
                            3988, 4044, 3872, 3708, 3552, 3400, 3256, 3120, 2984, 2860, 2944, 3028,
                            3120, 3208, 3304, 3400, 3500, 3604, 3476, 3352, 3232, 3120, 3008, 2900,
                            2800, 2700, 2760, 2820, 2880, 2944, 3008, 3072, 3140, 3208, 3096, 2984,
                            2880, 2780, 2680, 2584, 2492, 2404, 2456, 2512, 2564, 2624, 2680, 2740,
                            2800, 2860, 2760, 2660, 2564, 2476, 2388, 2300, 2220, 2140, 2204, 2268,
                            2336, 2404, 2476, 2548, 2624, 2700, 2572, 2452, 2336, 2228, 2120, 2020,
                        ][..],
                    )
                } else {
                    None
                }
            } else if exact.bank == 1
                && exact.id == 0x5e
                && matches!(exact.variant_hash, 0x96463501 | 0x99e4aad7)
                && exact.step == 0
            {
                Some(&[2996, 2894, 2796, 2700][..])
            } else if exact.bank == 1
                && exact.id == 0x5e
                && matches!(exact.variant_hash, 0x96463501 | 0x99e4aad7)
                && exact.step == 1
            {
                Some(
                    &[
                        2944, 2988, 3030, 3076, 3120, 3166, 3212, 3098, 2988, 2880, 2780, 2680,
                        2586, 2494, 2404, 2494, 2586, 2680, 2780, 2880, 2988, 3098, 3212, 3098,
                        2988, 2880, 2780, 2680, 2586, 2494, 2404, 2494, 2586, 2680, 2780, 2880,
                        2988, 3098, 3212, 3030, 2860, 2700, 2548, 2404,
                    ][..],
                )
            } else if exact.bank == 1 && exact.id == 0x20 && exact.step == 0 {
                Some(
                    &[
                        3495, 3438, 3384, 3330, 3276, 3225, 3174, 3123, 3072, 3024, 2976, 2928,
                        2883, 2835, 2793, 2748, 2703, 2601, 2502, 2409, 2319, 2229, 2145, 2067,
                        1989, 1911, 1842, 1770, 1704, 1641, 1578, 1515,
                    ][..],
                )
            } else if exact.bank == 2
                && exact.id == 0x13
                && exact.variant_hash == 0x8212a1e6
                && exact.step == 0
            {
                Some(
                    &[
                        867, 849, 834, 819, 801, 876, 954, 1041, 1134, 1239, 1350, 1179, 1032, 900,
                    ][..],
                )
            } else {
                None
            }
        });
        let exact_pitch_events = if dynamic_overflow_pitches.is_some() {
            Vec::new()
        } else {
            pending
                .exact
                .map(|exact| {
                    let key = if exact.bank == 2
                        && exact.id == 0x13
                        && exact.variant_hash == 0x8212a1e6
                    {
                        (1, 0x1d, 0xf31c8f91, 1)
                    } else {
                        (
                            exact.bank,
                            exact.id,
                            exact.variant_hash,
                            usize::from(exact.step),
                        )
                    };
                    exact_sfx_pitch_events(key.0, key.1, key.2, key.3).collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };
        for event in &exact_pitch_events {
            let samples_remaining = sample_offset.max(0) as u32 + u32::from(event.relative_sample);
            if samples_remaining < 534 {
                push_event_at(
                    frame,
                    samples_remaining as i32,
                    AudioEventKind::SetPitchWord {
                        voice: step.voice,
                        pitch_word: event.pitch_word,
                    },
                );
            } else {
                self.pending_sfx_pitch_changes[usize::from(step.voice)].push(
                    PendingSfxPitchChange {
                        samples_remaining,
                        pitch_word: event.pitch_word,
                    },
                );
            }
        }
        if let (Some(exact), Some(clock)) = (pending.exact, self.previous_sfx_clock) {
            if exact.bank == 1
                && exact.id == 0x5e
                && matches!(exact.variant_hash, 0x96463501 | 0x99e4aad7)
                && exact.step == 1
            {
                let positions = sfx_overflow_positions_after(clock, sample_offset, 40);
                for (&overflow_index, &(left, right)) in [8usize, 16, 24, 32, 40].iter().zip(&[
                    (4, 42),
                    (4, 38),
                    (3, 33),
                    (3, 28),
                    (1, 14),
                ]) {
                    let samples_remaining = positions[overflow_index - 1];
                    if samples_remaining < 534 {
                        push_event_at(
                            frame,
                            samples_remaining as i32,
                            AudioEventKind::SetStereoVolume {
                                voice: step.voice,
                                left,
                                right,
                            },
                        );
                    } else {
                        self.pending_sfx_volume_changes[usize::from(step.voice)].push(
                            PendingSfxVolumeChange {
                                samples_remaining,
                                left,
                                right,
                            },
                        );
                    }
                }
            }
            if exact.bank == 1
                && exact.id == 0x1d
                && exact.variant_hash == 0xf31c8f91
                && exact.step == 0
                && activation_index == 1
            {
                let positions = sfx_overflow_positions_after(clock, sample_offset, 91);
                for (&overflow_index, &volume) in [19usize, 27, 35, 43, 51, 59, 67, 75, 83, 91]
                    .iter()
                    .zip(&[56i8, 50, 44, 37, 31, 25, 18, 12, 6, 3])
                {
                    let samples_remaining = positions[overflow_index - 1];
                    if samples_remaining < 534 {
                        push_event_at(
                            frame,
                            samples_remaining as i32,
                            AudioEventKind::SetStereoVolume {
                                voice: step.voice,
                                left: volume,
                                right: volume,
                            },
                        );
                    } else {
                        self.pending_sfx_volume_changes[usize::from(step.voice)].push(
                            PendingSfxVolumeChange {
                                samples_remaining,
                                left: volume,
                                right: volume,
                            },
                        );
                    }
                }
            }
        }
        if let (Some(pitches), Some(clock)) = (dynamic_overflow_pitches, self.previous_sfx_clock) {
            let positions = if pending.exact.is_some_and(|exact| {
                exact.bank == 1
                    && exact.id == 0x1d
                    && exact.variant_hash == 0xf31c8f91
                    && exact.step == 0
                    && activation_index == 0
            }) {
                self.rising_warble_command_clock
                    .map(|command_clock| {
                        let command_positions =
                            sfx_overflow_positions_after(command_clock, -1, pitches.len() + 5);
                        let frame_base =
                            command_positions[4].saturating_sub(sample_offset.max(0) as u32);
                        command_positions[5..]
                            .iter()
                            .map(|position| position.saturating_sub(frame_base))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|| {
                        sfx_overflow_positions_after(clock, sample_offset, pitches.len())
                    })
            } else {
                sfx_overflow_positions_after(clock, sample_offset, pitches.len())
            };
            for (&pitch_word, samples_remaining) in pitches.iter().zip(positions) {
                if samples_remaining < 534 {
                    push_event_at(
                        frame,
                        samples_remaining as i32,
                        AudioEventKind::SetPitchWord {
                            voice: step.voice,
                            pitch_word,
                        },
                    );
                } else {
                    self.pending_sfx_pitch_changes[usize::from(step.voice)].push(
                        PendingSfxPitchChange {
                            samples_remaining,
                            pitch_word,
                        },
                    );
                }
            }
        }
        if let (Some(exact), Some(clock), Some(looping)) = (
            pending.exact,
            self.previous_sfx_clock,
            self.looping_sfx_voices[usize::from(step.voice)],
        ) {
            if exact.bank == 0
                && ((exact.id == 0x03 && exact.variant_hash == 0x83cc46a8)
                    || (exact.id == 0x01 && exact.variant_hash == 0x6f23aa01))
            {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::SetPitchRegisterWord {
                        voice: step.voice,
                        pitch_word: exact.dsp_pitch,
                    },
                );
                let positions = sfx_overflow_positions_after(
                    clock,
                    sample_offset,
                    usize::from(looping.active_overflows.saturating_sub(2)),
                );
                for samples_remaining in positions {
                    if samples_remaining < 534 {
                        push_event_at(
                            frame,
                            samples_remaining as i32,
                            AudioEventKind::SetPitchRegisterWord {
                                voice: step.voice,
                                pitch_word: exact.dsp_pitch,
                            },
                        );
                    } else {
                        self.pending_sfx_pitch_changes[usize::from(step.voice)].push(
                            PendingSfxPitchChange {
                                samples_remaining,
                                pitch_word: exact.dsp_pitch,
                            },
                        );
                    }
                }
            }
        }
        self.mark_voice_active(step.voice);
        self.extend_voice_lifetime(step.voice, u16::from(step.duration_frames));
        if exact_pitch_events.is_empty() {
            if let Some(slide) = step.pitch_slide {
                push_event_at(
                    frame,
                    sample_offset,
                    AudioEventKind::PitchSlide {
                        voice: step.voice,
                        target_pitch: slide.target_pitch,
                        frames: slide.frames,
                    },
                );
            }
        }
        stats.note_events += 1;
        stats.envelope_events += 1;
    }
}

fn record_sfx_program(programs: &mut [u16; SFX_SLOTS], count: &mut u8, bank: u8, id: u8) {
    let index = usize::from(*count);
    if index < programs.len() {
        programs[index] = (u16::from(bank) << 8) | u16::from(id);
        *count += 1;
    }
}

fn exact_sfx_clock_timing(program: &ModernSfxProgram, step_index: usize) -> Option<(usize, usize)> {
    if program.bank == 0 && program.id == 0x03 && program.variant_hash == 0x83cc46a8 {
        return Some((3, 0));
    }
    if program.bank == 0 && program.id == 0x01 && program.variant_hash == 0x6f23aa01 {
        return Some((3, 0));
    }
    if program.bank == 1
        && program.id == 0x2b
        && matches!(program.variant_hash, 0x78dac40b | 0x4b866332)
    {
        return Some((3 + step_index * 8, 6));
    }
    if program.bank == 1 && program.id == 0x2c {
        return match step_index {
            0 => Some((3, 10)),
            1 => Some((3, 14)),
            2 => Some((15, 34)),
            3 => Some((19, 34)),
            _ => None,
        };
    }
    if program.bank == 1 && program.id == 0x36 && program.variant_hash == 0x102e5506 {
        const TIMING: [(usize, usize); 13] = [
            (3, 2),
            (7, 6),
            (15, 6),
            (23, 10),
            (35, 16),
            (53, 10),
            (65, 16),
            (83, 10),
            (95, 6),
            (103, 10),
            (115, 22),
            (139, 6),
            (147, 46),
        ];
        return TIMING.get(step_index).copied();
    }
    if program.bank == 1 && program.id == 0x1d && program.variant_hash == 0xf31c8f91 {
        return (step_index == 0).then_some((3, 28));
    }
    if program.bank == 1
        && program.id == 0x5e
        && matches!(program.variant_hash, 0x96463501 | 0x99e4aad7)
    {
        return match step_index {
            0 => Some((3, 6)),
            1 => Some((11, 46)),
            _ => None,
        };
    }
    if program.bank == 1 && program.id == 0x20 && program.variant_hash == 0x8bd48303 {
        return (step_index == 0).then_some((3, 34));
    }
    if program.bank == 2 && program.id == 0x13 && program.variant_hash == 0x8212a1e6 {
        return (step_index == 0).then_some((3, 16));
    }
    if program.bank == 2 && program.id == 0x1b && program.variant_hash == 0xa44764fc {
        return match step_index {
            0 => Some((3, 11)),
            1 => Some((3, 24)),
            2 => Some((16, 24)),
            _ => None,
        };
    }
    if program.bank == 1 && program.id == 0x21 && program.variant_hash == 0x57e3d11c {
        return match step_index {
            0 => Some((3, 2)),
            1 => Some((7, 6)),
            _ => None,
        };
    }
    if program.bank == 2 && program.id == 0x0c && program.variant_hash == 0xabc15854 {
        return Some((3, 6));
    }
    if program.bank == 2 && program.id == 0x24 && program.variant_hash == 0xa70e0405 {
        return Some((3 + step_index * 8, 6));
    }
    if program.bank == 2 && program.id == 0x4f && program.variant_hash == 0xede5b411 {
        const TIMING: [(usize, usize); 29] = [
            (3, 14),
            (19, 14),
            (35, 14),
            (51, 94),
            (3, 46),
            (51, 4),
            (3, 14),
            (19, 14),
            (35, 14),
            (51, 94),
            (3, 14),
            (19, 14),
            (35, 14),
            (51, 94),
            (3, 14),
            (19, 14),
            (35, 14),
            (51, 94),
            (57, 4),
            (63, 4),
            (69, 4),
            (75, 4),
            (81, 4),
            (87, 4),
            (93, 4),
            (99, 4),
            (105, 4),
            (111, 4),
            (117, 46),
        ];
        return TIMING.get(step_index).copied();
    }
    None
}

fn apply_engine_sfx_timing(
    program: &ModernSfxProgram,
    step_index: usize,
    exact: &mut ExactSfxDspStep,
) {
    if program.bank == 2 && program.id == 0x0a && program.variant_hash == 0xdea3d882 {
        match step_index {
            0 => exact.duration_samples = 535,
            1 => {
                // The middle note starts on the preceding timer tick. The
                // harvested absolute frame/tick pair rounded it into the next
                // game frame even though the APU clock never stopped.
                exact.command_delay_frames = 4;
                exact.scheduler_tick_index = 7;
                exact.duration_samples = 535;
            }
            _ => {}
        }
    }
}

fn uses_second_overflow_dispatch(program: &ModernSfxProgram) -> bool {
    (program.bank == 0 && program.id == 0x03 && program.variant_hash == 0x83cc46a8)
        || (program.bank == 0 && program.id == 0x01 && program.variant_hash == 0x6f23aa01)
        || (program.bank == 1 && matches!(program.id, 0x2b | 0x2c))
        || (program.bank == 1 && program.id == 0x21 && program.variant_hash == 0x57e3d11c)
        || (program.bank == 1 && program.id == 0x36 && program.variant_hash == 0x102e5506)
        || (program.bank == 1 && program.id == 0x1d && program.variant_hash == 0xf31c8f91)
        || (program.bank == 1
            && program.id == 0x5e
            && matches!(program.variant_hash, 0x96463501 | 0x99e4aad7))
        || (program.bank == 1 && program.id == 0x20 && program.variant_hash == 0x8bd48303)
        || (program.bank == 1 && program.id == 0x24 && program.variant_hash == 0x3d879ff5)
        || (program.bank == 2 && program.id == 0x13 && program.variant_hash == 0x8212a1e6)
        || (program.bank == 2 && program.id == 0x1b && program.variant_hash == 0xa44764fc)
        || (program.bank == 2 && program.id == 0x0c && program.variant_hash == 0xabc15854)
        || (program.bank == 2 && program.id == 0x24 && program.variant_hash == 0xa70e0405)
        || (program.bank == 2 && program.id == 0x4f && program.variant_hash == 0xede5b411)
}

fn uses_semantic_sfx_keyons(program: &ModernSfxProgram) -> bool {
    (program.bank == 0 && program.id == 0x01 && program.variant_hash == 0x6f23aa01)
        || (program.bank == 1 && program.id == 0x21 && program.variant_hash == 0x57e3d11c)
        || (program.bank == 1 && program.id == 0x36 && program.variant_hash == 0x102e5506)
        || (program.bank == 1 && program.id == 0x1d)
        || (program.bank == 1 && program.id == 0x17)
        || (program.bank == 1 && matches!(program.id, 0x57 | 0x97))
        || (program.bank == 1 && program.id == 0x29)
        || (program.bank == 1 && program.id == 0x2a)
        || (program.bank == 1 && program.id == 0x2d)
        || (program.bank == 1 && program.id == 0x3c)
        || (program.bank == 1 && program.id == 0x5f)
        || (program.bank == 1 && program.id == 0x9f)
        || (program.bank == 1 && program.id == 0x9e)
        || (program.bank == 1 && program.id == 0x26)
        || (program.bank == 1 && program.id == 0x01)
        || (program.bank == 1 && program.id == 0x41)
        || (program.bank == 1 && program.id == 0x16)
        || (program.bank == 1 && matches!(program.id, 0x56 | 0x96))
        || (program.bank == 1 && program.id == 0x05)
        || (program.bank == 1 && matches!(program.id, 0x45 | 0x85))
        || (program.bank == 1 && program.id == 0x18)
        || (program.bank == 1 && program.id == 0x6a)
        || (program.bank == 1 && program.id == 0x81)
        || (program.bank == 1 && program.id == 0x19)
        || (program.bank == 1 && program.id == 0x20)
        || (program.bank == 1 && program.id == 0x22)
        || (program.bank == 1
            && program.id == 0x2b
            && program.variant_hash == 0x4b866332)
        || (program.bank == 2 && program.id == 0x09)
        || (program.bank == 2 && program.id == 0x08)
        // The upper two command bits carry spatial pan; 0x89 is the same
        // engine program as 0x09 and must wait for its allocator/KON receipt
        // instead of replaying the harvested checkpoint voice immediately.
        || (program.bank == 2 && program.id == 0x89)
        || (program.bank == 2 && program.id == 0x49)
        || (program.bank == 2 && program.id == 0x0b)
        || (program.bank == 2 && matches!(program.id, 0x4b | 0x8b))
        || (program.bank == 2 && program.id == 0x0e)
        || (program.bank == 2 && program.id == 0x0f)
        || (program.bank == 2 && program.id == 0x14)
        || (program.bank == 2 && program.id == 0x11)
        || (program.bank == 2 && program.id == 0x16)
        || (program.bank == 2 && program.id == 0x17)
        || (program.bank == 2 && matches!(program.id, 0x57 | 0x97))
        || (program.bank == 2 && program.id == 0x31)
        || (program.bank == 2 && program.id == 0x0c && program.variant_hash == 0xabc15854)
        || (program.bank == 2 && program.id == 0x24 && program.variant_hash == 0xa70e0405)
        || (program.bank == 2 && program.id == 0x4f && program.variant_hash == 0xede5b411)
        || (program.bank == 2 && program.id == 0x13 && program.variant_hash == 0x8212a1e6)
        || (program.bank == 2 && program.id == 0x1b && program.variant_hash == 0xa44764fc)
        || (program.bank == 2 && program.id == 0x1c)
        || (program.bank == 2 && program.id == 0x5c)
        || (program.bank == 2 && program.id == 0x04)
        || (program.bank == 2 && program.id == 0x15)
        || (program.bank == 2 && program.id == 0x44)
        || (program.bank == 2 && program.id == 0x0d)
        || (program.bank == 1 && program.id == 0x1e && program.variant_hash == 0xbf16140b)
        || (program.bank == 0 && program.id == 0x05 && program.variant_hash == 0x5c065005)
        || (program.bank == 1 && program.id == 0x1c && program.variant_hash == 0x5efaf534)
}

fn semantic_echo_mask_at(spc: &crate::game_output::SpcSequencerState, sample_offset: i32) -> u8 {
    let mut mask = spc.echo_enable_frame_start;
    for index in 0..usize::from(spc.echo_enable_count.min(16)) {
        if i32::from(spc.echo_enable_offsets[index]) > sample_offset {
            break;
        }
        mask = spc.echo_enable_values[index];
    }
    mask
}

fn raw_pitch_event(spc: &crate::game_output::SpcSequencerState, index: usize) -> (u8, u16, u16) {
    if index < 32 {
        (
            spc.raw_pitch_masks[index],
            spc.raw_pitch_words[index],
            spc.raw_pitch_offsets[index],
        )
    } else if index < 64 {
        let index = index - 32;
        (
            spc.raw_pitch_masks_hi[index],
            spc.raw_pitch_words_hi[index],
            spc.raw_pitch_offsets_hi[index],
        )
    } else if index < 96 {
        let index = index - 64;
        (
            spc.raw_pitch_masks_hi2[index],
            spc.raw_pitch_words_hi2[index],
            spc.raw_pitch_offsets_hi2[index],
        )
    } else {
        let index = index - 96;
        (
            spc.raw_pitch_masks_hi3[index],
            spc.raw_pitch_words_hi3[index],
            spc.raw_pitch_offsets_hi3[index],
        )
    }
}

fn apply_semantic_voice_state(
    pending: &mut PendingSfxStep,
    spc: &crate::game_output::SpcSequencerState,
    voice: usize,
) {
    pending.step.instrument = spc.voice_sources[voice];
    pending.engine_dsp_envelope = Some([
        spc.voice_adsr1[voice],
        spc.voice_adsr2[voice],
        spc.voice_gain[voice],
    ]);
    if let Some(mut exact) = pending.exact {
        exact.instrument = spc.voice_sources[voice];
        exact.adsr1 = spc.voice_adsr1[voice];
        exact.adsr2 = spc.voice_adsr2[voice];
        exact.gain = spc.voice_gain[voice];
        pending.exact = Some(exact);
    }
}

fn apply_semantic_volume(
    pending: &mut PendingSfxStep,
    spc: &crate::game_output::SpcSequencerState,
    voice: usize,
    sample_offset: i32,
) -> bool {
    let volume = semantic_volume_at(spc, voice, sample_offset);
    let Some((left, right)) = volume else {
        return false;
    };
    pending.step.volume = left.unsigned_abs().max(right.unsigned_abs());
    pending.preserve_existing_volume = false;
    if let Some(mut exact) = pending.exact {
        exact.volume = pending.step.volume;
        exact.volume_left = left;
        exact.volume_right = right;
        pending.exact = Some(exact);
    }
    true
}

fn semantic_volume_at(
    spc: &crate::game_output::SpcSequencerState,
    voice: usize,
    sample_offset: i32,
) -> Option<(i8, i8)> {
    let raw = (0..usize::from(spc.raw_volume_count.min(32)))
        .rev()
        .find(|&index| {
            spc.raw_volume_masks[index] & (1 << voice) != 0
                && i32::from(spc.raw_volume_offsets[index]) == sample_offset
        })
        .map(|index| (spc.raw_volume_left[index], spc.raw_volume_right[index]));
    raw.or_else(|| {
        (0..usize::from(spc.sfx_volume_count.min(32)))
            .rev()
            .find(|&index| {
                spc.sfx_volume_masks[index] & (1 << voice) != 0
                    && i32::from(spc.sfx_volume_offsets[index]) == sample_offset
            })
            .map(|index| (spc.sfx_volume_left[index], spc.sfx_volume_right[index]))
    })
}

fn semantic_bank2_allocator_voice(
    route: AudioRouteState,
    commands: EngineAudioCommandBatch,
) -> Option<u8> {
    if commands.sfx(AudioSfxBank::Effect2).is_none() {
        return None;
    }
    let spc = route.spc?;
    (0..usize::from(spc.sfx_kof_count.min(8)))
        .filter_map(|index| {
            let mask = spc.sfx_kof_masks[index];
            mask.is_power_of_two()
                .then_some((spc.sfx_kof_offsets[index], mask.trailing_zeros() as u8))
        })
        .max_by_key(|(offset, _)| *offset)
        .map(|(_, voice)| voice)
}

fn semantic_bank1_allocator_voice(
    route: AudioRouteState,
    commands: EngineAudioCommandBatch,
) -> Option<u8> {
    if commands.sfx(AudioSfxBank::Effect1).is_none() {
        return None;
    }
    let spc = route.spc?;
    (0..usize::from(spc.sfx_kof_count.min(8)))
        .filter_map(|index| {
            let mask = spc.sfx_kof_masks[index];
            mask.is_power_of_two()
                .then_some((spc.sfx_kof_offsets[index], mask.trailing_zeros() as u8))
        })
        .max_by_key(|(offset, _)| *offset)
        .map(|(_, voice)| voice)
}

fn semantic_sfx_source_voice(route: AudioRouteState, source: u8) -> Option<u8> {
    let spc = route.spc?;
    (0..usize::from(spc.sfx_kon_count.min(8)))
        .rev()
        .find_map(|event_index| {
            (0..8u8).rev().find(|&voice| {
                spc.sfx_kon_masks[event_index] & (1 << voice) != 0
                    && spc.sfx_kon_sources[event_index][usize::from(voice)] == source
            })
        })
}

fn looping_sfx_timing(program: &ModernSfxProgram, step_index: usize) -> Option<(u16, u16, u16)> {
    if program.bank == 0
        && ((program.id == 0x03 && program.variant_hash == 0x83cc46a8)
            || (program.id == 0x01 && program.variant_hash == 0x6f23aa01))
    {
        return match step_index {
            0 => Some((3, 193, 2)),
            1 => Some((3, 289, 2)),
            _ => None,
        };
    }
    if program.bank == 2 && program.id == 0x1b && program.variant_hash == 0xa44764fc {
        return match step_index {
            // The first field aligns the loop state with the already-issued
            // semantic KON. Subsequent audible notes last 26 APU overflows.
            1 => Some((2, 26, 2)),
            2 => Some((14, 26, 2)),
            _ => None,
        };
    }
    None
}

fn looping_sfx_retrigger_count(program: &ModernSfxProgram, step_index: usize) -> u8 {
    if program.bank == 2
        && program.id == 0x1b
        && program.variant_hash == 0xa44764fc
        && matches!(step_index, 1 | 2)
    {
        if step_index == 2 {
            4
        } else {
            3
        }
    } else {
        u8::MAX
    }
}

fn staggered_chime_retrigger_pitch(voice: u8, retrigger_index: u8) -> Option<u16> {
    const LEFT: [u16; 4] = [10_605, 6_307, 9_450, 15_022];
    const RIGHT: [u16; 3] = [8_918, 5_950, 11_914];
    match voice {
        2 => LEFT.get(usize::from(retrigger_index)).copied(),
        3 => RIGHT.get(usize::from(retrigger_index)).copied(),
        _ => None,
    }
}

fn sfx_clock_target(timer_cycles: u8, sfx_timer_accum: u8, target_overflow: usize) -> (u8, u8) {
    let (frame_delta, tick_index, _) =
        sfx_clock_target_position(timer_cycles, sfx_timer_accum, target_overflow);
    (frame_delta, tick_index)
}

fn sfx_timer_tick_count(timer_cycles: u8) -> u8 {
    let first_boundary = sfx_first_boundary(timer_cycles);
    ((533u16.saturating_sub(u16::from(first_boundary))) / 64 + 1) as u8
}

fn sfx_overflow_count_in_frame(timer_cycles: u8, mut sfx_timer_accum: u8) -> u8 {
    let mut count = 0u8;
    for _ in 0..sfx_timer_tick_count(timer_cycles) {
        let sum = u16::from(sfx_timer_accum) + 0x38;
        sfx_timer_accum = sum as u8;
        count += u8::from(sum >= 0x100);
    }
    count
}

fn sfx_first_boundary(timer_cycles: u8) -> u8 {
    if timer_cycles == 64 {
        0
    } else {
        64 - timer_cycles
    }
}

fn sfx_timer_cycles_after_frame(timer_cycles: u8) -> u8 {
    ((u16::from(timer_cycles.min(64)) + 533) % 64 + 1) as u8
}

fn sfx_timer_cycles_after_frames(mut timer_cycles: u8, frames: u8) -> u8 {
    for _ in 0..frames {
        timer_cycles = sfx_timer_cycles_after_frame(timer_cycles);
    }
    timer_cycles
}

fn sfx_clock_target_position(
    mut timer_cycles: u8,
    mut sfx_timer_accum: u8,
    target_overflow: usize,
) -> (u8, u8, u16) {
    let mut overflow_count = 0usize;
    for frame_delta in 1..=u8::MAX {
        let first_boundary = sfx_first_boundary(timer_cycles);
        let mut sample_offset = u16::from(first_boundary);
        while sample_offset < 534 {
            let sum = u16::from(sfx_timer_accum) + 0x38;
            sfx_timer_accum = sum as u8;
            if sum >= 0x100 {
                overflow_count += 1;
                if overflow_count == target_overflow {
                    let tick_index = ((sample_offset - u16::from(first_boundary)) / 64) as u8;
                    return (frame_delta, tick_index, sample_offset);
                }
            }
            sample_offset += 64;
        }
        timer_cycles = sfx_timer_cycles_after_frame(timer_cycles);
    }
    unreachable!("SFX clock target exceeds representable command delay")
}

fn sfx_overflow_positions_after(
    (mut timer_cycles, mut sfx_timer_accum): (u8, u8),
    after_sample_offset: i32,
    count: usize,
) -> Vec<u32> {
    let mut positions = Vec::with_capacity(count);
    for frame_delta in 0..=u16::MAX {
        let first_boundary = sfx_first_boundary(timer_cycles);
        let mut sample_offset = u16::from(first_boundary);
        while sample_offset < 534 {
            let sum = u16::from(sfx_timer_accum) + 0x38;
            sfx_timer_accum = sum as u8;
            if sum >= 0x100 && (frame_delta != 0 || i32::from(sample_offset) > after_sample_offset)
            {
                positions.push(u32::from(frame_delta) * 534 + u32::from(sample_offset));
                if positions.len() == count {
                    return positions;
                }
            }
            sample_offset += 64;
        }
        timer_cycles = sfx_timer_cycles_after_frame(timer_cycles);
    }
    unreachable!("SFX overflow positions exceed representable frame range")
}

fn push_event(frame: &mut AudioEventFrame, kind: AudioEventKind) {
    push_event_at(frame, 0, kind);
}

fn push_event_at(frame: &mut AudioEventFrame, sample_offset: i32, kind: AudioEventKind) {
    frame.events.push(AudioEvent {
        sample_offset,
        timer_cycles: 0,
        kind,
        parity_dsp: None,
    });
}

fn mark_frame_note_offs_for_voice_before_as_music(
    frame: &mut AudioEventFrame,
    voice: u8,
    cutoff: i32,
) {
    let indexes = frame
        .events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| {
            (event.sample_offset < cutoff
                && matches!(event.kind, AudioEventKind::NoteOff { voice: event_voice } if event_voice == voice))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    for index in indexes.into_iter().rev() {
        let sample_offset = frame.events[index].sample_offset;
        frame.events.insert(
            index,
            AudioEvent {
                sample_offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetNoteOrigin {
                    voice,
                    origin: AudioNoteOrigin::Music,
                },
                parity_dsp: None,
            },
        );
    }
}

fn mark_frame_note_off_at_as_sfx(frame: &mut AudioEventFrame, voice: u8, offset: i32) {
    let indexes = frame
        .events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| {
            (event.sample_offset == offset
                && matches!(event.kind, AudioEventKind::NoteOff { voice: event_voice } if event_voice == voice))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    for index in indexes.into_iter().rev() {
        frame.events.insert(
            index,
            AudioEvent {
                sample_offset: offset,
                timer_cycles: 0,
                kind: AudioEventKind::SetNoteOrigin {
                    voice,
                    origin: AudioNoteOrigin::Sfx,
                },
                parity_dsp: None,
            },
        );
    }
}

fn discard_frame_music_events_for_voice_at_or_after(
    frame: &mut AudioEventFrame,
    voice: u8,
    cutoff: i32,
) {
    let Some(origin_index) = frame.events.iter().rposition(|event| {
        matches!(
            event.kind,
            AudioEventKind::SetNoteOrigin {
                voice: event_voice,
                origin: AudioNoteOrigin::Music,
            } if event_voice == voice
        )
    }) else {
        return;
    };

    let mut index = 0usize;
    frame.events.retain(|event| {
        let keep = index < origin_index
            || audio_event_voice(&event.kind) != Some(voice)
            || event.sample_offset < cutoff;
        index += 1;
        keep
    });
}

fn latest_music_note_offset_before(frame: &AudioEventFrame, cutoff: i32) -> Option<i32> {
    let mut origins = [None; 8];
    let mut latest = None;
    for event in &frame.events {
        match event.kind {
            AudioEventKind::SetNoteOrigin { voice, origin } => {
                origins[usize::from(voice)] = Some(origin);
            }
            AudioEventKind::NoteOn { voice, .. }
            | AudioEventKind::RetriggerVoice { voice }
            | AudioEventKind::KeyOnVoice { voice, .. }
                if origins[usize::from(voice)] == Some(AudioNoteOrigin::Music)
                    && event.sample_offset < cutoff =>
            {
                latest = Some(latest.map_or(event.sample_offset, |offset: i32| {
                    offset.max(event.sample_offset)
                }));
            }
            _ => {}
        }
    }
    latest
}

fn audio_event_voice(kind: &AudioEventKind) -> Option<u8> {
    match *kind {
        AudioEventKind::SetNoteOrigin { voice, .. }
        | AudioEventKind::SetPitchWord { voice, .. }
        | AudioEventKind::SetPitchRegisterWord { voice, .. }
        | AudioEventKind::SetStereoVolume { voice, .. }
        | AudioEventKind::SetDspEnvelope { voice, .. }
        | AudioEventKind::SetEnvelopeRateCounter { voice, .. }
        | AudioEventKind::NoteOn { voice, .. }
        | AudioEventKind::RetriggerVoice { voice }
        | AudioEventKind::KeyOnVoice { voice, .. }
        | AudioEventKind::NoteOff { voice }
        | AudioEventKind::SetDuration { voice, .. }
        | AudioEventKind::PitchSlide { voice, .. }
        | AudioEventKind::SetNoise { voice, .. }
        | AudioEventKind::SetPan { voice, .. }
        | AudioEventKind::SetEchoSend { voice, .. }
        | AudioEventKind::SetEnvelope { voice, .. }
        | AudioEventKind::VoiceParameter { voice, .. } => Some(voice),
        _ => None,
    }
}

fn first_nonzero<const N: usize>(values: [u8; N]) -> u8 {
    values.into_iter().find(|value| *value != 0).unwrap_or(0)
}

const fn default_music_master_volume() -> u16 {
    0x6000
}

const fn default_music_tempo() -> u8 {
    0x10
}

fn driver_tempo_for_track(track: u8) -> u8 {
    match track {
        // Title/file-select theme. This is the SPC driver's E7 tempo, not a
        // host-frame approximation; it determines the exact F1 fade cadence.
        0x0b => 0x21,
        _ => 0x20,
    }
}

fn catalog_echo_send(catalog_echo: bool, voice: u8, echo_mask: u8) -> bool {
    catalog_echo && echo_mask & (1 << voice) != 0
}

fn pitch_for_code(code: u8) -> u8 {
    36 + (code & 0x3f)
}

fn instrument_for_code(code: u8) -> u8 {
    code >> 4
}

fn tempo_for_track(track: u8) -> u8 {
    96 + (track & 0x1f)
}

fn fold_program_hash(accum: u32, program_hash: u32) -> u32 {
    let mut hash = if accum == 0 { 2166136261 } else { accum };
    for byte in program_hash.to_le_bytes() {
        hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_echo_send_never_invents_a_global_echo_enable_bit() {
        assert!(!catalog_echo_send(true, 7, 0x1f));
        assert!(catalog_echo_send(true, 7, 0x9f));
        assert!(!catalog_echo_send(true, 4, 0));
        assert!(!catalog_echo_send(false, 7, 0xff));
    }

    #[test]
    fn keyon_without_an_echo_event_inherits_the_global_echo_mask() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.music_echo_mask = 0x1f;
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        push_event_at(
            &mut frame,
            164,
            AudioEventKind::KeyOnVoice {
                voice: 6,
                source: 16,
                adsr1: 142,
                adsr2: 224,
                gain: 184,
                volume_left: 63,
                volume_right: 63,
                rate_counter: 0,
            },
        );

        sequencer.ensure_keyons_have_echo_state(&mut frame);

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 164
                && matches!(
                    event.kind,
                    AudioEventKind::SetEchoSend {
                        voice: 6,
                        enabled: false,
                    }
                )
        }));
    }

    #[test]
    fn same_sample_echo_writes_preserve_driver_order() {
        let mut sequencer = ModernAudioSequencer::default();
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);

        sequencer.emit_music_globals_at_position(0x0b, 1000, &mut frame);

        let values = frame
            .events
            .iter()
            .filter_map(|event| match event.kind {
                AudioEventKind::GlobalParameter {
                    register: 0x4d,
                    value,
                } if event.sample_offset == 28 => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(values, vec![63, 31]);
        assert_eq!(sequencer.music_echo_mask, 31);
    }

    #[test]
    fn same_sample_echo_chain_ends_on_the_narrowest_driver_mask() {
        let mut sequencer = ModernAudioSequencer::default();
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);

        sequencer.emit_music_globals_at_position(0x03, 2701, &mut frame);

        let values = frame
            .events
            .iter()
            .filter_map(|event| match event.kind {
                AudioEventKind::GlobalParameter {
                    register: 0x4d,
                    value,
                } if event.sample_offset == 444 => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(values, vec![63, 31, 15, 7, 3]);
        assert_eq!(sequencer.music_echo_mask, 3);
    }

    #[test]
    fn bank2_id9_preserves_music_writes_before_its_voice_takeover() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.previous_sfx_clock = Some((54, 104));
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        let takeover_offset = sfx_overflow_positions_after((54, 104), -1, 51)[1] as i32;
        push_event_at(
            &mut frame,
            takeover_offset - 128,
            AudioEventKind::VoiceParameter {
                voice: 5,
                parameter: VoiceParameterKind::PitchLow,
                value: 0x8b,
            },
        );
        push_event_at(
            &mut frame,
            0,
            AudioEventKind::KeyOnVoice {
                voice: 5,
                source: 12,
                adsr1: 0,
                adsr2: 0,
                gain: 0,
                volume_left: 0,
                volume_right: 0,
                rate_counter: 0,
            },
        );

        sequencer.reconcile_bank2_id9_command(
            true,
            &mut frame,
            &mut ModernAudioSequenceStats::default(),
        );

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == takeover_offset - 128
                && matches!(
                    event.kind,
                    AudioEventKind::VoiceParameter {
                        voice: 5,
                        parameter: VoiceParameterKind::PitchLow,
                        value: 0x8b,
                    }
                )
        }));
        assert!(!frame.events.iter().any(|event| {
            event.sample_offset == 0
                && matches!(
                    event.kind,
                    AudioEventKind::KeyOnVoice {
                        voice: 5,
                        source: 12,
                        ..
                    }
                )
        }));
    }

    #[test]
    fn mixed_music_keyon_restores_the_note_before_bank2_id9_takes_it() {
        let mut sequencer = ModernAudioSequencer {
            last_music_track: 0x03,
            music_frame_position: 3255,
            ..ModernAudioSequencer::default()
        };
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        let mut stats = ModernAudioSequenceStats::default();

        sequencer.emit_music_latch_side_effects(&mut frame, &mut stats);

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 288
                && matches!(
                    event.kind,
                    AudioEventKind::NoteOn {
                        voice: 5,
                        instrument: 10,
                        ..
                    }
                )
        }));
    }
    use crate::game_output::{AudioQueueState, MusicControlState, SpcSequencerState};

    #[test]
    fn sfx_clock_reproduces_phase_dependent_program_boundaries() {
        let cases = [
            ((6, 104), [(2, 3), (6, 7), (11, 1), (15, 5)]),
            ((8, 72), [(2, 4), (6, 7), (11, 2), (15, 6)]),
            ((54, 168), [(2, 1), (6, 5), (10, 8), (15, 3)]),
            ((42, 120), [(2, 3), (6, 6), (11, 0), (15, 4)]),
        ];

        for ((timer_cycles, sfx_timer_accum), expected) in cases {
            let actual = std::array::from_fn(|step| {
                sfx_clock_target(timer_cycles, sfx_timer_accum, 3 + step * 8)
            });
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn zero_phase_waits_until_sample_64_for_its_first_timer_tick() {
        assert_eq!(sfx_first_boundary(0), 64);
        assert_eq!(sfx_timer_tick_count(0), 8);
        assert_eq!(sfx_first_boundary(1), 63);
        assert_eq!(sfx_timer_tick_count(1), 8);
        assert_eq!(sfx_first_boundary(64), 0);
        assert_eq!(sfx_timer_tick_count(64), 9);
        assert_eq!(sfx_timer_cycles_after_frame(42), 64);
        assert_eq!(sfx_timer_cycles_after_frame(64), 22);
    }

    #[test]
    fn route_bank_one_effect_01_uses_observed_two_frame_lead_in() {
        let program = lookup_sfx_program_for_context(
            1,
            0x01,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0x5f,
            },
        )
        .unwrap();
        assert_eq!(program.variant_hash, 0x38d22304);

        let exact = exact_sfx_dsp_step(1, 0x01, program.variant_hash, 0, program.steps[0]).unwrap();
        assert_eq!(exact.command_delay_frames, 2);
        assert_eq!(exact.scheduler_tick_index, 4);
        assert_eq!(exact.volume_left, 44);

        let retrigger =
            exact_sfx_dsp_step(1, 0x01, program.variant_hash, 1, program.steps[1]).unwrap();
        assert_eq!(retrigger.command_delay_frames, 4);
        assert_eq!(retrigger.scheduler_tick_index, 6);
        assert_eq!(retrigger.volume_left, 75);
    }

    #[test]
    fn route_bank_one_effect_2b_uses_observed_scheduler_boundaries() {
        let program = lookup_sfx_program_for_context(
            1,
            0x2b,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0x7f,
            },
        )
        .unwrap();
        assert_eq!(program.variant_hash, 0x78dac40b);

        let first = exact_sfx_dsp_step(1, 0x2b, program.variant_hash, 0, program.steps[0]).unwrap();
        let second =
            exact_sfx_dsp_step(1, 0x2b, program.variant_hash, 1, program.steps[1]).unwrap();
        let third = exact_sfx_dsp_step(1, 0x2b, program.variant_hash, 2, program.steps[2]).unwrap();
        let fourth =
            exact_sfx_dsp_step(1, 0x2b, program.variant_hash, 3, program.steps[3]).unwrap();
        assert_eq!(
            (first.command_delay_frames, first.scheduler_tick_index),
            (2, 3)
        );
        assert_eq!(
            (second.command_delay_frames, second.scheduler_tick_index),
            (6, 7)
        );
        assert_eq!(
            (third.command_delay_frames, third.scheduler_tick_index),
            (11, 1)
        );
        assert_eq!(
            (fourth.command_delay_frames, fourth.scheduler_tick_index),
            (15, 5)
        );
        assert_eq!(
            (
                first.duration_samples,
                second.duration_samples,
                third.duration_samples,
                fourth.duration_samples
            ),
            (1792, 1728, 1792, 1728)
        );
    }

    #[test]
    fn bank_zero_effect_03_waits_for_its_observed_key_on_boundary() {
        let program = lookup_sfx_program_for_context(
            0,
            0x03,
            ModernSfxRuntimeContext {
                source_slot: 0,
                active_voice_mask: 0x24,
            },
        )
        .unwrap();
        assert_eq!(program.variant_hash, 0x83cc46a8);

        for (step_index, expected_pitch) in [(0, 450), (1, 356)] {
            let exact = exact_sfx_dsp_step(
                0,
                0x03,
                program.variant_hash,
                step_index,
                program.steps[step_index],
            )
            .unwrap();
            assert_eq!(
                (exact.command_delay_frames, exact.scheduler_tick_index),
                (2, 4)
            );
            assert_eq!(exact.dsp_pitch, expected_pitch);
        }
    }

    #[test]
    fn bank_zero_effect_03_retriggers_from_persistent_sfx_clock_state() {
        let program = lookup_sfx_program_for_context(
            0,
            0x03,
            ModernSfxRuntimeContext {
                source_slot: 0,
                active_voice_mask: 0x24,
            },
        )
        .unwrap();
        let exact = exact_sfx_dsp_step(0, 0x03, program.variant_hash, 0, program.steps[0]).unwrap();
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.previous_sfx_clock = Some((42, 72));
        sequencer.looping_sfx_voices[7] = Some(LoopingSfxVoice {
            step: program.steps[0],
            exact,
            overflows_remaining: 1,
            active: true,
            active_overflows: 193,
            gap_overflows: 2,
            retriggers_remaining: u8::MAX,
            retrigger_index: 0,
            staggered_chime: false,
        });

        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        let mut stats = ModernAudioSequenceStats::default();
        sequencer.advance_looping_sfx(&mut frame, &mut stats);
        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 214 && matches!(event.kind, AudioEventKind::NoteOff { voice: 7 })
        }));

        sequencer.previous_sfx_clock = Some((0, 8));
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        sequencer.advance_looping_sfx(&mut frame, &mut stats);
        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 320
                && matches!(
                    event.kind,
                    AudioEventKind::NoteOn { voice: 7, .. }
                        | AudioEventKind::KeyOnVoice { voice: 7, .. }
                )
        }));
        let looping = sequencer.looping_sfx_voices[7].unwrap();
        assert!(looping.active);
        assert_eq!(looping.overflows_remaining, 193);
    }

    #[test]
    fn duration_bounded_sfx_voice_expires_from_variant_context() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0xfe, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };

        sequencer.sequence_route(route);
        assert_eq!(sequencer.active_voice_mask & (1 << 1), 1 << 1);

        for _ in 0..6 {
            sequencer.sequence_route(route);
        }

        assert_eq!(sequencer.active_voice_mask & (1 << 1), 0);
    }

    #[test]
    fn same_voice_catalog_steps_advance_across_declared_frame_durations() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0, 0x0a],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };

        let program = lookup_sfx_program_for_context(
            2,
            0x0a,
            ModernSfxRuntimeContext {
                source_slot: 2,
                active_voice_mask: 0,
            },
        )
        .unwrap();
        assert_eq!(program.variant_hash, 0xdea3d882);
        assert_eq!(
            exact_sfx_dsp_step(2, 0x0a, program.variant_hash, 0, program.steps[0])
                .unwrap()
                .command_delay_frames,
            2
        );

        let command = sequencer.sequence_route(route);
        let lead_in = sequencer.sequence_route(route);
        let first = sequencer.sequence_route(route);
        let inter_step_gap = sequencer.sequence_route(route);
        let inter_step_gap_2 = sequencer.sequence_route(route);
        let second = sequencer.sequence_route(route);

        assert!(!command
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. })));
        assert!(!lead_in
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. })));

        assert_eq!(
            first
                .events
                .iter()
                .filter(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. }))
                .count(),
            1
        );
        assert!([inter_step_gap, inter_step_gap_2].iter().all(|frame| !frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. }))));
        assert_eq!(
            second
                .events
                .iter()
                .filter(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 7, .. }))
                .count(),
            1
        );
    }

    #[test]
    fn catalog_steps_emit_route_derived_stereo_pan() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0x03, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };
        sequencer.sequence_route(route);
        sequencer.sequence_route(route);
        let frame = sequencer.sequence_route(route);

        assert!(frame.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::SetPan {
                voice: 7,
                pan: -127
            }
        )));
        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::SetPan { voice: 6, pan: 127 })));
    }

    #[test]
    fn sequences_music_track_into_play_and_note_intents() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            music: MusicControlState {
                music_control: 0x12,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        };
        let frame = sequencer.sequence_route(route);

        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 0x12 })));
        assert!(!frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { .. })));
        assert_eq!(sequencer.last_stats().music_commands, 1);
        assert_eq!(sequencer.last_stats().note_events, 0);

        for _ in 0..3 {
            let lead_in = sequencer.sequence_route(route);
            assert!(!lead_in
                .events
                .iter()
                .any(|event| matches!(event.kind, AudioEventKind::NoteOn { .. })));
        }

        let first_notes = sequencer.sequence_route(route);
        assert_eq!(
            first_notes
                .events
                .iter()
                .filter(|event| matches!(event.kind, AudioEventKind::NoteOn { instrument: 10, .. }))
                .count(),
            2
        );
        assert_eq!(sequencer.last_stats().note_events, 2);
    }

    #[test]
    fn queued_track_outlives_stale_music_control_after_live_port_clears() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0x14, 0, 0, 0],
                ..AudioQueueState::default()
            },
            music: MusicControlState {
                music_control: 0x10,
                queued_music_control: 0x10,
                last_music_control: 0x14,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        });

        let continued = sequencer.sequence_route(AudioRouteState {
            music: MusicControlState {
                music_control: 0x10,
                queued_music_control: 0x10,
                last_music_control: 0x14,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        });

        assert_eq!(sequencer.last_music_track, 0x14);
        assert!(!continued
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 0x10 })));
    }

    #[test]
    fn transition_command_keeps_advancing_the_active_track() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0x01, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });
        let before = sequencer.music_frame_position;

        let transition = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0xf1, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        assert_eq!(sequencer.last_music_track, 0x01);
        assert_eq!(sequencer.music_frame_position, before + 1);
        assert!(!transition
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::SetMusicVolume { .. })));
        assert_eq!(sequencer.last_stats().ignored_commands, 0);

        let fading = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0xf1, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });
        assert!(fading.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::SetMusicVolume { value } if value < 0x60
        )));
        assert_eq!(sequencer.last_stats().ignored_commands, 0);
    }

    #[test]
    fn looping_house_music_keeps_emitting_after_the_capture_frontier() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0x03, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        let mut notes_after_capture = 0usize;
        for frame_index in 1..=7_000 {
            let frame = sequencer.sequence_route(AudioRouteState {
                music: MusicControlState {
                    last_music_control: 0x03,
                    ..MusicControlState::default()
                },
                ..AudioRouteState::default()
            });
            if frame_index > 5_200 {
                notes_after_capture += frame
                    .events
                    .iter()
                    .filter(|event| matches!(event.kind, AudioEventKind::NoteOn { .. }))
                    .count();
            }
        }

        assert!(
            notes_after_capture > 0,
            "track 3 must loop instead of becoming permanently silent after its captured events end"
        );
    }

    #[test]
    fn transition_fade_runs_for_the_full_128_music_ticks() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0x0b, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0xf1, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        let mut volume_changes = Vec::new();
        for _ in 0..160 {
            let frame = sequencer.sequence_route(AudioRouteState {
                queue: AudioQueueState {
                    input: [0xf1, 0, 0, 0],
                    ..AudioQueueState::default()
                },
                ..AudioRouteState::default()
            });
            volume_changes.extend(frame.events.iter().filter_map(|event| match event.kind {
                AudioEventKind::SetMusicVolume { value } => Some(value),
                _ => None,
            }));
        }

        assert_eq!(volume_changes.len(), 0x80);
        assert_eq!(volume_changes.first(), Some(&0x5f));
        assert_eq!(volume_changes.last(), Some(&0));
        assert!(volume_changes.windows(2).all(|pair| pair[1] <= pair[0]));
        assert_eq!(sequencer.music_master_fade_ticks, 0);
        assert_eq!(sequencer.music_master_volume, 0);
    }

    #[test]
    fn receipt_mode_does_not_apply_music_master_fade_twice() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.engine_receipt_mode = true;
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0x0b, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0xf1, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        let fading = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0xf1, 0, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(sequencer.music_master_fade_ticks < 0x80);
        assert!(!fading
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::SetMusicVolume { .. })));
    }

    #[test]
    fn zero_lead_in_track_emits_its_first_notes_on_the_command_frame() {
        let mut sequencer = ModernAudioSequencer::default();
        let frame = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0x07, 0, 0, 0],
                ..AudioQueueState::default()
            },
            music: MusicControlState {
                last_music_control: 0x07,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(frame.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn { instrument: 0, .. }
                | AudioEventKind::NoteOn { instrument: 9, .. }
                | AudioEventKind::NoteOn { instrument: 10, .. }
                | AudioEventKind::NoteOn { instrument: 22, .. }
        )));
        assert_eq!(sequencer.music_frame_position, 0);
    }

    #[test]
    fn active_track_one_uses_trace_backed_delayed_multi_note_sequence() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            music: MusicControlState {
                last_music_control: 0x01,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        };

        let start = sequencer.sequence_route(route);
        assert!(start
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 0x01 })));
        assert!(!start
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 0, .. })));

        for _ in 0..5 {
            let frame = sequencer.sequence_route(route);
            assert!(!frame
                .events
                .iter()
                .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 0, .. })));
        }
        let first_note = sequencer.sequence_route(route);
        assert!(first_note.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn {
                voice: 1,
                pitch: 60,
                ..
            }
        )));

        for _ in 0..5 {
            sequencer.sequence_route(route);
        }
        let second_note = sequencer.sequence_route(route);
        assert!(second_note.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn {
                voice: 0,
                pitch: 53,
                ..
            }
        )));
    }

    #[test]
    fn active_track_0b_starts_trace_backed_stereo_chord_after_lead_in() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            music: MusicControlState {
                last_music_control: 0x0b,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        };

        let start = sequencer.sequence_route(route);
        assert!(start
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 0x0b })));
        for _ in 0..24 {
            let frame = sequencer.sequence_route(route);
            assert!(!frame
                .events
                .iter()
                .any(|event| matches!(event.kind, AudioEventKind::NoteOn { .. })));
        }

        let chord = sequencer.sequence_route(route);
        assert_eq!(
            chord
                .events
                .iter()
                .filter(|event| matches!(event.kind, AudioEventKind::NoteOn { .. }))
                .count(),
            2
        );
        assert!(chord.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn {
                voice: 0,
                pitch: 90,
                ..
            }
        )));
        assert!(chord
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::SetPan { voice: 0, pan: 127 })));
    }

    #[test]
    fn sequences_sfx_ports_once_until_command_changes() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0x34, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };

        let first = sequencer.sequence_route(route);
        let first_stats = sequencer.last_stats();
        let second = sequencer.sequence_route(route);

        assert!(first
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlaySfx { bank: 0, id: 0x34 })));
        assert!(first
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 1, .. })));
        assert!(first.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::PitchSlide {
                voice: 1,
                target_pitch: 38,
                frames: 8
            }
        )));
        assert_eq!(first_stats.known_sfx_commands, 1);
        assert_eq!(first_stats.unknown_sfx_commands, 0);
        assert_eq!(first_stats.known_sfx_program_count, 1);
        assert_eq!(first_stats.known_sfx_programs[0], 0x0034);
        assert_ne!(first_stats.program_hash, 0);
        assert!(!second.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::PlaySfx { .. } | AudioEventKind::NoteOn { .. }
        )));
    }

    #[test]
    fn clearing_sfx_port_does_not_key_off_slot_numbered_music_voice() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0x01, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        let cleared = sequencer.sequence_route(AudioRouteState::default());

        assert!(!cleared
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOff { voice: 2 })));
    }

    #[test]
    fn apui00_music_command_is_not_duplicated_as_bank_six_sfx() {
        let mut sequencer = ModernAudioSequencer::default();
        let frame = sequencer.sequence_route(AudioRouteState {
            music: MusicControlState {
                apui00: 1,
                ..MusicControlState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlayMusic { track: 1 })));
        assert!(!frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlaySfx { bank: 6, .. })));
        assert_eq!(sequencer.last_stats().sfx_commands, 0);
    }

    #[test]
    fn semantic_sfx_waits_for_kon_and_emits_route_derived_echo_send() {
        let mut sequencer = ModernAudioSequencer::default();
        let command_route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0x01, 0],
                ..AudioQueueState::default()
            },
            spc: Some(SpcSequencerState {
                raw_volume_masks: [0x80; 32],
                raw_volume_count: 1,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        };
        let command = sequencer.sequence_route(command_route);
        assert!(!command.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::NoteOn { voice: 7, .. } | AudioEventKind::SetEchoSend { voice: 7, .. }
        )));

        let mut sfx_kon_sources = [[0; 8]; 8];
        sfx_kon_sources[0][7] = 3;
        let receipt = sequencer.sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                is_chan_on: 0x80,
                sfx_kon_masks: [0x80, 0, 0, 0, 0, 0, 0, 0],
                sfx_kon_owned_masks: [0x80, 0, 0, 0, 0, 0, 0, 0],
                sfx_kon_offsets: [123, 0, 0, 0, 0, 0, 0, 0],
                sfx_kon_count: 1,
                sfx_kon_sources,
                sfx_setup_masks: [0x80, 0, 0, 0, 0, 0, 0, 0],
                sfx_setup_sources: [3, 0, 0, 0, 0, 0, 0, 0],
                sfx_setup_offsets: [123, 0, 0, 0, 0, 0, 0, 0],
                sfx_setup_count: 1,
                echo_enable_frame_start: 0,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });
        assert!(receipt.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::SetEchoSend {
                voice: 7,
                enabled: false
            } if event.sample_offset == 123
        )));
        assert!(receipt.events.iter().any(|event| matches!(
            event.kind,
            AudioEventKind::KeyOnVoice { voice: 7, source: 3, .. }
                if event.sample_offset == 123
        )));
    }

    #[test]
    fn engine_receipts_authoritatively_drive_sfx_keyoffs_and_voice_ownership() {
        let mut sequencer = ModernAudioSequencer::default();
        let keyoff = sequencer.sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                is_chan_on: 0x80,
                sfx_kof_masks: [0x80, 0, 0, 0, 0, 0, 0, 0],
                sfx_kof_offsets: [106, 0, 0, 0, 0, 0, 0, 0],
                sfx_kof_count: 1,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        assert!(keyoff.events.iter().any(|event| {
            event.sample_offset == 106 && matches!(event.kind, AudioEventKind::NoteOff { voice: 7 })
        }));
        assert_eq!(sequencer.last_stats().sfx_voice_mask, 0x80);

        sequencer.sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState::default()),
            ..AudioRouteState::default()
        });
        assert_eq!(sequencer.last_stats().sfx_voice_mask, 0);
    }

    #[test]
    fn title_startup_sfx_matches_delayed_keyoff_and_engine_ownership_window() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0, 0x0a],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };
        let mut keyoffs = Vec::new();
        let mut ownership = Vec::new();

        for frame_number in 1..=27 {
            let frame = sequencer.sequence_route(route);
            keyoffs.extend(frame.events.iter().filter_map(|event| {
                matches!(event.kind, AudioEventKind::NoteOff { voice: 7 })
                    .then_some((frame_number, event.sample_offset))
            }));
            ownership.push(sequencer.last_stats().sfx_voice_mask);
        }

        assert!(keyoffs.contains(&(2, 106)));
        assert!(keyoffs.contains(&(26, 410)));
        assert_eq!(ownership[0], 0);
        assert!(ownership[1..26].iter().all(|mask| *mask == 0x80));
        assert_eq!(ownership[26], 0);
    }

    #[test]
    fn multi_step_exact_sfx_keeps_dsp_parameters_on_every_keyon() {
        let mut sequencer = ModernAudioSequencer::default();
        let route = AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0x2b, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        };
        let mut exact_keyons = 0;
        let mut exact_keyoffs = 0;

        for frame_number in 1..=20 {
            let frame = sequencer.sequence_route(route);
            exact_keyoffs += frame
                .events
                .iter()
                .filter(|event| matches!(event.kind, AudioEventKind::NoteOff { voice: 7 }))
                .count();
            for event in frame.events.iter().filter(|event| {
                matches!(
                    event.kind,
                    AudioEventKind::NoteOn {
                        voice: 7,
                        instrument: 14,
                        ..
                    }
                )
            }) {
                assert!(frame.events.iter().any(|candidate| {
                    candidate.sample_offset == event.sample_offset
                        && matches!(
                            candidate.kind,
                            AudioEventKind::SetPitchWord {
                                voice: 7,
                                pitch_word: 8414,
                            }
                        )
                }), "frame {frame_number} emitted a catalog SFX key-on without its exact pitch: {:?}", frame.events);
                exact_keyons += 1;
            }
        }

        assert_eq!(exact_keyons, 4);
        assert_eq!(exact_keyoffs, 5);
    }

    #[test]
    fn raw_keyoff_receipts_are_not_truncated_after_eight_writes() {
        let mut raw_kof_masks = [0; 32];
        let mut raw_kof_offsets = [0; 32];
        raw_kof_masks[8] = 0x02;
        raw_kof_offsets[8] = 298;

        let frame = ModernAudioSequencer::default().sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                raw_kof_masks,
                raw_kof_offsets,
                raw_kof_count: 9,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 298 && matches!(event.kind, AudioEventKind::NoteOff { voice: 1 })
        }));
    }

    #[test]
    fn raw_envelope_register_receipts_preserve_active_voice_updates() {
        let mut raw_envelope_masks = [0; 32];
        let mut raw_envelope_registers = [0; 32];
        let mut raw_envelope_values = [0; 32];
        let mut raw_envelope_offsets = [0; 32];
        raw_envelope_masks[0] = 0x09;
        raw_envelope_registers[0] = 6;
        raw_envelope_values[0] = 0xe0;
        raw_envelope_offsets[0] = 91;

        let frame = ModernAudioSequencer::default().sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                raw_envelope_masks,
                raw_envelope_registers,
                raw_envelope_values,
                raw_envelope_offsets,
                raw_envelope_count: 1,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        for voice in [0, 3] {
            assert!(frame.events.iter().any(|event| {
                event.sample_offset == 91
                    && matches!(
                        event.kind,
                        AudioEventKind::VoiceParameter {
                            voice: event_voice,
                            parameter: VoiceParameterKind::Adsr2,
                            value: 0xe0,
                        } if event_voice == voice
                    )
            }));
        }
    }

    #[test]
    fn full_raw_volume_receipt_buffer_is_still_authoritative() {
        let mut raw_volume_masks = [0; 32];
        let mut raw_volume_left = [0; 32];
        let mut raw_volume_right = [0; 32];
        let mut raw_volume_offsets = [0; 32];
        raw_volume_masks[31] = 0x01;
        raw_volume_left[31] = 15;
        raw_volume_right[31] = -2;
        raw_volume_offsets[31] = 524;

        let frame = ModernAudioSequencer::default().sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                raw_volume_masks,
                raw_volume_left,
                raw_volume_right,
                raw_volume_offsets,
                raw_volume_count: 32,
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 524
                && matches!(
                    event.kind,
                    AudioEventKind::SetStereoVolume {
                        voice: 0,
                        left: 15,
                        right: -2,
                    }
                )
        }));
    }

    #[test]
    fn music_keyon_uses_volume_at_kon_not_end_of_frame_volume() {
        let mut sfx_kon_rate_counters = [[0; 8]; 8];
        sfx_kon_rate_counters[0][7] = 1;
        let mut sfx_kon_volume_left = [[0; 8]; 8];
        let mut sfx_kon_volume_right = [[0; 8]; 8];
        sfx_kon_volume_left[0][7] = 23;
        sfx_kon_volume_right[0][7] = 0;
        let mut raw_volume_masks = [0; 32];
        let mut raw_volume_left = [0; 32];
        let mut raw_volume_right = [0; 32];
        let mut raw_volume_offsets = [0; 32];
        raw_volume_masks[..2].copy_from_slice(&[0x80, 0x80]);
        raw_volume_left[..2].copy_from_slice(&[23, 23]);
        raw_volume_right[..2].copy_from_slice(&[0, 1]);
        raw_volume_offsets[..2].copy_from_slice(&[50, 498]);

        let frame = ModernAudioSequencer::default().sequence_route(AudioRouteState {
            spc: Some(SpcSequencerState {
                sfx_kon_masks: [0x80, 0, 0, 0, 0, 0, 0, 0],
                sfx_kon_offsets: [370, 0, 0, 0, 0, 0, 0, 0],
                sfx_kon_count: 1,
                sfx_kon_rate_counters,
                sfx_kon_volume_left,
                sfx_kon_volume_right,
                raw_volume_masks,
                raw_volume_left,
                raw_volume_right,
                raw_volume_offsets,
                raw_volume_count: 2,
                voice_volume_left: [23; 8],
                voice_volume_right: [1; 8],
                ..SpcSequencerState::default()
            }),
            ..AudioRouteState::default()
        });

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 370
                && matches!(
                    event.kind,
                    AudioEventKind::KeyOnVoice {
                        voice: 7,
                        volume_left: 23,
                        volume_right: 0,
                        rate_counter: 1,
                        ..
                    }
                )
        }));
    }

    #[test]
    fn engine_receipt_mode_suppresses_route_derived_music_registers() {
        let mut sequencer = ModernAudioSequencer::default();
        let mut fallback =
            AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        sequencer.emit_music_globals_at_position(1, 6, &mut fallback);
        assert!(fallback
            .events
            .iter()
            .any(|event| { matches!(event.kind, AudioEventKind::VoiceParameter { .. }) }));

        sequencer.engine_receipt_mode = true;
        let mut live = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        sequencer.emit_music_globals_at_position(1, 6, &mut live);
        assert!(!live.events.iter().any(|event| {
            matches!(
                event.kind,
                AudioEventKind::VoiceParameter { .. } | AudioEventKind::GlobalParameter { .. }
            )
        }));
    }

    #[test]
    fn engine_keyons_use_ownership_at_each_write_not_end_of_frame() {
        let mut masks = [0; 8];
        masks[0] = 0x30;
        masks[1] = 0x20;
        let mut owned_masks = [0; 8];
        owned_masks[1] = 0x20;
        let mut offsets = [0; 8];
        offsets[0] = 10;
        offsets[1] = 368;
        let spc = SpcSequencerState {
            is_chan_on: 0xe0,
            sfx_kon_masks: masks,
            sfx_kon_owned_masks: owned_masks,
            sfx_kon_offsets: offsets,
            sfx_kon_count: 2,
            ..SpcSequencerState::default()
        };
        let mut sequencer = ModernAudioSequencer::default();
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        let mut stats = ModernAudioSequenceStats::default();

        sequencer.emit_engine_music_keyons(Some(spc), &mut frame, &mut stats);

        for voice in [4, 5] {
            assert!(frame.events.iter().any(|event| {
                event.sample_offset == 10
                    && matches!(
                        event.kind,
                        AudioEventKind::KeyOnVoice {
                            voice: event_voice,
                            ..
                        } if event_voice == voice
                    )
            }));
        }
        assert!(!frame.events.iter().any(|event| {
            event.sample_offset == 368
                && matches!(event.kind, AudioEventKind::KeyOnVoice { voice: 5, .. })
        }));
    }

    #[test]
    fn retained_music_kon_bits_are_emitted_after_sfx_reconciliation() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.last_music_track = 3;
        sequencer.music_frame_position = 600;
        let mut frame = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        push_event_at(
            &mut frame,
            10,
            AudioEventKind::NoteOn {
                voice: 4,
                pitch: 12,
                instrument: 10,
                volume: 18,
            },
        );

        let mut stats = ModernAudioSequenceStats::default();
        sequencer.emit_music_latch_side_effects(&mut frame, &mut stats);

        assert!(frame.events.iter().any(|event| {
            event.sample_offset == 10
                && matches!(event.kind, AudioEventKind::NoteOn { voice: 5, .. })
        }));
        assert!(!frame.events.iter().any(|event| {
            event.sample_offset == 10
                && matches!(event.kind, AudioEventKind::RetriggerVoice { voice: 4 })
        }));
    }

    #[test]
    fn unknown_sfx_uses_fallback_and_reports_coverage_gap() {
        let mut sequencer = ModernAudioSequencer::default();
        let frame = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0xfe, 0, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { voice: 1, .. })));
        assert_eq!(sequencer.last_stats().known_sfx_commands, 0);
        assert_eq!(sequencer.last_stats().unknown_sfx_commands, 1);
        assert_eq!(sequencer.last_stats().unknown_sfx_program_count, 1);
        assert_eq!(sequencer.last_stats().unknown_sfx_programs[0], 0x00fe);
        assert_eq!(sequencer.last_stats().fallback_sfx_commands, 1);
        assert_eq!(sequencer.last_stats().program_hash, 0);
    }

    #[test]
    fn context_only_sfx_is_known_without_inventing_a_fallback_note() {
        let mut sequencer = ModernAudioSequencer::default();
        let frame = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0, 0x83],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        assert!(frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::PlaySfx { bank: 2, id: 0x83 })));
        assert!(!frame
            .events
            .iter()
            .any(|event| matches!(event.kind, AudioEventKind::NoteOn { .. })));
        assert_eq!(sequencer.last_stats().known_sfx_commands, 1);
        assert_eq!(sequencer.last_stats().fallback_sfx_commands, 0);
    }

    #[test]
    fn title_sword_sfx_uses_exact_sample_duration_for_keyoff() {
        let program = lookup_sfx_program_for_context(
            1,
            0x2c,
            ModernSfxRuntimeContext {
                source_slot: 1,
                active_voice_mask: 0,
            },
        )
        .unwrap();
        let step_index = program
            .steps
            .iter()
            .position(|step| step.duration_frames == 0)
            .expect("title sword program must include an exact-duration tail");
        let step = program.steps[step_index];
        let exact = exact_sfx_dsp_step(
            program.bank,
            program.id,
            program.variant_hash,
            step_index,
            step,
        )
        .unwrap();
        assert!(exact.duration_samples > 534);

        let mut sequencer = ModernAudioSequencer::default();
        let mut start = AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        let mut stats = ModernAudioSequenceStats::default();
        sequencer.emit_sfx_step_at(
            &mut start,
            PendingSfxStep {
                step,
                exact: Some(exact),
                engine_dsp_envelope: None,
                delay_after_previous: 0,
                preserve_existing_volume: false,
                volume_via_parameters: false,
                refresh_repeat_on_keyon: false,
                preserve_inactive_pitch_latch: false,
                engine_keyoff_owned: false,
            },
            0,
            &mut stats,
        );

        let keyoff_frame = exact.duration_samples / 534;
        for _ in 1..keyoff_frame {
            let mut frame =
                AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
            sequencer.advance_sfx_keyoffs(&mut frame, &mut stats);
            assert!(!frame
                .events
                .iter()
                .any(|event| matches!(event.kind, AudioEventKind::NoteOff { voice } if voice == step.voice)));
        }

        let mut keyoff =
            AudioEventFrame::from_route_and_dsp_writes(AudioRouteState::default(), &[]);
        sequencer.advance_sfx_keyoffs(&mut keyoff, &mut stats);
        assert!(keyoff.events.iter().any(|event| {
            event.sample_offset == (exact.duration_samples % 534) as i32
                && matches!(event.kind, AudioEventKind::NoteOff { voice } if voice == step.voice)
        }));
        assert_ne!(sequencer.active_voice_mask & (1 << step.voice), 0);
        sequencer.release_finished_sfx_ownership();
        assert_eq!(sequencer.active_voice_mask & (1 << step.voice), 0);
    }

    #[test]
    fn title_sword_interrupt_disables_echo_at_the_keyoff_boundary() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.set_sfx_clock_phase(24, 216);
        let frame = sequencer.sequence_route(AudioRouteState {
            queue: AudioQueueState {
                input: [0, 0, 0x2c, 0],
                ..AudioQueueState::default()
            },
            ..AudioRouteState::default()
        });

        let keyoffs = frame
            .events
            .iter()
            .filter_map(|event| match event.kind {
                AudioEventKind::NoteOff { voice } if event.sample_offset > 0 => {
                    Some((voice, event.sample_offset))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(!keyoffs.is_empty());
        for (voice, sample_offset) in keyoffs {
            assert!(frame.events.iter().any(|event| {
                event.sample_offset == sample_offset
                    && matches!(
                        event.kind,
                        AudioEventKind::SetEchoSend {
                            voice: echo_voice,
                            enabled: false,
                        } if echo_voice == voice
                    )
            }));
        }
    }

    #[test]
    fn startup_boot_chime_schedule_is_clock_relative() {
        let mut sequencer = ModernAudioSequencer::default();
        sequencer.set_sfx_clock_phase(0, 200);
        for _ in 0..82 {
            sequencer.sequence_route(AudioRouteState::default());
        }
        let mut keyons = Vec::new();
        let mut keyoffs = Vec::new();
        for frame_index in 82..90 {
            let route = AudioRouteState {
                queue: AudioQueueState {
                    input: if frame_index == 82 {
                        [0, 0, 0, 0x0a]
                    } else {
                        [0; 4]
                    },
                    ..AudioQueueState::default()
                },
                ..AudioRouteState::default()
            };
            let frame = sequencer.sequence_route(route);
            for event in frame.events {
                match event.kind {
                    AudioEventKind::NoteOn { voice: 7, .. } => {
                        keyons.push((frame_index, event.sample_offset));
                    }
                    AudioEventKind::NoteOff { voice: 7 } => {
                        keyoffs.push((frame_index, event.sample_offset));
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(keyons, vec![(84, 360), (86, 444), (89, 58)]);
        assert_eq!(keyoffs, vec![(83, 94), (85, 361), (87, 445)]);
    }

    #[test]
    fn startup_boot_chime_is_silent_until_its_first_scheduled_keyon() {
        let mut sequencer = ModernAudioSequencer::default();
        let mut renderer = crate::modern_audio::ModernAudioEngine::default();
        sequencer.set_sfx_clock_phase(0, 200);
        let mut audio = vec![0i16; 534 * 2];

        for _ in 0..82 {
            let frame = sequencer.sequence_route(AudioRouteState::default());
            renderer.render_frame(&frame, &mut audio, 534, 2);
        }

        for frame_index in 82..=84 {
            let frame = sequencer.sequence_route(AudioRouteState {
                queue: AudioQueueState {
                    input: if frame_index == 82 {
                        [0, 0, 0, 0x0a]
                    } else {
                        [0; 4]
                    },
                    ..AudioQueueState::default()
                },
                ..AudioRouteState::default()
            });
            renderer.render_frame(&frame, &mut audio, 534, 2);
            if frame_index < 84 {
                assert!(
                    audio.iter().all(|sample| *sample == 0),
                    "startup audio leaked before frame 84 at frame {frame_index}"
                );
            } else {
                assert!(audio.iter().any(|sample| *sample != 0));
            }
        }
    }
}
