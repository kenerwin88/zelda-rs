//! `zelda3 --snes9x-dump-frames CORE ROM SRAM INPUT FRAMES OUT_DIR [HASHES]`
//!
//! Runs the pinned Snes9x core cold from the given SRAM with the route's
//! input script and writes `OUT_DIR/snes9x-<frame>.png` for each listed
//! frame (comma-separated, `lo-hi` ranges allowed; zero-based `retro_run`
//! frames, the cache's frame numbers). Frames in between are run without
//! copying video. With an `oracle-av-hashes.jsonl`, each dumped frame's
//! canonical hash is checked against the cache, so a picture is known to be
//! the authority's before it is used to judge a miss.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::process;
use std::sync::atomic::Ordering;

use crate::image_output::write_rgba_frame_png;
use crate::input_script::InputScript;
use crate::libretro_core::{LIBRETRO_CAPTURE_ENABLED, LibretroCore};
use crate::snes9x_compare::{canonical_oracle_video_digest, snes9x_pixel_stride, snes9x_rgba_pixel_at};

fn parse_frames(spec: &str) -> BTreeSet<u32> {
    let mut frames = BTreeSet::new();
    for part in spec.split(',').filter(|part| !part.is_empty()) {
        let (lo, hi) = part.split_once('-').unwrap_or((part, part));
        let parse = |value: &str| {
            value.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("--snes9x-dump-frames: bad frame {value:?}");
                process::exit(2);
            })
        };
        frames.extend(parse(lo)..=parse(hi));
    }
    frames
}

fn cache_video_hashes(path: &Path, frames: &BTreeSet<u32>) -> HashMap<u32, String> {
    let text = fs::read_to_string(path).unwrap_or_else(|error| {
        eprintln!("--snes9x-dump-frames: read {}: {error}", path.display());
        process::exit(2);
    });
    text.lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|record| {
            let frame = u32::try_from(record["frame"].as_u64()?).ok()?;
            let sha = record["video"]["sha256"].as_str()?.to_string();
            frames.contains(&frame).then_some((frame, sha))
        })
        .collect()
}

pub(crate) fn run_snes9x_dump_frames(args: &[String]) {
    let [core, rom, sram, input, frames, out_dir, rest @ ..] = args else {
        eprintln!("usage: zelda3 --snes9x-dump-frames CORE ROM SRAM INPUT FRAMES OUT_DIR [HASHES]");
        process::exit(2);
    };
    let frames = parse_frames(frames);
    let Some(&last) = frames.last() else {
        eprintln!("--snes9x-dump-frames: no frames");
        process::exit(2);
    };
    let hashes = rest.first().map(|path| cache_video_hashes(Path::new(path), &frames));
    let out_dir = Path::new(out_dir);
    fs::create_dir_all(out_dir).unwrap_or_else(|error| {
        eprintln!("--snes9x-dump-frames: create {}: {error}", out_dir.display());
        process::exit(2);
    });
    let sram = fs::read(sram).unwrap_or_else(|error| {
        eprintln!("--snes9x-dump-frames: read SRAM {sram}: {error}");
        process::exit(2);
    });
    let script = InputScript::from_path(input).unwrap_or_else(|error| {
        eprintln!("--snes9x-dump-frames: read input {input}: {error}");
        process::exit(2);
    });
    let mut core = LibretroCore::load_with_sram(core, rom, Some(&sram)).unwrap_or_else(|error| {
        eprintln!("--snes9x-dump-frames: load core: {error}");
        process::exit(2);
    });
    for frame in 0..=last {
        let input = script.input_for_frame(frame);
        if !frames.contains(&frame) {
            LIBRETRO_CAPTURE_ENABLED.store(false, Ordering::Relaxed);
            core.run_frame_discard_with_input(input);
            continue;
        }
        LIBRETRO_CAPTURE_ENABLED.store(true, Ordering::Relaxed);
        let captured = core.run_frame_with_input(input);
        let Some(stride) = snes9x_pixel_stride(captured.pixel_format) else {
            eprintln!("--snes9x-dump-frames: unsupported pixel format {}", captured.pixel_format);
            process::exit(1);
        };
        let (width, height) = (captured.video_width, captured.video_height);
        let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
        for y in 0..height as usize {
            for x in 0..width as usize {
                let pixel = snes9x_rgba_pixel_at(&captured, y * captured.video_pitch + x * stride).unwrap_or([0; 4]);
                rgba.extend_from_slice(&pixel);
            }
        }
        let path = out_dir.join(format!("snes9x-{frame}.png"));
        write_rgba_frame_png(&path, &rgba, width, height).unwrap_or_else(|error| {
            eprintln!("--snes9x-dump-frames: write {}: {error}", path.display());
            process::exit(1);
        });
        let sha = canonical_oracle_video_digest(&captured)
            .ok()
            .and_then(|digest| digest["sha256"].as_str().map(str::to_string))
            .unwrap_or_default();
        let verdict = match hashes.as_ref().map(|hashes| hashes.get(&frame)) {
            None => "unchecked",
            Some(Some(expected)) if *expected == sha => "cache-match",
            Some(Some(_)) => "CACHE-MISMATCH",
            Some(None) => "not-in-cache",
        };
        println!("snes9x-frame {frame} {verdict} {}", path.display());
    }
}
