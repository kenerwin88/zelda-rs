use super::*;

/// Stable, feature-independent modern audio payload used by snapshot v2 and
/// nested inside v3. Host backend selection and MSU streaming resources are
/// intentionally rebuilt after restore.
#[derive(serde::Serialize, serde::Deserialize)]
struct ModernAudioStateSnapshot {
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

impl ModernAudioStateSnapshot {
    fn capture(state: &AudioState) -> Self {
        let apu = &state.modern_apu;
        Self {
            modern_audio: state.modern_audio.clone(),
            modern_sequence: state.modern_sequence.clone(),
            apu_write_ents: apu.write_history,
            apu_write: apu.pending_write,
            apu_write_ent_pos: apu.write_position,
            apu_write_count: apu.write_count,
            apu_total_write: apu.total_writes,
            input_ports: apu.input_ports,
            port_to_snes: apu.output_ports,
            modern_sample_ram: state.legacy_modern_apu_ram(),
            volume_transition_step_float: state.volume_transition_step_float,
            volume_transition_target_float: state.volume_transition_target_float,
            config_audio_freq: state.config_audio_freq,
            config_msuvolume: state.config_msuvolume,
            config_resume_msu: state.config_resume_msu,
            config_msu_path: state.config_msu_path.clone(),
        }
    }

    fn into_audio_state(self) -> AudioState {
        let mut state = AudioState::default();
        state.modern_audio = self.modern_audio;
        state.modern_sequence = self.modern_sequence;
        state.modern_apu.write_history = self.apu_write_ents;
        state.modern_apu.pending_write = self.apu_write;
        state.modern_apu.write_position = self.apu_write_ent_pos;
        state.modern_apu.write_count = self.apu_write_count;
        state.modern_apu.total_writes = self.apu_total_write;
        state.modern_apu.input_ports = self.input_ports;
        state.modern_apu.output_ports = self.port_to_snes;
        state.modern_apu.import_legacy_ram(&self.modern_sample_ram);
        if let Some(bank_id) = crate::modern_sample_bank::identify_spc_ram(&self.modern_sample_ram)
        {
            state.modern_audio.select_sample_bank(bank_id);
        }
        #[cfg(feature = "audio-oracle")]
        {
            state.modern_sample_ram = self.modern_sample_ram;
        }
        state.volume_transition_step_float = self.volume_transition_step_float;
        state.volume_transition_target_float = self.volume_transition_target_float;
        state.config_audio_freq = self.config_audio_freq;
        state.config_msuvolume = self.config_msuvolume;
        state.config_resume_msu = self.config_resume_msu;
        state.config_msu_path = self.config_msu_path;
        state
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AudioSnapshotV3 {
    modern: ModernAudioStateSnapshot,
    oracle_sidecar: Option<Vec<u8>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AudioSnapshotV4 {
    modern: ModernAudioStateSnapshot,
    oracle_sidecar: Option<Vec<u8>>,
    sample_bank_id: u8,
}

/// Version-5 modern payload. Unlike v2-v4 this contains only state the modern
/// runtime actually consumes; the legacy 64 KiB SPC RAM image is deliberately
/// absent.
#[derive(serde::Serialize, serde::Deserialize)]
struct CompactModernAudioStateSnapshot {
    modern_apu: ModernApuState,
    volume_transition_step_float: [f32; 4],
    volume_transition_target_float: [f32; 4],
    config_audio_freq: u32,
    config_msuvolume: u8,
    config_resume_msu: bool,
    config_msu_path: Option<String>,
    modern_audio: ModernAudioEngine,
    modern_sequence: ModernAudioSequencer,
}

impl CompactModernAudioStateSnapshot {
    fn capture(state: &AudioState) -> Self {
        Self {
            modern_apu: state.modern_apu,
            volume_transition_step_float: state.volume_transition_step_float,
            volume_transition_target_float: state.volume_transition_target_float,
            config_audio_freq: state.config_audio_freq,
            config_msuvolume: state.config_msuvolume,
            config_resume_msu: state.config_resume_msu,
            config_msu_path: state.config_msu_path.clone(),
            modern_audio: state.modern_audio.clone(),
            modern_sequence: state.modern_sequence.clone(),
        }
    }

    fn into_audio_state(self) -> AudioState {
        let mut state = AudioState::default();
        state.modern_apu = self.modern_apu;
        state.volume_transition_step_float = self.volume_transition_step_float;
        state.volume_transition_target_float = self.volume_transition_target_float;
        state.config_audio_freq = self.config_audio_freq;
        state.config_msuvolume = self.config_msuvolume;
        state.config_resume_msu = self.config_resume_msu;
        state.config_msu_path = self.config_msu_path;
        state.modern_audio = self.modern_audio;
        state.modern_sequence = self.modern_sequence;
        state
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AudioSnapshotV5 {
    modern: CompactModernAudioStateSnapshot,
    oracle_sidecar: Option<Vec<u8>>,
    sample_bank_id: u8,
}

fn capture_oracle_sidecar(state: &AudioState) -> Result<Option<Vec<u8>>, String> {
    #[cfg(feature = "audio-oracle")]
    {
        return bincode::serialize(&crate::spc_player::spc_player_snapshot(state.spc_player))
            .map(Some)
            .map_err(|error| format!("oracle snapshot sidecar encode: {error}"));
    }
    #[cfg(not(feature = "audio-oracle"))]
    {
        let _ = state;
        Ok(None)
    }
}

#[derive(Clone, Copy)]
enum OracleShadowRestore {
    Preserve,
    FromSidecar,
}

fn restore_oracle_sidecar(
    state: AudioState,
    oracle_sidecar: Option<Vec<u8>>,
    shadow_restore: OracleShadowRestore,
) -> Result<(AudioState, bool), String> {
    let has_oracle_sidecar = oracle_sidecar.is_some();
    #[cfg(feature = "audio-oracle")]
    {
        let mut state = state;
        if let Some(sidecar) = oracle_sidecar {
            let oracle = bincode::deserialize(&sidecar)
                .map_err(|error| format!("oracle snapshot sidecar decode: {error}"))?;
            crate::spc_player::spc_player_destroy(state.spc_player);
            state.spc_player = crate::spc_player::spc_player_from_snapshot(oracle);
            if matches!(shadow_restore, OracleShadowRestore::FromSidecar) {
                state.modern_sample_ram = crate::spc_player::spc_player_save_ram(state.spc_player);
            }
        }
        return Ok((state, has_oracle_sidecar));
    }
    #[cfg(not(feature = "audio-oracle"))]
    {
        let _ = (oracle_sidecar, shadow_restore);
        Ok((state, has_oracle_sidecar))
    }
}

fn capture_v3(state: &AudioState) -> Result<AudioSnapshotV3, String> {
    Ok(AudioSnapshotV3 {
        modern: ModernAudioStateSnapshot::capture(state),
        oracle_sidecar: capture_oracle_sidecar(state)?,
    })
}

fn restore_v3(snapshot: AudioSnapshotV3) -> Result<(AudioState, bool), String> {
    restore_oracle_sidecar(
        snapshot.modern.into_audio_state(),
        snapshot.oracle_sidecar,
        OracleShadowRestore::Preserve,
    )
}

pub(super) fn decode_v3(payload: &[u8]) -> Result<(AudioState, bool), String> {
    let snapshot: AudioSnapshotV3 = bincode::deserialize(payload)
        .map_err(|error| format!("audio snapshot v3 decode: {error}"))?;
    restore_v3(snapshot)
}

pub(super) fn encode_v5(state: &AudioState) -> Result<(Vec<u8>, bool), String> {
    let oracle_sidecar = capture_oracle_sidecar(state)?;
    let has_oracle_sidecar = oracle_sidecar.is_some();
    let payload = bincode::serialize(&AudioSnapshotV5 {
        modern: CompactModernAudioStateSnapshot::capture(state),
        oracle_sidecar,
        sample_bank_id: state.modern_audio.sample_bank_id(),
    })
    .map_err(|error| format!("audio snapshot v5 encode: {error}"))?;
    Ok((payload, has_oracle_sidecar))
}

pub(super) fn decode_v5(payload: &[u8]) -> Result<(AudioState, bool), String> {
    let snapshot: AudioSnapshotV5 = bincode::deserialize(payload)
        .map_err(|error| format!("audio snapshot v5 decode: {error}"))?;
    if !crate::modern_sample_bank::is_valid_bank(snapshot.sample_bank_id) {
        return Err(format!(
            "audio snapshot v5 has unknown sample bank {}",
            snapshot.sample_bank_id
        ));
    }
    let mut state = snapshot.modern.into_audio_state();
    state
        .modern_audio
        .select_sample_bank(snapshot.sample_bank_id);
    restore_oracle_sidecar(
        state,
        snapshot.oracle_sidecar,
        OracleShadowRestore::FromSidecar,
    )
}

pub(super) fn decode_v4(payload: &[u8]) -> Result<(AudioState, bool), String> {
    let snapshot: AudioSnapshotV4 = bincode::deserialize(payload)
        .map_err(|error| format!("audio snapshot v4 decode: {error}"))?;
    let sample_bank_id = snapshot.sample_bank_id;
    if !crate::modern_sample_bank::is_valid_bank(sample_bank_id) {
        return Err(format!(
            "audio snapshot v4 has unknown sample bank {sample_bank_id}"
        ));
    }
    let (mut state, has_oracle_sidecar) = restore_v3(AudioSnapshotV3 {
        modern: snapshot.modern,
        oracle_sidecar: snapshot.oracle_sidecar,
    })?;
    state.modern_audio.select_sample_bank(sample_bank_id);
    Ok((state, has_oracle_sidecar))
}

impl serde::Serialize for AudioState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        AudioSnapshotV5 {
            modern: CompactModernAudioStateSnapshot::capture(self),
            oracle_sidecar: capture_oracle_sidecar(self).map_err(serde::ser::Error::custom)?,
            sample_bank_id: self.modern_audio.sample_bank_id(),
        }
        .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AudioState {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let snapshot = AudioSnapshotV5::deserialize(deserializer)?;
        let sample_bank_id = snapshot.sample_bank_id;
        if !crate::modern_sample_bank::is_valid_bank(sample_bank_id) {
            return Err(serde::de::Error::custom(format!(
                "audio state has unknown sample bank {sample_bank_id}"
            )));
        }
        restore_oracle_sidecar(
            snapshot.modern.into_audio_state(),
            snapshot.oracle_sidecar,
            OracleShadowRestore::FromSidecar,
        )
        .map(|(mut state, _)| {
            state.modern_audio.select_sample_bank(sample_bank_id);
            state
        })
        .map_err(serde::de::Error::custom)
    }
}

pub(super) fn decode_v2(payload: &[u8]) -> Result<AudioState, String> {
    bincode::deserialize::<ModernAudioStateSnapshot>(payload)
        .map(ModernAudioStateSnapshot::into_audio_state)
        .map_err(|error| format!("audio snapshot v2 decode: {error}"))
}

#[cfg(test)]
pub(super) fn encode_v2_for_test(state: &AudioState) -> Vec<u8> {
    bincode::serialize(&ModernAudioStateSnapshot::capture(state)).unwrap()
}

#[cfg(test)]
pub(super) fn encode_v3_without_sidecar_for_test(state: &AudioState) -> Vec<u8> {
    bincode::serialize(&AudioSnapshotV3 {
        modern: ModernAudioStateSnapshot::capture(state),
        oracle_sidecar: None,
    })
    .unwrap()
}

#[cfg(test)]
pub(super) fn encode_v4_for_test(state: &AudioState) -> (Vec<u8>, bool) {
    let snapshot = capture_v3(state).unwrap();
    let has_oracle_sidecar = snapshot.oracle_sidecar.is_some();
    let payload = bincode::serialize(&AudioSnapshotV4 {
        modern: snapshot.modern,
        oracle_sidecar: snapshot.oracle_sidecar,
        sample_bank_id: state.modern_audio.sample_bank_id(),
    })
    .unwrap();
    (payload, has_oracle_sidecar)
}

#[cfg(all(test, not(feature = "audio-oracle")))]
pub(super) fn encode_v3_with_opaque_sidecar_for_test(
    state: &AudioState,
    sidecar: Vec<u8>,
) -> Vec<u8> {
    bincode::serialize(&AudioSnapshotV3 {
        modern: ModernAudioStateSnapshot::capture(state),
        oracle_sidecar: Some(sidecar),
    })
    .unwrap()
}

/// Version-1 oracle payload. Its field order is frozen to the former
/// feature-selected `AudioStateSnapshot` representation.
#[cfg(feature = "audio-oracle")]
#[derive(serde::Serialize, serde::Deserialize)]
struct OracleAudioStateSnapshotV1 {
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

#[cfg(feature = "audio-oracle")]
impl OracleAudioStateSnapshotV1 {
    #[cfg(test)]
    fn capture(state: &AudioState) -> Self {
        let apu = &state.modern_apu;
        Self {
            spc_player: crate::spc_player::spc_player_snapshot(state.spc_player),
            apu_write_ents: apu.write_history,
            apu_write: apu.pending_write,
            apu_write_ent_pos: apu.write_position,
            apu_write_count: apu.write_count,
            apu_total_write: apu.total_writes,
            input_ports: apu.input_ports,
            port_to_snes: apu.output_ports,
            modern_sample_ram: state.legacy_modern_apu_ram(),
            volume_transition_step_float: state.volume_transition_step_float,
            volume_transition_target_float: state.volume_transition_target_float,
            config_audio_freq: state.config_audio_freq,
            config_msuvolume: state.config_msuvolume,
            config_resume_msu: state.config_resume_msu,
            config_msu_path: state.config_msu_path.clone(),
            modern_audio: state.modern_audio.clone(),
            modern_sequence: state.modern_sequence.clone(),
        }
    }

    fn into_audio_state(self) -> AudioState {
        let mut state = AudioState::default();
        crate::spc_player::spc_player_destroy(state.spc_player);
        state.spc_player = crate::spc_player::spc_player_from_snapshot(self.spc_player);
        state.modern_audio = self.modern_audio;
        state.modern_sequence = self.modern_sequence;
        state.modern_apu.write_history = self.apu_write_ents;
        state.modern_apu.pending_write = self.apu_write;
        state.modern_apu.write_position = self.apu_write_ent_pos;
        state.modern_apu.write_count = self.apu_write_count;
        state.modern_apu.total_writes = self.apu_total_write;
        state.modern_apu.input_ports = self.input_ports;
        state.modern_apu.output_ports = self.port_to_snes;
        state.modern_apu.import_legacy_ram(&self.modern_sample_ram);
        state.modern_sample_ram = self.modern_sample_ram;
        state.volume_transition_step_float = self.volume_transition_step_float;
        state.volume_transition_target_float = self.volume_transition_target_float;
        state.config_audio_freq = self.config_audio_freq;
        state.config_msuvolume = self.config_msuvolume;
        state.config_resume_msu = self.config_resume_msu;
        state.config_msu_path = self.config_msu_path;
        state
    }
}

#[cfg(feature = "audio-oracle")]
pub(super) fn decode_v1(payload: &[u8]) -> Result<AudioState, String> {
    bincode::deserialize::<OracleAudioStateSnapshotV1>(payload)
        .map(OracleAudioStateSnapshotV1::into_audio_state)
        .map_err(|error| format!("audio snapshot v1 decode: {error}"))
}

#[cfg(all(test, feature = "audio-oracle"))]
pub(super) fn encode_v1_for_test(state: &AudioState) -> Vec<u8> {
    bincode::serialize(&OracleAudioStateSnapshotV1::capture(state)).unwrap()
}

#[derive(serde::Deserialize)]
#[cfg(feature = "audio-oracle")]
struct LegacyAudioStateSnapshot {
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
    fn into_audio_state(self) -> AudioState {
        let mut state = AudioState::default();
        crate::spc_player::spc_player_destroy(state.spc_player);
        state.spc_player = crate::spc_player::spc_player_from_snapshot(self.spc_player);
        state.modern_apu.write_history = self.apu_write_ents;
        state.modern_apu.pending_write = self.apu_write;
        state.modern_apu.write_position = self.apu_write_ent_pos;
        state.modern_apu.write_count = self.apu_write_count;
        state.modern_apu.total_writes = self.apu_total_write;
        state.modern_apu.input_ports = self.input_ports;
        state.modern_apu.output_ports = self.port_to_snes;
        state.modern_apu.import_legacy_ram(&self.spc_ram);
        state.modern_sample_ram = self.spc_ram;
        state.volume_transition_step_float = self.volume_transition_step_float;
        state.volume_transition_target_float = self.volume_transition_target_float;
        state.config_audio_freq = self.config_audio_freq;
        state.config_msuvolume = self.config_msuvolume;
        state.config_resume_msu = self.config_resume_msu;
        state.config_msu_path = self.config_msu_path;
        state
    }
}

#[cfg(feature = "audio-oracle")]
pub(super) fn decode_headerless(payload: &[u8]) -> Result<AudioState, String> {
    decode_v1(payload).or_else(|v1_error| {
        bincode::deserialize::<LegacyAudioStateSnapshot>(payload)
            .map(LegacyAudioStateSnapshot::into_audio_state)
            .or_else(|legacy_error| {
                decode_v2(payload).map_err(|v2_error| {
                    format!(
                        "audio snapshot decode: v1={v1_error}; legacy={legacy_error}; v2={v2_error}"
                    )
                })
            })
    })
}
