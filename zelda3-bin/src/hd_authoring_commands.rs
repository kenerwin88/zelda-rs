use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process;

use crate::gpu_capture::render_hd_capture_from_game;
use crate::image_output::{decode_rgba_png, write_rgba_frame_png};
use crate::load_translated_replay_state;

fn write_reference_palette_png(
    path: &str,
    cgram_rgba: &[[u8; 4]; 256],
) -> Result<(), Box<dyn Error>> {
    let mut rgba = Vec::with_capacity(cgram_rgba.len() * 4);
    for px in cgram_rgba {
        rgba.extend_from_slice(px);
    }
    write_rgba_frame_png(Path::new(path), &rgba, 256, 1)
}

pub(crate) fn run_dump_hd_capture(args: &[String]) {
    let targets: Vec<u32> = args.iter().filter_map(|s| s.parse::<u32>().ok()).collect();
    if targets.is_empty() {
        eprintln!("usage: zelda3 --dump-hd-capture <frame> [frame...]");
        process::exit(2);
    }
    let max_frame = *targets.iter().max().unwrap();

    let atlas = match renderer::modern_source_atlas::load_modern_source_atlas(Path::new(".")) {
        Ok(atlas) => atlas,
        Err(e) => {
            eprintln!("failed to load source atlas: {e}");
            process::exit(1);
        }
    };

    const OUT_DIR: &str = "hd_art/capture";
    if let Err(e) = fs::create_dir_all(OUT_DIR) {
        eprintln!("failed to create {OUT_DIR}: {e}");
        process::exit(1);
    }

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

    let mut first_capture = true;
    let mut captured = 0usize;
    let mut completed = game.state_recorder.replay_frame_counter;
    while completed < max_frame && game.state_recorder.replay_mode {
        game.zelda_run_frame_with_replay_input_override(0, None);
        completed = completed.wrapping_add(1);

        if !targets.contains(&completed) {
            continue;
        }

        let capture = match render_hd_capture_from_game(&mut game, &atlas) {
            Ok(Some(capture)) => capture,
            Ok(None) => {
                eprintln!("frame {completed}: Mode 7 placement metadata not supported; skipping");
                continue;
            }
            Err(e) => {
                eprintln!("failed to render HD capture frame {completed} via asset GPU path: {e}");
                process::exit(1);
            }
        };
        let png_path = format!("{OUT_DIR}/frame_{completed}.png");
        if let Err(e) = write_rgba_frame_png(Path::new(&png_path), &capture.rgba, 256, 224) {
            eprintln!("failed to write {png_path}: {e}");
            process::exit(1);
        }

        let map_path = format!("{OUT_DIR}/frame_{completed}.map.json");
        let json = match serde_json::to_vec_pretty(&capture.placements) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("failed to serialize placement map: {e}");
                process::exit(1);
            }
        };
        if let Err(e) = fs::write(&map_path, &json) {
            eprintln!("failed to write {map_path}: {e}");
            process::exit(1);
        }

        if first_capture {
            let pal_path = format!("{OUT_DIR}/reference_palette.png");
            if let Err(e) = write_reference_palette_png(&pal_path, &capture.cgram_rgba) {
                eprintln!("failed to write {pal_path}: {e}");
                process::exit(1);
            }
            first_capture = false;
        }
        captured += 1;
        eprintln!(
            "captured frame {completed}: {} placements",
            capture.placements.len()
        );
    }

    if captured < targets.len() {
        eprintln!(
            "[warn] replay ended at frame {completed} before capturing all {} requested frame(s) ({captured} captured)",
            targets.len()
        );
    }
    println!(
        "dumped hd capture: {captured}/{} frame(s) -> {OUT_DIR}/",
        targets.len()
    );
}

pub(crate) fn run_slice_hd_cells(args: &[String]) {
    use renderer::hd_authoring::{slice_hd_cell, HdPlacement};

    let scale: u32 = args.first().and_then(|s| s.parse().ok()).unwrap_or(4);

    const CAPTURE_DIR: &str = "hd_art/capture";
    const SR_DIR: &str = "hd_art/sr";
    const CELLS_DIR: &str = "hd_art/cells";
    if let Err(e) = fs::create_dir_all(CELLS_DIR) {
        eprintln!("failed to create {CELLS_DIR}: {e}");
        process::exit(1);
    }

    let mut frame_nums: Vec<u32> = match fs::read_dir(CAPTURE_DIR) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name();
                let name = name.to_str()?;
                let n = name.strip_prefix("frame_")?.strip_suffix(".map.json")?;
                n.parse::<u32>().ok()
            })
            .collect(),
        Err(e) => {
            eprintln!("failed to read {CAPTURE_DIR}: {e}");
            process::exit(1);
        }
    };
    frame_nums.sort_unstable();

    let mut written_keys: HashSet<String> = HashSet::new();
    let mut written: Vec<(String, String)> = Vec::new();

    for n in frame_nums {
        let map_path = format!("{CAPTURE_DIR}/frame_{n}.map.json");
        let placements: Vec<HdPlacement> = match fs::read(&map_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        {
            Some(p) => p,
            None => {
                eprintln!("[warn] failed to read/parse {map_path}; skipping frame {n}");
                continue;
            }
        };

        let sr_path = format!("{SR_DIR}/frame_{n}.x{scale}.png");
        let (sr, sr_w, sr_h) = match decode_rgba_png(Path::new(&sr_path)) {
            Some(decoded) => decoded,
            None => {
                eprintln!("[warn] SR frame {sr_path} missing/unreadable; skipping frame {n}");
                continue;
            }
        };

        for p in &placements {
            if written_keys.contains(&p.key) {
                continue;
            }
            if u64::from_str_radix(p.key.trim_start_matches("0x"), 16).is_err() {
                eprintln!("[warn] bad placement key {:?}; skipping", p.key);
                continue;
            }
            let Some(cell) = slice_hd_cell(&sr, sr_w, sr_h, p.x, p.y, p.w, p.h, scale) else {
                continue;
            };
            let rel_path = format!("cells/{}.png", p.key);
            let cell_path = format!("{CELLS_DIR}/{}.png", p.key);
            if let Err(e) = write_rgba_frame_png(
                Path::new(&cell_path),
                &cell,
                p.w as u32 * scale,
                p.h as u32 * scale,
            ) {
                eprintln!("failed to write {cell_path}: {e}");
                process::exit(1);
            }
            written_keys.insert(p.key.clone());
            written.push((p.key.clone(), rel_path));
        }
    }

    #[derive(serde::Serialize)]
    struct OutManifest {
        reference_palette: String,
        overrides: Vec<OutOverride>,
    }
    #[derive(serde::Serialize)]
    struct OutOverride {
        key: String,
        rgba: String,
    }

    let overrides: Vec<OutOverride> = written
        .iter()
        .map(|(k, path)| OutOverride {
            key: k.clone(),
            rgba: path.clone(),
        })
        .collect();
    let manifest = OutManifest {
        reference_palette: "capture/reference_palette.png".into(),
        overrides,
    };
    if let Err(e) = fs::write(
        "hd_art/manifest.json",
        serde_json::to_vec_pretty(&manifest).unwrap(),
    ) {
        eprintln!("failed to write hd_art/manifest.json: {e}");
        process::exit(1);
    }
    println!("wrote {} cells + hd_art/manifest.json", written.len());
}
