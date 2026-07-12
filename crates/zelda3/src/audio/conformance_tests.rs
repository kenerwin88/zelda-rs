use super::*;
use crate::game_output::{
    AudioBackendMode, AudioEventFrame, AudioEventKind, AUDIO_INTERNAL_SAMPLES_PER_FRAME,
};

const CHANNELS: usize = 2;

#[derive(Clone, Copy, Debug)]
enum ScenarioStep {
    EngineCommand(EngineCommand),
    RenderFrames(usize),
}

#[derive(Clone, Copy, Debug)]
enum EngineCommand {
    Music(u8),
    StopMusic,
    Sfx { bank: SfxBank, id: u8 },
    SimultaneousSfx([(SfxBank, u8); 2]),
}

#[derive(Clone, Copy, Debug)]
enum SfxBank {
    Menu,
    World,
}

impl SfxBank {
    const fn catalog_id(self) -> u8 {
        match self {
            Self::Menu => 0,
            Self::World => 1,
        }
    }
}

impl EngineCommand {
    fn ports(self) -> [u8; 4] {
        let mut ports = [0; 4];
        match self {
            Self::Music(track) => ports[0] = track,
            Self::StopMusic => ports[0] = 0xf0,
            Self::Sfx { bank, id } => ports[usize::from(bank.catalog_id()) + 1] = id,
            Self::SimultaneousSfx(commands) => {
                for (bank, id) in commands {
                    ports[usize::from(bank.catalog_id()) + 1] = id;
                }
            }
        }
        ports
    }
}

#[derive(Clone, Copy, Debug)]
struct SemanticAudioScenario {
    name: &'static str,
    steps: &'static [ScenarioStep],
}

#[derive(Debug)]
struct ScenarioResult {
    frames: Vec<AudioEventFrame>,
    pcm: Vec<i16>,
}

fn run_scenario(scenario: SemanticAudioScenario) -> ScenarioResult {
    let mut state = ZeldaState::new();
    let mut frames = Vec::new();
    let mut pcm = Vec::new();
    let mut pending_command = None;

    for step in scenario.steps {
        match *step {
            ScenarioStep::EngineCommand(command) => {
                for (port, value) in command.ports().into_iter().enumerate() {
                    state.zelda_apu_write(0x2140 + port as u32, value);
                }
                state.zelda_push_apu_state();
                pending_command = Some(command);
            }
            ScenarioStep::RenderFrames(count) => {
                for _ in 0..count {
                    let (frame, frame_pcm) = render_modern_frame(&mut state);
                    assert_render_invariants(scenario.name, &state);
                    if let Some(command) = pending_command.take() {
                        assert_command_was_emitted(
                            scenario.name,
                            std::slice::from_ref(&frame),
                            command,
                        );
                        if matches!(command, EngineCommand::SimultaneousSfx(_)) {
                            assert!(
                                state.zelda_modern_audio_last_stats().triggered_voices >= 2,
                                "{} did not key on both overlapping effects: {:?}",
                                scenario.name,
                                frame.events
                            );
                        }
                    }
                    frames.push(frame);
                    pcm.extend(frame_pcm);
                }
            }
        }
    }
    assert!(
        pending_command.is_none(),
        "{} ended without rendering its final command",
        scenario.name
    );

    ScenarioResult { frames, pcm }
}

fn render_modern_frame(state: &mut ZeldaState) -> (AudioEventFrame, Vec<i16>) {
    let mut pcm = vec![0; AUDIO_INTERNAL_SAMPLES_PER_FRAME * CHANNELS];
    let frame = state.zelda_render_audio_with_backend(
        AudioBackendMode::Modern,
        &mut pcm,
        AUDIO_INTERNAL_SAMPLES_PER_FRAME as i32,
        CHANNELS as i32,
    );
    (frame, pcm)
}

fn assert_render_invariants(name: &str, state: &ZeldaState) {
    let sequence = state.zelda_modern_audio_sequence_last_stats();
    assert_eq!(
        sequence.unknown_sfx_commands, 0,
        "{name} used an unknown SFX program: {:?}",
        sequence.unknown_sfx_programs
    );
    assert_eq!(
        sequence.fallback_sfx_commands, 0,
        "{name} entered the heuristic SFX fallback"
    );
    assert_eq!(
        state.zelda_modern_audio_last_stats().ignored_events,
        0,
        "{name} emitted an event the modern renderer ignored"
    );
}

fn assert_command_was_emitted(name: &str, frames: &[AudioEventFrame], command: EngineCommand) {
    let emitted = |predicate: &dyn Fn(&AudioEventKind) -> bool| {
        frames
            .iter()
            .flat_map(|frame| &frame.events)
            .any(|event| predicate(&event.kind))
    };
    match command {
        EngineCommand::Music(track) => assert!(
            emitted(
                &|kind| matches!(kind, AudioEventKind::PlayMusic { track: actual } if *actual == track)
            ),
            "{name} never emitted PlayMusic({track:#04x})"
        ),
        EngineCommand::StopMusic => assert!(
            emitted(&|kind| matches!(kind, AudioEventKind::StopMusic)),
            "{name} never emitted StopMusic"
        ),
        EngineCommand::Sfx { bank, id } => {
            assert_sfx_was_emitted(name, frames, bank.catalog_id(), id)
        }
        EngineCommand::SimultaneousSfx(commands) => {
            for (bank, id) in commands {
                assert_sfx_was_emitted(name, frames, bank.catalog_id(), id);
            }
        }
    }
}

fn assert_sfx_was_emitted(name: &str, frames: &[AudioEventFrame], bank: u8, id: u8) {
    assert!(
        frames.iter().flat_map(|frame| &frame.events).any(|event| {
            matches!(event.kind, AudioEventKind::PlaySfx { bank: actual_bank, id: actual_id }
                if actual_bank == bank && actual_id == id)
        }),
        "{name} never emitted PlaySfx({bank}, {id:#04x})"
    );
}

const MENU_CURSOR: SemanticAudioScenario = SemanticAudioScenario {
    name: "menu_cursor",
    steps: &[
        ScenarioStep::EngineCommand(EngineCommand::Sfx {
            bank: SfxBank::Menu,
            id: 0x34,
        }),
        ScenarioStep::RenderFrames(3),
    ],
};

const OVERLAPPING_EFFECTS: SemanticAudioScenario = SemanticAudioScenario {
    name: "overlapping_menu_and_world_effects",
    steps: &[
        ScenarioStep::EngineCommand(EngineCommand::SimultaneousSfx([
            (SfxBank::Menu, 0x34),
            (SfxBank::World, 0x88),
        ])),
        ScenarioStep::RenderFrames(3),
    ],
};

const MUSIC_INTERRUPTION: SemanticAudioScenario = SemanticAudioScenario {
    name: "music_start_then_stop",
    steps: &[
        ScenarioStep::EngineCommand(EngineCommand::Music(0x01)),
        ScenarioStep::RenderFrames(80),
        ScenarioStep::EngineCommand(EngineCommand::StopMusic),
        ScenarioStep::RenderFrames(2),
    ],
};

#[test]
fn semantic_audio_matrix_is_engine_driven_and_uses_catalog_only() {
    for scenario in [MENU_CURSOR, OVERLAPPING_EFFECTS, MUSIC_INTERRUPTION] {
        let result = run_scenario(scenario);
        assert!(
            !result.frames.is_empty(),
            "{} rendered no frames",
            scenario.name
        );
        assert!(
            result.pcm.iter().any(|sample| *sample != 0),
            "{} rendered only silence",
            scenario.name
        );
        if scenario.name == MUSIC_INTERRUPTION.name {
            assert_music_was_actively_interrupted(&result.frames);
        }
    }
}

fn assert_music_was_actively_interrupted(frames: &[AudioEventFrame]) {
    let has_event = |frame: &AudioEventFrame, predicate: &dyn Fn(&AudioEventKind) -> bool| {
        frame.events.iter().any(|event| predicate(&event.kind))
    };
    let play = frames
        .iter()
        .position(|frame| {
            has_event(frame, &|kind| {
                matches!(kind, AudioEventKind::PlayMusic { .. })
            })
        })
        .unwrap();
    let note = frames
        .iter()
        .position(|frame| has_event(frame, &|kind| matches!(kind, AudioEventKind::NoteOn { .. })))
        .expect("music interruption never reached an active note");
    let stop = frames
        .iter()
        .position(|frame| has_event(frame, &|kind| matches!(kind, AudioEventKind::StopMusic)))
        .unwrap();
    assert!(play <= note && note < stop);
    assert!(!frames[stop + 1..]
        .iter()
        .any(|frame| has_event(frame, &|kind| matches!(kind, AudioEventKind::NoteOn { .. }))));
}

#[test]
fn semantic_snapshot_continuation_is_sample_exact() {
    std::thread::Builder::new()
        .name("semantic-audio-conformance-snapshot".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(semantic_snapshot_continuation_is_sample_exact_inner)
        .unwrap()
        .join()
        .unwrap();
}

fn semantic_snapshot_continuation_is_sample_exact_inner() {
    let mut uninterrupted = ZeldaState::new();
    uninterrupted.zelda_apu_write(0x2141, 0x34);
    uninterrupted.zelda_push_apu_state();
    let (_, lead_in) = render_modern_frame(&mut uninterrupted);
    assert_render_invariants("snapshot lead-in", &uninterrupted);
    assert!(lead_in.iter().any(|sample| *sample != 0));
    assert!(uninterrupted.zelda_modern_audio_last_stats().active_voices > 0);

    let snapshot = uninterrupted.zelda_audio_snapshot_bytes();
    let mut resumed = ZeldaState::new();
    resumed.zelda_audio_restore_from_bytes(&snapshot).unwrap();
    let mut continuation_was_audible = false;
    for continuation_frame in 0..4 {
        let (expected_events, expected) = render_modern_frame(&mut uninterrupted);
        let (actual_events, actual) = render_modern_frame(&mut resumed);
        assert_render_invariants("uninterrupted snapshot continuation", &uninterrupted);
        assert_render_invariants("resumed snapshot continuation", &resumed);
        continuation_was_audible |= expected.iter().any(|sample| *sample != 0);
        assert_eq!(
            actual_events, expected_events,
            "event drift at continuation frame {continuation_frame}"
        );
        assert_eq!(
            actual, expected,
            "PCM drift at continuation frame {continuation_frame}"
        );
    }
    assert!(
        continuation_was_audible,
        "snapshot continuation compared only silence"
    );
}

#[test]
fn ordinary_music_restore_clears_modern_scenario_state() {
    let mut state = ZeldaState::new();
    state.zelda_apu_write(0x2141, 0x34);
    state.zelda_push_apu_state();
    let _ = render_modern_frame(&mut state);

    state.zelda_restore_music_after_load_locked(false);

    assert_eq!(
        state.zelda_modern_audio_state(),
        (
            crate::modern_audio_sequence::ModernAudioSequencer::default(),
            crate::modern_audio::ModernAudioEngine::default(),
        )
    );
    let (post_restore_events, post_restore_pcm) = render_modern_frame(&mut state);
    assert_render_invariants("ordinary music restore", &state);
    assert!(!post_restore_events.events.iter().any(|event| matches!(
        event.kind,
        AudioEventKind::PlaySfx { .. } | AudioEventKind::NoteOn { .. }
    )));
    assert_eq!(state.zelda_modern_audio_last_stats().triggered_voices, 0);
    assert!(post_restore_pcm.iter().all(|sample| *sample == 0));
}
