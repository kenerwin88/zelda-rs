use std::path::{Path, PathBuf};
use std::process;

use crate::input_script::InputScript;

pub(crate) struct ReplaySaveConfig {
    pub(crate) rom_path: String,
    pub(crate) replay_path: String,
    pub(crate) max_frames: u32,
    pub(crate) dump_frame_path: Option<PathBuf>,
    pub(crate) audio_trace_log: u32,
    pub(crate) coverage_log: Option<PathBuf>,
    pub(crate) asset_gpu_smoke: bool,
    pub(crate) asset_gpu_progress_interval: u32,
    pub(crate) asset_gpu_missing_assets_out: Option<PathBuf>,
    pub(crate) asset_gpu_checkpoint_dir: Option<PathBuf>,
    pub(crate) asset_gpu_checkpoint_interval: u32,
    pub(crate) ppu_mode_summary: bool,
    pub(crate) save_state_path: Option<PathBuf>,
    pub(crate) save_state_at: Vec<(u32, PathBuf)>,
    pub(crate) load_state_path: Option<PathBuf>,
    pub(crate) load_sram_path: Option<PathBuf>,
    pub(crate) input_script: InputScript,
    pub(crate) input_script_overlay: Option<InputScript>,
    pub(crate) stop_replay_after_load: bool,
}

pub(crate) fn parse_replay_save_args_or_exit(args: &[String]) -> ReplaySaveConfig {
    let (rom_path, replay_path) = match (args.first(), args.get(1)) {
        (Some(rom), Some(replay)) => (rom.clone(), replay.clone()),
        _ => {
            eprintln!(
                "usage: zelda3 --replay-save <path-to-rom.sfc> <replay.sav> [frames] [--dump-frame <out.png>] [--audio-trace-log <stride>] [--asset-gpu-smoke] [--asset-gpu-progress <stride>] [--missing-assets-out <path>] [--stop-after-first-missing] [--asset-gpu-checkpoint-dir <dir>] [--asset-gpu-checkpoint-interval <frames>] [--input-script <path>] [--input-script-overlay <path>] [--stop-replay-after-load] [--save-state <checkpoint.sav>] [--load-state <checkpoint.sav>] [--load-sram <path>] [--coverage-log <path>]"
            );
            process::exit(2);
        }
    };
    let mut max_frames = u32::MAX;
    let mut dump_frame_path = None::<PathBuf>;
    let mut audio_trace_log = 0u32;
    let mut coverage_log: Option<PathBuf> = None;
    let mut asset_gpu_smoke = false;
    let mut asset_gpu_progress_interval = 10_000u32;
    let mut asset_gpu_missing_assets_out = None::<PathBuf>;
    let mut asset_gpu_checkpoint_dir = None::<PathBuf>;
    let mut asset_gpu_checkpoint_interval = 10_000u32;
    let ppu_mode_summary = std::env::var("ZELDA3_PPU_MODE_SUMMARY").is_ok();
    let mut save_state_path = None::<PathBuf>;
    let mut save_state_at: Vec<(u32, PathBuf)> = Vec::new();
    let mut load_state_path = None::<PathBuf>;
    let mut load_sram_path = None::<PathBuf>;
    let mut input_script = InputScript::default();
    let mut input_script_overlay = None::<InputScript>;
    let mut stop_replay_after_load = false;
    let mut i = 2usize;
    if let Some(candidate) = args.get(i) {
        if !candidate.starts_with("--") {
            max_frames = candidate.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("invalid frame count: {candidate}");
                process::exit(2);
            });
            i += 1;
        }
    }
    while i < args.len() {
        match args[i].as_str() {
            "--dump-frame" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--dump-frame requires a path");
                    process::exit(2);
                });
                dump_frame_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--audio-trace-log" => {
                let stride = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--audio-trace-log requires a stride");
                    process::exit(2);
                });
                audio_trace_log = stride.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --audio-trace-log stride: {stride}");
                    process::exit(2);
                });
                if audio_trace_log == 0 {
                    eprintln!("--audio-trace-log stride must be greater than zero");
                    process::exit(2);
                }
                i += 2;
            }
            "--asset-gpu-smoke" => {
                asset_gpu_smoke = true;
                i += 1;
            }
            "--asset-gpu-progress" => {
                let stride = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--asset-gpu-progress requires a stride");
                    process::exit(2);
                });
                asset_gpu_progress_interval = stride.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --asset-gpu-progress stride: {stride}");
                    process::exit(2);
                });
                i += 2;
            }
            "--missing-assets-out" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--missing-assets-out requires a path");
                    process::exit(2);
                });
                asset_gpu_smoke = true;
                asset_gpu_missing_assets_out = Some(PathBuf::from(path));
                i += 2;
            }
            "--stop-after-first-missing" => {
                asset_gpu_smoke = true;
                i += 1;
            }
            "--asset-gpu-checkpoint-dir" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--asset-gpu-checkpoint-dir requires a directory");
                    process::exit(2);
                });
                asset_gpu_checkpoint_dir = Some(PathBuf::from(path));
                i += 2;
            }
            "--asset-gpu-checkpoint-interval" => {
                let frames = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--asset-gpu-checkpoint-interval requires a frame count");
                    process::exit(2);
                });
                asset_gpu_checkpoint_interval = frames.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --asset-gpu-checkpoint-interval frame count: {frames}");
                    process::exit(2);
                });
                i += 2;
            }
            "--save-state" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--save-state requires a path");
                    process::exit(2);
                });
                save_state_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--save-state-at" => {
                let Some(spec) = args.get(i + 1) else {
                    eprintln!("--save-state-at <frame>:<path>");
                    process::exit(2);
                };
                let (f, path) = spec.split_once(':').unwrap_or_else(|| {
                    eprintln!("--save-state-at <frame>:<path>");
                    process::exit(2);
                });
                let frame = f.parse().unwrap_or_else(|_| {
                    eprintln!("--save-state-at <frame>:<path>: invalid frame number");
                    process::exit(2);
                });
                save_state_at.push((frame, PathBuf::from(path)));
                i += 2;
            }
            "--load-state" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-state requires a path");
                    process::exit(2);
                });
                load_state_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--load-sram" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                });
                load_sram_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--input-script" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--input-script requires a path");
                    process::exit(2);
                });
                input_script = match InputScript::from_path(Path::new(path)) {
                    Ok(script) => script,
                    Err(e) => {
                        eprintln!("failed to parse input script {}: {e}", path);
                        process::exit(2);
                    }
                };
                i += 2;
            }
            "--input-script-overlay" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--input-script-overlay requires a path");
                    process::exit(2);
                });
                input_script_overlay = Some(match InputScript::from_path(Path::new(path)) {
                    Ok(script) => script,
                    Err(e) => {
                        eprintln!("failed to parse input script overlay {}: {e}", path);
                        process::exit(2);
                    }
                });
                i += 2;
            }
            "--stop-replay-after-load" => {
                stop_replay_after_load = true;
                i += 1;
            }
            "--coverage-log" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--coverage-log requires a path");
                    process::exit(2);
                };
                coverage_log = Some(PathBuf::from(path));
                i += 2;
            }
            flag => {
                eprintln!("unknown --replay-save option: {flag}");
                process::exit(2);
            }
        }
    }

    if load_state_path.is_some() && load_sram_path.is_some() {
        eprintln!(
            "--load-sram cannot be combined with --load-state; checkpoints already include SRAM"
        );
        process::exit(2);
    }

    ReplaySaveConfig {
        rom_path,
        replay_path,
        max_frames,
        dump_frame_path,
        audio_trace_log,
        coverage_log,
        asset_gpu_smoke,
        asset_gpu_progress_interval,
        asset_gpu_missing_assets_out,
        asset_gpu_checkpoint_dir,
        asset_gpu_checkpoint_interval,
        ppu_mode_summary,
        save_state_path,
        save_state_at,
        load_state_path,
        load_sram_path,
        input_script,
        input_script_overlay,
        stop_replay_after_load,
    }
}
