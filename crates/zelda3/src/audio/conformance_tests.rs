use super::*;
use crate::game_output::{
    AudioEventFrame, AudioEventKind, AudioSfxBank, EngineAudioCommand,
    AUDIO_INTERNAL_SAMPLES_PER_FRAME,
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

    const fn engine_bank(self) -> AudioSfxBank {
        match self {
            Self::Menu => AudioSfxBank::Ambient,
            Self::World => AudioSfxBank::Effect1,
        }
    }
}

impl EngineCommand {
    fn emit(self, state: &mut ZeldaState) {
        match self {
            Self::Music(track) => {
                state.zelda_emit_audio_command(EngineAudioCommand::PlayMusic { track })
            }
            Self::StopMusic => state.zelda_emit_audio_command(EngineAudioCommand::StopMusic),
            Self::Sfx { bank, id } => state.zelda_emit_audio_command(
                EngineAudioCommand::from_sfx_port_value(bank.engine_bank(), id),
            ),
            Self::SimultaneousSfx(commands) => {
                for (bank, id) in commands {
                    state.zelda_emit_audio_command(EngineAudioCommand::from_sfx_port_value(
                        bank.engine_bank(),
                        id,
                    ));
                }
            }
        }
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
                command.emit(&mut state);
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
    let frame = state.zelda_render_audio(
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

#[test]
fn every_catalogued_sfx_command_is_reachable_from_the_typed_engine_bus() {
    for (bank, id) in crate::modern_sfx_catalog::conformance_commands() {
        assert!(
            bank < 3,
            "catalogued SFX bank {bank} has no engine APUI port"
        );
        let mut state = ZeldaState::new();
        state.zelda_emit_audio_command(EngineAudioCommand::from_sfx_port_value(
            AudioSfxBank::ALL[usize::from(bank)],
            id,
        ));
        state.zelda_push_apu_state();

        let (frame, _) = render_modern_frame(&mut state);
        assert_render_invariants("catalog-wide SFX command", &state);
        assert_sfx_was_emitted("catalog-wide SFX command", &[frame], bank, id);
        assert_eq!(
            state
                .zelda_modern_audio_sequence_last_stats()
                .known_sfx_commands,
            1,
            "catalogued SFX ({bank}, {id:#04x}) did not resolve through the engine path"
        );
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
        .position(|frame| {
            has_event(frame, &|kind| {
                matches!(
                    kind,
                    AudioEventKind::NoteOn { .. } | AudioEventKind::DspKeyOn { .. }
                )
            })
        })
        .expect("music interruption never reached an active note");
    let stop = frames
        .iter()
        .position(|frame| has_event(frame, &|kind| matches!(kind, AudioEventKind::StopMusic)))
        .unwrap();
    assert!(play <= note && note < stop);
    assert!(!frames[stop + 1..].iter().any(|frame| {
        has_event(frame, &|kind| {
            matches!(
                kind,
                AudioEventKind::NoteOn { .. } | AudioEventKind::DspKeyOn { .. }
            )
        })
    }));
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

#[derive(Clone, Debug)]
enum GeneratedAudioOperation {
    EmitBatch(Vec<EngineAudioCommand>),
    RenderFrames(u8),
    SnapshotRoundTrip,
}

fn generated_audio_command_strategy() -> proptest::strategy::BoxedStrategy<EngineAudioCommand> {
    use proptest::prelude::*;

    let tracks = (1..=0xef)
        .filter(|track| crate::modern_music_catalog::packed_track(*track).is_some())
        .collect::<Vec<_>>();
    let sfx_commands = crate::modern_sfx_catalog::conformance_commands();

    prop_oneof![
        5 => proptest::sample::select(sfx_commands).prop_map(|(bank, id)| {
            EngineAudioCommand::from_sfx_port_value(
                AudioSfxBank::ALL[usize::from(bank)],
                id,
            )
        }),
        3 => proptest::sample::select(tracks)
            .prop_map(|track| EngineAudioCommand::PlayMusic { track }),
        1 => Just(EngineAudioCommand::StopMusic),
        1 => Just(EngineAudioCommand::ClearMusic),
        1 => (0xf1u8..=0xff).prop_map(|value| EngineAudioCommand::MusicControl { value }),
        2 => proptest::sample::select(AudioSfxBank::ALL.to_vec())
            .prop_map(|bank| EngineAudioCommand::ClearSfx { bank }),
    ]
    .boxed()
}

fn generated_audio_operation_strategy() -> proptest::strategy::BoxedStrategy<GeneratedAudioOperation>
{
    use proptest::prelude::*;

    prop_oneof![
        5 => proptest::collection::vec(generated_audio_command_strategy(), 1..=4)
            .prop_map(GeneratedAudioOperation::EmitBatch),
        4 => (1u8..=4).prop_map(GeneratedAudioOperation::RenderFrames),
        1 => Just(GeneratedAudioOperation::SnapshotRoundTrip),
    ]
    .boxed()
}

fn generated_audio_program_strategy(
) -> impl proptest::strategy::Strategy<Value = Vec<GeneratedAudioOperation>> {
    use proptest::prelude::*;

    proptest::collection::vec(generated_audio_operation_strategy(), 1..=24).prop_map(
        |mut operations| {
            operations.insert(
                operations.len() / 2,
                GeneratedAudioOperation::SnapshotRoundTrip,
            );
            // The queue is a 16-entry hardware-shaped latch. Drain it completely
            // so every retained generated command reaches the sequencer.
            operations.push(GeneratedAudioOperation::RenderFrames(16));
            operations
        },
    )
}

fn check_generated_render_invariants(
    operation_index: usize,
    state: &ZeldaState,
) -> proptest::test_runner::TestCaseResult {
    let sequence = state.zelda_modern_audio_sequence_last_stats();
    let renderer = state.zelda_modern_audio_last_stats();
    proptest::prop_assert_eq!(
        sequence.unknown_sfx_commands,
        0,
        "operation {} used unknown SFX programs: {:?}",
        operation_index,
        sequence.unknown_sfx_programs
    );
    proptest::prop_assert_eq!(
        sequence.fallback_sfx_commands,
        0,
        "operation {} entered the heuristic SFX fallback",
        operation_index
    );
    proptest::prop_assert_eq!(
        renderer.ignored_events,
        0,
        "operation {} emitted unsupported renderer events",
        operation_index
    );
    proptest::prop_assert!(
        renderer.active_voices <= 8,
        "operation {operation_index} reported {} active voices",
        renderer.active_voices
    );
    Ok(())
}

fn run_generated_audio_program(
    program: &[GeneratedAudioOperation],
) -> proptest::test_runner::TestCaseResult {
    let mut uninterrupted = ZeldaState::new();
    let mut checkpointed = ZeldaState::new();

    for (operation_index, operation) in program.iter().enumerate() {
        match operation {
            GeneratedAudioOperation::EmitBatch(commands) => {
                for command in commands {
                    uninterrupted.zelda_emit_audio_command(*command);
                    checkpointed.zelda_emit_audio_command(*command);
                }
                uninterrupted.zelda_push_apu_state();
                checkpointed.zelda_push_apu_state();
            }
            GeneratedAudioOperation::RenderFrames(count) => {
                for render_index in 0..*count {
                    let (expected_events, expected_pcm) = render_modern_frame(&mut uninterrupted);
                    let (actual_events, actual_pcm) = render_modern_frame(&mut checkpointed);
                    check_generated_render_invariants(operation_index, &uninterrupted)?;
                    check_generated_render_invariants(operation_index, &checkpointed)?;
                    proptest::prop_assert_eq!(
                        &actual_events,
                        &expected_events,
                        "event drift after operation {}, render {}",
                        operation_index,
                        render_index
                    );
                    proptest::prop_assert_eq!(
                        &actual_pcm,
                        &expected_pcm,
                        "PCM drift after operation {}, render {}",
                        operation_index,
                        render_index
                    );
                }
            }
            GeneratedAudioOperation::SnapshotRoundTrip => {
                let snapshot = checkpointed.zelda_audio_snapshot_bytes();
                let mut restored = ZeldaState::new();
                restored
                    .zelda_audio_restore_from_bytes(&snapshot)
                    .map_err(proptest::test_runner::TestCaseError::fail)?;
                checkpointed = restored;
            }
        }
    }

    Ok(())
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(256))]

    #[test]
    fn generated_audio_programs_are_deterministic_and_snapshot_stable(
        program in generated_audio_program_strategy(),
    ) {
        run_generated_audio_program(&program)?;
    }
}
