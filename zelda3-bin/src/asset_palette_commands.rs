use std::path::Path;
use std::process;

use crate::image_output::write_rgba_frame_png;
use crate::load_translated_replay_state;

/// Replay to a target frame and snapshot the live CGRAM as a 256x1 RGBA
/// "reference palette" PNG - the authoring palette for HD `ArtSidecar`
/// overrides. An artist renders base art under this palette (or upscales it)
/// and ships this same PNG as the sidecar manifest's `reference_palette`; the
/// detail-modulate shader then re-lights HD art through the LIVE CGRAM every
/// frame. Because `detail = override / reference`, art authored as
/// `reference[idx]` gives `detail == 1` -> exact parity at this frame's palette,
/// and graceful recolor as the runtime palette changes.
///
/// NOTE: the shader uses a SINGLE global reference palette (256 entries), so all
/// HD art should be authored under ONE canonical CGRAM state - pick a
/// representative frame for the area/module you are upscaling.
///
/// Usage: `zelda3 --dump-reference-palette <frame> [out.png]` (needs the 7
/// timing-hack env vars, like every replay run). Emits
/// `developer_tilesets/reference_palette.png`.
pub(crate) fn run_dump_reference_palette(args: &[String]) {
    use renderer::tile_atlas::expand_cgram_to_rgba8;

    let target_frame = match args.first().map(|s| s.parse::<u32>()) {
        Some(Ok(f)) => f,
        _ => {
            eprintln!("usage: zelda3 --dump-reference-palette <frame> [out.png]");
            process::exit(2);
        }
    };
    const DEFAULT_OUT: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/reference_palette.png"
    );
    let out_path = args.get(1).map(String::as_str).unwrap_or(DEFAULT_OUT);
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let replay = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );

    let mut game = load_translated_replay_state(rom);
    if let Err(e) = game.replay_save_file(Path::new(replay)) {
        eprintln!("failed to load replay save {replay}: {e}");
        process::exit(1);
    }
    let mut frames = game.state_recorder.replay_frame_counter;
    while frames < target_frame && game.state_recorder.replay_mode {
        game.zelda_run_frame_with_replay_input_override(0, None);
        frames = frames.wrapping_add(1);
    }
    if frames < target_frame {
        eprintln!(
            "[warn] replay ended at frame {frames} before target {target_frame}; \
             capturing CGRAM at {frames}"
        );
    }

    let rgba = expand_cgram_to_rgba8(&game.ppu.cgram);
    if let Err(e) = write_rgba_frame_png(Path::new(out_path), &rgba, 256, 1) {
        eprintln!("failed to write reference palette PNG {out_path}: {e}");
        process::exit(1);
    }
    println!("dumped reference palette frame={frames} -> {out_path} (256x1 RGBA)");
}
