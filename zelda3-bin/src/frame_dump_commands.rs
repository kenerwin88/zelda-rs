use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use snes::ppu::PpuRenderFlags;

use crate::image_output::write_argb_frame_png;
use crate::render_diagnostics::{
    render_diagnostic_overworld_screen_bgra, run_diagnostic_play_frame_bgra,
};
use crate::{
    apply_sram_to_game_or_exit, load_play_or_checkpoint, load_replay_save_checkpoint,
    load_translated_replay_state, parse_u16_auto, read_file_or_exit, read_le_u16, InputScript,
    PLAYER_IS_INDOORS,
};

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
    let render_flags = PpuRenderFlags::empty();
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
    let mut frame = vec![0u8; width as usize * height as usize * 4];
    for frame_no in 0..frames {
        let input = input_script.input_for_frame(start_frame.wrapping_add(frame_no));
        run_diagnostic_play_frame_bgra(&mut game, input, &mut frame, render_flags);
    }
    if let Err(e) = write_argb_frame_png(&out_path, &frame, width, height) {
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
    let mut frame = vec![0u8; width as usize * height as usize * 4];
    render_diagnostic_overworld_screen_bgra(&mut game, &mut frame);
    if let Err(e) = write_argb_frame_png(&out_path, &frame, width, height) {
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
    let mut frame = vec![0u8; width as usize * height as usize * 4];
    for _ in 0..frames {
        run_diagnostic_play_frame_bgra(&mut game, 0, &mut frame, PpuRenderFlags::empty());
    }
    if let Err(e) = write_argb_frame_png(&out_path, &frame, width, height) {
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
