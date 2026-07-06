use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::gpu_capture::{render_live_game_gpu_frame_rgba, ModernAssetGpuReadbackRenderer};
use crate::image_output::write_rgba_frame_png;
use crate::input_script::InputScript;
use crate::{
    apply_sram_to_game_or_exit, load_play_or_checkpoint, load_replay_save_checkpoint,
    load_translated_replay_state, parse_u16_auto, read_file_or_exit, read_le_u16,
    PLAYER_IS_INDOORS,
};

#[derive(Debug, PartialEq, Eq)]
struct ScriptedAssetGpuSmokeOptions {
    rom_path: String,
    frames: u32,
    input_script_path: Option<PathBuf>,
    load_sram: Option<PathBuf>,
    load_state: Option<PathBuf>,
}

fn parse_scripted_asset_gpu_smoke_options(args: &[String]) -> ScriptedAssetGpuSmokeOptions {
    let usage = "usage: zelda3 --smoke-asset-gpu <path-to-rom.sfc> <frames> [--input-script <path>] [--load-sram <path>] [--load-state <path>]";
    let rom_path = match args.first() {
        Some(path) => path.clone(),
        None => {
            eprintln!("{usage}");
            process::exit(2);
        }
    };
    let frames = match args.get(1).and_then(|s| s.parse().ok()) {
        Some(frames) => frames,
        None => {
            eprintln!("{usage}");
            process::exit(2);
        }
    };
    let mut input_script_path = None;
    let mut load_sram = None;
    let mut load_state = None;
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--input-script" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--input-script requires a path");
                    process::exit(2);
                });
                input_script_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--load-state" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-state requires a path");
                    process::exit(2);
                });
                load_state = Some(PathBuf::from(path));
                i += 2;
            }
            "--load-sram" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                });
                load_sram = Some(PathBuf::from(path));
                i += 2;
            }
            flag => {
                eprintln!("unknown smoke-asset-gpu option: {flag}");
                process::exit(2);
            }
        }
    }
    if load_state.is_some() && load_sram.is_some() {
        eprintln!("--smoke-asset-gpu cannot combine --load-sram with --load-state");
        process::exit(2);
    }
    ScriptedAssetGpuSmokeOptions {
        rom_path,
        frames,
        input_script_path,
        load_sram,
        load_state,
    }
}

pub(crate) fn run_smoke_asset_gpu(args: &[String]) {
    let options = parse_scripted_asset_gpu_smoke_options(args);
    let input_script = match options.input_script_path.as_deref() {
        Some(path) => match InputScript::from_path(path) {
            Ok(script) => script,
            Err(e) => {
                eprintln!("failed to parse input script {}: {e}", path.display());
                process::exit(2);
            }
        },
        None => InputScript::default(),
    };
    let (mut game, start_frame) =
        load_play_or_checkpoint(&options.rom_path, options.load_state.as_deref());
    if let Some(path) = options.load_sram.as_deref() {
        let sram = read_file_or_exit(path, "SRAM");
        apply_sram_to_game_or_exit(&mut game, path, &sram);
    }
    let renderer = match ModernAssetGpuReadbackRenderer::load_from_env() {
        Ok(renderer) => renderer,
        Err(e) => {
            eprintln!("failed to initialize modern asset GPU readback: {e}");
            process::exit(1);
        }
    };
    for frame_no in 0..options.frames {
        let absolute_frame = start_frame.wrapping_add(frame_no);
        let input = input_script.input_for_frame(absolute_frame);
        game.zelda_run_frame(input as i32);
        if let Err(e) = renderer.render_game_rgba(&mut game) {
            eprintln!(
                "asset GPU smoke failed frame={} absolute_frame={} input=0x{input:04x}: {e}",
                frame_no.wrapping_add(1),
                absolute_frame.wrapping_add(1),
            );
            process::exit(1);
        }
    }
    println!(
        "asset GPU smoke passed frames={} start_frame={} end_frame={} main={:02x}; sub={:02x}; mode={}; screen={:02x}/{:02x}; cgram_nonzero={}; oam_nonzero={}",
        options.frames,
        start_frame,
        start_frame.wrapping_add(options.frames),
        game.ram[0x10],
        game.ram[0x11],
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.cgram.iter().filter(|&&v| v != 0).count(),
        game.ppu.oam.iter().filter(|&&v| v != 0).count(),
    );
}

pub(crate) fn run_dump_frame(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --dump-frame <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--load-state <path>]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = match args.get(1).and_then(|s| s.parse().ok()) {
        Some(frames) => frames,
        None => {
            eprintln!(
                "usage: zelda3 --dump-frame <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--load-state <path>]"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(2) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-frame <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--load-state <path>]"
            );
            process::exit(2);
        }
    };
    let mut input_script = InputScript::default();
    let mut load_sram = None;
    let mut load_state = None;
    let mut i = 3usize;
    while i < args.len() {
        match args[i].as_str() {
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
            "--load-state" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-state requires a path");
                    process::exit(2);
                });
                load_state = Some(PathBuf::from(path));
                i += 2;
            }
            "--load-sram" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                });
                load_sram = Some(PathBuf::from(path));
                i += 2;
            }
            flag => {
                eprintln!("unknown dump-frame option: {flag}");
                process::exit(2);
            }
        }
    }
    if load_state.is_some() && load_sram.is_some() {
        eprintln!("--dump-frame cannot combine --load-sram with --load-state");
        process::exit(2);
    }

    let (mut game, start_frame) = load_play_or_checkpoint(rom_path, load_state.as_deref());
    if let Some(path) = load_sram.as_deref() {
        let sram = read_file_or_exit(path, "SRAM");
        apply_sram_to_game_or_exit(&mut game, path, &sram);
    }
    let width = 256u32;
    let height = 224u32;
    for frame_no in 0..frames {
        let input = input_script.input_for_frame(start_frame.wrapping_add(frame_no));
        game.zelda_run_frame(input as i32);
    }
    let rgba = match render_live_game_gpu_frame_rgba(&mut game, width, height) {
        Ok(rgba) => rgba,
        Err(e) => {
            eprintln!("failed to render dump frame via modern asset GPU path: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = write_rgba_frame_png(&out_path, &rgba, width, height) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "dumped frame {frames} to {}; main={:02x}; sub={:02x}; mode={}; screen={:02x}/{:02x}; cgram_nonzero={}; oam_nonzero={}",
        out_path.display(),
        game.ram[0x10],
        game.ram[0x11],
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.cgram.iter().filter(|&&v| v != 0).count(),
        game.ppu.oam.iter().filter(|&&v| v != 0).count(),
    );
}

pub(crate) fn run_dump_overworld_screen(args: &[String]) {
    let rom_path = match args.first() {
        Some(path) => path,
        None => {
            eprintln!("usage: zelda3 --dump-overworld-screen <path-to-rom.sfc> <screen> <out.png>");
            process::exit(2);
        }
    };
    let screen = match args.get(1).and_then(|s| parse_u16_auto(s)) {
        Some(screen) => screen,
        None => {
            eprintln!("usage: zelda3 --dump-overworld-screen <path-to-rom.sfc> <screen> <out.png>");
            process::exit(2);
        }
    };
    let out_path = match args.get(2) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("usage: zelda3 --dump-overworld-screen <path-to-rom.sfc> <screen> <out.png>");
            process::exit(2);
        }
    };

    let mut game = load_translated_replay_state(rom_path);
    let loaded = game.parity_probe_overworld_screen(screen);
    let width = 256u32;
    let height = 224u32;
    let rgba = match render_live_game_gpu_frame_rgba(&mut game, width, height) {
        Ok(rgba) => rgba,
        Err(e) => {
            eprintln!("failed to render overworld screen via modern asset GPU path: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = write_rgba_frame_png(&out_path, &rgba, width, height) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "dumped overworld screen requested=0x{screen:04x} loaded=0x{loaded:04x} to {}; mode={}; screen={:02x}/{:02x}; bg1_tm={:04x}; bg1_chr={:04x}; bg2_tm={:04x}; bg2_chr={:04x}",
        out_path.display(),
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.bg_layer[0].tilemap_adr,
        game.ppu.bg_layer[0].tile_adr,
        game.ppu.bg_layer[1].tilemap_adr,
        game.ppu.bg_layer[1].tile_adr,
    );
}

pub(crate) fn run_scan_replay_checkpoints(args: &[String]) {
    let rom_path = match args.first() {
        Some(path) => path,
        None => {
            eprintln!(
                "usage: zelda3 --scan-replay-checkpoints <path-to-rom.sfc> <checkpoint-dir> [screen]"
            );
            process::exit(2);
        }
    };
    let checkpoint_dir = match args.get(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --scan-replay-checkpoints <path-to-rom.sfc> <checkpoint-dir> [screen]"
            );
            process::exit(2);
        }
    };
    let target_screen = args.get(2).and_then(|value| parse_u16_auto(value));
    let mut checkpoints = match fs::read_dir(&checkpoint_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sav"))
            .collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("failed to read {}: {e}", checkpoint_dir.display());
            process::exit(1);
        }
    };
    checkpoints.sort();

    for path in checkpoints {
        let mut game = load_translated_replay_state(rom_path);
        if let Err(e) = load_replay_save_checkpoint(&mut game, &path) {
            eprintln!("failed to load {}: {e}", path.display());
            continue;
        }
        let indoors = game.ram[PLAYER_IS_INDOORS] != 0;
        let room = read_le_u16(&game.ram, 0x48e);
        let screen = read_le_u16(&game.ram, 0x8a);
        if target_screen.is_some_and(|target| indoors || screen != target) {
            continue;
        }
        println!(
            "{} indoors={} room=0x{room:04x} ow=0x{screen:04x} main={:02x} sub={:02x} mode={} screen={:02x}/{:02x} bg1_tm={:04x} bg1_chr={:04x} bg2_tm={:04x} bg2_chr={:04x} cgram_nonzero={} vram_nonzero={}",
            path.display(),
            indoors,
            game.ram[0x10],
            game.ram[0x11],
            game.ppu.mode,
            game.ppu.screen_enabled[0],
            game.ppu.screen_enabled[1],
            game.ppu.bg_layer[0].tilemap_adr,
            game.ppu.bg_layer[0].tile_adr,
            game.ppu.bg_layer[1].tilemap_adr,
            game.ppu.bg_layer[1].tile_adr,
            game.ppu.cgram.iter().filter(|&&v| v != 0).count(),
            game.ppu.vram.iter().filter(|&&v| v != 0).count(),
        );
    }
}

pub(crate) fn run_dump_replay_checkpoint_ppu(args: &[String]) {
    let rom_path = match args.first() {
        Some(path) => path,
        None => {
            eprintln!(
                "usage: zelda3 --dump-replay-checkpoint-ppu <path-to-rom.sfc> <checkpoint.sav> <out.png> [frames]"
            );
            process::exit(2);
        }
    };
    let checkpoint_path = match args.get(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-replay-checkpoint-ppu <path-to-rom.sfc> <checkpoint.sav> <out.png> [frames]"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(2) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-replay-checkpoint-ppu <path-to-rom.sfc> <checkpoint.sav> <out.png> [frames]"
            );
            process::exit(2);
        }
    };
    let frames = args
        .get(3)
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    let mut game = load_translated_replay_state(rom_path);
    if let Err(e) = load_replay_save_checkpoint(&mut game, &checkpoint_path) {
        eprintln!("failed to load {}: {e}", checkpoint_path.display());
        process::exit(1);
    }
    let width = 256u32;
    let height = 224u32;
    for _ in 0..frames {
        game.zelda_run_frame(0);
    }
    let rgba = match render_live_game_gpu_frame_rgba(&mut game, width, height) {
        Ok(rgba) => rgba,
        Err(e) => {
            eprintln!("failed to render replay checkpoint via modern asset GPU path: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = write_rgba_frame_png(&out_path, &rgba, width, height) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "dumped replay checkpoint {} frames={frames} to {}; indoors={} ow=0x{:04x} main={:02x} sub={:02x} mode={} screen={:02x}/{:02x} bg1_tm={:04x} bg1_chr={:04x}",
        checkpoint_path.display(),
        out_path.display(),
        game.ram[PLAYER_IS_INDOORS] != 0,
        read_le_u16(&game.ram, 0x8a),
        game.ram[0x10],
        game.ram[0x11],
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.bg_layer[0].tilemap_adr,
        game.ppu.bg_layer[0].tile_adr,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_asset_gpu_options_accept_input_script_and_state() {
        let args = vec![
            "saves/zelda3.sfc".to_string(),
            "12000".to_string(),
            "--input-script".to_string(),
            "scripts/inputs/branch.txt".to_string(),
            "--load-state".to_string(),
            ".cache/branch.sav".to_string(),
        ];

        let options = parse_scripted_asset_gpu_smoke_options(&args);

        assert_eq!(
            options,
            ScriptedAssetGpuSmokeOptions {
                rom_path: "saves/zelda3.sfc".to_string(),
                frames: 12000,
                input_script_path: Some(PathBuf::from("scripts/inputs/branch.txt")),
                load_sram: None,
                load_state: Some(PathBuf::from(".cache/branch.sav")),
            }
        );
    }

    #[test]
    fn smoke_asset_gpu_options_accept_sram_without_script() {
        let args = vec![
            "saves/zelda3.sfc".to_string(),
            "5".to_string(),
            "--load-sram".to_string(),
            "route.srm".to_string(),
        ];

        let options = parse_scripted_asset_gpu_smoke_options(&args);

        assert_eq!(
            options,
            ScriptedAssetGpuSmokeOptions {
                rom_path: "saves/zelda3.sfc".to_string(),
                frames: 5,
                input_script_path: None,
                load_sram: Some(PathBuf::from("route.srm")),
                load_state: None,
            }
        );
    }
}
