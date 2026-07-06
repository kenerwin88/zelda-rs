use std::path::{Path, PathBuf};
use std::process;

use crate::gpu_compare::{
    gpu_render_compare_run, modern_index_compare_run_from_env, GpuRenderCompareRun,
    ModernIndexCompareRun,
};
use crate::input_script::InputScript;

pub(crate) struct ReplaySaveConfig {
    pub(crate) rom_path: String,
    pub(crate) replay_path: String,
    pub(crate) max_frames: u32,
    pub(crate) dump_frame_path: Option<PathBuf>,
    pub(crate) render_hash_log: u32,
    pub(crate) audio_trace_log: u32,
    pub(crate) fingerprint_log: Option<PathBuf>,
    pub(crate) fingerprint_frame: Option<u32>,
    pub(crate) coverage_log: Option<PathBuf>,
    pub(crate) gpu_render_compare: GpuRenderCompareRun,
    pub(crate) modern_index_compare: ModernIndexCompareRun,
    pub(crate) asset_gpu_smoke: bool,
    pub(crate) ppu_mode_summary: bool,
    pub(crate) render_hash_dump_frame: Option<(u32, PathBuf)>,
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
                "usage: zelda3 --replay-save <path-to-rom.sfc> <replay.sav> [frames] [--dump-frame <out.png>] [--render-hash-log <stride>] [--audio-trace-log <stride>] [--gpu-render-compare <stride>] [--gpu-render-compare-quiet] [--modern-index-compare <stride>] [--require-full-gpu-path] [--require-modern-index-parity] [--asset-gpu-smoke] [--render-hash-dump-frame <frame> <out.png>] [--input-script <path>] [--input-script-overlay <path>] [--stop-replay-after-load] [--save-state <checkpoint.sav>] [--load-state <checkpoint.sav>] [--load-sram <path>] [--fingerprint-log <path>] [--fingerprint-frame <frame>] [--coverage-log <path>]"
            );
            process::exit(2);
        }
    };
    let mut max_frames = u32::MAX;
    let mut dump_frame_path = None::<PathBuf>;
    let mut render_hash_log = 0u32;
    let mut audio_trace_log = 0u32;
    let mut fingerprint_log: Option<PathBuf> = None;
    let mut fingerprint_frame = None::<u32>;
    let mut coverage_log: Option<PathBuf> = None;
    let mut gpu_render_compare = gpu_render_compare_run(0, false);
    let mut modern_index_compare = modern_index_compare_run_from_env();
    let mut asset_gpu_smoke = false;
    let ppu_mode_summary = std::env::var("ZELDA3_PPU_MODE_SUMMARY").is_ok();
    let mut render_hash_dump_frame = None::<(u32, PathBuf)>;
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
            "--render-hash-log" => {
                let stride = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--render-hash-log requires a stride");
                    process::exit(2);
                });
                render_hash_log = stride.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --render-hash-log stride: {stride}");
                    process::exit(2);
                });
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
            "--gpu-render-compare" => {
                let stride = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--gpu-render-compare requires a stride");
                    process::exit(2);
                });
                let stride = stride.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --gpu-render-compare stride: {stride}");
                    process::exit(2);
                });
                if !gpu_render_compare.set_stride(stride) {
                    eprintln!("--gpu-render-compare stride must be greater than zero");
                    process::exit(2);
                }
                i += 2;
            }
            "--gpu-render-compare-quiet" => {
                gpu_render_compare.set_quiet();
                i += 1;
            }
            "--modern-index-compare" => {
                let stride = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--modern-index-compare requires a stride");
                    process::exit(2);
                });
                let stride = stride.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --modern-index-compare stride: {stride}");
                    process::exit(2);
                });
                if !modern_index_compare.set_stride(stride) {
                    eprintln!("--modern-index-compare stride must be greater than zero");
                    process::exit(2);
                }
                i += 2;
            }
            "--require-full-gpu-path" => {
                modern_index_compare.set_require_full_gpu_path();
                i += 1;
            }
            "--require-modern-index-parity" => {
                modern_index_compare.set_require_modern_index_parity();
                i += 1;
            }
            "--asset-gpu-smoke" => {
                asset_gpu_smoke = true;
                i += 1;
            }
            "--render-hash-dump-frame" => {
                let frame = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--render-hash-dump-frame requires a frame");
                    process::exit(2);
                });
                let path = args.get(i + 2).unwrap_or_else(|| {
                    eprintln!("--render-hash-dump-frame requires a path");
                    process::exit(2);
                });
                let frame = frame.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --render-hash-dump-frame frame: {frame}");
                    process::exit(2);
                });
                render_hash_dump_frame = Some((frame, PathBuf::from(path)));
                i += 3;
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
            "--fingerprint-log" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--fingerprint-log requires a path");
                    process::exit(2);
                };
                fingerprint_log = Some(PathBuf::from(path));
                i += 2;
            }
            "--fingerprint-frame" => {
                let Some(frame) = args.get(i + 1) else {
                    eprintln!("--fingerprint-frame requires a frame");
                    process::exit(2);
                };
                fingerprint_frame = Some(frame.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --fingerprint-frame frame: {frame}");
                    process::exit(2);
                }));
                i += 2;
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

    if let Err(e) = modern_index_compare.validate() {
        eprintln!("{e}");
        process::exit(2);
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
        render_hash_log,
        audio_trace_log,
        fingerprint_log,
        fingerprint_frame,
        coverage_log,
        gpu_render_compare,
        modern_index_compare,
        asset_gpu_smoke,
        ppu_mode_summary,
        render_hash_dump_frame,
        save_state_path,
        save_state_at,
        load_state_path,
        load_sram_path,
        input_script,
        input_script_overlay,
        stop_replay_after_load,
    }
}
