use super::*;

fn distinctive_brr_bank(nibble: u8) -> Vec<u8> {
    let mut ram = vec![0u8; 0x10000];
    for source in 0..=u8::MAX {
        let address = 0x5000usize + usize::from(source) * 9;
        let entry = 0x3c00usize + usize::from(source) * 4;
        ram[entry..entry + 2].copy_from_slice(&(address as u16).to_le_bytes());
        ram[entry + 2..entry + 4].copy_from_slice(&(address as u16).to_le_bytes());
        ram[address] = 0x81;
        ram[address + 1..address + 9].fill(nibble);
    }
    ram
}

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
fn modern_default_entry_point_preserves_msu_pcm_mixing() {
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
    let mut audio = [0i16; 4];

    state.zelda_render_audio(&mut audio, 2, 2);

    assert_eq!(audio, [1000, -1000, 2000, -2000]);

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

#[test]
fn audio_route_state_exposes_queued_apu_ports_without_rendering() {
    let mut state = ZeldaState::new();
    state.zelda_apu_write(0x2140, 0x12);
    state.zelda_apu_write(0x2141, 0x34);
    state.zelda_push_apu_state();

    let route = state.zelda_audio_route_state();

    assert_eq!(route.queue.pos, 1);
    assert_eq!(route.queue.count, 1);
    assert_eq!(route.queue.total, 1);
    assert_eq!(route.queue.write, [0x12, 0x34, 0, 0]);
    assert_eq!(route.queue.pending, [0x12, 0x34, 0, 0]);
    assert_eq!(route.queue.input, [0, 0, 0, 0]);
    assert!(route.spc.is_none());
}

#[test]
fn game_frame_output_collects_runtime_render_and_audio_facts() {
    let mut state = ZeldaState::new();
    state.zelda_apu_write(0x2142, 0x56);
    state.zelda_push_apu_state();
    state.ppu.mode = 7;
    state.ppu.forced_blank = true;
    state.ppu.brightness = 0x0f;
    state.ppu.screen_enabled = [0x11, 0x04];

    let output = state.zelda_game_frame_output();

    assert_eq!(output.render.mode, 7);
    assert!(output.render.forced_blank);
    assert_eq!(output.render.brightness, 0x0f);
    assert_eq!(output.render.screen_enabled, [0x11, 0x04]);
    assert_eq!(output.audio.queue.pending, [0, 0, 0x56, 0]);
    assert!(output.audio.events.len() >= 2);
}

#[test]
fn modern_backend_renders_from_typed_events() {
    let mut state = ZeldaState::new();
    state.zelda_apu_write(0x2141, 0x88);
    state.zelda_push_apu_state();
    let mut audio = [123i16; 8];

    let frame = state.zelda_render_audio(&mut audio, 4, 2);
    let stats = state.zelda_modern_audio_last_stats();
    let sequence = state.zelda_modern_audio_sequence_last_stats();

    assert!(
        audio.iter().any(|sample| *sample != 0),
        "frame={frame:#?} stats={stats:#?} sequence={sequence:#?}"
    );
    assert_eq!(frame.queue.input, [0, 0x88, 0, 0]);
    assert_eq!(frame.unresolved_dsp_writes, 0);
    assert_eq!(stats.triggered_voices, 1);
    assert_eq!(stats.ignored_events, 0);
    assert_eq!(stats.samples_per_channel, 4);
    assert_eq!(stats.channels, 2);
    assert_ne!(stats.checksum, 0);
    assert!(frame.events.iter().any(|event| matches!(
        event.kind,
        crate::game_output::AudioEventKind::PlaySfx { id: 0x88, .. }
    )));
    assert!(frame.events.iter().any(|event| matches!(
        event.kind,
        crate::game_output::AudioEventKind::NoteOn { voice: 1, .. }
    )));
    assert_eq!(sequence.sfx_commands, 1);
    assert_eq!(sequence.note_events, 1);
}

#[test]
fn modern_only_audio_state_does_not_embed_an_spc_address_space() {
    assert!(
        std::mem::size_of::<super::AudioState>() < 32 * 1024,
        "modern-only AudioState should contain typed bridge state, not 64 KiB of SPC RAM"
    );
}

#[test]
fn modern_only_audio_snapshot_is_compact() {
    let mut state = ZeldaState::new();
    state.zelda_set_spc_startup_phase(72, 0);
    state.zelda_apu_write(0x2140, 0x34);
    state.zelda_apu_write(0x2141, 0x56);
    state.zelda_save_music_state_to_ram_locked();
    let snapshot = state.zelda_audio_snapshot_bytes();
    assert!(
        snapshot.len() < 32 * 1024,
        "modern-only audio snapshot unexpectedly contains an SPC RAM image: {} bytes",
        snapshot.len()
    );
    let mut restored = ZeldaState::new();
    restored.zelda_audio_restore_from_bytes(&snapshot).unwrap();
    let compat = restored.zelda_modern_audio_compat_ram();
    assert_eq!(compat[0x43], 72);
    assert_eq!(&compat[0x410..0x414], &[0x34, 0x56, 0, 0]);
}

#[test]
fn modern_audio_consumes_typed_engine_commands_instead_of_apui_bytes() {
    use crate::game_output::{AudioPan, AudioSfxBank, EngineAudioCommand};

    let mut state = ZeldaState::new();
    state.zelda_emit_audio_command(EngineAudioCommand::PlaySfx {
        bank: AudioSfxBank::Effect1,
        effect: 0x0a,
        pan: AudioPan::Right,
    });
    state.zelda_push_apu_state();
    let mut audio = [0i16; 32];

    let frame = state.zelda_render_audio(&mut audio, 16, 2);

    assert!(frame.events.iter().any(|event| {
        matches!(
            event.kind,
            crate::game_output::AudioEventKind::PlaySfx { bank: 1, id: 0x4a }
        )
    }));
}

#[test]
fn gameplay_sound_latch_becomes_a_typed_nmi_command() {
    let mut state = ZeldaState::new();
    state.set_sound_effect_1(0x4a);
    state.interrupt_nmi_audio_parts_locked();
    state.zelda_push_apu_state();
    let mut audio = [0i16; 32];

    let frame = state.zelda_render_audio(&mut audio, 16, 2);

    assert!(frame.events.iter().any(|event| {
        matches!(
            event.kind,
            crate::game_output::AudioEventKind::PlaySfx { bank: 1, id: 0x4a }
        )
    }));
}

#[test]
fn audio_nmi_generation_publishes_then_consumes_after_interruptible_caller_returns() {
    let mut state = ZeldaState::new();
    state.set_ambient_sound_effect(1);
    state.set_sound_effect_2(12);
    state.next_audio_nmi_generation = AudioNmiGeneration::PreviouslyPublishedPorts;

    state.interrupt_nmi_audio_parts_for_generation();
    state.zelda_push_apu_state();
    let mut retained_audio = [0i16; 32];
    let retained_frame = state.zelda_render_audio(&mut retained_audio, 16, 2);

    assert_eq!(state.game_state.system_signals.ambient_sound_effect(), 1);
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0);
    assert!(!retained_frame.events.iter().any(|event| {
        matches!(
            event.kind,
            crate::game_output::AudioEventKind::PlaySfx { bank: 0, id: 1 }
        )
    }));

    state.interrupt_nmi_audio_parts_for_generation();
    state.zelda_push_apu_state();
    let mut published_audio = [0i16; 32];
    let published_frame = state.zelda_render_audio(&mut published_audio, 16, 2);

    assert_eq!(state.game_state.system_signals.ambient_sound_effect(), 1);
    assert!(published_frame.events.iter().any(|event| {
        matches!(
            event.kind,
            crate::game_output::AudioEventKind::PlaySfx { bank: 0, id: 1 }
        )
    }));

    state.interrupt_nmi_audio_parts_for_generation();
    state.zelda_push_apu_state();
    let mut consumed_audio = [0i16; 32];
    state.zelda_render_audio(&mut consumed_audio, 16, 2);

    assert_eq!(state.game_state.system_signals.ambient_sound_effect(), 0);
    assert_eq!(state.zelda_debug_apu_write_ports()[1], 1);
}

#[test]
fn rom_startup_audio_stays_silent_until_the_boot_chime_keyon() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    let mut audio = vec![0i16; 534 * 2];
    let mut first_nonzero = None;

    for frame in 0..=90 {
        state.zelda_run_frame(0);
        state.zelda_render_audio(&mut audio, 534, 2);
        if first_nonzero.is_none() {
            first_nonzero = audio
                .chunks_exact(2)
                .position(|sample| sample != [0, 0])
                .map(|offset| (frame, offset));
        }
    }
    assert_eq!(first_nonzero, Some((84, 362)));
}

#[test]
fn typed_engine_command_latches_preserve_last_write_wins() {
    use crate::game_output::{AudioPan, AudioSfxBank, EngineAudioCommand};

    let mut state = ZeldaState::new();
    state.zelda_emit_audio_command(EngineAudioCommand::PlaySfx {
        bank: AudioSfxBank::Effect2,
        effect: 0x0a,
        pan: AudioPan::Center,
    });
    state.zelda_emit_audio_command(EngineAudioCommand::PlaySfx {
        bank: AudioSfxBank::Effect2,
        effect: 0x0b,
        pan: AudioPan::Left,
    });
    state.zelda_push_apu_state();
    let mut audio = [0i16; 32];

    let frame = state.zelda_render_audio(&mut audio, 16, 2);
    let commands = frame
        .events
        .iter()
        .filter_map(|event| match event.kind {
            crate::game_output::AudioEventKind::PlaySfx { bank: 2, id } => Some(id),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(commands, vec![0x8b]);
}

#[test]
fn modern_runtime_acknowledges_typed_commands_without_apui_reads() {
    use crate::game_output::{AudioPan, AudioSfxBank, EngineAudioCommand};

    let command = EngineAudioCommand::PlaySfx {
        bank: AudioSfxBank::Ambient,
        effect: 0x12,
        pan: AudioPan::Left,
    };
    let mut state = ZeldaState::new();
    state.zelda_emit_audio_command(command);
    state.zelda_push_apu_state();
    let mut audio = [0i16; 32];

    state.zelda_render_audio(&mut audio, 16, 2);

    assert!(state.zelda_audio_command_acknowledged(command));
    assert!(
        !state.zelda_audio_command_acknowledged(EngineAudioCommand::ClearSfx {
            bank: AudioSfxBank::Ambient,
        })
    );
}

#[test]
fn production_audio_state_has_no_modern_apui_transport() {
    let source = include_str!("audio.rs");

    assert!(
        !source.contains("ModernApuState"),
        "production audio still owns an APUI-shaped modern state"
    );
    assert!(
        !source.contains("rehydrate_commands_from_ports"),
        "production audio still rebuilds typed commands from APUI bytes"
    );
}

#[test]
fn queued_typed_engine_commands_survive_audio_snapshot_restore() {
    use crate::game_output::{AudioPan, AudioSfxBank, EngineAudioCommand};

    let mut state = ZeldaState::new();
    state.zelda_emit_audio_command(EngineAudioCommand::PlaySfx {
        bank: AudioSfxBank::Ambient,
        effect: 0x12,
        pan: AudioPan::Center,
    });
    state.zelda_push_apu_state();
    let snapshot = state.zelda_audio_snapshot_bytes();
    let mut restored = ZeldaState::new();
    restored.zelda_audio_restore_from_bytes(&snapshot).unwrap();
    let mut audio = [0i16; 32];

    let frame = restored.zelda_render_audio(&mut audio, 16, 2);

    assert!(frame.events.iter().any(|event| {
        matches!(
            event.kind,
            crate::game_output::AudioEventKind::PlaySfx { bank: 0, id: 0x12 }
        )
    }));
}

#[test]
fn modern_backend_uses_owned_brr_bank_and_ignores_legacy_ram_replacement() {
    fn render(state: &mut ZeldaState, frame: &crate::game_output::AudioEventFrame) -> Vec<i16> {
        let mut audio = vec![0i16; 735 * 2];
        state
            .audio
            .modern
            .renderer
            .render_frame(frame, &mut audio, 735, 2);
        audio
    }

    let mut frame = crate::game_output::AudioEventFrame::from_route_and_dsp_writes(
        crate::game_output::AudioRouteState::default(),
        &[],
    );
    frame.events.push(crate::game_output::AudioEvent {
        sample_offset: 0,
        timer_cycles: 0,
        kind: crate::game_output::AudioEventKind::SetPitchWord {
            voice: 0,
            pitch_word: 0x1000,
        },
        parity_dsp: None,
    });
    frame.events.push(crate::game_output::AudioEvent {
        sample_offset: 4,
        timer_cycles: 0,
        kind: crate::game_output::AudioEventKind::NoteOn {
            voice: 0,
            pitch: 60,
            instrument: 3,
            volume: 127,
        },
        parity_dsp: None,
    });
    let bank_a = distinctive_brr_bank(0x11);
    let bank_b = distinctive_brr_bank(0x77);
    let mut replaced = ZeldaState::new();
    replaced.load_audio_apu_ram_c_saveload(&bank_a);
    let first_a = render(&mut replaced, &frame);
    let mut unchanged = replaced.clone();

    replaced.load_audio_apu_ram_c_saveload(&bank_b);
    unchanged.load_audio_apu_ram_c_saveload(&bank_a);
    let after_b = render(&mut replaced, &frame);
    let after_a = render(&mut unchanged, &frame);

    assert!(first_a.iter().any(|sample| *sample != 0));
    assert_eq!(after_a, after_b);
}

#[test]
fn normal_audio_entry_point_uses_selected_modern_backend() {
    let mut normal = ZeldaState::new();
    normal.zelda_apu_write(0x2141, 0x88);
    normal.zelda_push_apu_state();
    let mut explicit = normal.clone();
    let mut normal_audio = [0i16; 16];
    let mut explicit_audio = [0i16; 16];

    normal.zelda_render_audio(&mut normal_audio, 8, 2);
    explicit.zelda_render_audio(&mut explicit_audio, 8, 2);

    assert_eq!(normal_audio, explicit_audio);
    assert!(normal_audio.iter().any(|sample| *sample != 0));
}

#[test]
fn modern_backend_acknowledges_apui_commands_without_the_legacy_interpreter() {
    let mut state = ZeldaState::new();
    state.zelda_apu_write(0x2141, 0x88);
    state.zelda_push_apu_state();
    let mut audio = [0i16; 16];

    state.zelda_render_audio(&mut audio, 8, 2);

    assert_eq!(state.zelda_apu_read(0x2141), 0x88);
}

#[test]
fn ordinary_music_restore_discards_future_modern_audio_state() {
    let mut state = ZeldaState::new();
    state.zelda_apu_write(0x2141, 0x88);
    state.zelda_push_apu_state();
    let mut audio = [0i16; 32];
    state.zelda_render_audio(&mut audio, 16, 2);
    assert_ne!(
        state.zelda_modern_audio_state(),
        (
            crate::modern_audio_sequence::ModernAudioSequencer::default(),
            crate::modern_audio::ModernAudioEngine::default(),
        )
    );

    state.zelda_restore_music_after_load_locked(false);

    assert_eq!(
        state.zelda_modern_audio_state(),
        (
            crate::modern_audio_sequence::ModernAudioSequencer::default(),
            crate::modern_audio::ModernAudioEngine::default(),
        )
    );
}

#[test]
fn zero_sample_modern_callback_defers_command_until_audio_can_advance() {
    let mut deferred = ZeldaState::new();
    deferred.zelda_apu_write(0x2141, 0x88);
    deferred.zelda_push_apu_state();
    let mut direct = deferred.clone();
    let mut empty = [];

    deferred.zelda_render_audio(&mut empty, 0, 2);

    let mut deferred_audio = [0i16; 16];
    let mut direct_audio = [0i16; 16];
    deferred.zelda_render_audio(&mut deferred_audio, 8, 2);
    direct.zelda_render_audio(&mut direct_audio, 8, 2);

    assert_eq!(deferred_audio, direct_audio);
    assert_eq!(
        deferred.zelda_modern_audio_state(),
        direct.zelda_modern_audio_state()
    );
    assert_eq!(
        deferred.zelda_apu_read(0x2141),
        direct.zelda_apu_read(0x2141)
    );
}

#[test]
fn audio_snapshot_has_versioned_header_and_accepts_preheader_payload() {
    std::thread::Builder::new()
        .name("versioned-audio-snapshot".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(audio_snapshot_has_versioned_header_and_accepts_preheader_payload_inner)
        .unwrap()
        .join()
        .unwrap();
}

fn audio_snapshot_has_versioned_header_and_accepts_preheader_payload_inner() {
    use crate::game_output::{AudioPan, AudioSfxBank};

    fn with_header(version: u16, flags: u16, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(AUDIO_SNAPSHOT_HEADER_BYTES + payload.len());
        bytes.extend_from_slice(&AUDIO_SNAPSHOT_MAGIC);
        bytes.extend_from_slice(&version.to_le_bytes());
        bytes.extend_from_slice(&flags.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }

    let mut state = ZeldaState::new();
    state.select_modern_sample_bank(2);
    let snapshot = state.zelda_audio_snapshot_bytes();
    assert_eq!(&snapshot[..4], b"Z3AU");
    assert_eq!(
        u16::from_le_bytes([snapshot[4], snapshot[5]]),
        AUDIO_SNAPSHOT_VERSION
    );
    let expected_flags = 0;
    assert_eq!(
        u16::from_le_bytes([snapshot[6], snapshot[7]]),
        expected_flags
    );
    assert_eq!(
        u32::from_le_bytes(snapshot[8..12].try_into().unwrap()) as usize,
        snapshot.len() - 12
    );

    let mut restored = ZeldaState::new();
    restored.zelda_audio_restore_from_bytes(&snapshot).unwrap();
    assert_eq!(
        restored.zelda_modern_audio_state(),
        state.zelda_modern_audio_state()
    );

    let v2_payload = snapshot_state::encode_v2_for_test(&state.audio);
    let v2 = with_header(2, 0, &v2_payload);
    restored
        .zelda_audio_restore_from_bytes(&v2)
        .expect("version-2 modern snapshot migrates");
    restored
        .zelda_audio_restore_from_bytes(&v2_payload)
        .expect("preheader modern snapshot remains readable");

    let (v4_payload, v4_has_sidecar) = snapshot_state::encode_v4_for_test(&state.audio);
    let v4_flags = if v4_has_sidecar {
        AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR
    } else {
        0
    };
    restored
        .zelda_audio_restore_from_bytes(&with_header(4, v4_flags, &v4_payload))
        .expect("version-4 sample-bank snapshot migrates");

    state.zelda_emit_audio_command(EngineAudioCommand::PlaySfx {
        bank: AudioSfxBank::Ambient,
        effect: 0x12,
        pan: AudioPan::Left,
    });
    state.zelda_push_apu_state();
    let (v5_payload, v5_has_sidecar) = snapshot_state::encode_v5_for_test(&state.audio);
    let v5_flags = if v5_has_sidecar {
        AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR
    } else {
        0
    };
    restored
        .zelda_audio_restore_from_bytes(&with_header(5, v5_flags, &v5_payload))
        .expect("version-5 APUI transport snapshot migrates into typed runtime state");
    assert_eq!(
        restored.zelda_modern_audio_state(),
        state.zelda_modern_audio_state()
    );

    let modern_v3_payload = snapshot_state::encode_v3_without_sidecar_for_test(&state.audio);
    restored
        .zelda_audio_restore_from_bytes(&with_header(3, 0, &modern_v3_payload))
        .expect("portable modern-only v3 snapshot migrates");

    {
        let portable_payload =
            snapshot_state::encode_v3_with_opaque_sidecar_for_test(&state.audio, vec![1, 2, 3, 4]);
        restored
            .zelda_audio_restore_from_bytes(&with_header(
                3,
                AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR,
                &portable_payload,
            ))
            .expect("modern build ignores opaque oracle sidecar");
    }

    let truncated = &snapshot[..snapshot.len() - 1];
    assert!(restored
        .zelda_audio_restore_from_bytes(truncated)
        .unwrap_err()
        .contains("length mismatch"));
    let mut appended = snapshot.clone();
    appended.push(0);
    assert!(restored
        .zelda_audio_restore_from_bytes(&appended)
        .unwrap_err()
        .contains("length mismatch"));

    let mut unsupported = snapshot.clone();
    let unsupported_version = AUDIO_SNAPSHOT_VERSION + 1;
    unsupported[4..6].copy_from_slice(&unsupported_version.to_le_bytes());
    assert_eq!(
        restored.zelda_audio_restore_from_bytes(&unsupported),
        Err(format!(
            "unsupported audio snapshot version {unsupported_version}"
        ))
    );

    let mut invalid_bank = snapshot.clone();
    *invalid_bank.last_mut().unwrap() = 0xff;
    assert!(restored
        .zelda_audio_restore_from_bytes(&invalid_bank)
        .unwrap_err()
        .contains("unknown sample bank"));

    let mut mismatched_flag = snapshot;
    let mismatched_flags = expected_flags ^ AUDIO_SNAPSHOT_FLAG_ORACLE_SIDECAR;
    mismatched_flag[6..8].copy_from_slice(&mismatched_flags.to_le_bytes());
    assert_eq!(
        restored.zelda_audio_restore_from_bytes(&mismatched_flag),
        Err("audio snapshot oracle sidecar flag mismatch".to_string())
    );
}
