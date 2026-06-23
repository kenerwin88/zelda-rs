use super::*;

fn msu1_header(repeat_position: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"MSU1");
    bytes.extend_from_slice(&repeat_position.to_le_bytes());
    bytes
}

fn opuz_header(repeat_position: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"OPUZ");
    bytes.extend_from_slice(&repeat_position.to_le_bytes());
    bytes
}

fn unique_temp_msu_dir() -> std::path::PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("zelda3-rs-msu-test-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn resume_info(tag: u32, actual_track: u8) -> MsuResumeInfoState {
    MsuResumeInfoState {
        tag,
        offset: 0x1122_3344,
        samples_until_repeat: 0x5566_7788,
        range_cur: 0x99aa,
        range_repeat: 0xbbcc,
        initial_packet_bytes: 0x1122_3344_5566_7788,
        orig_track: 5,
        actual_track,
    }
}

#[test]
fn msu_open_saves_active_resume_info_to_alt_slot_like_c() {
    let mut state = ZeldaState::new();
    state.audio.msu_player.enabled = 1;
    state.audio.msu_player.state = MSU_STATE_PLAYING;
    state.audio.msu_player.resume_info = resume_info(0x1234_5678, 5);

    state.msu_player_open(5, false);

    assert_eq!(
        state.msu_resume_info(MsuResumeSlot::Alternate),
        resume_info(0x1234_5678, 5)
    );
    assert_eq!(state.audio.msu_player.state, MSU_STATE_IDLE);
}

#[test]
fn audio_config_feeds_msu_volume_and_resume_settings() {
    let mut state = ZeldaState::new();
    state.zelda_configure_audio(48_000, 77, true, Some("msu/".to_string()));
    state.zelda_enable_msu(1);

    assert_eq!(state.audio.config_audio_freq, 48_000);
    assert_eq!(state.audio.config_msuvolume, 77);
    assert!(state.audio.config_resume_msu);
    assert_eq!(state.audio.config_msu_path.as_deref(), Some("msu/"));
    let expected = 255.0 * 77.0 * (1.0 / 255.0 / 100.0);
    assert!((state.audio.volume_transition_target_float[3] - expected).abs() < 0.00001);
}

#[test]
fn msu_open_and_mix_pcm_matches_c_streaming_path() {
    let dir = unique_temp_msu_dir();
    let path = dir.join("5.pcm");
    let mut bytes = msu1_header(0);
    for sample in [1000i16, -1000, 2000, -2000] {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(&path, bytes).unwrap();

    let mut state = ZeldaState::new();
    state.zelda_configure_audio(
        44_100,
        100,
        false,
        Some(format!("{}/", dir.to_string_lossy())),
    );
    state.zelda_enable_msu(1);
    state.msu_player_open(5, false);

    assert!(state.audio.msu_player.has_file);
    assert_eq!(state.audio.msu_player.state, MSU_STATE_PLAYING);
    assert_eq!(state.audio.msu_player.total_samples_in_file, 2);
    assert_eq!(state.audio.msu_player.samples_until_repeat, 2);

    let mut audio = [0i16; 4];
    state.msu_player_mix(&mut audio, 2);

    assert_eq!(audio, [1000, -1000, 2000, -2000]);
    assert_eq!(state.audio.msu_player.resume_info.offset, 0);
    assert_eq!(state.audio.msu_player.cur_file_offs, 2);

    let _ = fs::remove_file(path);
    let _ = fs::remove_dir(dir);
}

#[test]
fn msu_open_opuz_keeps_file_and_resume_state_until_mix_like_c() {
    let dir = unique_temp_msu_dir();
    let path = dir.join("5.opuz");
    fs::write(&path, opuz_header(0)).unwrap();

    let mut state = ZeldaState::new();
    state.zelda_configure_audio(
        48_000,
        100,
        false,
        Some(format!("{}/", dir.to_string_lossy())),
    );
    state.zelda_enable_msu(MSU_FEATURE_OPUZ);
    state.msu_player_open(5, false);

    assert!(state.audio.msu_player.has_file);
    assert!(state.audio.msu_player.has_opus);
    assert_eq!(state.audio.msu_player.state, MSU_STATE_PLAYING);
    assert_eq!(state.audio.msu_player.resume_info.orig_track, 5);
    assert_eq!(state.audio.msu_player.resume_info.actual_track, 5);
    assert_eq!(state.audio.msu_player.range_cur, 8);

    let _ = fs::remove_file(path);
    let _ = fs::remove_dir(dir);
}

#[test]
fn opuz_packet_range_header_updates_resume_cursors_before_decoder_boundary() {
    let dir = unique_temp_msu_dir();
    let path = dir.join("5.opuz");
    let mut encoder = opus::Encoder::new(48_000, Channels::Stereo, opus::Application::Audio)
        .expect("create opus encoder");
    let pcm = [0i16; 960 * 2];
    let mut packet = [0u8; 1275];
    let packet_len = encoder
        .encode(&pcm, &mut packet)
        .expect("encode opus packet");
    let mut bytes = opuz_header(0);
    bytes.extend_from_slice(&18u32.to_le_bytes());
    bytes.extend_from_slice(&960u32.to_le_bytes());
    bytes.extend_from_slice(&0x4003u16.to_le_bytes());
    bytes.extend_from_slice(&(packet_len as u16).to_le_bytes());
    bytes.extend_from_slice(&packet[..packet_len]);
    fs::write(&path, bytes).unwrap();

    let mut state = ZeldaState::new();
    state.zelda_configure_audio(
        48_000,
        100,
        false,
        Some(format!("{}/", dir.to_string_lossy())),
    );
    state.zelda_enable_msu(MSU_FEATURE_OPUZ);
    state.msu_player_open(5, false);

    assert_eq!(
        ZeldaState::msu_player_prepare_opuz_packet(&mut state.audio.msu_player),
        OpuzPacketStatus::Decoded(960)
    );
    assert_eq!(state.audio.msu_player.samples_until_repeat, 960);
    assert_eq!(state.audio.msu_player.preskip, 3);
    assert_eq!(state.audio.msu_player.range_repeat, 8);
    assert_eq!(state.audio.msu_player.range_cur, 18);
    assert_eq!(state.audio.msu_player.resume_info.range_repeat, 8);
    assert_eq!(state.audio.msu_player.resume_info.range_cur, 18);
    assert_eq!(state.audio.msu_player.resume_info.offset, 18);
    assert_eq!(state.audio.msu_player.resume_info.samples_until_repeat, 963);
    assert_eq!(state.audio.msu_player.resume_info.initial_packet_bytes, {
        let mut initial = [0; 8];
        let initial_len = (2 + packet_len).min(initial.len());
        let mut encoded = Vec::with_capacity(2 + packet_len);
        encoded.extend_from_slice(&(packet_len as u16).to_le_bytes());
        encoded.extend_from_slice(&packet[..packet_len]);
        initial[..initial_len].copy_from_slice(&encoded[..initial_len]);
        u64::from_le_bytes(initial)
    });
    assert_eq!(state.audio.msu_player.cur_file_offs, 20 + packet_len as u32);

    let _ = fs::remove_file(path);
    let _ = fs::remove_dir(dir);
}

#[test]
fn msu_pcm_non_repeating_track_finishes_like_c() {
    let dir = unique_temp_msu_dir();
    let path = dir.join("1.pcm");
    let mut bytes = msu1_header(0);
    for sample in [321i16, -321, 654, -654] {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(&path, bytes).unwrap();

    let mut state = ZeldaState::new();
    state.zelda_configure_audio(
        44_100,
        100,
        false,
        Some(format!("{}/", dir.to_string_lossy())),
    );
    state.zelda_enable_msu(1);
    state.msu_player_open(1, false);

    let mut audio = [0i16; 8];
    state.msu_player_mix(&mut audio, 4);

    assert_eq!(&audio[..4], &[321, -321, 654, -654]);
    assert_eq!(state.audio.msu_player.state, MSU_STATE_FINISHED_PLAYING);
    assert!(!state.audio.msu_player.has_file);

    let _ = fs::remove_file(path);
    let _ = fs::remove_dir(dir);
}
