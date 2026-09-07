//! Split out of `snes9x_compare.rs` by topic (replay_bundle). Mechanical move:
//! bodies are unchanged; private items became `pub(crate)` and `super::`
//! paths became `crate::` (the parent's super is the crate root).

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplayBundle {
    pub(crate) dir: PathBuf,
    pub(crate) input_script: PathBuf,
    pub(crate) rom_random_script: PathBuf,
    pub(crate) initial_sram: PathBuf,
    pub(crate) frames_completed: u32,
    pub(crate) manifest_sha256: String,
    pub(crate) input_sha256: String,
    pub(crate) rom_random_sha256: String,
    pub(crate) initial_sram_sha256: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReplayBundleManifest {
    pub(crate) schema: u32,
    pub(crate) frames_completed: Option<u32>,
    pub(crate) rom: ReplayBundleRom,
    #[serde(default)]
    pub(crate) rom_random_replay: Option<ReplayBundleArtifact>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReplayBundleRom {
    pub(crate) sha256: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReplayBundleArtifact {
    #[serde(default)]
    pub(crate) sha256: Option<String>,
}

pub(crate) fn resolve_replay_bundle(
    dir: &Path,
    frames: u32,
    rom_path: &Path,
) -> Result<ReplayBundle, String> {
    let manifest_path = dir.join("manifest.json");
    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest: ReplayBundleManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    if manifest.schema != 1 {
        return Err(format!(
            "{} has unsupported replay-bundle schema {}",
            manifest_path.display(),
            manifest.schema
        ));
    }
    let frames_completed = manifest.frames_completed.ok_or_else(|| {
        format!(
            "{} has no frames_completed receipt; the replay bundle is incomplete or still running",
            manifest_path.display()
        )
    })?;
    if frames > frames_completed {
        return Err(format!(
            "replay bundle {} is proven through frame {frames_completed}, but this run requests frame {frames}; choose a bundle with sufficient coverage",
            dir.display()
        ));
    }

    let actual_rom_sha256 = parity::runner::sha256_file(rom_path)
        .map_err(|error| format!("failed to hash ROM {}: {error}", rom_path.display()))?;
    if !manifest.rom.sha256.eq_ignore_ascii_case(&actual_rom_sha256) {
        return Err(format!(
            "replay bundle {} belongs to ROM {}, but {} hashes to {}",
            dir.display(),
            manifest.rom.sha256,
            rom_path.display(),
            actual_rom_sha256
        ));
    }

    let required_file = |name: &str| -> Result<PathBuf, String> {
        let path = dir.join(name);
        if !path.is_file() {
            return Err(format!(
                "replay bundle {} is missing required artifact {name}",
                dir.display()
            ));
        }
        Ok(path)
    };
    let input_script = required_file("input.txt")?;
    let rom_random_script = required_file("rom-random.txt")?;
    let initial_sram = required_file("initial.srm")?;
    let rom_random_sha256 = parity::runner::sha256_file(&rom_random_script).map_err(|error| {
        format!(
            "failed to hash replay bundle artifact {}: {error}",
            rom_random_script.display()
        )
    })?;
    if let Some(expected) = manifest
        .rom_random_replay
        .as_ref()
        .and_then(|artifact| artifact.sha256.as_deref())
    {
        if !expected.eq_ignore_ascii_case(&rom_random_sha256) {
            return Err(format!(
                "replay bundle {} has a modified rom-random.txt: manifest says {expected}, file hashes to {rom_random_sha256}",
                dir.display()
            ));
        }
    }

    Ok(ReplayBundle {
        dir: dir.to_path_buf(),
        input_sha256: parity::runner::sha256_file(&input_script).map_err(|error| {
            format!(
                "failed to hash replay bundle artifact {}: {error}",
                input_script.display()
            )
        })?,
        initial_sram_sha256: parity::runner::sha256_file(&initial_sram).map_err(|error| {
            format!(
                "failed to hash replay bundle artifact {}: {error}",
                initial_sram.display()
            )
        })?,
        manifest_sha256: parity::runner::sha256_file(&manifest_path).map_err(|error| {
            format!(
                "failed to hash replay bundle manifest {}: {error}",
                manifest_path.display()
            )
        })?,
        rom_random_sha256,
        input_script,
        rom_random_script,
        initial_sram,
        frames_completed,
    })
}

pub(crate) fn validate_replay_source_parents(
    sources: &[(&str, Option<&Path>)],
    allow_mixed: bool,
) -> Result<(), String> {
    if allow_mixed {
        return Ok(());
    }
    let mut first = None::<(&str, PathBuf)>;
    for (name, path) in sources {
        let Some(path) = path else {
            continue;
        };
        let canonical = fs::canonicalize(path)
            .map_err(|error| format!("failed to resolve {name} {}: {error}", path.display()))?;
        let parent = canonical.parent().ok_or_else(|| {
            format!(
                "cannot determine replay provenance directory for {name} {}",
                path.display()
            )
        })?;
        if let Some((first_name, first_parent)) = first.as_ref() {
            if parent != first_parent {
                return Err(format!(
                    "mixed replay provenance is unsafe: {first_name} comes from {}, but {name} comes from {}; use --replay-bundle <dir>, or pass --allow-mixed-replay-provenance only for an intentional diagnostic",
                    first_parent.display(),
                    parent.display()
                ));
            }
        } else {
            first = Some((name, parent.to_path_buf()));
        }
    }
    Ok(())
}

pub(crate) fn replay_save_recorded_frames(path: &Path) -> Result<u32, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    if bytes.len() < 8 {
        return Err("replay save is shorter than its 8-byte version/frame header".to_string());
    }
    let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    if version != 1 {
        return Err(format!(
            "unsupported replay save version {version}; expected 1"
        ));
    }
    let frames = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if frames == 0 {
        return Err("replay save contains no frames".to_string());
    }
    Ok(frames)
}

/// Validate that an unmodified Snes9x oracle can finish a recorded route.
///
/// `ZeldaState` is used only to parse the replay container's input commands.
/// It does not execute gameplay, render video, or synthesize audio in this
/// mode; every recorded controller state is fed directly to Snes9x.
pub(crate) fn run_validate_snes9x_replay(args: &[String]) {
    let (core_path, rom_path, replay_path, sram_path) = match (
        args.first(),
        args.get(1),
        args.get(2),
        args.get(3),
    ) {
        (Some(core), Some(rom), Some(replay), Some(sram)) => (
            core.as_str(),
            rom.as_str(),
            Path::new(replay),
            Path::new(sram),
        ),
        _ => {
            eprintln!(
                    "usage: zelda3 --validate-snes9x-replay <snes9x_libretro.dylib> <rom.sfc> <replay.sav> <sram.dat> [--expected-core-sha256 <sha>] [--expected-rom-sha256 <sha>]"
                );
            process::exit(2);
        }
    };
    let mut expected_core_sha256 = None::<String>;
    let mut expected_rom_sha256 = None::<String>;
    let mut i = 4usize;
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
            flag => {
                eprintln!("unknown --validate-snes9x-replay option: {flag}");
                process::exit(2);
            }
        }
    }

    verify_expected_sha256(core_path, "libretro core", expected_core_sha256.as_deref());
    verify_expected_sha256(rom_path, "ROM", expected_rom_sha256.as_deref());
    let frames = replay_save_recorded_frames(replay_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to read replay save {}: {error}",
            replay_path.display()
        );
        process::exit(2);
    });
    let replay_sha256 = parity::runner::sha256_file(replay_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to hash replay save {}: {error}",
            replay_path.display()
        );
        process::exit(2);
    });
    let sram_sha256 = parity::runner::sha256_file(sram_path).unwrap_or_else(|error| {
        eprintln!("failed to hash SRAM {}: {error}", sram_path.display());
        process::exit(2);
    });
    let sram = read_file_or_exit(sram_path, "SRAM");

    let mut replay_decoder = load_play_state(rom_path);
    replay_decoder
        .replay_save_file(replay_path)
        .unwrap_or_else(|error| {
            eprintln!(
                "failed to load replay save {}: {error}",
                replay_path.display()
            );
            process::exit(2);
        });
    if replay_decoder.state_recorder.total_frames != frames {
        eprintln!(
            "replay header/parser frame count mismatch: header={frames} parser={}",
            replay_decoder.state_recorder.total_frames
        );
        process::exit(2);
    }

    let mut oracle =
        LibretroCore::load_with_sram(core_path, rom_path, Some(&sram)).unwrap_or_else(|error| {
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
    println!(
        "validating Snes9x replay: core={} version={} frames={} replay_sha256={} sram_sha256={}",
        oracle.library_name, oracle.library_version, frames, replay_sha256, sram_sha256,
    );

    LIBRETRO_CAPTURE_ENABLED.store(false, Ordering::Relaxed);
    let mut recorder = std::mem::take(&mut replay_decoder.state_recorder);
    let mut first_credits_frame = None::<u32>;
    let mut final_credits_frame = None::<u32>;
    let mut final_state = [0u8; 3];
    let mut nonzero_input_frames = 0u32;
    let mut input_hash = 0xcbf29ce484222325u64;
    for frame in 0..frames {
        let input = replay_decoder.state_recorder_read_next_replay_state(&mut recorder);
        if input != 0 {
            nonzero_input_frames = nonzero_input_frames.saturating_add(1);
        }
        for byte in input.to_le_bytes() {
            input_hash ^= u64::from(byte);
            input_hash = input_hash.wrapping_mul(0x100000001b3);
        }
        oracle.run_frame_discard_with_input(input);
        let ram = oracle
            .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
            .unwrap_or_else(|| {
                eprintln!("Snes9x did not expose system RAM after frame {frame}");
                process::exit(1);
            });
        if ram.len() <= 0xb0 {
            eprintln!("Snes9x system RAM is too short: {} bytes", ram.len());
            process::exit(1);
        }
        final_state = [ram[0x10], ram[0x11], ram[0xb0]];
        if final_state[0] == 0x1a {
            first_credits_frame.get_or_insert(frame);
            if final_state[1] == 0x26 {
                final_credits_frame.get_or_insert(frame);
            }
        }
        let completed = frame + 1;
        if completed % 100_000 == 0 || completed == frames {
            println!(
                "Snes9x replay progress {completed}/{frames}: module={:02x}/{:02x}/{:02x}",
                final_state[0], final_state[1], final_state[2]
            );
        }
    }
    LIBRETRO_CAPTURE_ENABLED.store(true, Ordering::Relaxed);
    replay_decoder.state_recorder = recorder;

    if replay_decoder.state_recorder.replay_mode {
        eprintln!("replay input stream was not fully consumed after {frames} frames");
        process::exit(1);
    }
    let Some(first_credits_frame) = first_credits_frame else {
        eprintln!(
            "Snes9x did not reach credits module 1A; final module={:02x}/{:02x}/{:02x}",
            final_state[0], final_state[1], final_state[2]
        );
        process::exit(1);
    };
    let Some(final_credits_frame) = final_credits_frame else {
        eprintln!(
            "Snes9x entered credits at frame {first_credits_frame} but did not reach final credits state 1A/26; final module={:02x}/{:02x}/{:02x}",
            final_state[0], final_state[1], final_state[2]
        );
        process::exit(1);
    };
    println!(
        "Snes9x replay validated: consumed={frames} nonzero_input_frames={nonzero_input_frames} input_fnv64={input_hash:016x} credits_first_frame={first_credits_frame} final_credits_frame={final_credits_frame} final_module={:02x}/{:02x}/{:02x}",
        final_state[0], final_state[1], final_state[2]
    );
}

/// Run a native Snes9x boundary forward with a deterministic input script.
///
/// This is deliberately oracle-only: it makes route-wide CPU/NMI/DMA tracing
/// available even when the translated runtime has not reached that boundary
/// yet. The trace core remains controlled by its `ZELDA3_SNES9X_TRACE_*`
/// environment variables.
pub(crate) fn run_snes9x_script(args: &[String]) {
    let (core_path, rom_path, state_path, input_path, frames) = match (
        args.first(),
        args.get(1),
        args.get(2),
        args.get(3),
        args.get(4),
    ) {
        (Some(core), Some(rom), Some(state), Some(input), Some(frames)) => {
            let frames = frames.parse::<u32>().unwrap_or_else(|error| {
                eprintln!("invalid frame count `{frames}`: {error}");
                process::exit(2);
            });
            (
                core.as_str(),
                rom.as_str(),
                Path::new(state),
                Path::new(input),
                frames,
            )
        }
        _ => {
            eprintln!(
                "usage: zelda3 --run-snes9x-script <snes9x_libretro.dylib> <rom.sfc> <oracle.state> <input.txt> <frames> [--load-sram <path>] [--input-frame-offset <n>] [--save-state <path>] [--dump-wram <path>] [--dump-sram <path>] [--expected-core-sha256 <sha>] [--expected-rom-sha256 <sha>]"
            );
            process::exit(2);
        }
    };

    let mut load_sram = None::<PathBuf>;
    let mut input_frame_offset = 0u32;
    let mut save_state = None::<PathBuf>;
    let mut dump_wram = None::<PathBuf>;
    let mut dump_sram = None::<PathBuf>;
    let mut expected_core_sha256 = None::<String>;
    let mut expected_rom_sha256 = None::<String>;
    let mut i = 5usize;
    while i < args.len() {
        match args[i].as_str() {
            "--load-sram" => {
                load_sram = Some(PathBuf::from(args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                })));
                i += 2;
            }
            "--input-frame-offset" => {
                input_frame_offset = args
                    .get(i + 1)
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--input-frame-offset requires an unsigned integer");
                        process::exit(2);
                    });
                i += 2;
            }
            "--save-state" => {
                save_state = Some(PathBuf::from(args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--save-state requires a path");
                    process::exit(2);
                })));
                i += 2;
            }
            "--dump-wram" => {
                dump_wram = Some(PathBuf::from(args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--dump-wram requires a path");
                    process::exit(2);
                })));
                i += 2;
            }
            "--dump-sram" => {
                dump_sram = Some(PathBuf::from(args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--dump-sram requires a path");
                    process::exit(2);
                })));
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
            flag => {
                eprintln!("unknown --run-snes9x-script option: {flag}");
                process::exit(2);
            }
        }
    }

    verify_expected_sha256(core_path, "libretro core", expected_core_sha256.as_deref());
    verify_expected_sha256(rom_path, "ROM", expected_rom_sha256.as_deref());
    let input_script = InputScript::from_path(input_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to parse input script {}: {error}",
            input_path.display()
        );
        process::exit(2);
    });
    let state = read_file_or_exit(state_path, "Snes9x state");
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
    oracle.unserialize_state(&state).unwrap_or_else(|error| {
        eprintln!(
            "failed to restore Snes9x state {}: {error}",
            state_path.display()
        );
        process::exit(2);
    });

    LIBRETRO_CAPTURE_ENABLED.store(false, Ordering::Relaxed);
    for frame in 0..frames {
        let script_frame = input_frame_offset.wrapping_add(frame);
        oracle.run_frame_discard_with_input(input_script.input_for_frame(script_frame));
        let completed = frame + 1;
        if completed % 100_000 == 0 {
            println!("Snes9x script progress {completed}/{frames}");
        }
    }
    LIBRETRO_CAPTURE_ENABLED.store(true, Ordering::Relaxed);

    if let Some(path) = save_state {
        let state = oracle.serialize_state().unwrap_or_else(|error| {
            eprintln!("failed to serialize final Snes9x state: {error}");
            process::exit(1);
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| {
                eprintln!("failed to create {}: {error}", parent.display());
                process::exit(1);
            });
        }
        fs::write(&path, state).unwrap_or_else(|error| {
            eprintln!("failed to write {}: {error}", path.display());
            process::exit(1);
        });
    }

    for (path, memory_id, label) in [
        (dump_wram, RETRO_MEMORY_SYSTEM_RAM, "WRAM"),
        (dump_sram, RETRO_MEMORY_SAVE_RAM, "SRAM"),
    ] {
        let Some(path) = path else {
            continue;
        };
        let bytes = oracle.memory_bytes(memory_id).unwrap_or_else(|| {
            eprintln!("failed to read final Snes9x {label}");
            process::exit(1);
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| {
                eprintln!("failed to create {}: {error}", parent.display());
                process::exit(1);
            });
        }
        fs::write(&path, bytes).unwrap_or_else(|error| {
            eprintln!("failed to write {}: {error}", path.display());
            process::exit(1);
        });
    }

    let ram = oracle
        .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
        .unwrap_or_default();
    let byte = |address: usize| ram.get(address).copied().unwrap_or(0);
    println!(
        "Snes9x script completed {frames} frame(s): module={:02x}/{:02x}/{:02x} room={:04x}",
        byte(0x10),
        byte(0x11),
        byte(0xb0),
        u16::from_le_bytes([byte(0xa0), byte(0xa1)])
    );
}
