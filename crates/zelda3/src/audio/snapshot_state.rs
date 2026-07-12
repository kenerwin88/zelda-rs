use super::*;

/// Feature-selected wire payload for `AudioState`. Version 1 (oracle builds)
/// replaces the `spc_player` raw pointer with a deep pointer-free snapshot;
/// version 2 (normal builds) omits legacy SPC/DSP state. The `msu_player` is
/// intentionally not round-tripped: MSU (external music
/// streaming) is disabled in headless replay, it owns non-serde state
/// (`OpusDecoder`), and on restore it is reconstructed as `MsuPlayer::default()`.
/// Runtime backend selection is host configuration and is likewise rebuilt
/// from its modern default or an operator override after restore. Modern-owned
/// sample RAM, sequencing, rendering, queue, and configuration state remain.
#[derive(serde::Serialize, serde::Deserialize)]
struct AudioStateSnapshot {
    #[cfg(feature = "audio-oracle")]
    spc_player: crate::spc_player::SpcPlayerSnapshot,
    apu_write_ents: [ApuWriteEnt; 16],
    apu_write: ApuWriteEnt,
    apu_write_ent_pos: u8,
    apu_write_count: u8,
    apu_total_write: u8,
    input_ports: [u8; 4],
    port_to_snes: [u8; 4],
    #[serde(with = "serde_big_array::BigArray")]
    modern_sample_ram: [u8; 0x10000],
    volume_transition_step_float: [f32; 4],
    volume_transition_target_float: [f32; 4],
    config_audio_freq: u32,
    config_msuvolume: u8,
    config_resume_msu: bool,
    config_msu_path: Option<String>,
    #[serde(default)]
    modern_audio: ModernAudioEngine,
    #[serde(default)]
    modern_sequence: ModernAudioSequencer,
}

#[derive(serde::Deserialize)]
#[cfg(feature = "audio-oracle")]
pub(super) struct LegacyAudioStateSnapshot {
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

#[cfg(feature = "audio-oracle")]
impl LegacyAudioStateSnapshot {
    pub(super) fn into_audio_state(self) -> AudioState {
        AudioState {
            spc_player: crate::spc_player::spc_player_from_snapshot(self.spc_player),
            backend: AudioBackendMode::default(),
            audio_has_rendered: false,
            msu_player: MsuPlayer::default(),
            modern_audio: ModernAudioEngine::default(),
            modern_sequence: ModernAudioSequencer::default(),
            apu_write_ents: self.apu_write_ents,
            apu_write: self.apu_write,
            apu_write_ent_pos: self.apu_write_ent_pos,
            apu_write_count: self.apu_write_count,
            apu_total_write: self.apu_total_write,
            input_ports: self.input_ports,
            port_to_snes: self.port_to_snes,
            modern_sample_ram: self.spc_ram,
            volume_transition_step_float: self.volume_transition_step_float,
            volume_transition_target_float: self.volume_transition_target_float,
            config_audio_freq: self.config_audio_freq,
            config_msuvolume: self.config_msuvolume,
            config_resume_msu: self.config_resume_msu,
            config_msu_path: self.config_msu_path,
        }
    }
}

impl serde::Serialize for AudioState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let snapshot = AudioStateSnapshot {
            #[cfg(feature = "audio-oracle")]
            spc_player: crate::spc_player::spc_player_snapshot(self.spc_player),
            modern_audio: self.modern_audio.clone(),
            modern_sequence: self.modern_sequence.clone(),
            apu_write_ents: self.apu_write_ents,
            apu_write: self.apu_write,
            apu_write_ent_pos: self.apu_write_ent_pos,
            apu_write_count: self.apu_write_count,
            apu_total_write: self.apu_total_write,
            input_ports: self.input_ports,
            port_to_snes: self.port_to_snes,
            modern_sample_ram: self.modern_sample_ram,
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
        #[cfg(feature = "audio-oracle")]
        let spc_player = crate::spc_player::spc_player_from_snapshot(snapshot.spc_player);
        Ok(Self {
            #[cfg(feature = "audio-oracle")]
            spc_player,
            backend: AudioBackendMode::default(),
            audio_has_rendered: false,
            msu_player: MsuPlayer::default(),
            modern_audio: snapshot.modern_audio,
            modern_sequence: snapshot.modern_sequence,
            apu_write_ents: snapshot.apu_write_ents,
            apu_write: snapshot.apu_write,
            apu_write_ent_pos: snapshot.apu_write_ent_pos,
            apu_write_count: snapshot.apu_write_count,
            apu_total_write: snapshot.apu_total_write,
            input_ports: snapshot.input_ports,
            port_to_snes: snapshot.port_to_snes,
            modern_sample_ram: snapshot.modern_sample_ram,
            volume_transition_step_float: snapshot.volume_transition_step_float,
            volume_transition_target_float: snapshot.volume_transition_target_float,
            config_audio_freq: snapshot.config_audio_freq,
            config_msuvolume: snapshot.config_msuvolume,
            config_resume_msu: snapshot.config_resume_msu,
            config_msu_path: snapshot.config_msu_path,
        })
    }
}
