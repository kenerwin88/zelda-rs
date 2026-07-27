// Methods ported from zelda3/src/audio.c and included inside ZeldaState.

use super::*;
use crate::config::{config_value_path, MSU_FEATURE_MSU_DELUXE, MSU_FEATURE_OPUZ};
use crate::game_output::{
    AudioEventFrame, AudioQueueState, AudioRouteState, DspWriteEvent, EngineAudioCommand,
    EngineAudioCommandBatch, GameFrameOutput, MusicControlState, RenderOutputFacts,
    RuntimeOutputFacts, SpcSequencerState,
};
use crate::game_state::constants::INIDISP_COPY;
use crate::modern_audio::{ModernAudioEngine, ModernAudioFrameStats};
use crate::modern_audio_sequence::{ModernAudioSequenceStats, ModernAudioSequencer};
use opus::{Channels, Decoder as OpusDecoder};
use std::borrow::Cow;
use std::fs;

#[path = "audio/msu_runtime.rs"]
mod msu_runtime;
#[path = "audio/snapshot_state.rs"]
mod snapshot_state;
const MSU_STATE_IDLE: u8 = 0;
const MSU_STATE_FINISHED_PLAYING: u8 = 1;
const MSU_STATE_RESUMING: u8 = 2;
const MSU_STATE_PLAYING: u8 = 3;
const AUDIO_SNAPSHOT_MAGIC: [u8; 4] = *b"Z3AU";
const AUDIO_SNAPSHOT_VERSION: u16 = 7;
const AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR: u16 = 1;
const AUDIO_SNAPSHOT_HEADER_BYTES: usize = 12;

const MSU_TRACK_REPEATS: [u8; 48] = [
    1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1,
    1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

const MSU_DELUXE_TRACK_ROUTE: [u8; 32] = [
    0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 2, 2, 2, 0, 0, 0, 2, 2, 0, 0, 0, 2, 0, 0, 0, 0,
];

const MSU_DELUXE_OVERWORLD_TRACKS: [u8; 160] = [
    37, 37, 42, 38, 38, 38, 38, 39, 37, 37, 42, 38, 38, 38, 38, 41, 42, 42, 42, 42, 42, 42, 40, 40,
    43, 43, 42, 47, 47, 42, 45, 45, 43, 43, 43, 47, 47, 42, 45, 45, 112, 112, 48, 42, 42, 42, 42,
    45, 44, 44, 48, 48, 48, 46, 46, 46, 44, 44, 44, 48, 48, 46, 46, 46, 49, 49, 51, 50, 50, 50, 50,
    50, 49, 49, 51, 50, 50, 50, 50, 51, 51, 51, 51, 51, 51, 51, 51, 51, 52, 52, 51, 56, 56, 51, 54,
    54, 52, 52, 52, 56, 56, 51, 54, 54, 58, 52, 57, 51, 51, 51, 51, 54, 53, 53, 57, 57, 57, 55, 55,
    110, 53, 53, 57, 57, 57, 55, 55, 110, 37, 41, 41, 42, 42, 42, 42, 42, 42, 41, 41, 42, 42, 42,
    42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42,
];

const MSU_DELUXE_ENTRANCE_TRACKS: [u8; 133] = [
    59, 59, 60, 61, 61, 61, 62, 62, 63, 64, 64, 64, 105, 65, 65, 66, 66, 62, 67, 62, 62, 68, 62,
    62, 68, 68, 62, 62, 62, 62, 62, 62, 62, 62, 62, 62, 69, 70, 71, 72, 73, 73, 73, 106, 102, 74,
    62, 62, 75, 75, 76, 77, 78, 68, 79, 80, 81, 62, 62, 62, 82, 75, 242, 59, 59, 76, 242, 242, 242,
    96, 83, 99, 59, 242, 242, 242, 84, 95, 104, 62, 85, 62, 62, 86, 242, 67, 103, 83, 83, 87, 76,
    88, 81, 98, 81, 88, 83, 89, 75, 97, 90, 91, 91, 100, 92, 93, 92, 242, 93, 107, 62, 75, 62, 67,
    62, 242, 242, 242, 73, 73, 73, 73, 102, 114, 81, 76, 62, 67, 62, 61, 94, 62, 103,
];

const MSU_VOLUME_TRANSITION_TARGETS: [u8; 4] = [0, 64, 255, 255];
const MSU_VOLUME_TRANSITION_STEPS: [u8; 4] = [7, 3, 3, 24];
const FEATURES0_MISC_BUG_FIXES_AUDIO: u32 = 4096;
const MSU1_TAG: u32 = (b'1' as u32) << 24 | (b'U' as u32) << 16 | (b'S' as u32) << 8 | b'M' as u32;
const OPUZ_TAG: u32 = (b'Z' as u32) << 24 | (b'U' as u32) << 16 | (b'P' as u32) << 8 | b'O' as u32;

pub(super) struct MsuPlayer {
    buffer_size: u32,
    buffer_pos: u32,
    preskip: u32,
    samples_until_repeat: u32,
    total_samples_in_file: u32,
    repeat_position: u32,
    cur_file_offs: u32,
    resume_info: MsuResumeInfoState,
    enabled: u8,
    state: u8,
    volume: f32,
    volume_step: f32,
    volume_target: f32,
    range_cur: u16,
    range_repeat: u16,
    has_file: bool,
    has_opus: bool,
    opus_decoder: Option<OpusDecoder>,
    pcm_data: Vec<i16>,
    opuz_data: Vec<u8>,
    buffer: [i16; 960 * 2],
}

impl Clone for MsuPlayer {
    fn clone(&self) -> Self {
        Self {
            buffer_size: self.buffer_size,
            buffer_pos: self.buffer_pos,
            preskip: self.preskip,
            samples_until_repeat: self.samples_until_repeat,
            total_samples_in_file: self.total_samples_in_file,
            repeat_position: self.repeat_position,
            cur_file_offs: self.cur_file_offs,
            resume_info: self.resume_info,
            enabled: self.enabled,
            state: self.state,
            volume: self.volume,
            volume_step: self.volume_step,
            volume_target: self.volume_target,
            range_cur: self.range_cur,
            range_repeat: self.range_repeat,
            has_file: self.has_file,
            has_opus: self.has_opus,
            opus_decoder: None,
            pcm_data: self.pcm_data.clone(),
            opuz_data: self.opuz_data.clone(),
            buffer: self.buffer,
        }
    }
}

impl Default for MsuPlayer {
    fn default() -> Self {
        Self {
            buffer_size: 0,
            buffer_pos: 0,
            preskip: 0,
            samples_until_repeat: 0,
            total_samples_in_file: 0,
            repeat_position: 0,
            cur_file_offs: 0,
            resume_info: MsuResumeInfoState::default(),
            enabled: 0,
            state: MSU_STATE_IDLE,
            volume: 0.0,
            volume_step: 0.0,
            volume_target: 0.0,
            range_cur: 0,
            range_repeat: 0,
            has_file: false,
            has_opus: false,
            opus_decoder: None,
            pcm_data: Vec::new(),
            opuz_data: Vec::new(),
            buffer: [0; 960 * 2],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpuzPacketStatus {
    Decoded(u32),
    FinishedPlaying,
    ReadError,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ApuWriteEnt {
    ports: [u8; 4],
}

impl ApuWriteEnt {
    fn from_commands(commands: EngineAudioCommandBatch) -> Self {
        Self {
            ports: commands.legacy_ports(),
        }
    }

    fn decoded_commands(self) -> EngineAudioCommandBatch {
        EngineAudioCommandBatch::from_legacy_ports(self.ports)
    }
}

/// Production command transport. Typed batches are authoritative; APUI bytes
/// are projected only by legacy snapshot and oracle adapters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ModernAudioCommandQueue {
    write_history: [EngineAudioCommandBatch; 16],
    pending_write: EngineAudioCommandBatch,
    write_position: u8,
    write_count: u8,
    total_writes: u8,
    input_commands: EngineAudioCommandBatch,
    acknowledged_commands: EngineAudioCommandBatch,
}

impl ModernAudioCommandQueue {
    fn from_legacy_transport(
        write_history: [ApuWriteEnt; 16],
        pending_write: ApuWriteEnt,
        write_position: u8,
        write_count: u8,
        total_writes: u8,
        input_ports: [u8; 4],
        output_ports: [u8; 4],
    ) -> Self {
        Self {
            write_history: write_history.map(ApuWriteEnt::decoded_commands),
            pending_write: pending_write.decoded_commands(),
            write_position,
            write_count,
            total_writes,
            input_commands: EngineAudioCommandBatch::from_legacy_ports(input_ports),
            acknowledged_commands: EngineAudioCommandBatch::from_legacy_ports(output_ports),
        }
    }

    fn legacy_write_history(&self) -> [ApuWriteEnt; 16] {
        self.write_history.map(ApuWriteEnt::from_commands)
    }

    fn emit(&mut self, command: EngineAudioCommand) {
        self.pending_write.apply(command);
    }

    fn push(&mut self) {
        let pos = (self.write_position & 0xf) as usize;
        self.write_history[pos] = self.pending_write;
        self.write_position = self.write_position.wrapping_add(1);
        if self.write_count < 16 {
            self.write_count += 1;
        }
        self.total_writes = self.total_writes.wrapping_add(1);
    }

    fn pop(&mut self) {
        if self.write_count != 0 {
            let pos = self.write_position.wrapping_sub(self.write_count) & 0xf;
            self.input_commands = self.write_history[pos as usize];
            self.write_count -= 1;
        }
    }

    fn acknowledge_input(&mut self) {
        self.acknowledged_commands = self.input_commands;
    }

    fn acknowledge_legacy_ports(&mut self, ports: [u8; 4]) {
        self.acknowledged_commands = EngineAudioCommandBatch::from_legacy_ports(ports);
    }

    fn reset(&mut self) {
        self.write_position = 0;
        self.total_writes = 0;
        self.write_count = 0;
    }

    fn discard_unused_frames(&mut self) {
        if self.write_count != 0 {
            let pos = self.write_position.wrapping_sub(self.write_count) & 0xf;
            if self.input_commands == self.write_history[pos as usize] {
                if self.total_writes >= 16 {
                    self.total_writes = 14;
                    self.write_count -= 1;
                }
                return;
            }
        }
        self.total_writes = 0;
    }

    fn pending_commands(&self) -> EngineAudioCommandBatch {
        let pos = self.write_position.wrapping_sub(self.write_count) & 0xf;
        self.write_history[pos as usize]
    }

    fn route_state(&self) -> AudioQueueState {
        AudioQueueState {
            pos: self.write_position,
            count: self.write_count,
            total: self.total_writes,
            write: self.pending_write.legacy_ports(),
            pending: self.pending_commands().legacy_ports(),
            input: self.input_commands.legacy_ports(),
        }
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
struct ModernAudioRuntime {
    queue: ModernAudioCommandQueue,
    renderer: ModernAudioEngine,
    sequencer: ModernAudioSequencer,
    #[serde(default)]
    driver_clock: Option<crate::spc_driver_clock::AbsoluteDspEventClock>,
    #[serde(default)]
    sample_bank_id: u8,
    #[serde(default)]
    sample_bank_generation: u32,
}

/// Fields retained only for importing and exporting old C-style save blocks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct LegacyAudioCompatibilityState {
    saved_music_ports: [u8; 4],
    startup_sfx_timer_accum: u8,
}

impl LegacyAudioCompatibilityState {
    fn import_legacy_ram(&mut self, ram: &[u8]) {
        self.startup_sfx_timer_accum = ram.get(0x43).copied().unwrap_or(0);
        self.saved_music_ports = ram
            .get(0x410..0x414)
            .and_then(|ports| ports.try_into().ok())
            .unwrap_or([0; 4]);
    }

    fn export_legacy_ram(&self) -> [u8; 0x10000] {
        let mut ram = [0; 0x10000];
        ram[0x43] = self.startup_sfx_timer_accum;
        ram[0x410..0x414].copy_from_slice(&self.saved_music_ports);
        ram
    }
}

pub(super) struct AudioState {
    audio_has_rendered: bool,
    msu_player: MsuPlayer,
    modern: ModernAudioRuntime,
    legacy_compatibility: LegacyAudioCompatibilityState,
    volume_transition_step_float: [f32; 4],
    volume_transition_target_float: [f32; 4],
    config_audio_freq: u32,
    config_msuvolume: u8,
    config_resume_msu: bool,
    config_msu_path: Option<String>,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            audio_has_rendered: false,
            msu_player: MsuPlayer::default(),
            modern: ModernAudioRuntime::default(),
            legacy_compatibility: LegacyAudioCompatibilityState::default(),
            volume_transition_step_float: [0.0; 4],
            volume_transition_target_float: [0.0; 4],
            config_audio_freq: 0,
            config_msuvolume: 100,
            config_resume_msu: false,
            config_msu_path: None,
        }
    }
}

impl Clone for AudioState {
    fn clone(&self) -> Self {
        Self {
            audio_has_rendered: self.audio_has_rendered,
            msu_player: self.msu_player.clone(),
            modern: self.modern.clone(),
            legacy_compatibility: self.legacy_compatibility,
            volume_transition_step_float: self.volume_transition_step_float,
            volume_transition_target_float: self.volume_transition_target_float,
            config_audio_freq: self.config_audio_freq,
            config_msuvolume: self.config_msuvolume,
            config_resume_msu: self.config_resume_msu,
            config_msu_path: self.config_msu_path.clone(),
        }
    }
}

impl AudioState {
    fn legacy_modern_apu_ram(&self) -> [u8; 0x10000] {
        self.legacy_compatibility.export_legacy_ram()
    }
}

impl ZeldaState {
    /// Legacy APUI input adapter used by compatibility tests.
    #[cfg(test)]
    pub fn zelda_apu_write(&mut self, adr: u32, val: u8) {
        self.zelda_emit_audio_command(EngineAudioCommand::from_apui_write((adr as usize) & 3, val));
    }

    pub fn zelda_emit_audio_command(&mut self, command: EngineAudioCommand) {
        self.audio.modern.queue.emit(command);
    }

    pub fn zelda_debug_apu_write_ports(&self) -> [u8; 4] {
        self.audio.modern.queue.pending_write.legacy_ports()
    }

    pub fn zelda_engine_audio_commands(&self) -> EngineAudioCommandBatch {
        self.audio.modern.queue.input_commands
    }

    pub fn zelda_audio_command_acknowledged(&self, command: EngineAudioCommand) -> bool {
        let acknowledged = self.audio.modern.queue.acknowledged_commands;
        match command {
            EngineAudioCommand::ClearMusic
            | EngineAudioCommand::PlayMusic { .. }
            | EngineAudioCommand::StopMusic
            | EngineAudioCommand::MusicControl { .. } => {
                let mut expected = EngineAudioCommandBatch::default();
                expected.apply(command);
                acknowledged.music() == expected.music()
            }
            EngineAudioCommand::ClearSfx { bank } | EngineAudioCommand::PlaySfx { bank, .. } => {
                let mut expected = EngineAudioCommandBatch::default();
                expected.apply(command);
                acknowledged.sfx(bank) == expected.sfx(bank)
            }
        }
    }

    pub fn zelda_audio_route_state(&self) -> AudioRouteState {
        AudioRouteState {
            music: MusicControlState::from_game(self),
            queue: self.zelda_audio_queue_state(),
            spc: self.zelda_audio_spc_route_state(),
            sample_bank_id: self.audio.modern.sample_bank_id,
            sample_bank_generation: self.audio.modern.sample_bank_generation,
        }
    }

    fn zelda_modern_audio_route_state(&self) -> AudioRouteState {
        AudioRouteState {
            music: MusicControlState::from_game(self),
            queue: self.zelda_audio_queue_state(),
            spc: None,
            sample_bank_id: self.audio.modern.sample_bank_id,
            sample_bank_generation: self.audio.modern.sample_bank_generation,
        }
    }

    fn zelda_audio_queue_state(&self) -> AudioQueueState {
        self.audio.modern.queue.route_state()
    }

    fn zelda_audio_spc_route_state(&self) -> Option<SpcSequencerState> {
        None
    }

    pub fn zelda_audio_event_frame_from_dsp_writes(
        &self,
        writes: &[DspWriteEvent],
    ) -> AudioEventFrame {
        AudioEventFrame::from_route_and_dsp_writes(self.zelda_audio_route_state(), writes)
    }

    pub fn zelda_game_frame_output(&self) -> GameFrameOutput {
        let frame = &self.game_state.frame;
        GameFrameOutput {
            runtime: RuntimeOutputFacts {
                frame_counter: frame.frame_counter,
                main_module: frame.main_module,
                submodule: frame.submodule,
                subsubmodule: frame.subsubmodule,
                inidisp: self.ram[INIDISP_COPY],
            },
            render: RenderOutputFacts {
                mode: self.ppu.mode,
                forced_blank: self.ppu.forced_blank,
                brightness: self.ppu.brightness,
                screen_enabled: self.ppu.screen_enabled,
            },
            audio: AudioEventFrame::from_route_and_dsp_writes(self.zelda_audio_route_state(), &[]),
        }
    }

    pub fn zelda_modern_audio_last_stats(&self) -> ModernAudioFrameStats {
        self.audio.modern.renderer.last_stats()
    }

    pub fn zelda_modern_audio_sequence_last_stats(&self) -> ModernAudioSequenceStats {
        self.audio.modern.sequencer.last_stats()
    }

    pub fn zelda_audio_route_debug_json(&self) -> String {
        let queue = &self.audio.modern.queue;
        let write = queue.pending_write.legacy_ports();
        let pending = queue.pending_commands().legacy_ports();
        let input = queue.input_commands.legacy_ports();
        let mut out = format!(
            "\"queue\":{{\"pos\":{},\"count\":{},\"total\":{},\"write\":[{},{},{},{}],\"pending\":[{},{},{},{}],\"input\":[{},{},{},{}]",
            queue.write_position,
            queue.write_count,
            queue.total_writes,
            write[0],
            write[1],
            write[2],
            write[3],
            pending[0],
            pending[1],
            pending[2],
            pending[3],
            input[0],
            input[1],
            input[2],
            input[3],
        );
        out.push('}');
        out
    }

    /// Bincode-serialize the full live audio state (SPC player POD + DSP value +
    /// SPC RAM + APU queue + volume floats + config). Used to make checkpoint
    /// resume byte-identical to a from-scratch run: the C-style music saveload
    /// (`zelda_save_music_state_to_ram_locked`) only round-trips the sequencer
    /// *variables* packed through SPC RAM and resets `timer_cycles`/the APU queue,
    /// so the next frame's audio render diverges. This snapshot captures the exact
    /// runtime state instead.
    pub fn zelda_audio_snapshot_bytes(&self) -> Vec<u8> {
        let (payload, has_oracle_sidecar) =
            snapshot_state::encode_v7(&self.audio).expect("audio snapshot serialize failed");
        let flags = if has_oracle_sidecar {
            AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR
        } else {
            0
        };
        let mut bytes = Vec::with_capacity(AUDIO_SNAPSHOT_HEADER_BYTES + payload.len());
        bytes.extend_from_slice(&AUDIO_SNAPSHOT_MAGIC);
        bytes.extend_from_slice(&AUDIO_SNAPSHOT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&flags.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload);
        bytes
    }

    pub fn zelda_modern_audio_state(&self) -> (ModernAudioSequencer, ModernAudioEngine) {
        (
            self.audio.modern.sequencer.clone(),
            self.audio.modern.renderer.clone(),
        )
    }

    pub fn zelda_modern_audio_voice_debug_states(
        &self,
    ) -> [crate::modern_audio::ModernVoiceDebugState; 8] {
        self.audio.modern.renderer.voice_debug_states()
    }

    pub fn zelda_modern_audio_sfx_clock_checkpoint(&self) -> (u32, u8, u8) {
        self.audio.modern.sequencer.sfx_clock_checkpoint()
    }

    pub fn zelda_modern_audio_compat_ram(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.audio.legacy_compatibility.export_legacy_ram().to_vec())
    }

    /// Restore the full live audio state previously captured by
    /// `zelda_audio_snapshot_bytes`. Replaces `self.audio` wholesale (its `Drop`
    /// frees the prior SPC player); the deserialized `AudioState` re-creates the
    /// SPC player + DSP with all raw pointers correctly re-linked.
    pub fn zelda_audio_restore_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let (version, flags, payload) = if bytes.starts_with(&AUDIO_SNAPSHOT_MAGIC) {
            if bytes.len() < AUDIO_SNAPSHOT_HEADER_BYTES {
                return Err("audio snapshot header is truncated".to_string());
            }
            let version = u16::from_le_bytes([bytes[4], bytes[5]]);
            if !matches!(version, 1 | 2 | 3 | 4 | 5 | 6 | AUDIO_SNAPSHOT_VERSION) {
                return Err(format!("unsupported audio snapshot version {version}"));
            }
            let flags = u16::from_le_bytes([bytes[6], bytes[7]]);
            let supported_flags = if version >= 3 {
                AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR
            } else {
                0
            };
            if flags & !supported_flags != 0 {
                return Err(format!("unsupported audio snapshot flags {flags}"));
            }
            let payload_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
            let expected_len = AUDIO_SNAPSHOT_HEADER_BYTES
                .checked_add(payload_len)
                .ok_or_else(|| "audio snapshot length overflow".to_string())?;
            if bytes.len() != expected_len {
                return Err(format!(
                    "audio snapshot length mismatch: declared={payload_len} actual={}",
                    bytes.len().saturating_sub(AUDIO_SNAPSHOT_HEADER_BYTES)
                ));
            }
            (version, flags, &bytes[AUDIO_SNAPSHOT_HEADER_BYTES..])
        } else {
            (0, 0, bytes)
        };
        let restored = match version {
            AUDIO_SNAPSHOT_VERSION => {
                let (state, has_oracle_sidecar) = snapshot_state::decode_v7(payload)?;
                let flag_has_sidecar = flags & AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR != 0;
                if flag_has_sidecar != has_oracle_sidecar {
                    return Err("audio snapshot oracle sidecar flag mismatch".to_string());
                }
                state
            }
            6 => {
                let (state, has_oracle_sidecar) = snapshot_state::decode_v6(payload)?;
                let flag_has_sidecar = flags & AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR != 0;
                if flag_has_sidecar != has_oracle_sidecar {
                    return Err("audio snapshot oracle sidecar flag mismatch".to_string());
                }
                state
            }
            5 => {
                let (state, has_oracle_sidecar) = snapshot_state::decode_v5(payload)?;
                let flag_has_sidecar = flags & AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR != 0;
                if flag_has_sidecar != has_oracle_sidecar {
                    return Err("audio snapshot oracle sidecar flag mismatch".to_string());
                }
                state
            }
            4 => {
                let (state, has_oracle_sidecar) = snapshot_state::decode_v4(payload)?;
                let flag_has_sidecar = flags & AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR != 0;
                if flag_has_sidecar != has_oracle_sidecar {
                    return Err("audio snapshot oracle sidecar flag mismatch".to_string());
                }
                state
            }
            3 => {
                let (state, has_oracle_sidecar) = snapshot_state::decode_v3(payload)?;
                let flag_has_sidecar = flags & AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR != 0;
                if flag_has_sidecar != has_oracle_sidecar {
                    return Err("audio snapshot oracle sidecar flag mismatch".to_string());
                }
                state
            }
            2 => snapshot_state::decode_v2(payload)?,
            1 => {
                return Err("audio snapshot v1 predates the modern-only audio runtime".to_string());
            }
            0 => snapshot_state::decode_v2(payload)?,
            _ => unreachable!(),
        };
        self.audio = restored;
        Ok(())
    }

    pub fn zelda_push_apu_state(&mut self) {
        self.audio.modern.queue.push();
    }

    fn zelda_pop_apu_state(&mut self) {
        self.audio.modern.queue.pop();
    }

    pub fn zelda_discard_unused_audio_frames(&mut self) {
        self.audio.modern.queue.discard_unused_frames();
    }

    fn zelda_reset_apu_queue(&mut self) {
        self.audio.modern.queue.reset();
    }

    pub fn zelda_read_apui00(&self) -> u8 {
        self.game_state.system_signals.apui00()
    }

    /// Legacy APUI acknowledgement projection used by compatibility tests.
    #[cfg(test)]
    pub fn zelda_apu_read(&self, adr: u32) -> u8 {
        self.audio.modern.queue.acknowledged_commands.legacy_ports()[(adr as usize) & 3]
    }

    pub fn zelda_render_audio(
        &mut self,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
    ) -> AudioEventFrame {
        if samples <= 0 || channels <= 0 {
            return AudioEventFrame::from_route_and_dsp_writes(
                self.zelda_modern_audio_route_state(),
                &[],
            );
        }
        self.audio.audio_has_rendered = true;
        self.zelda_pop_apu_state();
        if self.audio.modern.driver_clock.is_none() {
            self.audio.modern.queue.acknowledge_input();
        }
        // The sequencer and SPC driver clock advance on the native 32 kHz DSP
        // timeline (534 samples per 60 Hz frame). Hosts request one frame of
        // audio at THEIR output rate (e.g. 735 @ 44.1 kHz, 801 @ 48 kHz);
        // advancing the drivers by that raw count would speed music and
        // acknowledgment timing up by rate/32000. Normalize exactly like the
        // engine's mixer does: near-native windows advance verbatim, anything
        // else advances one native frame, and the engine resamples its native
        // mix to the requested output size.
        let native_samples: u32 = {
            let requested = samples.max(0) as usize;
            const NATIVE: usize = crate::game_output::AUDIO_INTERNAL_SAMPLES_PER_FRAME;
            if (400..=NATIVE).contains(&requested) {
                requested as u32
            } else {
                NATIVE as u32
            }
        };
        let route = self.zelda_modern_audio_route_state();
        let frame = if let Some(clock) = self.audio.modern.driver_clock.as_mut() {
            let window = clock.advance(self.audio.modern.queue.input_commands, native_samples);
            let acknowledgements = clock.host_acknowledgements();
            self.audio
                .modern
                .queue
                .acknowledge_legacy_ports(acknowledgements);
            let mut frame = AudioEventFrame::from_route_and_dsp_writes(route, &window.writes);
            frame.sequenced = true;
            frame
        } else {
            self.audio
                .modern
                .sequencer
                .sequence_engine_commands_for_samples(
                    route,
                    self.audio.modern.queue.input_commands,
                    native_samples,
                )
        };
        self.audio
            .modern
            .renderer
            .render_frame(&frame, audio_buffer, samples, channels);
        if self.audio.msu_player.has_file && channels == 2 {
            self.msu_player_mix(audio_buffer, samples);
        }
        frame
    }

    pub fn zelda_set_rom_startup_audio_phase(&mut self, enabled: bool) {
        let phase = if enabled { 72 } else { 0 };
        self.zelda_set_spc_startup_phase(phase, 0);
        if enabled {
            // The SPC startup loop consumes its bootstrap timer work before
            // the first rendered frame. This is the equivalent phase at the
            // modern scheduler's first audio-frame boundary.
            self.audio.modern.sequencer.set_sfx_clock_phase(0, 200);
        }
    }

    pub fn zelda_set_spc_startup_phase(&mut self, sfx_timer_accum: u8, timer_cycles: u8) {
        self.audio.legacy_compatibility.startup_sfx_timer_accum = sfx_timer_accum;
        self.audio
            .modern
            .sequencer
            .set_sfx_clock_phase(timer_cycles, sfx_timer_accum);
    }

    pub fn zelda_is_music_playing(&self) -> bool {
        if self.audio.msu_player.state != MSU_STATE_IDLE {
            self.audio.msu_player.state != MSU_STATE_FINISHED_PLAYING
        } else {
            !matches!(
                self.audio.modern.queue.acknowledged_commands.music(),
                crate::game_output::AudioMusicCommand::Clear
            )
        }
    }

    pub fn zelda_restore_music_after_load_locked(&mut self, is_reset: bool) {
        self.audio.modern.renderer = ModernAudioEngine::default();
        self.audio.modern.sequencer = ModernAudioSequencer::default();
        self.audio.modern.renderer.sample_ram_changed();
        let commands = EngineAudioCommandBatch::from_legacy_ports(
            self.audio.legacy_compatibility.saved_music_ports,
        );
        self.audio.modern.queue.input_commands = commands;
        self.audio.modern.queue.pending_write = commands;
        if is_reset {
            self.audio.modern.queue.acknowledged_commands = EngineAudioCommandBatch::default();
        }
        let restored_sfx_timer_accum = self.audio.legacy_compatibility.startup_sfx_timer_accum;
        if restored_sfx_timer_accum != 0 {
            self.audio
                .modern
                .sequencer
                .set_sfx_clock_phase(0, restored_sfx_timer_accum);
        }
        if self.audio.msu_player.enabled != 0 {
            self.audio.msu_player.volume = 0.0;
            let resume = self.msu_resume_info(MsuResumeSlot::Primary);
            let system_signals = &self.game_state.system_signals;
            let current_music_control = system_signals.current_music_control();
            let last_music_control = system_signals.last_music_control();
            let track = if current_music_control == 0xf1 {
                resume.orig_track
            } else {
                current_music_control
            };
            self.msu_player_open(track as i32, true);
            if (0xf1..=0xf3).contains(&last_music_control) {
                let i = (last_music_control - 0xf1) as usize;
                let target = MSU_VOLUME_TRANSITION_TARGETS[i];
                let msu_volume = self.game_state.system_signals.msu_volume();
                if target != msu_volume {
                    let f = self.audio.volume_transition_target_float[3] * (1.0 / 255.0);
                    self.audio.msu_player.volume = msu_volume as f32 * f;
                    self.audio.msu_player.volume_target = target as f32 * f;
                    self.audio.msu_player.volume_step = self.audio.volume_transition_step_float[i];
                }
            }
            if self.audio.msu_player.state != 0 {
                self.zelda_emit_audio_command(EngineAudioCommand::StopMusic);
            }
        }
        self.zelda_reset_apu_queue();
    }

    pub fn zelda_save_music_state_to_ram_locked(&mut self) {
        self.audio.legacy_compatibility.saved_music_ports =
            self.audio.modern.queue.pending_write.legacy_ports();
        let msu_volume = (self.audio.msu_player.volume * 255.0) as u8;
        self.set_msu_volume(msu_volume);
        self.save_msu_resume_info(MsuResumeSlot::Primary, self.audio.msu_player.resume_info);
    }

    pub fn zelda_enable_msu(&mut self, enable: u8) {
        self.audio.msu_player.volume = 1.0;
        self.audio.msu_player.enabled = enable;
        let volscale = self.audio.config_msuvolume as f32 * (1.0 / 255.0 / 100.0);
        let freq = if self.audio.config_audio_freq == 0 {
            1.0
        } else {
            self.audio.config_audio_freq as f32
        };
        let stepscale = self.audio.config_msuvolume as f32 * (60.0 / 256.0 / 100.0) / freq;
        for i in 0..MSU_VOLUME_TRANSITION_STEPS.len() {
            self.audio.volume_transition_step_float[i] =
                MSU_VOLUME_TRANSITION_STEPS[i] as f32 * stepscale;
            self.audio.volume_transition_target_float[i] =
                MSU_VOLUME_TRANSITION_TARGETS[i] as f32 * volscale;
        }
    }

    pub fn zelda_configure_audio(
        &mut self,
        audio_freq: u32,
        msuvolume: u8,
        resume_msu: bool,
        msu_path: Option<String>,
    ) {
        self.audio.config_audio_freq = audio_freq;
        self.audio.config_msuvolume = msuvolume;
        self.audio.config_resume_msu = resume_msu;
        self.audio.config_msu_path = msu_path;
    }

    pub fn load_song_bank(&mut self, p: &[u8]) {
        if let Some(clock) = self.audio.modern.driver_clock.as_mut() {
            clock
                .upload_song_bank(p)
                .expect("compiled song bank must contain a valid SPC upload stream");
        }
        let bank_id = (0..3).find(|&bank_id| self.asset_raw(bank_id) == Some(p));
        if let Some(bank_id) = bank_id {
            self.audio.modern.sample_bank_id = bank_id as u8;
            self.audio.modern.renderer.select_sample_bank(bank_id as u8);
        }
        self.audio.modern.sample_bank_generation =
            self.audio.modern.sample_bank_generation.wrapping_add(1);
        self.audio.modern.renderer.sample_ram_changed();
    }

    pub(crate) fn select_modern_sample_bank(&mut self, bank_id: u8) {
        self.audio.modern.renderer.select_sample_bank(bank_id);
    }

    pub(crate) fn initialize_spc_driver_clock(
        &mut self,
        driver: &[u8],
        intro_bank: &[u8],
    ) -> Result<(), String> {
        self.audio.modern.driver_clock = Some(crate::spc_driver_clock::AbsoluteDspEventClock::new(
            driver, intro_bank,
        )?);
        Ok(())
    }

    pub(crate) fn clear_spc_driver_clock(&mut self) {
        self.audio.modern.driver_clock = None;
    }

    pub(crate) fn configure_spc_driver_clock_for_rom_bootstrap(&mut self) {
        if let Some(clock) = self.audio.modern.driver_clock.as_mut() {
            clock.configure_rom_bootstrap();
        }
    }

    pub fn zelda_spc_driver_clock_debug_summary(&self) -> Option<String> {
        self.audio
            .modern
            .driver_clock
            .as_ref()
            .map(|clock| clock.debug_state_summary())
    }

    pub fn zelda_begin_spc_driver_instruction_trace(&mut self) {
        if let Some(clock) = self.audio.modern.driver_clock.as_mut() {
            clock.begin_debug_instruction_trace();
        }
    }

    pub fn zelda_take_spc_driver_instruction_trace(
        &mut self,
    ) -> Option<(u64, Vec<snes::apu::SpcInstructionTrace>)> {
        self.audio
            .modern
            .driver_clock
            .as_mut()
            .map(|clock| clock.take_debug_instruction_trace())
    }

    pub fn zelda_audio_debug_summary(&self) -> String {
        "modern audio runtime active".to_string()
    }

    pub fn zelda_debug_full_apu_from_spc(&self) -> snes::apu::ApuState {
        let mut apu = snes::apu::ApuState::new();
        apu.reset();
        let spc_ram = self.audio.legacy_compatibility.export_legacy_ram();
        apu.ram.copy_from_slice(&spc_ram);
        apu.rom_readable = false;
        apu.spc.pc = 0x800;
        apu
    }

    pub(super) fn save_audio_apu_ram_c_saveload(&self) -> [u8; 0x10000] {
        self.audio.legacy_compatibility.export_legacy_ram()
    }

    pub(super) fn load_audio_apu_ram_c_saveload(&mut self, data: &[u8]) {
        self.audio.legacy_compatibility.import_legacy_ram(data);
        if let Some(bank_id) = crate::modern_sample_bank::identify_spc_ram(data) {
            self.audio.modern.renderer.select_sample_bank(bank_id);
        }
        self.audio.modern.renderer.sample_ram_changed();
    }

    pub(super) fn save_audio_dsp_c_saveload(&self) -> Vec<u8> {
        vec![0; snes::apu::DSP_SAVELOAD_SIZE]
    }

    pub(super) fn load_audio_dsp_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() == snes::apu::DSP_SAVELOAD_SIZE {
            Ok(())
        } else {
            Err(format!(
                "invalid inert DSP compatibility block: expected {}, got {}",
                snes::apu::DSP_SAVELOAD_SIZE,
                data.len()
            ))
        }
    }

    fn msu_resume_info(&self, slot: MsuResumeSlot) -> MsuResumeInfoState {
        self.game_state.system_signals.msu_resume_info(slot)
    }

    fn save_msu_resume_info(&mut self, slot: MsuResumeSlot, info: MsuResumeInfoState) {
        self.set_msu_resume_info(slot, info);
    }
}

#[cfg(test)]
fn upload_song_bank_to_ram(ram: &mut [u8; 0x10000], data: &[u8]) {
    let mut cursor = 0usize;
    while cursor + 2 <= data.len() {
        let length = usize::from(u16::from_le_bytes([data[cursor], data[cursor + 1]]));
        if length == 0 {
            break;
        }
        if cursor + 4 > data.len() || cursor + 4 + length > data.len() {
            break;
        }
        let mut target = u16::from_le_bytes([data[cursor + 2], data[cursor + 3]]);
        cursor += 4;
        for &byte in &data[cursor..cursor + length] {
            ram[usize::from(target)] = byte;
            target = target.wrapping_add(1);
        }
        cursor += length;
    }
}

#[cfg(test)]
mod song_bank_upload_tests {
    use super::upload_song_bank_to_ram;

    #[test]
    fn materializes_upload_blocks_at_their_spc_addresses() {
        let mut ram = [0u8; 0x10000];
        let upload = [
            3, 0, 0x00, 0x3c, 1, 2, 3, // directory block
            2, 0, 0xfe, 0xff, 4, 5, // wrapping block
            0, 0,
        ];

        upload_song_bank_to_ram(&mut ram, &upload);

        assert_eq!(&ram[0x3c00..0x3c03], &[1, 2, 3]);
        assert_eq!(ram[0xfffe], 4);
        assert_eq!(ram[0xffff], 5);
        assert_eq!(&ram[..8], &[0; 8]);
    }
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

#[cfg(test)]
#[path = "audio_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "audio/conformance_tests.rs"]
mod conformance_tests;
