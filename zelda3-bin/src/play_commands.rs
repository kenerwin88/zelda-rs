use std::env;
use std::panic::{self, AssertUnwindSafe};
use std::process;

use platform::{HostMenuAction, HostMenuInput, HostMenuMode, HostMenuState, NativeFrontendOptions};
use zelda3::ZeldaState;

use crate::developer_room_commands::{
    current_developer_location_from_ram, load_developer_destination,
};
use crate::{
    captured_panic_from, developer_destinations, install_crash_panic_hook,
    load_embedded_play_state, load_play_state, play_renderer, select_run_what,
    write_play_crash_report, TRACE_FILTERED_JOYPAD_H, TRACE_FILTERED_JOYPAD_L, TRACE_JOYPAD1H_LAST,
    TRACE_JOYPAD1L_LAST, TRACE_MAIN_MODULE_INDEX, TRACE_SELECTFILE_ARR2_1, TRACE_SELECTFILE_VAR10,
    TRACE_SELECTFILE_VAR11, TRACE_SELECTFILE_VAR3, TRACE_SELECTFILE_VAR5, TRACE_SELECTFILE_VAR7,
    TRACE_SELECTFILE_VAR9, TRACE_SUBMODULE_INDEX, TRACE_SUBSUBMODULE_INDEX,
};

pub(crate) fn run_frontend_smoke(args: &[String]) {
    let frames: u32 = args.first().map(|s| s.parse().unwrap_or(2)).unwrap_or(2);
    let mut game = load_embedded_play_state();
    let width = 256u32;
    let height = 224u32;
    let mut renderer = match play_renderer::configured_from_env(
        width,
        height,
        NativeFrontendOptions::from_env(3, false),
    ) {
        Ok(frontend) => frontend,
        Err(e) => {
            eprintln!("failed to initialize native frontend: {e}");
            process::exit(1);
        }
    };
    let renderer_name = renderer.name();

    let mut completed = 0u32;
    while completed < frames && !renderer.quit_requested() {
        let live_input = renderer.poll_input();
        game.zelda_run_frame(live_input as i32);
        renderer.present_frame(&mut game);
        completed += 1;
    }

    println!("frontend smoke completed frames={completed} renderer={renderer_name}");
}

pub(crate) fn run_play(rom_path: &str) {
    run_play_with_state(load_play_state(rom_path));
}

pub(crate) fn run_standalone_play() {
    run_play_with_state(load_embedded_play_state());
}

fn run_play_with_state(mut game: ZeldaState) {
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
