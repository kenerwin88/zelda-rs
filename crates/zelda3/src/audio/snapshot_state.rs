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
        let queue = &state.modern.queue;
        Self {
            modern_audio: state.modern.renderer.clone(),
            modern_sequence: state.modern.sequencer.clone(),
            apu_write_ents: queue.legacy_write_history(),
            apu_write: ApuWriteEnt::from_commands(queue.pending_write),
            apu_write_ent_pos: queue.write_position,
            apu_write_count: queue.write_count,
            apu_total_write: queue.total_writes,
            input_ports: queue.input_commands.legacy_ports(),
            port_to_snes: queue.acknowledged_commands.legacy_ports(),
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
        state.modern.renderer = self.modern_audio;
        state.modern.sequencer = self.modern_sequence;
        state.modern.queue = ModernAudioCommandQueue::from_legacy_transport(
            self.apu_write_ents,
            self.apu_write,
            self.apu_write_ent_pos,
            self.apu_write_count,
            self.apu_total_write,
            self.input_ports,
            self.port_to_snes,
        );
        state
            .legacy_compatibility
            .import_legacy_ram(&self.modern_sample_ram);
        if let Some(bank_id) = crate::modern_sample_bank::identify_spc_ram(&self.modern_sample_ram)
        {
            state.modern.renderer.select_sample_bank(bank_id);
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

/// Frozen APUI-shaped transport used only to decode version-5 snapshots.
#[derive(serde::Serialize, serde::Deserialize)]
struct LegacyApuTransportV5 {
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

impl LegacyApuTransportV5 {
    fn capture(state: &AudioState) -> Self {
        let queue = &state.modern.queue;
        Self {
            write_history: queue.legacy_write_history(),
            pending_write: ApuWriteEnt::from_commands(queue.pending_write),
            write_position: queue.write_position,
            write_count: queue.write_count,
            total_writes: queue.total_writes,
            input_ports: queue.input_commands.legacy_ports(),
            output_ports: queue.acknowledged_commands.legacy_ports(),
            saved_music_ports: state.legacy_compatibility.saved_music_ports,
            startup_sfx_timer_accum: state.legacy_compatibility.startup_sfx_timer_accum,
        }
    }

    fn apply(self, state: &mut AudioState) {
        state.modern.queue = ModernAudioCommandQueue::from_legacy_transport(
            self.write_history,
            self.pending_write,
            self.write_position,
            self.write_count,
            self.total_writes,
            self.input_ports,
            self.output_ports,
        );
        state.legacy_compatibility.saved_music_ports = self.saved_music_ports;
        state.legacy_compatibility.startup_sfx_timer_accum = self.startup_sfx_timer_accum;
    }
}

/// Version-5 modern payload retained as a compatibility decoder.
#[derive(serde::Serialize, serde::Deserialize)]
struct CompactModernAudioStateSnapshotV5 {
    modern_apu: LegacyApuTransportV5,
    volume_transition_step_float: [f32; 4],
    volume_transition_target_float: [f32; 4],
    config_audio_freq: u32,
    config_msuvolume: u8,
    config_resume_msu: bool,
    config_msu_path: Option<String>,
    modern_audio: ModernAudioEngine,
    modern_sequence: ModernAudioSequencer,
}

impl CompactModernAudioStateSnapshotV5 {
    fn capture(state: &AudioState) -> Self {
        Self {
            modern_apu: LegacyApuTransportV5::capture(state),
            volume_transition_step_float: state.volume_transition_step_float,
            volume_transition_target_float: state.volume_transition_target_float,
            config_audio_freq: state.config_audio_freq,
            config_msuvolume: state.config_msuvolume,
            config_resume_msu: state.config_resume_msu,
            config_msu_path: state.config_msu_path.clone(),
            modern_audio: state.modern.renderer.clone(),
            modern_sequence: state.modern.sequencer.clone(),
        }
    }

    fn into_audio_state(self) -> AudioState {
        let mut state = AudioState::default();
        self.modern_apu.apply(&mut state);
        state.volume_transition_step_float = self.volume_transition_step_float;
        state.volume_transition_target_float = self.volume_transition_target_float;
        state.config_audio_freq = self.config_audio_freq;
        state.config_msuvolume = self.config_msuvolume;
        state.config_resume_msu = self.config_resume_msu;
        state.config_msu_path = self.config_msu_path;
        state.modern.renderer = self.modern_audio;
        state.modern.sequencer = self.modern_sequence;
        state
    }
}

/// Version-6 payload: typed runtime state with legacy compatibility fields
/// isolated from production command transport.
#[derive(serde::Serialize, serde::Deserialize)]
struct CompactModernAudioStateSnapshotV6 {
    modern: ModernAudioRuntime,
    legacy_compatibility: LegacyAudioCompatibilityState,
    volume_transition_step_float: [f32; 4],
    volume_transition_target_float: [f32; 4],
    config_audio_freq: u32,
    config_msuvolume: u8,
    config_resume_msu: bool,
    config_msu_path: Option<String>,
}

impl CompactModernAudioStateSnapshotV6 {
    fn capture(state: &AudioState) -> Self {
        Self {
            modern: state.modern.clone(),
            legacy_compatibility: state.legacy_compatibility,
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
        state.modern = self.modern;
        state.legacy_compatibility = self.legacy_compatibility;
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
struct AudioSnapshotV5 {
    modern: CompactModernAudioStateSnapshotV5,
    oracle_sidecar: Option<Vec<u8>>,
    sample_bank_id: u8,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AudioSnapshotV6 {
    modern: CompactModernAudioStateSnapshotV6,
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
        modern: CompactModernAudioStateSnapshotV5::capture(state),
        oracle_sidecar,
        sample_bank_id: state.modern.renderer.sample_bank_id(),
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
        .modern
        .renderer
        .select_sample_bank(snapshot.sample_bank_id);
    restore_oracle_sidecar(
        state,
        snapshot.oracle_sidecar,
        OracleShadowRestore::FromSidecar,
    )
}

pub(super) fn encode_v6(state: &AudioState) -> Result<(Vec<u8>, bool), String> {
    let oracle_sidecar = capture_oracle_sidecar(state)?;
    let has_oracle_sidecar = oracle_sidecar.is_some();
    let payload = bincode::serialize(&AudioSnapshotV6 {
        modern: CompactModernAudioStateSnapshotV6::capture(state),
        oracle_sidecar,
        sample_bank_id: state.modern.renderer.sample_bank_id(),
    })
    .map_err(|error| format!("audio snapshot v6 encode: {error}"))?;
    Ok((payload, has_oracle_sidecar))
}

pub(super) fn decode_v6(payload: &[u8]) -> Result<(AudioState, bool), String> {
    let snapshot: AudioSnapshotV6 = bincode::deserialize(payload)
        .map_err(|error| format!("audio snapshot v6 decode: {error}"))?;
    if !crate::modern_sample_bank::is_valid_bank(snapshot.sample_bank_id) {
        return Err(format!(
            "audio snapshot v6 has unknown sample bank {}",
            snapshot.sample_bank_id
        ));
    }
    let mut state = snapshot.modern.into_audio_state();
    state
        .modern
        .renderer
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
    state.modern.renderer.select_sample_bank(sample_bank_id);
    Ok((state, has_oracle_sidecar))
}

impl serde::Serialize for AudioState {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        AudioSnapshotV6 {
            modern: CompactModernAudioStateSnapshotV6::capture(self),
            oracle_sidecar: capture_oracle_sidecar(self).map_err(serde::ser::Error::custom)?,
            sample_bank_id: self.modern.renderer.sample_bank_id(),
        }
        .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AudioState {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let snapshot = AudioSnapshotV6::deserialize(deserializer)?;
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
            state.modern.renderer.select_sample_bank(sample_bank_id);
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
        sample_bank_id: state.modern.renderer.sample_bank_id(),
    })
    .unwrap();
    (payload, has_oracle_sidecar)
}

#[cfg(test)]
pub(super) fn encode_v5_for_test(state: &AudioState) -> (Vec<u8>, bool) {
    encode_v5(state).unwrap()
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
        let queue = &state.modern.queue;
        Self {
            spc_player: crate::spc_player::spc_player_snapshot(state.spc_player),
            apu_write_ents: queue.legacy_write_history(),
            apu_write: ApuWriteEnt::from_commands(queue.pending_write),
            apu_write_ent_pos: queue.write_position,
            apu_write_count: queue.write_count,
            apu_total_write: queue.total_writes,
            input_ports: queue.input_commands.legacy_ports(),
            port_to_snes: queue.acknowledged_commands.legacy_ports(),
            modern_sample_ram: state.legacy_modern_apu_ram(),
            volume_transition_step_float: state.volume_transition_step_float,
            volume_transition_target_float: state.volume_transition_target_float,
            config_audio_freq: state.config_audio_freq,
            config_msuvolume: state.config_msuvolume,
            config_resume_msu: state.config_resume_msu,
            config_msu_path: state.config_msu_path.clone(),
            modern_audio: state.modern.renderer.clone(),
            modern_sequence: state.modern.sequencer.clone(),
        }
    }

    fn into_audio_state(self) -> AudioState {
        let mut state = AudioState::default();
        crate::spc_player::spc_player_destroy(state.spc_player);
        state.spc_player = crate::spc_player::spc_player_from_snapshot(self.spc_player);
        state.modern.renderer = self.modern_audio;
        state.modern.sequencer = self.modern_sequence;
        state.modern.queue = ModernAudioCommandQueue::from_legacy_transport(
            self.apu_write_ents,
            self.apu_write,
            self.apu_write_ent_pos,
            self.apu_write_count,
            self.apu_total_write,
            self.input_ports,
            self.port_to_snes,
        );
        state
            .legacy_compatibility
            .import_legacy_ram(&self.modern_sample_ram);
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
        state.modern.queue = ModernAudioCommandQueue::from_legacy_transport(
            self.apu_write_ents,
            self.apu_write,
            self.apu_write_ent_pos,
            self.apu_write_count,
            self.apu_total_write,
            self.input_ports,
            self.port_to_snes,
        );
        state.legacy_compatibility.import_legacy_ram(&self.spc_ram);
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
