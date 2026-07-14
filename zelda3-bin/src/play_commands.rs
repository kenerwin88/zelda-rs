use std::env;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::process;

use platform::{HostMenuAction, HostMenuInput, HostMenuMode, HostMenuState, NativeFrontendOptions};
use zelda3::{game_output::AudioBackendMode, ZeldaState};

use crate::audio_trace::replay_checksum_samples;
use crate::developer_room_commands::{
    current_developer_location_from_ram, load_developer_destination,
};
use crate::input_script::InputScript;
use crate::{
    apply_sram_to_game_or_exit, captured_panic_from, developer_destinations,
    install_crash_panic_hook, load_embedded_play_state, load_play_state, play_renderer,
    read_file_or_exit, select_run_what, write_play_crash_report, TRACE_FILTERED_JOYPAD_H,
    TRACE_FILTERED_JOYPAD_L, TRACE_JOYPAD1H_LAST, TRACE_JOYPAD1L_LAST, TRACE_MAIN_MODULE_INDEX,
    TRACE_SELECTFILE_ARR2_1, TRACE_SELECTFILE_VAR10, TRACE_SELECTFILE_VAR11, TRACE_SELECTFILE_VAR3,
    TRACE_SELECTFILE_VAR5, TRACE_SELECTFILE_VAR7, TRACE_SELECTFILE_VAR9, TRACE_SUBMODULE_INDEX,
    TRACE_SUBSUBMODULE_INDEX,
};

#[derive(Debug)]
struct FrontendSmokeOptions {
    frames: u32,
    frame_pacing: bool,
    require_audio: bool,
    rom_path: Option<String>,
    input_script: InputScript,
    load_sram: Option<String>,
    replay_save: Option<String>,
    replay_start_frame: u32,
}

fn configure_audio_backend_from_env(game: &mut ZeldaState) -> Result<AudioBackendMode, String> {
    let Some(value) = env::var_os("ZELDA3_AUDIO_BACKEND") else {
        return Ok(game.zelda_audio_backend());
    };
    let value = value
        .into_string()
        .map_err(|_| "ZELDA3_AUDIO_BACKEND must be valid UTF-8".to_string())?;
    let backend = AudioBackendMode::parse(&value).ok_or_else(|| {
        format!(
            "invalid ZELDA3_AUDIO_BACKEND={value:?}; expected modern, dsp-parity, or trace-only"
        )
    })?;
    game.zelda_set_audio_backend(backend)
        .map_err(str::to_string)?;
    Ok(backend)
}

fn parse_frontend_smoke_options(args: &[String]) -> Result<FrontendSmokeOptions, String> {
    let mut frames = 2u32;
    let mut frames_set = false;
    let mut frame_pacing = true;
    let mut require_audio = false;
    let mut rom_path = None;
    let mut input_script = InputScript::default();
    let mut load_sram = None;
    let mut replay_save = None;
    let mut replay_start_frame = 0;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--no-frame-pacing" => frame_pacing = false,
            "--require-audio" => require_audio = true,
            "--rom" => {
                let value = args.get(i + 1).ok_or_else(|| {
                    "usage: zelda3 --frontend-smoke [frames] [--no-frame-pacing] [--rom path] [--input-script path] [--load-sram path]; --rom requires a path".to_string()
                })?;
                rom_path = Some(value.clone());
                i += 1;
            }
            "--input-script" => {
                let value = args.get(i + 1).ok_or_else(|| {
                    "usage: zelda3 --frontend-smoke [frames] [--no-frame-pacing] [--rom path] [--input-script path] [--load-sram path]; --input-script requires a path".to_string()
                })?;
                input_script = InputScript::from_path(Path::new(value))
                    .map_err(|err| format!("failed to parse input script {value}: {err}"))?;
                i += 1;
            }
            "--load-sram" => {
                let value = args.get(i + 1).ok_or_else(|| {
                    "usage: zelda3 --frontend-smoke [frames] [--no-frame-pacing] [--rom path] [--input-script path] [--load-sram path]; --load-sram requires a path".to_string()
                })?;
                load_sram = Some(value.clone());
                i += 1;
            }
            "--replay-save" => {
                let value = args.get(i + 1).ok_or_else(|| {
                    "usage: zelda3 --frontend-smoke [frames] [--no-frame-pacing] [--rom path] [--input-script path] [--load-sram path] [--replay-save path] [--replay-start-frame n]; --replay-save requires a path".to_string()
                })?;
                replay_save = Some(value.clone());
                i += 1;
            }
            "--replay-start-frame" => {
                let value = args.get(i + 1).ok_or_else(|| {
                    "usage: zelda3 --frontend-smoke [frames] [--no-frame-pacing] [--rom path] [--input-script path] [--load-sram path] [--replay-save path] [--replay-start-frame n]; --replay-start-frame requires a frame".to_string()
                })?;
                replay_start_frame = value.parse().map_err(|_| {
                    format!("--frontend-smoke invalid --replay-start-frame={value:?}")
                })?;
                i += 1;
            }
            value if value.starts_with("--") => {
                return Err(format!(
                    "usage: zelda3 --frontend-smoke [frames] [--no-frame-pacing] [--rom path] [--input-script path] [--load-sram path]; unknown option {value:?}"
                ));
            }
            value if !frames_set => {
                frames = value.parse().map_err(|_| {
                    format!(
                        "usage: zelda3 --frontend-smoke [frames] [--no-frame-pacing] [--rom path] [--input-script path] [--load-sram path]; invalid frames={value:?}"
                    )
                })?;
                frames_set = true;
            }
            value => {
                return Err(format!(
                    "usage: zelda3 --frontend-smoke [frames] [--no-frame-pacing] [--rom path] [--input-script path] [--load-sram path]; unknown option {value:?}"
                ));
            }
        }
        i += 1;
    }
    Ok(FrontendSmokeOptions {
        frames,
        frame_pacing,
        require_audio,
        rom_path,
        input_script,
        load_sram,
        replay_save,
        replay_start_frame,
    })
}

pub(crate) fn run_frontend_smoke(args: &[String]) {
    let options = parse_frontend_smoke_options(args).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(2);
    });
    if options.replay_save.is_some() && options.rom_path.is_none() {
        eprintln!("--frontend-smoke --replay-save requires --rom");
        process::exit(2);
    }
    if options.replay_start_frame != 0 && options.replay_save.is_none() {
        eprintln!("--frontend-smoke --replay-start-frame requires --replay-save");
        process::exit(2);
    }
    let mut game = options
        .rom_path
        .as_deref()
        .map(load_play_state)
        .unwrap_or_else(load_embedded_play_state);
    let audio_backend = configure_audio_backend_from_env(&mut game).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(2);
    });
    if let Some(path) = options.load_sram.as_deref() {
        let sram = read_file_or_exit(Path::new(path), "SRAM");
        apply_sram_to_game_or_exit(&mut game, Path::new(path), &sram);
    }
    if let Some(path) = options.replay_save.as_deref() {
        if let Err(error) = game.replay_save_file(Path::new(path)) {
            eprintln!("failed to load frontend-smoke replay {path}: {error}");
            process::exit(1);
        }
        let mut completed = 0;
        while completed < options.replay_start_frame && game.state_recorder.replay_mode {
            game.zelda_run_frame(0);
            game.zelda_discard_unused_audio_frames();
            completed += 1;
        }
        if completed != options.replay_start_frame {
            eprintln!(
                "frontend-smoke replay ended at frame={completed}, expected={}",
                options.replay_start_frame
            );
            process::exit(1);
        }
        if !options.input_script.is_empty() {
            let mut state_recorder = std::mem::take(&mut game.state_recorder);
            ZeldaState::state_recorder_stop_replay(&mut state_recorder);
            game.state_recorder = state_recorder;
        }
    }
    let width = 256u32;
    let height = 224u32;
    let mut renderer = match play_renderer::configured_from_env(
        width,
        height,
        NativeFrontendOptions::from_env(3, false).with_frame_pacing(options.frame_pacing),
    ) {
        Ok(frontend) => frontend,
        Err(e) => {
            eprintln!("failed to initialize native frontend: {e}");
            process::exit(1);
        }
    };
    let renderer_name = renderer.name();
    let audio_samples = renderer.audio_samples_per_frame();
    let audio_channels = renderer.audio_channels();
    let mut audio = vec![0i16; audio_samples * audio_channels];
    let mut audio_peak = 0i16;
    let mut triggered_voices = 0u64;
    let mut understood_events = 0u64;
    let mut note_events = 0u64;
    let mut sfx_commands = 0u64;
    let mut audio_hash = 0u32;

    let mut completed = 0u32;
    while completed < options.frames && !renderer.quit_requested() {
        let live_input = if options.input_script.is_empty() {
            renderer.poll_input()
        } else {
            options.input_script.input_for_frame(completed)
        };
        game.zelda_run_frame(live_input as i32);
        renderer.present_frame(&mut game);
        game.zelda_render_audio(&mut audio, audio_samples as i32, audio_channels as i32);
        let stats = game.zelda_modern_audio_last_stats();
        triggered_voices += u64::from(stats.triggered_voices);
        understood_events += u64::from(stats.understood_events);
        let sequence_stats = game.zelda_modern_audio_sequence_last_stats();
        note_events += u64::from(sequence_stats.note_events);
        sfx_commands += u64::from(sequence_stats.sfx_commands);
        audio_peak = audio_peak.max(
            audio
                .iter()
                .map(|sample| sample.saturating_abs())
                .max()
                .unwrap_or(0),
        );
        audio_hash = audio_hash.rotate_left(5) ^ replay_checksum_samples(&audio);
        renderer.push_audio(&audio);
        game.zelda_discard_unused_audio_frames();
        completed += 1;
    }
    if !options.frame_pacing {
        renderer.wait_idle();
    }

    println!(
        "frontend smoke completed frames={completed} renderer={renderer_name} audio_backend={audio_backend:?} audio_peak={audio_peak} audio_hash=0x{audio_hash:08x} triggered_voices={triggered_voices} understood_events={understood_events} note_events={note_events} sfx_commands={sfx_commands}"
    );
    if options.require_audio && audio_peak == 0 {
        eprintln!("frontend smoke failed: requested audio verification but output was silent");
        process::exit(1);
    }
}

pub(crate) fn run_play(rom_path: &str) {
    run_play_with_state(load_play_state(rom_path));
}

pub(crate) fn run_standalone_play() {
    run_play_with_state(load_embedded_play_state());
}

fn run_play_with_state(mut game: ZeldaState) {
    let audio_backend = configure_audio_backend_from_env(&mut game).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(2);
    });
    if env::var_os("ZELDA3_AUDIO_BACKEND").is_some() {
        eprintln!("audio backend override: {audio_backend:?}");
    }
    let last_panic = install_crash_panic_hook();
    let width = 256u32;
    let height = 224u32;
    let mut renderer = match play_renderer::configured_from_env(
        width,
        height,
        NativeFrontendOptions::from_env(3, true),
    ) {
        Ok(frontend) => frontend,
        Err(e) => {
            eprintln!("failed to initialize native frontend: {e}");
            process::exit(1);
        }
    };
    let audio_samples = renderer.audio_samples_per_frame();
    let audio_channels = renderer.audio_channels();
    let mut audio = vec![0i16; audio_samples * audio_channels];
    let mut host_frame = 0u32;
    let mut game_started = env::var_os("ZELDA3_SKIP_HOST_MENU").is_some();
    let mut host_menu = HostMenuState::new(
        HostMenuMode::PreGame,
        developer_destinations::developer_destinations(),
    );
    if game_started {
        host_menu.close();
    }
    let trace_live_input = env::var_os("ZELDA3_TRACE_LIVE_INPUT").is_some();
    let mut last_traced_live_input = (
        u16::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
    );

    while !renderer.quit_requested() {
        let live_input = renderer.poll_input_with_menu(host_menu.is_open());
        let mut should_quit = false;
        for input in renderer.drain_host_menu_inputs() {
            if host_menu.is_open() {
                if let Some(action) = host_menu.handle_input(input) {
                    match action {
                        HostMenuAction::Resume => host_menu.close(),
                        HostMenuAction::StartQuest => {
                            game_started = true;
                            host_menu.close();
                        }
                        HostMenuAction::Quit | HostMenuAction::SaveAndQuit => {
                            should_quit = true;
                        }
                        HostMenuAction::SetPresentation(_)
                        | HostMenuAction::SetLighting(_)
                        | HostMenuAction::SetShadows(_)
                        | HostMenuAction::SetViewport(_)
                        | HostMenuAction::ResetRuntimeSettings(_) => {
                            renderer.apply_runtime_settings(host_menu.runtime_settings());
                        }
                        HostMenuAction::ShowControls(panel) => {
                            eprintln!("host menu controls panel selected: {panel:?}");
                        }
                        HostMenuAction::WarpToVerifiedDestination(id) => {
                            match load_developer_destination(id) {
                                Ok((next_game, next_frame)) => {
                                    game = next_game;
                                    game.zelda_set_audio_backend(audio_backend)
                                        .expect("fresh developer state accepts backend selection");
                                    host_frame = next_frame;
                                    game_started = true;
                                    host_menu.close();
                                    eprintln!(
                                        "developer destination loaded: {id} frame={next_frame}"
                                    );
                                }
                                Err(e) => {
                                    eprintln!("developer destination failed: {id}: {e}");
                                }
                            }
                        }
                    }
                }
            } else {
                match input {
                    HostMenuInput::Cancel => host_menu.open_ingame(),
                    HostMenuInput::CyclePresentation
                    | HostMenuInput::CycleLighting
                    | HostMenuInput::CycleShadows => {
                        if let Some(
                            HostMenuAction::SetPresentation(_)
                            | HostMenuAction::SetLighting(_)
                            | HostMenuAction::SetShadows(_),
                        ) = host_menu.handle_input(input)
                        {
                            renderer.apply_runtime_settings(host_menu.runtime_settings());
                        }
                    }
                    _ => {}
                }
            }
        }
        if should_quit {
            break;
        }
        if host_menu.is_open() {
            host_menu.set_current_developer_location(current_developer_location_from_ram(
                &game.ram, host_frame,
            ));
            renderer.present_menu_overlay(&host_menu);
            continue;
        }
        if !game_started {
            game_started = true;
        }
        let run_what = select_run_what(&game.ram);
        let pre_frame_game = game.clone();
        let mut crash_stage = "run_frame";
        let frame_result = panic::catch_unwind(AssertUnwindSafe(|| {
            game.zelda_run_frame(live_input as i32);
            crash_stage = renderer.name();
            renderer.present_frame(&mut game);
            crash_stage = "audio";
            game.zelda_render_audio(&mut audio, audio_samples as i32, audio_channels as i32);
            renderer.push_audio(&audio);
            game.zelda_discard_unused_audio_frames();
        }));
        if let Err(payload) = frame_result {
            let panic_info = captured_panic_from(last_panic.clone(), payload);
            write_play_crash_report(
                &pre_frame_game,
                host_frame,
                live_input,
                run_what,
                crash_stage,
                Some(&panic_info),
            );
            game.zelda_write_sram();
            process::exit(101);
        }
        let trace_state = (
            live_input,
            game.ram[TRACE_JOYPAD1H_LAST],
            game.ram[TRACE_JOYPAD1L_LAST],
            game.ram[TRACE_FILTERED_JOYPAD_H],
            game.ram[TRACE_FILTERED_JOYPAD_L],
            game.ram[TRACE_SELECTFILE_VAR3],
            game.ram[TRACE_SELECTFILE_VAR5],
            game.ram[TRACE_SELECTFILE_VAR7],
            game.ram[TRACE_SELECTFILE_VAR9],
            game.ram[TRACE_SELECTFILE_VAR10],
            game.ram[TRACE_SELECTFILE_VAR11],
            game.ram[TRACE_SELECTFILE_ARR2_1],
        );
        if trace_live_input && trace_state != last_traced_live_input {
            eprintln!(
                "live-input host_frame={host_frame} input=0x{live_input:04x} joyh=0x{:02x} joyl=0x{:02x} fh=0x{:02x} fl=0x{:02x} main={} sub={} subsub={} sel3={} sel5={} sel7={} sel9={} sel10={} sel11={} arr2_1={}",
                game.ram[TRACE_JOYPAD1H_LAST],
                game.ram[TRACE_JOYPAD1L_LAST],
                game.ram[TRACE_FILTERED_JOYPAD_H],
                game.ram[TRACE_FILTERED_JOYPAD_L],
                game.ram[TRACE_MAIN_MODULE_INDEX],
                game.ram[TRACE_SUBMODULE_INDEX],
                game.ram[TRACE_SUBSUBMODULE_INDEX],
                game.ram[TRACE_SELECTFILE_VAR3],
                game.ram[TRACE_SELECTFILE_VAR5],
                game.ram[TRACE_SELECTFILE_VAR7],
                game.ram[TRACE_SELECTFILE_VAR9],
                game.ram[TRACE_SELECTFILE_VAR10],
                game.ram[TRACE_SELECTFILE_VAR11],
                game.ram[TRACE_SELECTFILE_ARR2_1],
            );
            last_traced_live_input = trace_state;
        }
        host_frame = host_frame.wrapping_add(1);
    }
    game.zelda_write_sram();
}

#[cfg(test)]
fn apply_host_menu_action_for_test(
    menu: &mut HostMenuState,
    action: HostMenuAction,
    should_start: &mut bool,
    should_quit: &mut bool,
) {
    match action {
        HostMenuAction::Resume => menu.close(),
        HostMenuAction::StartQuest => {
            *should_start = true;
            menu.close();
        }
        HostMenuAction::Quit | HostMenuAction::SaveAndQuit => *should_quit = true,
        HostMenuAction::SetPresentation(_)
        | HostMenuAction::SetLighting(_)
        | HostMenuAction::SetShadows(_)
        | HostMenuAction::SetViewport(_)
        | HostMenuAction::ShowControls(_)
        | HostMenuAction::ResetRuntimeSettings(_)
        | HostMenuAction::WarpToVerifiedDestination(_) => {}
    }
}

#[cfg(test)]
mod host_menu_play_tests {
    use super::*;

    #[test]
    fn frontend_smoke_defaults_to_two_paced_frames() {
        let options = parse_frontend_smoke_options(&[]).unwrap();
        assert_eq!(options.frames, 2);
        assert!(options.frame_pacing);
        assert!(!options.require_audio);
        assert!(options.rom_path.is_none());
        assert!(options.input_script.is_empty());
        assert!(options.load_sram.is_none());
        assert!(options.replay_save.is_none());
        assert_eq!(options.replay_start_frame, 0);
    }

    #[test]
    fn frontend_smoke_accepts_no_frame_pacing() {
        let args = vec![
            "600".to_string(),
            "--no-frame-pacing".to_string(),
            "--require-audio".to_string(),
        ];
        let options = parse_frontend_smoke_options(&args).unwrap();
        assert_eq!(options.frames, 600);
        assert!(!options.frame_pacing);
        assert!(options.require_audio);
        assert!(options.rom_path.is_none());
        assert!(options.input_script.is_empty());
        assert!(options.load_sram.is_none());
    }

    #[test]
    fn frontend_smoke_accepts_live_scripted_paths() {
        let args = vec![
            "120".to_string(),
            "--rom".to_string(),
            "saves/zelda3.sfc".to_string(),
            "--load-sram".to_string(),
            "scripts/inputs/tas-us-full-completion-smv.sram".to_string(),
        ];
        let options = parse_frontend_smoke_options(&args).unwrap();
        assert_eq!(options.frames, 120);
        assert_eq!(options.rom_path.as_deref(), Some("saves/zelda3.sfc"));
        assert!(options.input_script.is_empty());
        assert_eq!(
            options.load_sram.as_deref(),
            Some("scripts/inputs/tas-us-full-completion-smv.sram")
        );
    }

    #[test]
    fn frontend_smoke_accepts_replay_fast_forward_for_live_render_regressions() {
        let args = vec![
            "250".to_string(),
            "--rom".to_string(),
            "saves/zelda3.sfc".to_string(),
            "--replay-save".to_string(),
            "saves/zelda3-combined-route.sav".to_string(),
            "--replay-start-frame".to_string(),
            "900".to_string(),
        ];
        let options = parse_frontend_smoke_options(&args).unwrap();
        assert_eq!(
            options.replay_save.as_deref(),
            Some("saves/zelda3-combined-route.sav")
        );
        assert_eq!(options.replay_start_frame, 900);
    }

    #[test]
    fn frontend_smoke_rejects_unknown_options() {
        let args = vec!["--fast".to_string()];
        let error = parse_frontend_smoke_options(&args).unwrap_err();
        assert_eq!(
            error,
            "usage: zelda3 --frontend-smoke [frames] [--no-frame-pacing] [--rom path] [--input-script path] [--load-sram path]; unknown option \"--fast\""
        );
    }

    #[test]
    fn menu_resume_action_closes_ingame_menu() {
        let mut menu = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        let mut should_quit = false;
        let mut should_start = false;
        apply_host_menu_action_for_test(
            &mut menu,
            HostMenuAction::Resume,
            &mut should_start,
            &mut should_quit,
        );
        assert!(!menu.is_open());
        assert!(!should_start);
        assert!(!should_quit);
    }

    #[test]
    fn menu_start_action_closes_pregame_menu_and_starts_game() {
        let mut menu = HostMenuState::new(HostMenuMode::PreGame, Vec::new());
        let mut should_quit = false;
        let mut should_start = false;
        apply_host_menu_action_for_test(
            &mut menu,
            HostMenuAction::StartQuest,
            &mut should_start,
            &mut should_quit,
        );
        assert!(!menu.is_open());
        assert!(should_start);
        assert!(!should_quit);
    }
}
