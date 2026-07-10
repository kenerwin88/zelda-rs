// Methods ported from zelda3/src/audio.c and included inside ZeldaState.

use super::*;
use crate::config::{config_value_path, MSU_FEATURE_MSU_DELUXE, MSU_FEATURE_OPUZ};
use crate::game_output::{
    AudioBackendMode, AudioEventFrame, AudioQueueState, AudioRouteState, DspWriteEvent,
    GameFrameOutput, MusicControlState, RenderOutputFacts, RuntimeOutputFacts, SpcSequencerState,
};
use crate::modern_audio::{ModernAudioEngine, ModernAudioFrameStats};
use crate::modern_audio_sequence::{ModernAudioSequenceStats, ModernAudioSequencer};
use opus::{Channels, Decoder as OpusDecoder};
use std::fs;

const MSU_STATE_IDLE: u8 = 0;
const MSU_STATE_FINISHED_PLAYING: u8 = 1;
const MSU_STATE_RESUMING: u8 = 2;
const MSU_STATE_PLAYING: u8 = 3;

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

pub(super) struct AudioState {
    spc_player: *mut crate::spc_player::SpcPlayer,
    msu_player: MsuPlayer,
    modern_audio: ModernAudioEngine,
    modern_sequence: ModernAudioSequencer,
    apu_write_ents: [ApuWriteEnt; 16],
    apu_write: ApuWriteEnt,
    apu_write_ent_pos: u8,
    apu_write_count: u8,
    apu_total_write: u8,
    input_ports: [u8; 4],
    port_to_snes: [u8; 4],
    spc_ram: [u8; 0x10000],
    volume_transition_step_float: [f32; 4],
    volume_transition_target_float: [f32; 4],
    config_audio_freq: u32,
    config_msuvolume: u8,
    config_resume_msu: bool,
    config_msu_path: Option<String>,
}

impl Default for AudioState {
    fn default() -> Self {
        let spc_player = crate::spc_player::spc_player_create();
        crate::spc_player::spc_player_initialize(spc_player);
        Self {
            spc_player,
            msu_player: MsuPlayer::default(),
            modern_audio: ModernAudioEngine::default(),
            modern_sequence: ModernAudioSequencer::default(),
            apu_write_ents: [ApuWriteEnt::default(); 16],
            apu_write: ApuWriteEnt::default(),
            apu_write_ent_pos: 0,
            apu_write_count: 0,
            apu_total_write: 0,
            input_ports: [0; 4],
            port_to_snes: [0; 4],
            spc_ram: [0; 0x10000],
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
            spc_player: crate::spc_player::spc_player_clone(self.spc_player),
            msu_player: self.msu_player.clone(),
            modern_audio: self.modern_audio.clone(),
            modern_sequence: self.modern_sequence.clone(),
            apu_write_ents: self.apu_write_ents,
            apu_write: self.apu_write,
            apu_write_ent_pos: self.apu_write_ent_pos,
            apu_write_count: self.apu_write_count,
            apu_total_write: self.apu_total_write,
            input_ports: self.input_ports,
            port_to_snes: self.port_to_snes,
            spc_ram: self.spc_ram,
            volume_transition_step_float: self.volume_transition_step_float,
            volume_transition_target_float: self.volume_transition_target_float,
            config_audio_freq: self.config_audio_freq,
            config_msuvolume: self.config_msuvolume,
            config_resume_msu: self.config_resume_msu,
            config_msu_path: self.config_msu_path.clone(),
        }
    }
}

impl Drop for AudioState {
    fn drop(&mut self) {
        crate::spc_player::spc_player_destroy(self.spc_player);
        self.spc_player = std::ptr::null_mut();
    }
}

/// Serializable mirror of `AudioState`. The `spc_player` raw pointer is replaced
/// by a deep, pointer-free snapshot (see `spc_player::SpcPlayerSnapshot`). The
/// `msu_player` is intentionally NOT round-tripped: MSU (external music
/// streaming) is disabled in headless replay, it owns non-serde state
/// (`OpusDecoder`), and on restore it is reconstructed as `MsuPlayer::default()`.
/// Every other field is byte-faithful.
#[derive(serde::Serialize, serde::Deserialize)]
struct AudioStateSnapshot {
    spc_player: crate::spc_player::SpcPlayerSnapshot,
    apu_write_ents: [ApuWriteEnt; 16],
    apu_write: ApuWriteEnt,
    apu_write_ent_pos: u8,
    apu_write_count: u8,
    apu_total_write: u8,
    input_ports: [u8; 4],
    port_to_snes: [u8; 4],
    #[serde(with = "serde_big_array::BigArray")]
    spc_ram: [u8; 0x10000],
    volume_transition_step_float: [f32; 4],
    volume_transition_target_float: [f32; 4],
    config_audio_freq: u32,
    config_msuvolume: u8,
    config_resume_msu: bool,
    config_msu_path: Option<String>,
}

impl serde::Serialize for AudioState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let snapshot = AudioStateSnapshot {
            spc_player: crate::spc_player::spc_player_snapshot(self.spc_player),
            apu_write_ents: self.apu_write_ents,
            apu_write: self.apu_write,
            apu_write_ent_pos: self.apu_write_ent_pos,
            apu_write_count: self.apu_write_count,
            apu_total_write: self.apu_total_write,
            input_ports: self.input_ports,
            port_to_snes: self.port_to_snes,
            spc_ram: self.spc_ram,
            volume_transition_step_float: self.volume_transition_step_float,
            volume_transition_target_float: self.volume_transition_target_float,
            config_audio_freq: self.config_audio_freq,
            config_msuvolume: self.config_msuvolume,
            config_resume_msu: self.config_resume_msu,
            config_msu_path: self.config_msu_path.clone(),
        };
        snapshot.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AudioState {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let snapshot = AudioStateSnapshot::deserialize(deserializer)?;
        let spc_player = crate::spc_player::spc_player_from_snapshot(snapshot.spc_player);
        Ok(Self {
            spc_player,
            msu_player: MsuPlayer::default(),
            modern_audio: ModernAudioEngine::default(),
            modern_sequence: ModernAudioSequencer::default(),
            apu_write_ents: snapshot.apu_write_ents,
            apu_write: snapshot.apu_write,
            apu_write_ent_pos: snapshot.apu_write_ent_pos,
            apu_write_count: snapshot.apu_write_count,
            apu_total_write: snapshot.apu_total_write,
            input_ports: snapshot.input_ports,
            port_to_snes: snapshot.port_to_snes,
            spc_ram: snapshot.spc_ram,
            volume_transition_step_float: snapshot.volume_transition_step_float,
            volume_transition_target_float: snapshot.volume_transition_target_float,
            config_audio_freq: snapshot.config_audio_freq,
            config_msuvolume: snapshot.config_msuvolume,
            config_resume_msu: snapshot.config_resume_msu,
            config_msu_path: snapshot.config_msu_path,
        })
    }
}

impl ZeldaState {
    fn remap_msu_deluxe_track(&self, mp: &MsuPlayer, track: u8) -> u8 {
        if mp.enabled & MSU_FEATURE_MSU_DELUXE == 0
            || track as usize >= MSU_DELUXE_TRACK_ROUTE.len()
        {
            return track;
        }
        match MSU_DELUXE_TRACK_ROUTE[track as usize] {
            1 => {
                let area = self.game_state.world.region.overworld_area_index() as usize & 0xff;
                if area < MSU_DELUXE_OVERWORLD_TRACKS.len() {
                    MSU_DELUXE_OVERWORLD_TRACKS[area]
                } else {
                    track
                }
            }
            2 => {
                let entrance = self.game_state.world.region.which_entrance() as usize;
                if entrance >= MSU_DELUXE_ENTRANCE_TRACKS.len()
                    || MSU_DELUXE_ENTRANCE_TRACKS[entrance] == 242
                {
                    track
                } else {
                    MSU_DELUXE_ENTRANCE_TRACKS[entrance]
                }
            }
            _ => track,
        }
    }

    pub fn zelda_is_playing_music_track(&self, track: u8) -> bool {
        let mp = &self.audio.msu_player;
        if mp.state != MSU_STATE_IDLE && mp.enabled & MSU_FEATURE_MSU_DELUXE != 0 {
            self.remap_msu_deluxe_track(mp, track) == mp.resume_info.actual_track
        } else {
            track == self.game_state.system_signals.current_music_control()
        }
    }

    pub fn zelda_is_playing_music_track_with_bug(&self, track: u8) -> bool {
        let mp = &self.audio.msu_player;
        if mp.state != MSU_STATE_IDLE && mp.enabled & MSU_FEATURE_MSU_DELUXE != 0 {
            self.remap_msu_deluxe_track(mp, track) == mp.resume_info.actual_track
        } else if self
            .game_state
            .enhanced_features
            .has(FEATURES0_MISC_BUG_FIXES_AUDIO)
        {
            track == self.game_state.system_signals.current_music_control()
        } else {
            track == self.game_state.system_signals.last_music_control()
        }
    }

    pub fn zelda_get_entrance_music_track(&self, i: i32) -> u8 {
        let mut rv = self
            .assets
            .as_ref()
            .and_then(|assets| assets.asset(27))
            .and_then(|asset| asset.get(i as usize))
            .copied()
            .unwrap_or(0);
        let mp = &self.audio.msu_player;
        if mp.state != MSU_STATE_IDLE && mp.enabled & MSU_FEATURE_MSU_DELUXE != 0 {
            let entrance = self.game_state.world.region.which_entrance() as usize;
            if rv == 242
                && entrance < MSU_DELUXE_ENTRANCE_TRACKS.len()
                && MSU_DELUXE_ENTRANCE_TRACKS[entrance] != 242
            {
                rv = 16;
            }
        }
        rv
    }

    pub fn zelda_play_msu_audio_track(&mut self, music_ctrl: u8) {
        if self.audio.msu_player.enabled == 0 {
            self.audio.msu_player.resume_info.tag = 0;
            self.zelda_apu_write(0x2140, music_ctrl);
            return;
        }
        if music_ctrl & 0xf0 != 0xf0 {
            self.msu_player_open(music_ctrl as i32, false);
        } else if (0xf1..=0xf3).contains(&music_ctrl) {
            let i = (music_ctrl - 0xf1) as usize;
            self.audio.msu_player.volume_target = self.audio.volume_transition_target_float[i];
            self.audio.msu_player.volume_step = self.audio.volume_transition_step_float[i];
        }
        if self.audio.msu_player.state == 0 {
            self.zelda_apu_write(0x2140, music_ctrl);
        } else {
            self.zelda_apu_write(0x2140, 0xf0);
        }
    }

    fn msu_player_close_file(mp: &mut MsuPlayer) {
        mp.has_file = false;
        mp.has_opus = false;
        mp.opus_decoder = None;
        mp.pcm_data.clear();
        mp.opuz_data.clear();
        if mp.state != MSU_STATE_FINISHED_PLAYING {
            mp.state = MSU_STATE_IDLE;
        }
        mp.resume_info = MsuResumeInfoState::default();
    }

    fn msu_player_open(&mut self, orig_track: i32, resume_from_snapshot: bool) {
        let actual_track = {
            let mp = &self.audio.msu_player;
            self.remap_msu_deluxe_track(mp, orig_track as u8)
        };

        let resume = if !resume_from_snapshot {
            let mut resume = MsuResumeInfoState::default();
            if self.game_state.frame.main_module == 9
                && actual_track == self.msu_resume_info(MsuResumeSlot::Alternate).actual_track
                && self.audio.config_resume_msu
            {
                resume = self.msu_resume_info(MsuResumeSlot::Alternate);
            }
            if self.audio.msu_player.state >= MSU_STATE_RESUMING {
                self.save_msu_resume_info(
                    MsuResumeSlot::Alternate,
                    self.audio.msu_player.resume_info,
                );
            }
            resume
        } else {
            self.msu_resume_info(MsuResumeSlot::Primary)
        };

        let volume_target = self.audio.volume_transition_target_float[3];
        let volume_step = self.audio.volume_transition_step_float[3];
        let config_msu_path = self.audio.config_msu_path.clone();
        let mp = &mut self.audio.msu_player;
        mp.volume_target = volume_target;
        mp.volume_step = volume_step;
        mp.state = MSU_STATE_IDLE;
        Self::msu_player_close_file(mp);
        if actual_track == 0 {
            return;
        }

        let ext = if mp.enabled & MSU_FEATURE_OPUZ != 0 {
            "opuz"
        } else {
            "pcm"
        };
        let prefix = config_msu_path.as_deref().unwrap_or("");
        let fname = format!("{prefix}{actual_track}.{ext}");
        let Ok(bytes) = fs::read(config_value_path(&fname)) else {
            Self::msu_player_close_file(mp);
            return;
        };
        if bytes.len() < 8 {
            Self::msu_player_close_file(mp);
            return;
        }

        let Some(file_tag) = read_le_u32(&bytes, 0) else {
            Self::msu_player_close_file(mp);
            return;
        };
        let Some(repeat_position) = read_le_u32(&bytes, 4) else {
            Self::msu_player_close_file(mp);
            return;
        };
        mp.repeat_position = repeat_position;
        mp.state = if resume.actual_track == actual_track && resume.tag == file_tag {
            MSU_STATE_RESUMING
        } else {
            MSU_STATE_PLAYING
        };
        if mp.state == MSU_STATE_RESUMING {
            mp.resume_info = resume;
        } else {
            mp.resume_info.orig_track = orig_track as u8;
            mp.resume_info.actual_track = actual_track;
            mp.resume_info.tag = file_tag;
            mp.resume_info.range_cur = 8;
        }
        mp.cur_file_offs = mp.resume_info.offset;
        mp.samples_until_repeat = mp.resume_info.samples_until_repeat;
        mp.range_cur = mp.resume_info.range_cur;
        mp.range_repeat = mp.resume_info.range_repeat;
        mp.buffer_size = 0;
        mp.buffer_pos = 0;
        mp.preskip = 0;

        if file_tag == OPUZ_TAG {
            let Ok(decoder) = OpusDecoder::new(48_000, Channels::Stereo) else {
                Self::msu_player_close_file(mp);
                return;
            };
            mp.opuz_data = bytes;
            mp.opus_decoder = Some(decoder);
            mp.has_file = true;
            mp.has_opus = true;
        } else if file_tag == MSU1_TAG {
            let sample_frames = ((bytes.len() - 8) / 4) as u32;
            mp.total_samples_in_file = sample_frames;
            mp.samples_until_repeat = sample_frames.wrapping_sub(mp.cur_file_offs);
            mp.pcm_data.clear();
            mp.pcm_data.reserve(sample_frames as usize * 2);
            for chunk in bytes[8..8 + sample_frames as usize * 4].chunks_exact(2) {
                mp.pcm_data.push(i16::from_le_bytes([chunk[0], chunk[1]]));
            }
            mp.has_file = true;
            mp.has_opus = false;
        } else {
            Self::msu_player_close_file(mp);
        }
    }

    fn msu_player_prepare_opuz_packet(mp: &mut MsuPlayer) -> OpuzPacketStatus {
        if mp.opus_decoder.is_none() {
            let Ok(decoder) = OpusDecoder::new(48_000, Channels::Stereo) else {
                return OpuzPacketStatus::ReadError;
            };
            mp.opus_decoder = Some(decoder);
        }

        loop {
            if mp.samples_until_repeat == 0 {
                if mp.range_cur == 0 {
                    return OpuzPacketStatus::FinishedPlaying;
                }
                if let Some(decoder) = &mut mp.opus_decoder {
                    if decoder.reset_state().is_err() {
                        return OpuzPacketStatus::ReadError;
                    }
                }
                let range_offset = mp.range_cur as usize;
                let Some(range_header) = mp.opuz_data.get(range_offset..range_offset + 10) else {
                    return OpuzPacketStatus::ReadError;
                };
                let Some(file_offs) = read_le_u32(range_header, 0) else {
                    return OpuzPacketStatus::ReadError;
                };
                if file_offs & 0xf000_0000 != 0 {
                    return OpuzPacketStatus::ReadError;
                }
                let Some(samples_until_repeat) = read_le_u32(range_header, 4) else {
                    return OpuzPacketStatus::ReadError;
                };
                let preskip = u16::from_le_bytes([range_header[8], range_header[9]]);
                mp.samples_until_repeat = samples_until_repeat;
                mp.preskip = (preskip & 0x3fff) as u32;
                if preskip & 0x4000 != 0 {
                    mp.range_repeat = mp.range_cur;
                }
                mp.range_cur = if preskip & 0x8000 != 0 {
                    mp.range_repeat
                } else {
                    mp.range_cur.wrapping_add(10)
                };
                mp.cur_file_offs = file_offs;
                mp.resume_info.range_repeat = mp.range_repeat;
                mp.resume_info.range_cur = mp.range_cur;
            }
            if mp.samples_until_repeat == 0 {
                return OpuzPacketStatus::ReadError;
            }

            let packet_offset = mp.cur_file_offs as usize;
            let Some(packet_header) = mp.opuz_data.get(packet_offset..packet_offset + 2) else {
                return OpuzPacketStatus::ReadError;
            };
            let packet_header = u16::from_le_bytes([packet_header[0], packet_header[1]]);
            let size = (packet_header & 0x7fff) as usize;
            if size > 1275 {
                return OpuzPacketStatus::ReadError;
            }
            let n = usize::from((packet_header >> 15) != 0);
            let packet_end = packet_offset.saturating_add(2).saturating_add(size);
            if packet_end > mp.opuz_data.len() {
                return OpuzPacketStatus::ReadError;
            }

            let mut initial_file_data = [0; 8];
            let initial_len = (2 + size).min(initial_file_data.len());
            initial_file_data[..initial_len]
                .copy_from_slice(&mp.opuz_data[packet_offset..packet_offset + initial_len]);
            let initial_file_data = u64::from_le_bytes(initial_file_data);
            if mp.state == MSU_STATE_RESUMING {
                mp.state = MSU_STATE_PLAYING;
                if mp.resume_info.initial_packet_bytes != initial_file_data {
                    return OpuzPacketStatus::ReadError;
                }
            }
            mp.resume_info.initial_packet_bytes = initial_file_data;
            mp.resume_info.samples_until_repeat = mp.samples_until_repeat.wrapping_add(mp.preskip);
            mp.resume_info.offset = mp.cur_file_offs;
            mp.cur_file_offs = mp.cur_file_offs.wrapping_add(2 + size as u32);

            let mut packet = Vec::with_capacity(size + n);
            if n != 0 {
                packet.push(0xfc);
            }
            packet.extend_from_slice(&mp.opuz_data[packet_offset + 2..packet_end]);

            let Some(decoder) = &mut mp.opus_decoder else {
                return OpuzPacketStatus::ReadError;
            };
            let Ok(r) = decoder.decode(&packet, &mut mp.buffer, false) else {
                return OpuzPacketStatus::ReadError;
            };
            if r == 0 {
                return OpuzPacketStatus::ReadError;
            }
            let r = r as u32;
            if r > mp.preskip {
                return OpuzPacketStatus::Decoded(r);
            }
            mp.preskip = mp.preskip.wrapping_sub(r);
        }
    }

    fn mix_to_buffer_with_volume(dst: &mut [i16], src: &[i16], n: usize, volume: f32) {
        for i in 0..n {
            let left = i * 2;
            let right = left + 1;
            if right >= dst.len() || right >= src.len() {
                break;
            }
            if volume == 1.0 {
                dst[left] = dst[left].wrapping_add(src[left]);
                dst[right] = dst[right].wrapping_add(src[right]);
            } else {
                let vol = (65536.0 * volume) as i32;
                dst[left] = dst[left].wrapping_add(((src[left] as i32 * vol) >> 16) as i16);
                dst[right] = dst[right].wrapping_add(((src[right] as i32 * vol) >> 16) as i16);
            }
        }
    }

    fn mix_to_buffer_with_volume_ramp(
        dst: &mut [i16],
        src: &[i16],
        n: usize,
        volume: f32,
        volume_step: f32,
        _ideal_target: f32,
    ) {
        let mut vol = (volume * 281474976710656.0) as i64;
        let step = (volume_step * 281474976710656.0) as i64;
        for i in 0..n {
            let left = i * 2;
            let right = left + 1;
            if right >= dst.len() || right >= src.len() {
                break;
            }
            let v = (vol >> 32) as i32;
            dst[left] = dst[left].wrapping_add(((src[left] as i32 * v) >> 16) as i16);
            dst[right] = dst[right].wrapping_add(((src[right] as i32 * v) >> 16) as i16);
            vol = vol.wrapping_add(step);
        }
    }

    fn mix_to_buffer(mp: &mut MsuPlayer, dst: &mut [i16], src: &[i16], mut n: u32) {
        if mp.volume != mp.volume_target {
            let step = if mp.volume < mp.volume_target {
                mp.volume_step
            } else {
                -mp.volume_step
            };
            let mut new_vol = mp.volume + step * n as f32;
            let mut curn = n;
            if if step >= 0.0 {
                new_vol >= mp.volume_target
            } else {
                new_vol < mp.volume_target
            } {
                let maxn = ((mp.volume_target - mp.volume) / step) as u32;
                curn = maxn.min(curn);
                new_vol = mp.volume_target;
            }
            let vol = mp.volume;
            mp.volume = new_vol;
            Self::mix_to_buffer_with_volume_ramp(dst, src, curn as usize, vol, step, new_vol);
            let skip = curn as usize * 2;
            if skip >= dst.len() || skip >= src.len() {
                return;
            }
            n -= curn;
            Self::mix_to_buffer_with_volume(&mut dst[skip..], &src[skip..], n as usize, mp.volume);
        } else {
            Self::mix_to_buffer_with_volume(dst, src, n as usize, mp.volume);
        }
    }

    pub fn msu_player_mix(&mut self, audio_buffer: &mut [i16], mut audio_samples: i32) {
        let mut audio_offset = 0usize;
        while audio_samples != 0 {
            let remaining = self
                .audio
                .msu_player
                .buffer_size
                .saturating_sub(self.audio.msu_player.buffer_pos);
            if remaining == 0 {
                if self.audio.msu_player.has_opus {
                    let orig_track = self.audio.msu_player.resume_info.orig_track;
                    match Self::msu_player_prepare_opuz_packet(&mut self.audio.msu_player) {
                        OpuzPacketStatus::FinishedPlaying => {
                            self.audio.msu_player.state = MSU_STATE_FINISHED_PLAYING;
                            Self::msu_player_close_file(&mut self.audio.msu_player);
                        }
                        OpuzPacketStatus::Decoded(r) => {
                            let n = r
                                .saturating_sub(self.audio.msu_player.preskip)
                                .min(self.audio.msu_player.samples_until_repeat);
                            self.audio.msu_player.samples_until_repeat =
                                self.audio.msu_player.samples_until_repeat.wrapping_sub(n);
                            self.audio.msu_player.buffer_pos = self.audio.msu_player.preskip;
                            self.audio.msu_player.buffer_size =
                                self.audio.msu_player.buffer_pos + n;
                            self.audio.msu_player.preskip = 0;
                        }
                        OpuzPacketStatus::ReadError => {
                            Self::msu_player_close_file(&mut self.audio.msu_player);
                            self.zelda_apu_write(0x2140, orig_track);
                            return;
                        }
                    }
                } else if self.audio.msu_player.has_file {
                    if self.audio.msu_player.samples_until_repeat == 0 {
                        let actual_track = self.audio.msu_player.resume_info.actual_track as usize;
                        if actual_track < MSU_TRACK_REPEATS.len()
                            && MSU_TRACK_REPEATS[actual_track] == 0
                        {
                            self.audio.msu_player.state = MSU_STATE_FINISHED_PLAYING;
                            Self::msu_player_close_file(&mut self.audio.msu_player);
                            return;
                        }
                        self.audio.msu_player.samples_until_repeat = self
                            .audio
                            .msu_player
                            .total_samples_in_file
                            .wrapping_sub(self.audio.msu_player.repeat_position);
                        if self.audio.msu_player.samples_until_repeat == 0 {
                            let orig_track = self.audio.msu_player.resume_info.orig_track;
                            Self::msu_player_close_file(&mut self.audio.msu_player);
                            self.zelda_apu_write(0x2140, orig_track);
                            return;
                        }
                        self.audio.msu_player.cur_file_offs = self.audio.msu_player.repeat_position;
                    }

                    let r = 960u32.min(self.audio.msu_player.samples_until_repeat);
                    let data_start = self.audio.msu_player.cur_file_offs as usize * 2;
                    let data_end = data_start.saturating_add(r as usize * 2);
                    if data_end > self.audio.msu_player.pcm_data.len() {
                        let orig_track = self.audio.msu_player.resume_info.orig_track;
                        Self::msu_player_close_file(&mut self.audio.msu_player);
                        self.zelda_apu_write(0x2140, orig_track);
                        return;
                    }
                    let source = self.audio.msu_player.pcm_data[data_start..data_end].to_vec();
                    self.audio.msu_player.buffer[..source.len()].copy_from_slice(&source);
                    self.audio.msu_player.resume_info.offset = self.audio.msu_player.cur_file_offs;
                    self.audio.msu_player.cur_file_offs =
                        self.audio.msu_player.cur_file_offs.wrapping_add(r);
                    let n = r
                        .saturating_sub(self.audio.msu_player.preskip)
                        .min(self.audio.msu_player.samples_until_repeat);
                    self.audio.msu_player.samples_until_repeat =
                        self.audio.msu_player.samples_until_repeat.wrapping_sub(n);
                    self.audio.msu_player.buffer_pos = self.audio.msu_player.preskip;
                    self.audio.msu_player.buffer_size = self.audio.msu_player.buffer_pos + n;
                    self.audio.msu_player.preskip = 0;
                } else if self.audio.msu_player.state == MSU_STATE_FINISHED_PLAYING {
                    Self::msu_player_close_file(&mut self.audio.msu_player);
                    return;
                } else {
                    break;
                }
            }
            let remaining = self
                .audio
                .msu_player
                .buffer_size
                .saturating_sub(self.audio.msu_player.buffer_pos);
            let nr = (audio_samples as u32).min(remaining);
            let buffer_pos = self.audio.msu_player.buffer_pos as usize * 2;
            let end = buffer_pos + nr as usize * 2;
            let source = self.audio.msu_player.buffer[buffer_pos..end].to_vec();
            self.audio.msu_player.buffer_pos += nr;
            let dst_end = audio_offset.saturating_add(nr as usize * 2);
            if dst_end > audio_buffer.len() {
                return;
            }
            Self::mix_to_buffer(
                &mut self.audio.msu_player,
                &mut audio_buffer[audio_offset..],
                &source,
                nr,
            );
            audio_samples -= nr as i32;
            audio_offset = dst_end;
        }
    }

    pub fn zelda_apu_write(&mut self, adr: u32, val: u8) {
        self.audio.apu_write.ports[(adr as usize) & 3] = val;
    }

    pub fn zelda_debug_apu_write_ports(&self) -> [u8; 4] {
        self.audio.apu_write.ports
    }

    pub fn zelda_audio_route_state(&self) -> AudioRouteState {
        let pending_pos = self
            .audio
            .apu_write_ent_pos
            .wrapping_sub(self.audio.apu_write_count)
            & 0xf;
        let queue = AudioQueueState {
            pos: self.audio.apu_write_ent_pos,
            count: self.audio.apu_write_count,
            total: self.audio.apu_total_write,
            write: self.audio.apu_write.ports,
            pending: self.audio.apu_write_ents[pending_pos as usize].ports,
            input: self.audio.input_ports,
        };
        let spc = unsafe { self.audio.spc_player.as_ref() }.map(|player| SpcSequencerState {
            spc_in: player.input_ports,
            spc_out: player.port_to_snes,
            timer_cycles: player.timer_cycles,
            main_tempo_accum: player.main_tempo_accum,
            block_count: player.block_count,
            key_on: player.key_ON,
            key_off: player.key_OFF,
            current_bit: player.current_bit,
            port1_active: player.port1_active,
            port2_active: player.port2_active,
            port3_active: player.port3_active,
            is_chan_on: player.is_chan_on,
            vol_dirty: player.vol_dirty,
            ch7_sfx: player.channel[7].sfx_which_sound,
            ch7_sfx_ptr: player.channel[7].sfx_sound_ptr,
            ch7_pattern: player.channel[7].pattern_order_ptr_for_chan,
            ch7_ticks: player.channel[7].note_ticks_left,
            ch7_keyoff_ticks: player.channel[7].note_keyoff_ticks_left,
        });
        AudioRouteState {
            music: MusicControlState::from_game(self),
            queue,
            spc,
        }
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
                inidisp: self.ram[0x13],
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
        let pending_pos = self
            .audio
            .apu_write_ent_pos
            .wrapping_sub(self.audio.apu_write_count)
            & 0xf;
        let pending = self.audio.apu_write_ents[pending_pos as usize].ports;
        let mut out = format!(
            "\"queue\":{{\"pos\":{},\"count\":{},\"total\":{},\"write\":[{},{},{},{}],\"pending\":[{},{},{},{}],\"input\":[{},{},{},{}]",
            self.audio.apu_write_ent_pos,
            self.audio.apu_write_count,
            self.audio.apu_total_write,
            self.audio.apu_write.ports[0],
            self.audio.apu_write.ports[1],
            self.audio.apu_write.ports[2],
            self.audio.apu_write.ports[3],
            pending[0],
            pending[1],
            pending[2],
            pending[3],
            self.audio.input_ports[0],
            self.audio.input_ports[1],
            self.audio.input_ports[2],
            self.audio.input_ports[3],
        );
        if let Some(player) = unsafe { self.audio.spc_player.as_ref() } {
            out.push_str(&format!(
                ",\"spc_in\":[{},{},{},{}],\"spc_out\":[{},{},{},{}],\"timer\":{},\"main_tempo_accum\":{},\"block_count\":{},\"key_on\":{},\"key_off\":{},\"current_bit\":{},\"port1_active\":{},\"port2_active\":{},\"port3_active\":{}",
                player.input_ports[0],
                player.input_ports[1],
                player.input_ports[2],
                player.input_ports[3],
                player.port_to_snes[0],
                player.port_to_snes[1],
                player.port_to_snes[2],
                player.port_to_snes[3],
                player.timer_cycles,
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
                ",\"is_chan_on\":{},\"vol_dirty\":{},\"ch7_sfx\":{},\"ch7_sfx_ptr\":{},\"ch7_pattern\":{},\"ch7_ticks\":{},\"ch7_keyoff_ticks\":{}",
                player.is_chan_on,
                player.vol_dirty,
                player.channel[7].sfx_which_sound,
                player.channel[7].sfx_sound_ptr,
                player.channel[7].pattern_order_ptr_for_chan,
                player.channel[7].note_ticks_left,
                player.channel[7].note_keyoff_ticks_left,
            ));
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
        let bytes = bincode::serialize(&self.audio).expect("audio snapshot serialize failed");
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
                self.audio.apu_total_write,
                self.audio.input_ports
            );
        }
        bytes
    }

    /// Restore the full live audio state previously captured by
    /// `zelda_audio_snapshot_bytes`. Replaces `self.audio` wholesale (its `Drop`
    /// frees the prior SPC player); the deserialized `AudioState` re-creates the
    /// SPC player + DSP with all raw pointers correctly re-linked.
    pub fn zelda_audio_restore_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let restored: AudioState =
            bincode::deserialize(bytes).map_err(|e| format!("audio snapshot decode: {e}"))?;
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
                restored.apu_total_write,
                restored.input_ports
            );
        }
        self.audio = restored;
        Ok(())
    }

    pub fn zelda_audio_dsp_hash(&self) -> u32 {
        let mut hash = 2166136261u32;
        for byte in crate::spc_player::spc_player_save_dsp_c_saveload(self.audio.spc_player) {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
        hash
    }

    pub fn zelda_push_apu_state(&mut self) {
        let pos = (self.audio.apu_write_ent_pos & 0xf) as usize;
        self.audio.apu_write_ents[pos] = self.audio.apu_write;
        self.audio.apu_write_ent_pos = self.audio.apu_write_ent_pos.wrapping_add(1);
        if self.audio.apu_write_count < 16 {
            self.audio.apu_write_count += 1;
        }
        self.audio.apu_total_write = self.audio.apu_total_write.wrapping_add(1);
    }

    fn zelda_pop_apu_state(&mut self) {
        if self.audio.apu_write_count != 0 {
            let pos = self
                .audio
                .apu_write_ent_pos
                .wrapping_sub(self.audio.apu_write_count)
                & 0xf;
            self.audio.input_ports = self.audio.apu_write_ents[pos as usize].ports;
            if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
                player.input_ports = self.audio.input_ports;
            }
            self.audio.apu_write_count -= 1;
        }
    }

    pub fn zelda_discard_unused_audio_frames(&mut self) {
        if self.audio.apu_write_count != 0 {
            let pos = self
                .audio
                .apu_write_ent_pos
                .wrapping_sub(self.audio.apu_write_count)
                & 0xf;
            if self.audio.input_ports == self.audio.apu_write_ents[pos as usize].ports {
                if self.audio.apu_total_write >= 16 {
                    self.audio.apu_total_write = 14;
                    self.audio.apu_write_count -= 1;
                }
                return;
            }
        }
        self.audio.apu_total_write = 0;
    }

    fn zelda_reset_apu_queue(&mut self) {
        self.audio.apu_write_ent_pos = 0;
        self.audio.apu_total_write = 0;
        self.audio.apu_write_count = 0;
    }

    pub fn zelda_read_apui00(&self) -> u8 {
        self.game_state.system_signals.apui00()
    }

    pub fn zelda_apu_read(&self, adr: u32) -> u8 {
        if let Some(player) = unsafe { self.audio.spc_player.as_ref() } {
            player.port_to_snes[(adr as usize) & 3]
        } else {
            self.audio.port_to_snes[(adr as usize) & 3]
        }
    }

    pub fn zelda_render_audio(&mut self, audio_buffer: &mut [i16], samples: i32, channels: i32) {
        self.zelda_pop_apu_state();
        let count = (samples.max(0) as usize).saturating_mul(channels.max(0) as usize);
        if samples > 0 && channels > 0 {
            if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
                crate::spc_player::spc_player_generate_samples(player);
                crate::spc_player::dsp_get_samples(
                    player.dsp,
                    audio_buffer,
                    samples as usize,
                    channels as usize,
                );
            } else {
                for value in audio_buffer.iter_mut().take(count) {
                    *value = 0;
                }
            }
        }
        if self.audio.msu_player.has_file && channels == 2 {
            self.msu_player_mix(audio_buffer, samples);
        }
    }

    pub fn zelda_render_audio_trace_dsp(
        &mut self,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
    ) -> Vec<(u8, u8, i32, u8)> {
        self.zelda_pop_apu_state();
        let count = (samples.max(0) as usize).saturating_mul(channels.max(0) as usize);
        let mut writes = Vec::new();
        if samples > 0 && channels > 0 {
            if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
                let mut hist = crate::spc_player::DspRegWriteHistory::default();
                player.reg_write_history = &mut hist;
                crate::spc_player::spc_player_generate_samples(player);
                player.reg_write_history = std::ptr::null_mut();
                writes.reserve(hist.count);
                for i in 0..hist.count {
                    writes.push((
                        hist.addr[i],
                        hist.val[i],
                        hist.sample_offset[i],
                        hist.timer_cycles[i],
                    ));
                }
                crate::spc_player::dsp_get_samples(
                    player.dsp,
                    audio_buffer,
                    samples as usize,
                    channels as usize,
                );
            } else {
                for value in audio_buffer.iter_mut().take(count) {
                    *value = 0;
                }
            }
        }
        if self.audio.msu_player.has_file && channels == 2 {
            self.msu_player_mix(audio_buffer, samples);
        }
        writes
    }

    pub fn zelda_render_audio_trace_dsp_events(
        &mut self,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
    ) -> Vec<DspWriteEvent> {
        self.zelda_render_audio_trace_dsp(audio_buffer, samples, channels)
            .into_iter()
            .map(|(addr, value, sample_offset, timer_cycles)| {
                DspWriteEvent::new(addr, value, sample_offset, timer_cycles)
            })
            .collect()
    }

    pub fn zelda_render_audio_with_backend(
        &mut self,
        backend: AudioBackendMode,
        audio_buffer: &mut [i16],
        samples: i32,
        channels: i32,
    ) -> AudioEventFrame {
        match backend {
            AudioBackendMode::DspParity => {
                let writes =
                    self.zelda_render_audio_trace_dsp_events(audio_buffer, samples, channels);
                self.zelda_audio_event_frame_from_dsp_writes(&writes)
            }
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
                self.zelda_pop_apu_state();
                let route = self.zelda_audio_route_state();
                let frame = self.audio.modern_sequence.sequence_route(route);
                self.audio
                    .modern_audio
                    .render_frame(&frame, audio_buffer, samples, channels);
                frame
            }
        }
    }

    pub fn zelda_set_rom_startup_audio_phase(&mut self, enabled: bool) {
        let phase = if enabled { 72 } else { 0 };
        self.zelda_set_spc_startup_phase(phase, 0);
    }

    pub fn zelda_set_spc_startup_phase(&mut self, sfx_timer_accum: u8, timer_cycles: u8) {
        if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
            player.sfx_timer_accum = sfx_timer_accum;
            player.timer_cycles = timer_cycles.min(64);
            player.ram[0x0043] = sfx_timer_accum;
        }
        self.audio.spc_ram[0x0043] = sfx_timer_accum;
    }

    pub fn zelda_is_music_playing(&self) -> bool {
        if self.audio.msu_player.state != MSU_STATE_IDLE {
            self.audio.msu_player.state != MSU_STATE_FINISHED_PLAYING
        } else {
            self.zelda_apu_read(0x2140) != 0
        }
    }

    pub fn zelda_restore_music_after_load_locked(&mut self, is_reset: bool) {
        crate::spc_player::spc_player_copy_variables_from_ram(self.audio.spc_player);
        if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
            player.timer_cycles = 0;
            player
                .input_ports
                .copy_from_slice(&player.ram[0x410..0x414]);
            self.audio.input_ports = player.input_ports;
            self.audio
                .apu_write
                .ports
                .copy_from_slice(&player.input_ports);
        } else {
            self.audio
                .apu_write
                .ports
                .copy_from_slice(&self.audio.spc_ram[0x410..0x414]);
        }
        if is_reset {
            self.audio.port_to_snes = [0; 4];
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
        crate::spc_player::spc_player_copy_variables_to_ram(self.audio.spc_player);
        if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
            player.ram[0x410..0x414].copy_from_slice(&self.audio.apu_write.ports);
            self.audio.spc_ram = player.ram;
        } else {
            self.audio.spc_ram[0x410..0x414].copy_from_slice(&self.audio.apu_write.ports);
        }
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
        let len = p.len().min(self.audio.spc_ram.len());
        self.audio.spc_ram[..len].copy_from_slice(&p[..len]);
        if let Some(player) = unsafe { self.audio.spc_player.as_mut() } {
            crate::spc_player::spc_player_upload(player, p.as_ptr());
        }
    }

    pub fn zelda_audio_debug_summary(&self) -> String {
        crate::spc_player::spc_player_debug_summary(self.audio.spc_player)
    }

    pub fn zelda_debug_full_apu_from_spc(&self) -> snes::apu::ApuState {
        let mut apu = snes::apu::ApuState::new();
        apu.reset();
        let spc_ram = crate::spc_player::spc_player_save_ram(self.audio.spc_player);
        apu.ram.copy_from_slice(&spc_ram);
        apu.rom_readable = false;
        apu.spc.pc = 0x800;
        apu
    }

    pub(super) fn save_audio_apu_ram_c_saveload(&self) -> [u8; 0x10000] {
        crate::spc_player::spc_player_save_ram(self.audio.spc_player)
    }

    pub(super) fn load_audio_apu_ram_c_saveload(&mut self, data: &[u8]) {
        let len = data.len().min(self.audio.spc_ram.len());
        self.audio.spc_ram[..len].copy_from_slice(&data[..len]);
        if len < self.audio.spc_ram.len() {
            self.audio.spc_ram[len..].fill(0);
        }
        crate::spc_player::spc_player_load_ram(self.audio.spc_player, &self.audio.spc_ram);
    }

    pub(super) fn save_audio_dsp_c_saveload(&self) -> Vec<u8> {
        crate::spc_player::spc_player_save_dsp_c_saveload(self.audio.spc_player)
    }

    pub(super) fn load_audio_dsp_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        crate::spc_player::spc_player_load_dsp_c_saveload(self.audio.spc_player, data)
    }

    fn msu_resume_info(&self, slot: MsuResumeSlot) -> MsuResumeInfoState {
        self.game_state.system_signals.msu_resume_info(slot)
    }

    fn save_msu_resume_info(&mut self, slot: MsuResumeSlot, info: MsuResumeInfoState) {
        self.set_msu_resume_info(slot, info);
    }
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

#[cfg(test)]
#[path = "audio_tests.rs"]
mod tests;
