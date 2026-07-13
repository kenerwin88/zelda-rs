// Methods ported from zelda3/src/audio.c and included inside ZeldaState.

use super::*;
use crate::config::{config_value_path, MSU_FEATURE_MSU_DELUXE, MSU_FEATURE_OPUZ};
use crate::game_output::{
    AudioBackendMode, AudioEventFrame, AudioQueueState, AudioRouteState, DspWriteEvent,
    GameFrameOutput, MusicControlState, RenderOutputFacts, RuntimeOutputFacts, SpcSequencerState,
};
use crate::game_state::constants::INIDISP_COPY;
use crate::modern_audio::{ModernAudioEngine, ModernAudioFrameStats};
use crate::modern_audio_sequence::{ModernAudioSequenceStats, ModernAudioSequencer};
use opus::{Channels, Decoder as OpusDecoder};
use std::borrow::Cow;
use std::fs;

#[cfg(not(feature = "audio-oracle"))]
#[path = "audio/modern_only_runtime.rs"]
mod modern_only_runtime;
#[path = "audio/msu_runtime.rs"]
mod msu_runtime;
#[cfg(feature = "audio-oracle")]
#[path = "audio/oracle_runtime.rs"]
mod oracle_runtime;
#[path = "audio/snapshot_state.rs"]
mod snapshot_state;
const MSU_STATE_IDLE: u8 = 0;
const MSU_STATE_FINISHED_PLAYING: u8 = 1;
const MSU_STATE_RESUMING: u8 = 2;
const MSU_STATE_PLAYING: u8 = 3;
const AUDIO_SNAPSHOT_MAGIC: [u8; 4] = *b"Z3AU";
const AUDIO_SNAPSHOT_VERSION: u16 = 5;
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

/// Compact modern-owned state for the SNES/APU command bridge.
///
/// The modern renderer consumes semantic commands and canonical sample banks;
/// it does not emulate the SPC700 address space. The two legacy RAM locations
/// retained here are compatibility fields used by old C-style save blocks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ModernApuState {
    write_history: [ApuWriteEnt; 16],
    pending_write: ApuWriteEnt,
    write_position: u8,
    write_count: u8,
    total_writes: u8,
    input_ports: [u8; 4],
    output_ports: [u8; 4],
    saved_music_ports: [u8; 4],
    startup_sfx_timer_accum: u8,
}

impl ModernApuState {
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
    #[cfg(feature = "audio-oracle")]
    spc_player: *mut crate::spc_player::SpcPlayer,
    backend: AudioBackendMode,
    audio_has_rendered: bool,
    msu_player: MsuPlayer,
    modern_audio: ModernAudioEngine,
    modern_sequence: ModernAudioSequencer,
    modern_apu: ModernApuState,
    #[cfg(feature = "audio-oracle")]
    modern_sample_ram: [u8; 0x10000],
    volume_transition_step_float: [f32; 4],
    volume_transition_target_float: [f32; 4],
    config_audio_freq: u32,
    config_msuvolume: u8,
    config_resume_msu: bool,
    config_msu_path: Option<String>,
}

impl Default for AudioState {
    fn default() -> Self {
        #[cfg(feature = "audio-oracle")]
        let spc_player = crate::spc_player::spc_player_create();
        #[cfg(feature = "audio-oracle")]
        crate::spc_player::spc_player_initialize(spc_player);
        Self {
            #[cfg(feature = "audio-oracle")]
            spc_player,
            backend: AudioBackendMode::default(),
            audio_has_rendered: false,
            msu_player: MsuPlayer::default(),
            modern_audio: ModernAudioEngine::default(),
            modern_sequence: ModernAudioSequencer::default(),
            modern_apu: ModernApuState::default(),
            #[cfg(feature = "audio-oracle")]
            modern_sample_ram: [0; 0x10000],
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
            #[cfg(feature = "audio-oracle")]
            spc_player: crate::spc_player::spc_player_clone(self.spc_player),
            backend: self.backend,
            audio_has_rendered: self.audio_has_rendered,
            msu_player: self.msu_player.clone(),
            modern_audio: self.modern_audio.clone(),
            modern_sequence: self.modern_sequence.clone(),
            modern_apu: self.modern_apu,
            #[cfg(feature = "audio-oracle")]
            modern_sample_ram: self.modern_sample_ram,
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
        #[cfg(feature = "audio-oracle")]
        {
            return self.modern_sample_ram;
        }
        #[cfg(not(feature = "audio-oracle"))]
        self.modern_apu.export_legacy_ram()
    }
}

impl Drop for AudioState {
    fn drop(&mut self) {
        #[cfg(feature = "audio-oracle")]
        {
            crate::spc_player::spc_player_destroy(self.spc_player);
            self.spc_player = std::ptr::null_mut();
        }
    }
}

impl ZeldaState {
    pub fn zelda_apu_write(&mut self, adr: u32, val: u8) {
        self.audio.modern_apu.pending_write.ports[(adr as usize) & 3] = val;
    }

    pub fn zelda_debug_apu_write_ports(&self) -> [u8; 4] {
        self.audio.modern_apu.pending_write.ports
    }

    pub fn zelda_audio_route_state(&self) -> AudioRouteState {
        AudioRouteState {
            music: MusicControlState::from_game(self),
            queue: self.zelda_audio_queue_state(),
            spc: self.zelda_audio_spc_route_state(),
        }
    }

    fn zelda_modern_audio_route_state(&self) -> AudioRouteState {
        AudioRouteState {
            music: MusicControlState::from_game(self),
            queue: self.zelda_audio_queue_state(),
            spc: None,
        }
    }

    fn zelda_audio_queue_state(&self) -> AudioQueueState {
        let apu = &self.audio.modern_apu;
        let pending_pos = apu.write_position.wrapping_sub(apu.write_count) & 0xf;
        AudioQueueState {
            pos: apu.write_position,
            count: apu.write_count,
            total: apu.total_writes,
            write: apu.pending_write.ports,
            pending: apu.write_history[pending_pos as usize].ports,
            input: apu.input_ports,
        }
    }

    #[cfg(not(feature = "audio-oracle"))]
    fn zelda_audio_spc_route_state(&self) -> Option<SpcSequencerState> {
        None
    }

    #[cfg(feature = "audio-oracle")]
    fn zelda_audio_spc_route_state(&self) -> Option<SpcSequencerState> {
        unsafe { self.audio.spc_player.as_ref() }.map(|player| SpcSequencerState {
            spc_in: player.input_ports,
            spc_out: player.port_to_snes,
            timer_cycles: player.timer_cycles,
            sfx_timer_accum: player.sfx_timer_accum,
            main_tempo_accum: player.main_tempo_accum,
            block_count: player.block_count,
            key_on: player.key_ON,
            key_off: player.key_OFF,
            current_bit: player.current_bit,
            port1_active: player.port1_active,
            port2_active: player.port2_active,
            port3_active: player.port3_active,
            is_chan_on: player.is_chan_on,
            echo_enable_mask: player.semantic_echo_enable_mask,
            echo_enable_frame_start: player.semantic_echo_enable_frame_start,
            echo_enable_values: player.semantic_echo_enable_values,
            echo_enable_offsets: player.semantic_echo_enable_offsets,
            echo_enable_count: player.semantic_echo_enable_count,
            echo_volume_registers: player.semantic_echo_volume_registers,
            echo_volume_values: player.semantic_echo_volume_values,
            echo_volume_offsets: player.semantic_echo_volume_offsets,
            echo_volume_count: player.semantic_echo_volume_count,
            global_registers: player.semantic_global_registers,
            global_values: player.semantic_global_values,
            global_offsets: player.semantic_global_offsets,
            global_count: player.semantic_global_count,
            voice_sources: player.semantic_voice_sources,
            voice_adsr1: player.semantic_voice_adsr1,
            voice_adsr2: player.semantic_voice_adsr2,
            voice_gain: player.semantic_voice_gain,
            voice_volume_left: player.semantic_voice_volume_left,
            voice_volume_right: player.semantic_voice_volume_right,
            vol_dirty: player.vol_dirty,
            ch7_sfx: player.channel[7].sfx_which_sound,
            ch7_sfx_ptr: player.channel[7].sfx_sound_ptr,
            ch7_pattern: player.channel[7].pattern_order_ptr_for_chan,
            ch7_ticks: player.channel[7].note_ticks_left,
            ch7_keyoff_ticks: player.channel[7].note_keyoff_ticks_left,
            sfx_kof_masks: player.semantic_sfx_kof_masks,
            sfx_kof_offsets: player.semantic_sfx_kof_offsets,
            sfx_kof_count: player.semantic_sfx_kof_count,
            raw_kof_masks: player.semantic_raw_kof_masks,
            raw_kof_offsets: player.semantic_raw_kof_offsets,
            raw_kof_count: player.semantic_raw_kof_count,
            sfx_kon_masks: player.semantic_sfx_kon_masks,
            sfx_kon_owned_masks: player.semantic_sfx_kon_owned_masks,
            sfx_kon_offsets: player.semantic_sfx_kon_offsets,
            sfx_kon_rate_counters: player.semantic_sfx_kon_rate_counters,
            sfx_kon_sources: player.semantic_sfx_kon_sources,
            sfx_kon_adsr1: player.semantic_sfx_kon_adsr1,
            sfx_kon_adsr2: player.semantic_sfx_kon_adsr2,
            sfx_kon_gain: player.semantic_sfx_kon_gain,
            sfx_kon_volume_left: player.semantic_sfx_kon_volume_left,
            sfx_kon_volume_right: player.semantic_sfx_kon_volume_right,
            sfx_kon_count: player.semantic_sfx_kon_count,
            sfx_echo_masks: player.semantic_sfx_echo_masks,
            sfx_echo_enabled: player.semantic_sfx_echo_enabled,
            sfx_echo_offsets: player.semantic_sfx_echo_offsets,
            sfx_echo_count: player.semantic_sfx_echo_count,
            sfx_pitch_masks: player.semantic_sfx_pitch_masks,
            sfx_pitch_words: player.semantic_sfx_pitch_words,
            sfx_pitch_offsets: player.semantic_sfx_pitch_offsets,
            sfx_pitch_count: player.semantic_sfx_pitch_count,
            raw_pitch_masks: player.semantic_raw_pitch_masks,
            raw_pitch_words: player.semantic_raw_pitch_words,
            raw_pitch_offsets: player.semantic_raw_pitch_offsets,
            raw_pitch_masks_hi: player.semantic_raw_pitch_masks_hi,
            raw_pitch_words_hi: player.semantic_raw_pitch_words_hi,
            raw_pitch_offsets_hi: player.semantic_raw_pitch_offsets_hi,
            raw_pitch_masks_hi2: player.semantic_raw_pitch_masks_hi2,
            raw_pitch_words_hi2: player.semantic_raw_pitch_words_hi2,
            raw_pitch_offsets_hi2: player.semantic_raw_pitch_offsets_hi2,
            raw_pitch_masks_hi3: player.semantic_raw_pitch_masks_hi3,
            raw_pitch_words_hi3: player.semantic_raw_pitch_words_hi3,
            raw_pitch_offsets_hi3: player.semantic_raw_pitch_offsets_hi3,
            raw_pitch_count: player.semantic_raw_pitch_count,
            sfx_volume_masks: player.semantic_sfx_volume_masks,
            sfx_volume_left: player.semantic_sfx_volume_left,
            sfx_volume_right: player.semantic_sfx_volume_right,
            sfx_volume_offsets: player.semantic_sfx_volume_offsets,
            sfx_volume_count: player.semantic_sfx_volume_count,
            raw_volume_masks: player.semantic_raw_volume_masks,
            raw_volume_left: player.semantic_raw_volume_left,
            raw_volume_right: player.semantic_raw_volume_right,
            raw_volume_offsets: player.semantic_raw_volume_offsets,
            raw_volume_count: player.semantic_raw_volume_count,
            raw_envelope_masks: player.semantic_raw_envelope_masks,
            raw_envelope_registers: player.semantic_raw_envelope_registers,
            raw_envelope_values: player.semantic_raw_envelope_values,
            raw_envelope_offsets: player.semantic_raw_envelope_offsets,
            raw_envelope_count: player.semantic_raw_envelope_count,
            sfx_setup_masks: player.semantic_sfx_setup_masks,
            sfx_setup_sources: player.semantic_sfx_setup_sources,
            sfx_setup_adsr1: player.semantic_sfx_setup_adsr1,
            sfx_setup_adsr2: player.semantic_sfx_setup_adsr2,
            sfx_setup_gain: player.semantic_sfx_setup_gain,
            sfx_setup_offsets: player.semantic_sfx_setup_offsets,
            sfx_setup_count: player.semantic_sfx_setup_count,
        })
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
        self.audio.modern_audio.last_stats()
    }

    pub fn zelda_modern_audio_sequence_last_stats(&self) -> ModernAudioSequenceStats {
        self.audio.modern_sequence.last_stats()
    }

    pub fn zelda_audio_route_debug_json(&self) -> String {
        let apu = &self.audio.modern_apu;
        let pending_pos = apu.write_position.wrapping_sub(apu.write_count) & 0xf;
        let pending = apu.write_history[pending_pos as usize].ports;
        let mut out = format!(
            "\"queue\":{{\"pos\":{},\"count\":{},\"total\":{},\"write\":[{},{},{},{}],\"pending\":[{},{},{},{}],\"input\":[{},{},{},{}]",
            apu.write_position,
            apu.write_count,
            apu.total_writes,
            apu.pending_write.ports[0],
            apu.pending_write.ports[1],
            apu.pending_write.ports[2],
            apu.pending_write.ports[3],
            pending[0],
            pending[1],
            pending[2],
            pending[3],
            apu.input_ports[0],
            apu.input_ports[1],
            apu.input_ports[2],
            apu.input_ports[3],
        );
        #[cfg(feature = "audio-oracle")]
        if let Some(player) = unsafe { self.audio.spc_player.as_ref() } {
            out.push_str(&format!(
                ",\"spc_in\":[{},{},{},{}],\"spc_out\":[{},{},{},{}],\"timer\":{},\"sfx_timer_accum\":{},\"main_tempo_accum\":{},\"block_count\":{},\"key_on\":{},\"key_off\":{},\"current_bit\":{},\"port1_active\":{},\"port2_active\":{},\"port3_active\":{}",
                player.input_ports[0],
                player.input_ports[1],
                player.input_ports[2],
                player.input_ports[3],
                player.port_to_snes[0],
                player.port_to_snes[1],
                player.port_to_snes[2],
                player.port_to_snes[3],
                player.timer_cycles,
                player.sfx_timer_accum,
                player.main_tempo_accum,
                player.block_count,
                player.key_ON,
                player.key_OFF,
                player.current_bit,
                player.port1_active,
                player.port2_active,
                player.port3_active,
            ));
            out.push_str(&format!(
                ",\"is_chan_on\":{},\"vol_dirty\":{},\"sfx_sound_ptr_cur\":{},\"ch7_sfx\":{},\"ch7_sfx_ptr\":{},\"ch7_pattern\":{},\"ch7_ticks\":{},\"ch7_keyoff_ticks\":{},\"sfx_kof_count\":{},\"sfx_kof_masks\":{:?},\"sfx_kof_offsets\":{:?},\"sfx_echo_count\":{},\"sfx_echo_masks\":{:?},\"sfx_echo_enabled\":{:?},\"sfx_echo_offsets\":{:?}",
                player.is_chan_on,
                player.vol_dirty,
                player.sfx_sound_ptr_cur,
                player.channel[7].sfx_which_sound,
                player.channel[7].sfx_sound_ptr,
                player.channel[7].pattern_order_ptr_for_chan,
                player.channel[7].note_ticks_left,
                player.channel[7].note_keyoff_ticks_left,
                player.semantic_sfx_kof_count,
                player.semantic_sfx_kof_masks,
                player.semantic_sfx_kof_offsets,
                player.semantic_sfx_echo_count,
                player.semantic_sfx_echo_masks,
                player.semantic_sfx_echo_enabled,
                player.semantic_sfx_echo_offsets,
            ));
            out.push_str(",\"sfx_channels\":[");
            for (index, channel) in player.channel.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"voice\":{},\"sound\":{},\"sound_ptr\":{},\"pan\":{},\"countdown\":{},\"note_length_left\":{},\"active\":{}}}",
                    index,
                    channel.sfx_which_sound,
                    channel.sfx_sound_ptr,
                    channel.sfx_pan,
                    channel.sfx_arr_countdown,
                    channel.sfx_note_length_left,
                    u8::from(player.is_chan_on & (1u8 << index) != 0),
                ));
            }
            out.push(']');
        }
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
            snapshot_state::encode_v5(&self.audio).expect("audio snapshot serialize failed");
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
        #[cfg(feature = "audio-oracle")]
        if std::env::var("ZELDA3_DBG_AUDIO_FP").is_ok() {
            let tc = unsafe { self.audio.spc_player.as_ref() }
                .map(|p| p.timer_cycles)
                .unwrap_or(255);
            let so = unsafe { self.audio.spc_player.as_ref() }
                .and_then(|p| unsafe { p.dsp.as_ref() })
                .map(|d| d.sampleOffset)
                .unwrap_or(-1);
            eprintln!(
                "[AUDIO_FP] snapshot: bytes={} timer_cycles={tc} dsp.sampleOffset={so} apu_total_write={} input_ports={:?}",
                bytes.len(),
                self.audio.modern_apu.total_writes,
                self.audio.modern_apu.input_ports
            );
        }
        bytes
    }

    pub fn zelda_modern_audio_state(&self) -> (ModernAudioSequencer, ModernAudioEngine) {
        (
            self.audio.modern_sequence.clone(),
            self.audio.modern_audio.clone(),
        )
    }

    pub fn zelda_modern_audio_compat_ram(&self) -> Cow<'_, [u8]> {
        #[cfg(feature = "audio-oracle")]
        {
            return Cow::Borrowed(&self.audio.modern_sample_ram);
        }
        #[cfg(not(feature = "audio-oracle"))]
        Cow::Owned(self.audio.modern_apu.export_legacy_ram().to_vec())
    }

    pub fn zelda_audio_live_spc_ram(&self) -> [u8; 0x10000] {
        #[cfg(feature = "audio-oracle")]
        {
            crate::spc_player::spc_player_save_ram(self.audio.spc_player)
        }
        #[cfg(not(feature = "audio-oracle"))]
        {
            panic!("live SPC RAM requires the audio-oracle feature")
        }
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
            if !matches!(version, 1 | 2 | 3 | 4 | AUDIO_SNAPSHOT_VERSION) {
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
                #[cfg(feature = "audio-oracle")]
                {
                    snapshot_state::decode_v1(payload)?
                }
                #[cfg(not(feature = "audio-oracle"))]
                {
                    return Err(
                        "audio snapshot v1 requires an audio-oracle build for migration"
                            .to_string(),
                    );
                }
            }
            0 => {
                #[cfg(feature = "audio-oracle")]
                {
                    snapshot_state::decode_headerless(payload)?
                }
                #[cfg(not(feature = "audio-oracle"))]
                {
                    snapshot_state::decode_v2(payload)?
                }
            }
            _ => unreachable!(),
        };
        #[cfg(feature = "audio-oracle")]
        if std::env::var("ZELDA3_DBG_AUDIO_FP").is_ok() {
            let tc = unsafe { restored.spc_player.as_ref() }
                .map(|p| p.timer_cycles)
                .unwrap_or(255);
            let so = unsafe { restored.spc_player.as_ref() }
                .and_then(|p| unsafe { p.dsp.as_ref() })
                .map(|d| d.sampleOffset)
                .unwrap_or(-1);
            eprintln!(
                "[AUDIO_FP] restore: bytes={} timer_cycles={tc} dsp.sampleOffset={so} apu_total_write={} input_ports={:?}",
                bytes.len(),
                restored.modern_apu.total_writes,
                restored.modern_apu.input_ports
            );
        }
        self.audio = restored;
        Ok(())
    }

    pub fn zelda_push_apu_state(&mut self) {
        let apu = &mut self.audio.modern_apu;
        let pos = (apu.write_position & 0xf) as usize;
        apu.write_history[pos] = apu.pending_write;
        apu.write_position = apu.write_position.wrapping_add(1);
        if apu.write_count < 16 {
            apu.write_count += 1;
        }
        apu.total_writes = apu.total_writes.wrapping_add(1);
    }

    fn zelda_pop_apu_state(&mut self) {
        let apu = &mut self.audio.modern_apu;
        if apu.write_count != 0 {
            let pos = apu.write_position.wrapping_sub(apu.write_count) & 0xf;
            apu.input_ports = apu.write_history[pos as usize].ports;
            apu.write_count -= 1;
        }
    }

    pub fn zelda_discard_unused_audio_frames(&mut self) {
        let apu = &mut self.audio.modern_apu;
        if apu.write_count != 0 {
            let pos = apu.write_position.wrapping_sub(apu.write_count) & 0xf;
            if apu.input_ports == apu.write_history[pos as usize].ports {
                if apu.total_writes >= 16 {
                    apu.total_writes = 14;
                    apu.write_count -= 1;
                }
                return;
            }
        }
        apu.total_writes = 0;
    }

    fn zelda_reset_apu_queue(&mut self) {
        self.audio.modern_apu.write_position = 0;
        self.audio.modern_apu.total_writes = 0;
        self.audio.modern_apu.write_count = 0;
    }

    pub fn zelda_read_apui00(&self) -> u8 {
        self.game_state.system_signals.apui00()
    }

    pub fn zelda_apu_read(&self, adr: u32) -> u8 {
        if self.audio.backend == AudioBackendMode::Modern {
            return self.audio.modern_apu.output_ports[(adr as usize) & 3];
        }
        #[cfg(feature = "audio-oracle")]
        if let Some(player) = unsafe { self.audio.spc_player.as_ref() } {
            return player.port_to_snes[(adr as usize) & 3];
        }
        self.audio.modern_apu.output_ports[(adr as usize) & 3]
    }

    pub fn zelda_render_audio(&mut self, audio_buffer: &mut [i16], samples: i32, channels: i32) {
        self.zelda_render_audio_with_backend(self.audio.backend, audio_buffer, samples, channels);
    }

    pub fn zelda_audio_backend(&self) -> AudioBackendMode {
        self.audio.backend
    }

    pub const fn zelda_audio_oracle_available() -> bool {
        cfg!(feature = "audio-oracle")
    }

    pub fn zelda_set_audio_backend(
        &mut self,
        backend: AudioBackendMode,
    ) -> Result<(), &'static str> {
        #[cfg(not(feature = "audio-oracle"))]
        if backend != AudioBackendMode::Modern {
            return Err("DSP audio backends require the audio-oracle feature");
        }
        if self.audio.audio_has_rendered && backend != self.audio.backend {
            return Err("audio backend selection is locked after rendering begins");
        }
        self.audio.backend = backend;
        Ok(())
    }

    pub fn zelda_render_audio_with_backend(
        &mut self,
        backend: AudioBackendMode,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
    ) -> AudioEventFrame {
        if samples > 0 && channels > 0 {
            self.audio.audio_has_rendered = true;
        }
        match backend {
            #[cfg(feature = "audio-oracle")]
            AudioBackendMode::DspParity => {
                let writes =
                    self.zelda_render_audio_trace_dsp_events(audio_buffer, samples, channels);
                self.zelda_audio_event_frame_from_dsp_writes(&writes)
            }
            #[cfg(feature = "audio-oracle")]
            AudioBackendMode::TraceOnly => {
                let count = (samples.max(0) as usize).saturating_mul(channels.max(0) as usize);
                let mut scratch = vec![0i16; count];
                let writes =
                    self.zelda_render_audio_trace_dsp_events(&mut scratch, samples, channels);
                for value in audio_buffer.iter_mut().take(count) {
                    *value = 0;
                }
                self.zelda_audio_event_frame_from_dsp_writes(&writes)
            }
            AudioBackendMode::Modern => {
                if samples <= 0 || channels <= 0 {
                    return AudioEventFrame::from_route_and_dsp_writes(
                        self.zelda_modern_audio_route_state(),
                        &[],
                    );
                }
                self.zelda_pop_apu_state();
                self.audio.modern_apu.output_ports = self.audio.modern_apu.input_ports;
                let route = self.zelda_modern_audio_route_state();
                let frame = self.audio.modern_sequence.sequence_route(route);
                self.audio
                    .modern_audio
                    .render_frame(&frame, audio_buffer, samples, channels);
                if self.audio.msu_player.has_file && channels == 2 {
                    self.msu_player_mix(audio_buffer, samples);
                }
                frame
            }
            #[cfg(not(feature = "audio-oracle"))]
            AudioBackendMode::DspParity | AudioBackendMode::TraceOnly => {
                panic!("DSP audio rendering requires the audio-oracle feature")
            }
        }
    }

    pub fn zelda_set_rom_startup_audio_phase(&mut self, enabled: bool) {
        let phase = if enabled { 72 } else { 0 };
        self.zelda_set_spc_startup_phase(phase, 0);
    }

    pub fn zelda_set_spc_startup_phase(&mut self, sfx_timer_accum: u8, timer_cycles: u8) {
        #[cfg(feature = "audio-oracle")]
        if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
            player.sfx_timer_accum = sfx_timer_accum;
            player.timer_cycles = timer_cycles.min(64);
            player.ram[0x0043] = sfx_timer_accum;
        }
        #[cfg(not(feature = "audio-oracle"))]
        let _ = timer_cycles;
        self.audio.modern_apu.startup_sfx_timer_accum = sfx_timer_accum;
        #[cfg(feature = "audio-oracle")]
        {
            self.audio.modern_sample_ram[0x0043] = sfx_timer_accum;
        }
    }

    pub fn zelda_is_music_playing(&self) -> bool {
        if self.audio.msu_player.state != MSU_STATE_IDLE {
            self.audio.msu_player.state != MSU_STATE_FINISHED_PLAYING
        } else {
            self.zelda_apu_read(0x2140) != 0
        }
    }

    pub fn zelda_restore_music_after_load_locked(&mut self, is_reset: bool) {
        self.audio.modern_audio = ModernAudioEngine::default();
        self.audio.modern_sequence = ModernAudioSequencer::default();
        self.audio.modern_audio.sample_ram_changed();
        #[cfg(feature = "audio-oracle")]
        crate::spc_player::spc_player_copy_variables_from_ram(self.audio.spc_player);
        #[cfg(feature = "audio-oracle")]
        if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
            player.timer_cycles = 0;
            player
                .input_ports
                .copy_from_slice(&player.ram[0x410..0x414]);
            self.audio.modern_apu.input_ports = player.input_ports;
            self.audio.modern_apu.pending_write.ports = player.input_ports;
        } else {
            self.audio.modern_apu.input_ports = self.audio.modern_apu.saved_music_ports;
            self.audio.modern_apu.pending_write.ports = self.audio.modern_apu.saved_music_ports;
        }
        #[cfg(not(feature = "audio-oracle"))]
        {
            self.audio.modern_apu.input_ports = self.audio.modern_apu.saved_music_ports;
            self.audio.modern_apu.pending_write.ports = self.audio.modern_apu.saved_music_ports;
        }
        if is_reset {
            self.audio.modern_apu.output_ports = [0; 4];
            #[cfg(feature = "audio-oracle")]
            crate::spc_player::spc_player_initialize(self.audio.spc_player);
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
                self.zelda_apu_write(0x2140, 0xf0);
            }
        }
        self.zelda_reset_apu_queue();
    }

    pub fn zelda_save_music_state_to_ram_locked(&mut self) {
        #[cfg(feature = "audio-oracle")]
        crate::spc_player::spc_player_copy_variables_to_ram(self.audio.spc_player);
        #[cfg(feature = "audio-oracle")]
        if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
            player.ram[0x410..0x414].copy_from_slice(&self.audio.modern_apu.pending_write.ports);
        }
        self.audio.modern_apu.saved_music_ports = self.audio.modern_apu.pending_write.ports;
        #[cfg(feature = "audio-oracle")]
        self.audio.modern_sample_ram[0x410..0x414]
            .copy_from_slice(&self.audio.modern_apu.saved_music_ports);
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
        let bank_id = (0..3).find(|&bank_id| self.asset_raw(bank_id) == Some(p));
        if let Some(bank_id) = bank_id {
            self.audio.modern_audio.select_sample_bank(bank_id as u8);
        }
        self.audio.modern_audio.sample_ram_changed();
        #[cfg(feature = "audio-oracle")]
        {
            upload_song_bank_to_ram(&mut self.audio.modern_sample_ram, p);
            if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
                crate::spc_player::spc_player_upload(player, p.as_ptr());
            }
        }
    }

    pub(crate) fn select_modern_sample_bank(&mut self, bank_id: u8) {
        self.audio.modern_audio.select_sample_bank(bank_id);
    }

    pub fn zelda_audio_debug_summary(&self) -> String {
        #[cfg(feature = "audio-oracle")]
        {
            crate::spc_player::spc_player_debug_summary(self.audio.spc_player)
        }
        #[cfg(not(feature = "audio-oracle"))]
        {
            "audio-oracle feature disabled; modern backend active".to_string()
        }
    }

    pub fn zelda_debug_full_apu_from_spc(&self) -> snes::apu::ApuState {
        let mut apu = snes::apu::ApuState::new();
        apu.reset();
        #[cfg(feature = "audio-oracle")]
        let spc_ram = crate::spc_player::spc_player_save_ram(self.audio.spc_player);
        #[cfg(not(feature = "audio-oracle"))]
        let spc_ram = self.audio.modern_apu.export_legacy_ram();
        apu.ram.copy_from_slice(&spc_ram);
        apu.rom_readable = false;
        apu.spc.pc = 0x800;
        apu
    }

    pub(super) fn save_audio_apu_ram_c_saveload(&self) -> [u8; 0x10000] {
        #[cfg(feature = "audio-oracle")]
        if self.audio.backend == AudioBackendMode::Modern || self.audio.spc_player.is_null() {
            return self.audio.modern_sample_ram;
        }
        #[cfg(feature = "audio-oracle")]
        return crate::spc_player::spc_player_save_ram(self.audio.spc_player);
        #[cfg(not(feature = "audio-oracle"))]
        self.audio.modern_apu.export_legacy_ram()
    }

    pub(super) fn load_audio_apu_ram_c_saveload(&mut self, data: &[u8]) {
        self.audio.modern_apu.import_legacy_ram(data);
        if let Some(bank_id) = crate::modern_sample_bank::identify_spc_ram(data) {
            self.audio.modern_audio.select_sample_bank(bank_id);
        }
        self.audio.modern_audio.sample_ram_changed();
        #[cfg(feature = "audio-oracle")]
        {
            let len = data.len().min(self.audio.modern_sample_ram.len());
            self.audio.modern_sample_ram[..len].copy_from_slice(&data[..len]);
            if len < self.audio.modern_sample_ram.len() {
                self.audio.modern_sample_ram[len..].fill(0);
            }
            crate::spc_player::spc_player_load_ram(
                self.audio.spc_player,
                &self.audio.modern_sample_ram,
            );
        }
    }

    #[cfg(feature = "audio-oracle")]
    pub(super) fn save_audio_dsp_c_saveload(&self) -> Vec<u8> {
        crate::spc_player::spc_player_save_dsp_c_saveload(self.audio.spc_player)
    }

    #[cfg(not(feature = "audio-oracle"))]
    pub(super) fn save_audio_dsp_c_saveload(&self) -> Vec<u8> {
        vec![0; snes::apu::DSP_SAVELOAD_SIZE]
    }

    #[cfg(feature = "audio-oracle")]
    pub(super) fn load_audio_dsp_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        crate::spc_player::spc_player_load_dsp_c_saveload(self.audio.spc_player, data)
    }

    #[cfg(not(feature = "audio-oracle"))]
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

#[cfg(any(feature = "audio-oracle", test))]
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
