//! Optional authoring harness: normal boot and recorded controller input, then
//! a room snapshot or interactive play. It never warps or patches game RAM.
//! Uses the same input parser, GPU capture, and frontend as the main binary.

#![cfg(not(test))]

#[allow(dead_code)]
#[path = "../src/gpu_capture.rs"]
mod gpu_capture;
#[allow(dead_code)]
#[path = "../src/image_output.rs"]
mod image_output;
#[allow(dead_code)]
#[path = "../src/input_script.rs"]
mod input_script;
#[allow(dead_code)]
#[path = "../src/play_renderer.rs"]
mod play_renderer;

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use zelda3::ZeldaState;

fn number(s: &str) -> Result<u32, Box<dyn Error>> {
    Ok(if let Some(s) = s.strip_prefix("0x") {
        u32::from_str_radix(s, 16)?
    } else {
        s.parse()?
    })
}

fn word(ram: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([ram[offset], ram[offset + 1]])
}

fn main() -> Result<(), Box<dyn Error>> {
    let usage = "dungeon-room-playtest --rom ROM --pack PACK --input-script INPUT --frames LIMIT --room ID --out NEW_DIR [--sram SRAM] [--snapshot-frame FRAME] [--play]";
    let mut values = std::collections::HashMap::new();
    let mut play = false;
    let mut args = std::env::args().skip(1);
    while let Some(key) = args.next() {
        if key == "--play" {
            play = true;
        } else if [
            "--rom",
            "--pack",
            "--input-script",
            "--frames",
            "--room",
            "--out",
            "--sram",
            "--snapshot-frame",
        ]
        .contains(&key.as_str())
        {
            let value = args.next().ok_or(usage)?;
            if values.insert(key, value).is_some() {
                return Err("duplicate option".into());
            }
        } else {
            return Err(usage.into());
        }
    }
    let required = |key: &str| values.get(key).map(String::as_str).ok_or(usage);
    let room = number(required("--room")?)?;
    if room >= 0x140 {
        return Err("room must be in 0..0x140".into());
    }
    let limit = number(required("--frames")?)?;
    let snapshot = values
        .get("--snapshot-frame")
        .map(|s| number(s))
        .transpose()?;
    if snapshot.is_some_and(|frame| frame >= limit) {
        return Err("snapshot frame must be below the frame limit".into());
    }
    let out = PathBuf::from(required("--out")?);
    let input = input_script::InputScript::from_path(required("--input-script")?)?;
    let rom = fs::read(required("--rom")?)?;
    let pack = fs::read(required("--pack")?)?;
    let mut game = ZeldaState::new();
    game.set_rom_startup_timing(true);
    game.set_rom(&rom);
    game.set_assets(&pack)?;
    game.zelda_configure_audio(44100, 100, false, None);
    game.zelda_set_language(None);
    if let Some(path) = values.get("--sram") {
        let sram = fs::read(path)?;
        if sram.len() != game.sram.len() {
            return Err("SRAM size mismatch".into());
        }
        game.sram.copy_from_slice(&sram);
    }
    // New directory only; never read or write the user's normal save files.
    fs::create_dir_all(out.parent().unwrap_or_else(|| std::path::Path::new(".")))?;
    fs::create_dir(&out)?;
    let mut transitions = Vec::new();
    let mut previous = None;
    let mut reached = None;
    for frame in 0..limit {
        if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            game.zelda_run_frame(i32::from(input.input_for_frame(frame)));
        })) {
            let message = panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("engine panic");
            fs::write(
                out.join("failure.json"),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "frame": frame, "error": message, "options": values,
                    "main": game.ram[0x10], "sub": game.ram[0x11],
                    "room": word(&game.ram, 0xa0), "parity_proof": false,
                }))?,
            )?;
            fs::write(
                out.join("transitions.json"),
                serde_json::to_vec_pretty(&transitions)?,
            )?;
            return Err(format!("engine failed at frame {frame}: {message}").into());
        }
        let location = (game.ram[0x10], game.ram[0x1b], word(&game.ram, 0xa0));
        if previous != Some(location) {
            println!(
                "frame={frame} main={} indoors={} room=0x{:03x}",
                location.0, location.1, location.2
            );
            transitions.push(serde_json::json!({"frame": frame, "main": location.0,
                "indoors": location.1, "room": location.2}));
            previous = Some(location);
        }
        let at_requested_room = location.1 == 1 && location.2 == room as u16;
        let capture_now = if let Some(snapshot) = snapshot {
            if frame == snapshot && !at_requested_room {
                return Err("requested room is not loaded at the requested snapshot frame".into());
            }
            frame == snapshot
        } else {
            at_requested_room && location.0 == 7 && game.ram[0x11] == 0
        };
        if capture_now {
            reached = Some(frame);
            break;
        }
    }
    fs::write(
        out.join("transitions.json"),
        serde_json::to_vec_pretty(&transitions)?,
    )?;
    let frame = reached
        .ok_or("requested room was not reached through normal play within the frame limit")?;
    fs::write(out.join("wram.bin"), &game.ram)?;
    let sprites = (0..16)
        .map(|i| {
            serde_json::json!({
                "slot": i, "state": game.ram[0xdd0 + i], "id": game.ram[0xe20 + i],
                "x": u16::from(game.ram[0xd10 + i]) | (u16::from(game.ram[0xd30 + i]) << 8),
                "y": u16::from(game.ram[0xd00 + i]) | (u16::from(game.ram[0xd20 + i]) << 8),
            })
        })
        .collect::<Vec<_>>();
    let receipt = serde_json::json!({"frame": frame, "room": room,
        "main": game.ram[0x10], "sub": game.ram[0x11],
        "player_x": word(&game.ram, 0x22), "player_y": word(&game.ram, 0x20),
        "sprites": sprites, "options": values,
        "normal_boot": true, "warped": false, "parity_proof": false,
        "snapshot_only": snapshot.is_some()});
    fs::write(out.join("room.json"), serde_json::to_vec_pretty(&receipt)?)?;
    // Share the parity tools' GPU exclusion lock; never overlap their captures.
    let gpu_lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open("/tmp/zelda3-snes9x-compare.lock")?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        // SAFETY: flock only inspects this open file descriptor. Closing the
        // owned file at exit releases the lock, including on error paths.
        if unsafe { libc::flock(gpu_lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err("another GPU/parity run is active; retry after it finishes".into());
        }
    }
    let rgba = gpu_capture::render_live_game_gpu_frame_rgba(&mut game, 256, 224)?;
    image_output::write_rgba_frame_png(&out.join("room.png"), &rgba, 256, 224)?;
    println!(
        "reached room 0x{room:03x} at frame {frame}; wrote {}",
        out.display()
    );
    if play {
        let mut frontend = play_renderer::configured_from_env(
            256,
            224,
            platform::NativeFrontendOptions::from_env(3, true),
        )?;
        frontend.set_window_title("Dungeon room playtest (temporary session)");
        let mut audio = vec![0; frontend.audio_samples_per_frame() * frontend.audio_channels()];
        while !frontend.quit_requested() {
            let input = frontend.poll_input();
            game.zelda_run_frame(i32::from(input));
            frontend.present_frame(&mut game);
            let samples = frontend.audio_samples_per_frame();
            let channels = frontend.audio_channels();
            game.zelda_render_audio(&mut audio, samples as i32, channels as i32);
            frontend.push_audio(&audio);
            game.zelda_discard_unused_audio_frames();
            frontend.wait_idle();
        }
    }
    Ok(())
}
