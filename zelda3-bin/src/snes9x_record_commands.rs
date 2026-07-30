//! Interactive Snes9x route recording (boundaries, takes, telemetry).

use crate::*;

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::Ordering;

use crate::snes9x_route_recorder::{
    BoundaryCapture, OracleFrameReceipt, RecorderIdentity, RecorderProject,
};
use crate::snes9x_segment_matrix::{
    milestone_mismatches, milestone_values, MatrixProof, NativeStateSetProof,
};
use platform::{NativeFrontendOptions, RecorderControl};
use zelda3::RUN_MAIN;

/// Play and record the pinned Snes9x oracle without involving Rust gameplay.
///
/// F5 captures a new native boundary. F9/F10 load the previous/next boundary
/// and begin a new take, preserving branch lineage in the project manifest.
pub(crate) fn run_record_snes9x_route(args: &[String]) {
    let (core_path, rom_path, project_dir) = match (args.first(), args.get(1), args.get(2)) {
        (Some(core), Some(rom), Some(project)) => (core.as_str(), rom.as_str(), Path::new(project)),
        _ => {
            eprintln!(
                "usage: zelda3 --record-snes9x-route <snes9x_libretro.dylib> <rom.sfc> <project-dir> [--load-sram <path>] [--start-boundary <number|latest>] [--max-frames <n>] [--allow-core-rollover] [--expected-core-sha256 <sha>] [--expected-rom-sha256 <sha>]"
            );
            process::exit(2);
        }
    };
    let mut load_sram = None::<PathBuf>;
    let mut start_boundary = None::<String>;
    let mut max_frames = None::<u32>;
    let mut expected_core_sha256 = None::<String>;
    let mut expected_rom_sha256 = None::<String>;
    let mut allow_core_rollover = false;
    let mut i = 3usize;
    while i < args.len() {
        match args[i].as_str() {
            "--load-sram" => {
                load_sram = Some(PathBuf::from(args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                })));
                i += 2;
            }
            "--start-boundary" => {
                start_boundary = Some(
                    args.get(i + 1)
                        .unwrap_or_else(|| {
                            eprintln!("--start-boundary requires a number or latest");
                            process::exit(2);
                        })
                        .clone(),
                );
                i += 2;
            }
            "--max-frames" => {
                max_frames = Some(
                    args.get(i + 1)
                        .and_then(|value| value.parse::<u32>().ok())
                        .unwrap_or_else(|| {
                            eprintln!("--max-frames requires an unsigned integer");
                            process::exit(2);
                        }),
                );
                i += 2;
            }
            "--expected-core-sha256" => {
                expected_core_sha256 = Some(
                    args.get(i + 1)
                        .unwrap_or_else(|| {
                            eprintln!("--expected-core-sha256 requires a hash");
                            process::exit(2);
                        })
                        .clone(),
                );
                i += 2;
            }
            "--expected-rom-sha256" => {
                expected_rom_sha256 = Some(
                    args.get(i + 1)
                        .unwrap_or_else(|| {
                            eprintln!("--expected-rom-sha256 requires a hash");
                            process::exit(2);
                        })
                        .clone(),
                );
                i += 2;
            }
            "--allow-core-rollover" => {
                allow_core_rollover = true;
                i += 1;
            }
            flag => {
                eprintln!("unknown --record-snes9x-route option: {flag}");
                process::exit(2);
            }
        }
    }

    verify_expected_sha256(core_path, "libretro core", expected_core_sha256.as_deref());
    verify_expected_sha256(rom_path, "ROM", expected_rom_sha256.as_deref());
    let core_sha256 = parity::runner::sha256_file(Path::new(core_path)).unwrap_or_else(|error| {
        eprintln!("failed to hash Snes9x core: {error}");
        process::exit(2);
    });
    let rom_sha256 = parity::runner::sha256_file(Path::new(rom_path)).unwrap_or_else(|error| {
        eprintln!("failed to hash ROM: {error}");
        process::exit(2);
    });
    let sram = load_sram
        .as_deref()
        .map(|path| read_file_or_exit(path, "SRAM"));
    let _compare_lock = acquire_snes9x_compare_lock();
    let mut oracle = LibretroCore::load_with_sram(core_path, rom_path, sram.as_deref())
        .unwrap_or_else(|error| {
            eprintln!("failed to initialize Snes9x libretro core: {error}");
            process::exit(1);
        });
    validate_required_libretro_core(
        Some(("Snes9x", "1.63")),
        &oracle.library_name,
        &oracle.library_version,
    )
    .unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(2);
    });
    let identity = RecorderIdentity {
        core_name: oracle.library_name.clone(),
        core_version: oracle.library_version.clone(),
        core_sha256,
        rom_sha256,
    };
    let mut project = RecorderProject::open(project_dir, identity, allow_core_rollover)
        .unwrap_or_else(|error| {
            eprintln!("failed to open recorder project: {error}");
            process::exit(2);
        });
    let width = oracle.geometry.base_width.max(1);
    let height = oracle.geometry.base_height.max(1);
    let mut renderer =
        play_renderer::configured_from_env(width, height, NativeFrontendOptions::from_env(3, true))
            .unwrap_or_else(|error| {
                eprintln!("failed to initialize recorder frontend: {error}");
                process::exit(1);
            });
    LIBRETRO_CAPTURE_ENABLED.store(true, Ordering::Relaxed);

    let mut last_rgba = Vec::new();
    let mut last_width = width;
    let mut last_height = height;
    let mut selected_boundary;
    if project.boundary_count() == 0 {
        last_rgba = vec![0; last_width as usize * last_height as usize * 4];
        for alpha in last_rgba.iter_mut().skip(3).step_by(4) {
            *alpha = 0xff;
        }
        selected_boundary =
            capture_recorder_boundary(&oracle, &mut project, &last_rgba, last_width, last_height)
                .unwrap_or_else(|error| {
                    eprintln!("failed to capture initial recorder boundary: {error}");
                    process::exit(1);
                });
        renderer.present_rgba_frame(&last_rgba, last_width, last_height);
    } else {
        selected_boundary = match start_boundary.as_deref().unwrap_or("latest") {
            "latest" => project.boundary_count() - 1,
            value => value.parse::<usize>().unwrap_or_else(|_| {
                eprintln!("invalid --start-boundary {value:?}; expected a number or latest");
                process::exit(2);
            }),
        };
        restore_recorder_boundary(&mut oracle, &project, selected_boundary).unwrap_or_else(
            |error| {
                eprintln!("failed to restore Snes9x boundary {selected_boundary}: {error}");
                process::exit(1);
            },
        );
        if project.has_pending_oracle_rollover() {
            project
                .commit_oracle_rollover(selected_boundary)
                .unwrap_or_else(|error| {
                    eprintln!("failed to record Snes9x oracle generation rollover: {error}");
                    process::exit(1);
                });
            eprintln!(
                "recorded Snes9x oracle generation rollover from boundary {selected_boundary}"
            );
        }
    }
    let mut take = project
        .begin_take(selected_boundary)
        .unwrap_or_else(|error| {
            eprintln!("failed to begin recorder take: {error}");
            process::exit(1);
        });
    renderer.set_window_title(&format!(
        "Snes9x Oracle Recorder — boundary {selected_boundary} take {take} — F5 save, F9/F10 load"
    ));
    renderer.request_window_attention();
    eprintln!(
        "Snes9x route recorder ready: project={} boundary={} take={} controls=F5 save, F9 previous, F10 next",
        project_dir.display(), selected_boundary, take
    );

    let mut total_frames = 0u32;
    while !renderer.quit_requested() && max_frames.is_none_or(|limit| total_frames < limit) {
        let input = renderer.poll_input();
        if renderer.quit_requested() {
            break;
        }
        let mut restored_boundary = false;
        for control in renderer.drain_recorder_controls() {
            match control {
                RecorderControl::SaveBoundary => {
                    if last_rgba.is_empty() {
                        eprintln!(
                            "recorder has no displayed Snes9x frame to attach to a boundary yet"
                        );
                        continue;
                    }
                    let boundary = capture_recorder_boundary(
                        &oracle,
                        &mut project,
                        &last_rgba,
                        last_width,
                        last_height,
                    )
                    .unwrap_or_else(|error| {
                        eprintln!("failed to capture recorder boundary: {error}");
                        process::exit(1);
                    });
                    project.finish_take(Some(boundary)).unwrap_or_else(|error| {
                        eprintln!("failed to finish recorder take: {error}");
                        process::exit(1);
                    });
                    selected_boundary = boundary;
                    take = project.begin_take(boundary).unwrap_or_else(|error| {
                        eprintln!("failed to begin recorder take: {error}");
                        process::exit(1);
                    });
                    renderer.set_window_title(&format!(
                        "Snes9x Oracle Recorder — boundary {boundary} take {take} — F5 save, F9/F10 load"
                    ));
                    eprintln!("saved Snes9x boundary {boundary}; recording take {take}");
                }
                RecorderControl::LoadPreviousBoundary | RecorderControl::LoadNextBoundary => {
                    let target = match control {
                        RecorderControl::LoadPreviousBoundary => {
                            selected_boundary.saturating_sub(1)
                        }
                        RecorderControl::LoadNextBoundary => {
                            (selected_boundary + 1).min(project.boundary_count().saturating_sub(1))
                        }
                        RecorderControl::SaveBoundary => unreachable!(),
                    };
                    if target == selected_boundary {
                        eprintln!(
                            "boundary {selected_boundary} is already the end of that direction"
                        );
                        continue;
                    }
                    project.finish_take(None).unwrap_or_else(|error| {
                        eprintln!("failed to finish recorder take before reload: {error}");
                        process::exit(1);
                    });
                    restore_recorder_boundary(&mut oracle, &project, target).unwrap_or_else(
                        |error| {
                            eprintln!("failed to restore Snes9x boundary {target}: {error}");
                            process::exit(1);
                        },
                    );
                    selected_boundary = target;
                    take = project.begin_take(target).unwrap_or_else(|error| {
                        eprintln!("failed to begin branched recorder take: {error}");
                        process::exit(1);
                    });
                    renderer.set_window_title(&format!(
                        "Snes9x Oracle Recorder — boundary {target} take {take} — F5 save, F9/F10 load"
                    ));
                    restored_boundary = true;
                    eprintln!("loaded Snes9x boundary {target}; recording branched take {take}");
                    break;
                }
            }
        }
        if restored_boundary {
            continue;
        }

        let capture = oracle.run_frame_with_input(input);
        if !capture.video.is_empty() {
            last_width = capture.video_width.max(1);
            last_height = capture.video_height.max(1);
            last_rgba = snes9x_frame_rgba(&capture).unwrap_or_else(|error| {
                eprintln!(
                    "failed to decode Snes9x video at recorder frame {total_frames}: {error}"
                );
                process::exit(1);
            });
        }
        let ram = oracle
            .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
            .unwrap_or_else(|| {
                eprintln!("Snes9x did not expose system RAM at recorder frame {total_frames}");
                process::exit(1);
            });
        let frame_in_take = project.active_take_frames() as u32;
        project
            .record_frame(OracleFrameReceipt::new(
                frame_in_take,
                input,
                fnv64_bytes(&last_rgba),
                fnv64_i16(&capture.audio),
                capture.audio.len() / 2,
                recorder_telemetry(ram),
            ))
            .unwrap_or_else(|error| {
                eprintln!("failed to record oracle frame receipt: {error}");
                process::exit(1);
            });
        if !capture.audio.is_empty() {
            let host_audio = resample_stereo_frame(
                &capture.audio,
                renderer.audio_samples_per_frame(),
                renderer.audio_channels(),
            );
            renderer.push_audio(&host_audio);
        }
        if !last_rgba.is_empty() {
            renderer.present_rgba_frame(&last_rgba, last_width, last_height);
        }
        total_frames = total_frames.saturating_add(1);
    }
    let closing_boundary = if project.active_take_frames() > 0 && !last_rgba.is_empty() {
        Some(
            capture_recorder_boundary(&oracle, &mut project, &last_rgba, last_width, last_height)
                .unwrap_or_else(|error| {
                    eprintln!("failed to auto-save recorder boundary on close: {error}");
                    process::exit(1);
                }),
        )
    } else {
        None
    };
    project
        .finish_take(closing_boundary)
        .unwrap_or_else(|error| {
            eprintln!("failed to finalize recorder take: {error}");
            process::exit(1);
        });
    if let Some(boundary) = closing_boundary {
        eprintln!("auto-saved Snes9x boundary {boundary} on close");
    }
    eprintln!(
        "Snes9x route recording closed: project={} session_frames={} boundaries={} takes={}",
        project_dir.display(),
        total_frames,
        project.boundary_count(),
        project.take_count(),
    );
}

pub(crate) fn capture_recorder_boundary(
    oracle: &LibretroCore,
    project: &mut RecorderProject,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<usize, String> {
    let state = oracle.serialize_state()?;
    let wram = oracle
        .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
        .ok_or_else(|| "Snes9x did not expose WRAM for boundary capture".to_string())?
        .to_vec();
    let vram = oracle
        .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
        .unwrap_or(&[])
        .to_vec();
    let sram = oracle
        .memory_bytes(RETRO_MEMORY_SAVE_RAM)
        .unwrap_or(&[])
        .to_vec();
    let telemetry = recorder_telemetry(&wram);
    project.capture_boundary(BoundaryCapture {
        state: &state,
        wram: &wram,
        vram: &vram,
        sram: &sram,
        screenshot_rgba: rgba,
        screenshot_width: width,
        screenshot_height: height,
        telemetry,
    })
}

pub(crate) fn restore_recorder_boundary(
    oracle: &mut LibretroCore,
    project: &RecorderProject,
    boundary: usize,
) -> Result<(), String> {
    let state = project.load_boundary_state(boundary)?;
    let sram = project.load_boundary_sram(boundary)?;
    oracle.unserialize_state(&state)?;
    oracle.replace_memory(RETRO_MEMORY_SAVE_RAM, &sram, "SRAM")
}

pub(crate) fn snes9x_frame_rgba(frame: &LibretroFrame) -> Result<Vec<u8>, String> {
    let stride = snes9x_pixel_stride(frame.pixel_format)
        .ok_or_else(|| format!("unsupported Snes9x pixel format {}", frame.pixel_format))?;
    let mut rgba = vec![0u8; frame.video_width as usize * frame.video_height as usize * 4];
    for y in 0..frame.video_height as usize {
        for x in 0..frame.video_width as usize {
            let source = y * frame.video_pitch + x * stride;
            let pixel = snes9x_rgba_pixel_at(frame, source)
                .ok_or_else(|| format!("Snes9x video frame is truncated at ({x}, {y})"))?;
            let destination = (y * frame.video_width as usize + x) * 4;
            rgba[destination..destination + 4].copy_from_slice(&pixel);
        }
    }
    Ok(rgba)
}

pub(crate) fn recorder_telemetry(ram: &[u8]) -> serde_json::Value {
    let byte = |offset: usize| ram.get(offset).copied().unwrap_or(0);
    let word = |offset: usize| u16::from_le_bytes([byte(offset), byte(offset + 1)]);
    serde_json::json!({
        "main": byte(0x10),
        "sub": byte(0x11),
        "subsub": byte(0xb0),
        "frame_counter": byte(0x1a),
        "bg_tile_animation_countdown": word(0xc00d),
        "link_dma_source_offset": word(0xc00f),
        "link_dma_countdown": word(0xc013),
        "link_dma_tile_offset": word(0xc015),
        "saved_module": byte(0x10c),
        "indoors": byte(0x1b),
        "room": word(0xa0),
        "overworld": word(0x8a),
        "x": word(0x22),
        "y": word(0x20),
        "z": word(0x24),
        "direction": byte(0x67),
        "facing": byte(0x2f),
        "player_state": byte(0x5d),
        "health": byte(0xf36d),
        "max_health": byte(0xf36c),
        "magic": byte(0xf36e),
        "equipped_item": byte(0x202),
        "item_in_hand": byte(0x301),
        "progression_flags": word(0xf366),
        "small_keys": byte(0xf36f),
        "follower": byte(0x3cc),
        "music": byte(0x12c),
        "queued_music": byte(0x132),
        "sfx_ambient": byte(0x12d),
        "sfx_1": byte(0x12e),
        "sfx_2": byte(0x12f),
        "ending": byte(0x10) == 0x1a,
        "final_credits": byte(0x10) == 0x1a && byte(0x11) == 0x26,
    })
}

pub(crate) fn fnv64_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn fnv64_i16(samples: &[i16]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for sample in samples {
        for byte in sample.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}

pub(crate) fn resample_stereo_frame(
    input: &[i16],
    output_frames: usize,
    output_channels: usize,
) -> Vec<i16> {
    if input.is_empty() || output_frames == 0 || output_channels == 0 {
        return Vec::new();
    }
    let input_frames = input.len() / 2;
    let mut output = vec![0i16; output_frames * output_channels];
    for frame in 0..output_frames {
        let source_frame = frame.saturating_mul(input_frames) / output_frames;
        let left = input[source_frame.min(input_frames - 1) * 2];
        let right = input[source_frame.min(input_frames - 1) * 2 + 1];
        for channel in 0..output_channels {
            output[frame * output_channels + channel] = if channel & 1 == 0 { left } else { right };
        }
    }
    output
}

/// Build the independent start-state side of the segmented Snes9x matrix.
///
/// Chapter inputs are decoded by `ZeldaState`, but only Snes9x executes them.
/// Every oracle boundary file is produced by libretro `retro_serialize` after
/// the preceding Snes9x chapter; no Rust or translated snapshot bytes are ever
/// loaded into the oracle core. Rust chapter-start checkpoints are written as
/// a separate production-lane artifact for later video/audio comparison.
pub(crate) fn run_build_snes9x_segment_matrix(args: &[String]) {
    let (core_path, rom_path, proof_path, chapter_dir, sram_path, output_dir) = match (
        args.first(),
        args.get(1),
        args.get(2),
        args.get(3),
        args.get(4),
        args.get(5),
    ) {
        (Some(core), Some(rom), Some(proof), Some(chapters), Some(sram), Some(output)) => (
            core.as_str(),
            rom.as_str(),
            Path::new(proof),
            Path::new(chapters),
            Path::new(sram),
            Path::new(output),
        ),
        _ => {
            eprintln!(
                "usage: zelda3 --build-snes9x-segment-matrix <snes9x_libretro.dylib> <rom.sfc> <combined-route-proof.json> <chapter-save-dir> <sram.dat> <output-dir> --expected-core-sha256 <sha> --expected-rom-sha256 <sha> [--oracle-start-dir <native-state-set-dir>] [--replace] [--continue-after-mismatch]"
            );
            process::exit(2);
        }
    };
    let mut expected_core_sha256 = None::<String>;
    let mut expected_rom_sha256 = None::<String>;
    let mut replace = false;
    let mut continue_after_mismatch = false;
    let mut oracle_start_dir = None::<PathBuf>;
    let mut i = 6usize;
    while i < args.len() {
        match args[i].as_str() {
            "--expected-core-sha256" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--expected-core-sha256 requires a hash");
                    process::exit(2);
                };
                expected_core_sha256 = Some(value.clone());
                i += 2;
            }
            "--expected-rom-sha256" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--expected-rom-sha256 requires a hash");
                    process::exit(2);
                };
                expected_rom_sha256 = Some(value.clone());
                i += 2;
            }
            "--replace" => {
                replace = true;
                i += 1;
            }
            "--continue-after-mismatch" => {
                continue_after_mismatch = true;
                i += 1;
            }
            "--oracle-start-dir" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--oracle-start-dir requires a directory");
                    process::exit(2);
                };
                oracle_start_dir = Some(PathBuf::from(value));
                i += 2;
            }
            flag => {
                eprintln!("unknown --build-snes9x-segment-matrix option: {flag}");
                process::exit(2);
            }
        }
    }
    let Some(expected_core_sha256) = expected_core_sha256 else {
        eprintln!("segmented oracle capture requires --expected-core-sha256");
        process::exit(2);
    };
    let Some(expected_rom_sha256) = expected_rom_sha256 else {
        eprintln!("segmented oracle capture requires --expected-rom-sha256");
        process::exit(2);
    };
    verify_expected_sha256(core_path, "libretro core", Some(&expected_core_sha256));
    verify_expected_sha256(rom_path, "ROM", Some(&expected_rom_sha256));

    let proof_bytes = read_file_or_exit(proof_path, "combined route proof");
    let proof = MatrixProof::from_slice(&proof_bytes).unwrap_or_else(|error| {
        eprintln!(
            "invalid combined route proof {}: {error}",
            proof_path.display()
        );
        process::exit(2);
    });
    proof.require_segment_count(13).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(2);
    });
    if proof.total_frames() != 1_073_092 {
        eprintln!(
            "segmented oracle matrix requires 1073092 aggregate frames, proof contains {}",
            proof.total_frames()
        );
        process::exit(2);
    }

    if output_dir.exists() {
        if !replace {
            eprintln!(
                "segmented oracle output already exists: {}; pass --replace to rebuild it",
                output_dir.display()
            );
            process::exit(2);
        }
        fs::remove_dir_all(output_dir).unwrap_or_else(|error| {
            eprintln!(
                "failed to replace segmented oracle output {}: {error}",
                output_dir.display()
            );
            process::exit(1);
        });
    }
    fs::create_dir_all(output_dir).unwrap_or_else(|error| {
        eprintln!(
            "failed to create segmented oracle output {}: {error}",
            output_dir.display()
        );
        process::exit(1);
    });

    let sram = read_file_or_exit(sram_path, "SRAM");
    let core_sha256 = parity::runner::sha256_file(Path::new(core_path)).unwrap();
    let rom_sha256 = parity::runner::sha256_file(Path::new(rom_path)).unwrap();
    let proof_sha256 = parity::runner::sha256_file(proof_path).unwrap();
    let sram_sha256 = parity::runner::sha256_file(sram_path).unwrap();
    let native_state_set = oracle_start_dir.as_ref().map(|directory| {
        let provenance_path = directory.join("provenance.json");
        let provenance = NativeStateSetProof::from_slice(&read_file_or_exit(
            &provenance_path,
            "Snes9x native state provenance",
        ))
        .unwrap_or_else(|error| {
            eprintln!("invalid {}: {error}", provenance_path.display());
            process::exit(2);
        });
        provenance
            .validate_engine_hashes(&core_sha256, &rom_sha256)
            .unwrap_or_else(|error| {
                eprintln!("invalid {}: {error}", provenance_path.display());
                process::exit(2);
            });
        (directory.clone(), provenance_path, provenance)
    });
    let mut receipts = Vec::<serde_json::Value>::new();
    let mut previous_oracle_end = None::<Vec<u8>>;
    let mut stopped_reason = None::<String>;
    let mut native_boundary_states = 0usize;
    let mut route_valid_so_far = true;

    LIBRETRO_CAPTURE_ENABLED.store(false, Ordering::Relaxed);
    let mut oracle =
        LibretroCore::load_with_sram(core_path, rom_path, Some(&sram)).unwrap_or_else(|error| {
            eprintln!("failed to initialize Snes9x for segment matrix: {error}");
            process::exit(1);
        });
    validate_required_libretro_core(
        Some(("Snes9x", "1.63")),
        &oracle.library_name,
        &oracle.library_version,
    )
    .unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(2);
    });
    let core_identity = serde_json::json!({
        "library_name": oracle.library_name,
        "library_version": oracle.library_version,
        "libretro_api_version": oracle.api_version,
    });
    for segment in proof.segments() {
        let chapter_path = chapter_dir.join(segment.source_name);
        let chapter_frames = replay_save_recorded_frames(&chapter_path).unwrap_or_else(|error| {
            eprintln!(
                "failed to read chapter replay {}: {error}",
                chapter_path.display()
            );
            process::exit(2);
        });
        if chapter_frames != segment.frames {
            eprintln!(
                "chapter {} frame count is {chapter_frames}, proof requires {}",
                segment.index + 1,
                segment.frames
            );
            process::exit(2);
        }
        let chapter_sha256 = parity::runner::sha256_file(&chapter_path).unwrap_or_else(|error| {
            eprintln!("failed to hash {}: {error}", chapter_path.display());
            process::exit(1);
        });
        let segment_dir = output_dir.join(format!("segment-{:02}", segment.index + 1));
        fs::create_dir_all(&segment_dir).unwrap_or_else(|error| {
            eprintln!("failed to create {}: {error}", segment_dir.display());
            process::exit(1);
        });

        let (oracle_start, oracle_origin) = if segment.index == 0 {
            (
                oracle.serialize_state().unwrap_or_else(|error| {
                    eprintln!("failed to serialize reset Snes9x state: {error}");
                    process::exit(1);
                }),
                "Snes9x retro_serialize after reset with supplied SRAM",
            )
        } else if let Some((directory, _, state_set)) = native_state_set.as_ref() {
            let state_proof = state_set
                .state_for_segment(segment.index + 1)
                .expect("validated native state set contains every boundary");
            let state_path = directory.join(&state_proof.path);
            let actual_sha256 = parity::runner::sha256_file(&state_path).unwrap_or_else(|error| {
                eprintln!("failed to hash {}: {error}", state_path.display());
                process::exit(2);
            });
            if actual_sha256 != state_proof.sha256 {
                eprintln!(
                    "Snes9x native state hash mismatch for segment {}: expected {}, got {actual_sha256}",
                    segment.index + 1,
                    state_proof.sha256
                );
                process::exit(2);
            }
            native_boundary_states += 1;
            (
                read_file_or_exit(&state_path, "Snes9x native start state"),
                "independently supplied Snes9x libretro retro_serialize state",
            )
        } else if let Some(state) = previous_oracle_end.take() {
            oracle.unserialize_state(&state).unwrap_or_else(|error| {
                eprintln!(
                    "failed to restore Snes9x-native start state for segment {}: {error}",
                    segment.index + 1
                );
                process::exit(1);
            });
            (state, "Snes9x retro_serialize after the preceding segment")
        } else {
            unreachable!("every non-initial chained segment has a preceding Snes9x state")
        };
        if segment.index != 0 && native_state_set.is_some() {
            oracle
                .unserialize_state(&oracle_start)
                .unwrap_or_else(|error| {
                    eprintln!(
                    "failed to restore independently supplied Snes9x state for segment {}: {error}",
                    segment.index + 1
                );
                    process::exit(1);
                });
        }
        let oracle_start_path = segment_dir.join("oracle_start.state");
        fs::write(&oracle_start_path, &oracle_start).unwrap_or_else(|error| {
            eprintln!("failed to write {}: {error}", oracle_start_path.display());
            process::exit(1);
        });
        let oracle_start_sha256 = parity::runner::sha256_file(&oracle_start_path).unwrap();

        let mut rust_start = load_play_state(rom_path);
        rust_start
            .replay_save_file(&chapter_path)
            .unwrap_or_else(|error| {
                eprintln!(
                    "failed to load Rust chapter replay {}: {error}",
                    chapter_path.display()
                );
                process::exit(2);
            });
        if segment.index == 0 {
            apply_sram_to_game_or_exit(&mut rust_start, sram_path, &sram);
        }
        if rust_start.state_recorder.total_frames != segment.frames {
            eprintln!(
                "Rust replay parser frame count for chapter {} is {}, expected {}",
                segment.index + 1,
                rust_start.state_recorder.total_frames,
                segment.frames
            );
            process::exit(2);
        }
        let rust_start_path = segment_dir.join("rust_start.z3state");
        let rust_checkpoint = PlayCrashCheckpoint {
            magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
            host_frame: 0,
            input: 0,
            run_what: RUN_MAIN,
            game: rust_start.clone(),
        };
        fs::write(
            &rust_start_path,
            bincode::serialize(&rust_checkpoint).expect("serialize Rust segment start"),
        )
        .unwrap_or_else(|error| {
            eprintln!("failed to write {}: {error}", rust_start_path.display());
            process::exit(1);
        });
        let rust_start_sha256 = parity::runner::sha256_file(&rust_start_path).unwrap();

        let mut recorder = std::mem::take(&mut rust_start.state_recorder);
        let mut input_hash = 0xcbf29ce484222325u64;
        let mut nonzero_input_frames = 0u32;
        for _ in 0..segment.frames {
            let input = rust_start.state_recorder_read_next_replay_state(&mut recorder);
            if input != 0 {
                nonzero_input_frames = nonzero_input_frames.saturating_add(1);
            }
            for byte in input.to_le_bytes() {
                input_hash ^= u64::from(byte);
                input_hash = input_hash.wrapping_mul(0x100000001b3);
            }
            oracle.run_frame_discard_with_input(input);
        }
        rust_start.state_recorder = recorder;
        if rust_start.state_recorder.replay_mode {
            eprintln!(
                "chapter {} input stream was not fully consumed after {} frames",
                segment.index + 1,
                segment.frames
            );
            process::exit(1);
        }

        let oracle_ram = oracle
            .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
            .unwrap_or_else(|| {
                eprintln!(
                    "Snes9x did not expose WRAM after segment {}",
                    segment.index + 1
                );
                process::exit(1);
            })
            .to_vec();
        let actual = milestone_values(&oracle_ram).unwrap_or_else(|error| {
            eprintln!("failed to read Snes9x milestone: {error}");
            process::exit(1);
        });
        let mismatches =
            milestone_mismatches(&oracle_ram, segment.expected).unwrap_or_else(|error| {
                eprintln!("failed to validate Snes9x milestone: {error}");
                process::exit(1);
            });
        let oracle_end = oracle.serialize_state().unwrap_or_else(|error| {
            eprintln!(
                "failed to serialize Snes9x end state for segment {}: {error}",
                segment.index + 1
            );
            process::exit(1);
        });
        let oracle_end_len = oracle_end.len();
        let oracle_end_path = segment_dir.join("oracle_end.state");
        fs::write(&oracle_end_path, &oracle_end).unwrap_or_else(|error| {
            eprintln!("failed to write {}: {error}", oracle_end_path.display());
            process::exit(1);
        });
        let oracle_end_sha256 = parity::runner::sha256_file(&oracle_end_path).unwrap();
        if segment.index + 1 < 13 && native_state_set.is_none() {
            previous_oracle_end = Some(oracle_end);
            native_boundary_states += 1;
        }

        let passed = mismatches.is_empty();
        let pair_eligible = if native_state_set.is_some() {
            passed
        } else {
            route_valid_so_far && passed
        };
        receipts.push(serde_json::json!({
            "segment": segment.index + 1,
            "name": segment.source_name,
            "frames": segment.frames,
            "cumulative_frames": segment.cumulative_frames,
            "chapter_replay": {
                "path": chapter_path,
                "sha256": chapter_sha256,
                "input_fnv64": format!("{input_hash:016x}"),
                "nonzero_input_frames": nonzero_input_frames,
            },
            "paired_starts": {
                "rust": {"path": rust_start_path, "sha256": rust_start_sha256},
                "oracle": {
                    "path": oracle_start_path,
                    "sha256": oracle_start_sha256,
                    "bytes": oracle_start.len(),
                    "origin": oracle_origin,
                    "converted_from_rust": false,
                },
            },
            "oracle_end": {
                "path": oracle_end_path,
                "sha256": oracle_end_sha256,
                "bytes": oracle_end_len,
            },
            "milestone": {
                "expected": segment.expected,
                "actual": actual,
                "mismatches": mismatches,
                "passed": passed,
            },
            "eligible_for_output_parity": pair_eligible,
        }));
        println!(
            "Snes9x segmented route {:02}/13 frames={} cumulative={} milestone={} module={}",
            segment.index + 1,
            segment.frames,
            segment.cumulative_frames,
            if passed { "passed" } else { "FAILED" },
            actual.get("main").map(String::as_str).unwrap_or("?")
        );
        if !passed {
            route_valid_so_far = false;
            stopped_reason.get_or_insert_with(|| {
                format!(
                    "segment {} missed its Snes9x milestone: {}",
                    segment.index + 1,
                    mismatches.join("; ")
                )
            });
            if !continue_after_mismatch && native_state_set.is_none() {
                break;
            }
        }
    }
    LIBRETRO_CAPTURE_ENABLED.store(true, Ordering::Relaxed);

    let completed_segments = receipts.len();
    let capture_complete = completed_segments == 13;
    let route_eligible = capture_complete && stopped_reason.is_none();
    let manifest = serde_json::json!({
        "schema": 1,
        "kind": "zelda3_snes9x_native_segment_matrix_capture_v1",
        "coverage_label": "segmented coverage",
        "continuous_playthrough": false,
        "oracle_independence": {
            "oracle": "Snes9x libretro",
            "state_creation": "retro_serialize from Snes9x execution only",
            "rust_state_conversion_allowed": false,
            "rust_state_conversion_used": false,
        },
        "status": if route_eligible { "captured" } else { "failed" },
        "core": {
            "path": core_path,
            "sha256": core_sha256,
            "expected_sha256": expected_core_sha256,
            "identity": core_identity,
        },
        "rom": {
            "path": rom_path,
            "sha256": rom_sha256,
            "expected_sha256": expected_rom_sha256,
        },
        "source_proof": {"path": proof_path, "sha256": proof_sha256},
        "native_state_set_provenance": native_state_set.as_ref().map(|(_, path, _)| path),
        "sram": {"path": sram_path, "sha256": sram_sha256},
        "segments": receipts,
        "summary": {
            "expected_segments": 13,
            "completed_segments": completed_segments,
            "expected_native_boundary_states": 12,
            "created_native_boundary_states": native_boundary_states,
            "aggregate_input_frames": proof.total_frames(),
            "capture_complete": capture_complete,
            "route_milestones_passed": route_eligible,
            "eligible_for_segmented_output_parity": route_eligible,
            "verified_video_audio_parity_frames": 0,
            "stopped_reason": stopped_reason,
        },
    });
    let manifest_path = output_dir.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to write {}: {error}", manifest_path.display());
        process::exit(1);
    });
    println!(
        "segmented oracle capture manifest: {}",
        manifest_path.display()
    );
    if !route_eligible {
        eprintln!(
            "segmented oracle capture is not coverage-eligible: {}",
            manifest["summary"]["stopped_reason"]
                .as_str()
                .unwrap_or("capture did not complete")
        );
        process::exit(1);
    }
}
