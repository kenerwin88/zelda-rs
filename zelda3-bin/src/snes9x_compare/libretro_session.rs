//! Split out of `snes9x_compare.rs` by topic (libretro_session). Mechanical move:
//! bodies are unchanged; private items became `pub(crate)` and `super::`
//! paths became `crate::` (the parent's super is the crate root).

use super::*;

pub(crate) fn validate_oracle_av_checkpoint_interval(interval: Option<u32>) -> Result<(), String> {
    if interval.is_some() {
        return Err(
            "oracle A/V checkpoints are unavailable: pinned Snes9x serialization mutates live DSP state; capture from reset without in-run checkpoints"
                .to_string(),
        );
    }
    Ok(())
}

pub(crate) fn validate_cold_evidence_invocation_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(
            "cold-evidence invocation ID must be 1..=128 ASCII letters, digits, '-', '_', or '.'"
                .to_string(),
        );
    }
    Ok(())
}

pub(crate) fn cold_evidence_run_nonce(
    session_dir: &Path,
    invocation_id: &str,
    unix_time_ns: u128,
    process_id: u32,
) -> String {
    let session_dir = fs::canonicalize(session_dir).unwrap_or_else(|_| session_dir.to_path_buf());
    parity::evidence::sha256_bytes(
        format!(
            "zelda3-cold-run-v1\0{unix_time_ns}\0{process_id}\0{}\0{invocation_id}",
            session_dir.to_string_lossy()
        )
        .as_bytes(),
    )
}

pub(crate) fn oracle_name_from_core_path(core_path: &str) -> String {
    let stem = Path::new(core_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("libretro");
    stem.strip_suffix("_libretro").unwrap_or(stem).to_string()
}

pub(crate) fn validate_libretro_frame_window(
    frames: u32,
    compare_from_frame: u32,
) -> Result<(), String> {
    if frames == 0 {
        return Err("libretro parity requires at least one frame".to_string());
    }
    if compare_from_frame >= frames {
        return Err(format!(
            "--compare-from-frame {compare_from_frame} leaves no compared frames in a {frames}-frame route"
        ));
    }
    Ok(())
}

pub(crate) fn resolve_engine_state_compare_start(
    compare_from_frame: u32,
    explicit_engine_start: Option<u32>,
    ignore_engine_state: bool,
) -> Result<Option<u32>, String> {
    if ignore_engine_state && explicit_engine_start.is_some() {
        return Err(
            "--ignore-engine-state cannot be combined with --compare-engine-state-from-frame"
                .to_string(),
        );
    }
    Ok(if ignore_engine_state {
        None
    } else {
        Some(explicit_engine_start.unwrap_or(compare_from_frame))
    })
}

pub(crate) fn first_dsp_write_timing_mismatch(
    rust: &[DspWriteEvent],
    oracle: &[LibretroDspRegisterWrite],
) -> Option<usize> {
    let shared = rust.len().min(oracle.len());
    (0..shared)
        .find(|&index| {
            let rust = rust[index];
            let oracle = oracle[index];
            i32::from(rust.addr) != oracle.register
                || i32::from(rust.value) != oracle.value
                || rust.sample_offset != oracle.output_sample
                || i32::from(rust.timer_cycles) != oracle.dsp_phase
        })
        .or_else(|| (rust.len() != oracle.len()).then_some(shared))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpcClockWitness {
    pub(crate) phase_delta: u8,
    pub(crate) pc: u16,
    pub(crate) opcode: u8,
    pub(crate) rust_tail: usize,
    pub(crate) oracle_tail: usize,
    pub(crate) rust_timer_divider: u8,
    pub(crate) oracle_timer_divider: u8,
}

pub(crate) fn last_spc_clock_witness(
    rust: &[snes::apu::SpcInstructionTrace],
    oracle: &[crate::libretro_core::LibretroSmpInstruction],
) -> Option<SpcClockWitness> {
    // A libretro video-frame boundary can split an SPC instruction differently
    // from the translated host window. Match CPU state near the two tails
    // instead of comparing list indexes, which would turn that harmless split
    // into a false clock divergence.
    const TAIL_SEARCH: usize = 64;
    let rust_start = rust.len().saturating_sub(TAIL_SEARCH);
    let oracle_start = oracle.len().saturating_sub(TAIL_SEARCH);
    let mut best = None::<(usize, SpcClockWitness)>;
    for (rust_index, rust_instruction) in rust.iter().enumerate().skip(rust_start) {
        for (oracle_index, oracle_instruction) in oracle.iter().enumerate().skip(oracle_start) {
            if i32::from(rust_instruction.pc) != oracle_instruction.program_counter
                || i32::from(rust_instruction.opcode) != oracle_instruction.opcode
                || i32::from(rust_instruction.a) != oracle_instruction.a
                || i32::from(rust_instruction.x) != oracle_instruction.x
                || i32::from(rust_instruction.y) != oracle_instruction.y
                || i32::from(rust_instruction.sp) != oracle_instruction.stack_pointer
            {
                continue;
            }
            let rust_tail = rust.len() - 1 - rust_index;
            let oracle_tail = oracle.len() - 1 - oracle_index;
            let tail_distance = rust_tail + oracle_tail;
            let phase_delta = (128i32
                - i32::from(rust_instruction.timer0_cycles)
                - oracle_instruction.timer0_stage1)
                .rem_euclid(128) as u8;
            let witness = SpcClockWitness {
                phase_delta,
                pc: rust_instruction.pc,
                opcode: rust_instruction.opcode,
                rust_tail,
                oracle_tail,
                rust_timer_divider: rust_instruction.timer0_divider,
                oracle_timer_divider: oracle_instruction.timer0_stage2 as u8,
            };
            if best
                .as_ref()
                .is_none_or(|(best_distance, _)| tail_distance < *best_distance)
            {
                best = Some((tail_distance, witness));
            }
        }
    }
    best.map(|(_, witness)| witness)
}

pub(crate) fn validate_required_libretro_core(
    required: Option<(&str, &str)>,
    actual_name: &str,
    actual_version: &str,
) -> Result<(), String> {
    let Some((required_name, required_version)) = required else {
        return Ok(());
    };
    if !actual_name
        .to_ascii_lowercase()
        .contains(&required_name.to_ascii_lowercase())
    {
        return Err(format!(
            "wrong libretro core: expected {required_name}, loaded {actual_name} {actual_version}"
        ));
    }
    if !required_version.is_empty() && !actual_version.starts_with(required_version) {
        return Err(format!(
            "wrong {required_name} version: expected {required_version}, loaded {actual_version}"
        ));
    }
    Ok(())
}

pub(crate) fn verify_expected_sha256(path: &str, label: &str, expected: Option<&str>) {
    if let Err(error) = expected_sha256_matches(Path::new(path), label, expected) {
        eprintln!("{error}");
        process::exit(1);
    }
}

pub(crate) fn expected_sha256_matches(
    path: &Path,
    label: &str,
    expected: Option<&str>,
) -> Result<(), String> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let actual = parity::runner::sha256_file(path)
        .map_err(|error| format!("failed to hash {label} {}: {error}", path.display()))?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!(
            "{label} hash mismatch for replay: expected {expected}, found {actual} at {}",
            path.display()
        ))
    }
}

/// Parse an explicit frame selection for diagnostic artifact capture.
///
/// The comparator intentionally keeps this separate from parity decisions:
/// it only controls which already-observed state boundaries are written to a
/// session directory.  Accepting ranges keeps an early-divergence capture
/// practical without a shell-generated list of thousands of frame numbers.
pub(crate) fn debug_frame_selection_from_env(primary: &str, legacy: Option<&str>) -> Vec<u32> {
    let value = env::var(primary)
        .ok()
        .or_else(|| legacy.and_then(|name| env::var(name).ok()));
    value
        .as_deref()
        .map(parse_debug_frame_selection)
        .unwrap_or_default()
}

pub(crate) fn parse_debug_frame_selection(value: &str) -> Vec<u32> {
    let mut frames = Vec::new();
    for part in value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let range = part
            .split_once("..=")
            .or_else(|| part.split_once('-'))
            .and_then(|(start, end)| {
                Some((
                    start.trim().parse::<u32>().ok()?,
                    end.trim().parse::<u32>().ok()?,
                ))
            });
        match range {
            Some((start, end)) if start <= end => frames.extend(start..=end),
            Some(_) => {}
            None => {
                if let Ok(frame) = part.parse() {
                    frames.push(frame);
                }
            }
        }
    }
    frames.sort_unstable();
    frames.dedup();
    frames
}

pub(crate) fn parse_debug_byte_range(value: &str) -> Option<std::ops::Range<usize>> {
    let (start, end) = value.split_once("..")?;
    let parse = |part: &str| {
        let part = part.trim();
        part.strip_prefix("0x")
            .or_else(|| part.strip_prefix("0X"))
            .map_or_else(
                || part.parse::<usize>().ok(),
                |hex| usize::from_str_radix(hex, 16).ok(),
            )
    };
    let start = parse(start)?;
    let end = parse(end)?;
    (start <= end).then_some(start..end)
}

pub(crate) fn append_u32_range(ranges: &mut Vec<(u32, u32)>, value: u32) {
    if let Some((_, end)) = ranges.last_mut() {
        if value == end.saturating_add(1) {
            *end = value;
            return;
        }
    }
    ranges.push((value, value));
}

pub(crate) fn format_u32_ranges(ranges: &[(u32, u32)]) -> String {
    ranges
        .iter()
        .map(|&(start, end)| {
            if start == end {
                start.to_string()
            } else {
                format!("{start}..{end}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn oracle_preframe_snapshot_required(
    _frame: u32,
    _frames: u32,
    compare_this_frame: bool,
) -> bool {
    // ZELDA3_DIAGNOSTIC_PREFRAME_SNAPSHOTS=1 opts a resumed diagnostic replay
    // into per-frame Snes9x serialization so the first video mismatch writes
    // its detailed pre-frame artifacts without a checkpoint at that exact
    // frame (paired checkpoints do not carry every poly-thread transient, so a
    // focused resume can miss the mismatch). Never set it on authoritative runs:
    // the perturbation below is real.
    if compare_this_frame && diagnostic_preframe_snapshots_enabled() {
        return true;
    }
    // Pinned Snes9x's save path is not observationally pure: SPC_DSP::copy_state
    // canonicalizes live BRR and echo-history storage while serializing. Calling
    // retro_serialize in the authoritative loop can therefore change later APU
    // handshakes and, eventually, the CPU instruction that crosses vblank.
    //
    // Keep the cold/live oracle execution free of in-run save-state calls. The
    // session's initial state and complete input stream remain replayable, and
    // the final state is captured only after comparison has ended. Focused
    // diagnostics that need a pre-frame state must replay with that frame as
    // their start boundary instead of perturbing every preceding host call.
    false
}

pub(crate) fn diagnostic_preframe_snapshots_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        env::var_os("ZELDA3_DIAGNOSTIC_PREFRAME_SNAPSHOTS").is_some_and(|value| value != "0")
    })
}

pub(crate) fn initialize_libretro_session(
    session_dir: Option<&Path>,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    game: &ZeldaState,
    initial_sram: &[u8],
    initial_oracle_state: &[u8],
    frames: u32,
    start_frame: u32,
    compare_from_frame: u32,
    compare_engine_state_from_frame: Option<u32>,
    skip_oracle_frames: u32,
    compare_video: bool,
    compare_audio: bool,
    audio_comparison: AudioComparisonMode,
    timing: AudioTimingOptions,
    replay_save: Option<&Path>,
    replay_bundle: Option<&ReplayBundle>,
    initial_input_script: &[u8],
    rom_random_script: Option<&Path>,
    live_oracle_rng: bool,
    scan_all: bool,
    cold_evidence_invocation_id: Option<&str>,
) -> Option<BufWriter<fs::File>> {
    let dir = session_dir?;
    fs::create_dir_all(dir).unwrap_or_else(|e| {
        eprintln!("failed to create libretro session {}: {e}", dir.display());
        process::exit(1);
    });
    let cold_evidence_run_nonce = cold_evidence_invocation_id.map(|invocation_id| {
        let unix_time_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        cold_evidence_run_nonce(dir, invocation_id, unix_time_ns, process::id())
    });
    for stale in [
        "input.txt",
        "audio_frame_ends.json",
        "audio_report.json",
        "first_audio_mismatch.json",
        "first_audio_mismatch_rust.wav",
        "first_audio_mismatch_oracle.wav",
        "av_hashes.jsonl",
        "oracle_last_before.state",
        "oracle_final.state",
        "rust_final.z3state",
        "result.json",
        "rom-random.txt",
    ] {
        match fs::remove_file(dir.join(stale)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                eprintln!("failed to remove stale libretro session {stale}: {error}");
                process::exit(1);
            }
        }
    }
    // Persist the source controller stream before entering the comparison
    // loop. A later panic (for example a ROM-random order assertion) must still
    // leave a replayable diagnostic session rather than requiring the caller
    // to reconstruct input.txt by hand.
    fs::write(dir.join("input.txt"), initial_input_script).unwrap_or_else(|e| {
        eprintln!("failed to seed libretro session input.txt: {e}");
        process::exit(1);
    });
    fs::write(dir.join("initial.srm"), initial_sram).unwrap_or_else(|e| {
        eprintln!("failed to write libretro session initial.srm: {e}");
        process::exit(1);
    });
    fs::write(dir.join("oracle_initial.state"), initial_oracle_state).unwrap_or_else(|e| {
        eprintln!("failed to write libretro session oracle_initial.state: {e}");
        process::exit(1);
    });
    let rust_initial = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: start_frame,
        input: 0,
        run_what: RUN_MAIN,
        game: game.clone(),
    };
    fs::write(
        dir.join("rust_initial.z3state"),
        bincode::serialize(&rust_initial).expect("serialize initial Rust parity state"),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write libretro session rust_initial.z3state: {e}");
        process::exit(1);
    });
    let core_sha256 = parity::runner::sha256_file(Path::new(core_path)).unwrap_or_else(|e| {
        eprintln!("failed to hash libretro core {core_path}: {e}");
        process::exit(1);
    });
    let rom_sha256 = parity::runner::sha256_file(Path::new(rom_path)).unwrap_or_else(|e| {
        eprintln!("failed to hash ROM {rom_path}: {e}");
        process::exit(1);
    });
    let replay_save_manifest = replay_save.map(|path| {
        let sha256 = parity::runner::sha256_file(path).unwrap_or_else(|e| {
            eprintln!("failed to hash replay save {}: {e}", path.display());
            process::exit(1);
        });
        serde_json::json!({ "path": path, "sha256": sha256 })
    });
    let replay_bundle_manifest = replay_bundle.map(|bundle| {
        serde_json::json!({
            "dir": bundle.dir,
            "frames_completed": bundle.frames_completed,
            "manifest_sha256": bundle.manifest_sha256,
            "input": {
                "artifact": "input.txt",
                "sha256": bundle.input_sha256,
            },
            "rom_random": {
                "artifact": "rom-random.txt",
                "sha256": bundle.rom_random_sha256,
            },
            "initial_sram": {
                "artifact": "initial.srm",
                "sha256": bundle.initial_sram_sha256,
            },
        })
    });
    let rom_random_manifest = rom_random_script.map(|path| {
        let bytes = fs::read(path).unwrap_or_else(|e| {
            eprintln!(
                "failed to read ROM random replay script {}: {e}",
                path.display()
            );
            process::exit(1);
        });
        fs::write(dir.join("rom-random.txt"), &bytes).unwrap_or_else(|e| {
            eprintln!("failed to persist ROM random replay script: {e}");
            process::exit(1);
        });
        let artifact_path = dir.join("rom-random.txt");
        serde_json::json!({
            "source_path": path,
            "artifact": "rom-random.txt",
            "sha256": parity::runner::sha256_file(&artifact_path).unwrap_or_else(|e| {
                eprintln!("failed to hash persisted ROM random replay script: {e}");
                process::exit(1);
            }),
        })
    });
    let mut artifacts = vec![
        "initial.srm",
        "rust_initial.z3state",
        "oracle_initial.state",
        "oracle_last_before.state",
        "input.txt",
        "frame_receipts.jsonl",
        "av_hashes.jsonl",
        "audio_frame_ends.json",
        "audio_report.json",
        "first_audio_mismatch.json",
        "first_audio_mismatch_rust.wav",
        "first_audio_mismatch_oracle.wav",
        "oracle_final.state",
        "rust_final.z3state",
        "result.json",
        "replay.sh",
    ];
    if rom_random_script.is_some() {
        artifacts.push("rom-random.txt");
    }
    if live_oracle_rng {
        artifacts.push(LIVE_ORACLE_RNG_TRACE_ARTIFACT);
    }
    if env::var_os("ZELDA3_CAPTURE_OBJ_STATE_LEDGER").is_some() {
        artifacts.extend([
            "obj_state_ledger.jsonl",
            "obj_state_first_cache_divergence.jsonl",
            "obj_state_first_rust_wram.bin",
            "obj_state_first_oracle_wram.bin",
            "obj_state_first_rust_vram.bin",
            "obj_state_first_oracle_vram.bin",
        ]);
    }
    let manifest = serde_json::json!({
        "schema": 1,
        "status": "running",
        "cold_evidence_invocation_id": cold_evidence_invocation_id,
        "cold_evidence_run_nonce": cold_evidence_run_nonce,
        "core": {
            "path": core_path,
            "sha256": core_sha256,
            "library_name": oracle.library_name,
            "library_version": oracle.library_version,
            "libretro_api_version": oracle.api_version,
        },
        "rom": { "path": rom_path, "sha256": rom_sha256 },
        "replay_save": replay_save_manifest,
        "replay_bundle": replay_bundle_manifest,
        "rom_random_replay": rom_random_manifest,
        "rom_random_authority": if live_oracle_rng {
            serde_json::json!({
                "mode": "live_oracle_trace",
                "artifact": LIVE_ORACLE_RNG_TRACE_ARTIFACT,
                "store_pc_low16": CARTRIDGE_RNG_STORE_PC_LOW16,
            })
        } else if rom_random_script.is_some() {
            serde_json::json!({"mode": "replay_script"})
        } else {
            serde_json::json!({"mode": "translated_fallback"})
        },
        "timing": {
            "fps": oracle.av_info.timing.fps,
            "sample_rate": oracle.av_info.timing.sample_rate,
            "frames_requested": frames,
            "start_frame": start_frame,
            "compare_from_frame": compare_from_frame,
            "fixed_oracle_startup_skip_frames": skip_oracle_frames,
            "dynamic_alignment": false,
        },
        "comparison_lanes": {
            "video": compare_video,
            "audio": compare_audio,
            "engine_state": compare_engine_state_from_frame.is_some(),
            "engine_state_from_frame": compare_engine_state_from_frame,
        },
        "av_hash_ledger": {
            "schema": 1,
            "coverage": "every compared frame for each enabled lane",
            "video_canonicalization": "visible row-major RGB bytes; alpha and libretro row padding excluded",
            "audio_canonicalization": "interleaved stereo signed 16-bit little-endian samples",
        },
        "audio": {
            "comparison": audio_comparison.as_str(),
            "window_sample_frames": timing.window_frames,
            "silence_threshold": timing.silence_threshold,
            "max_timing_error_sample_frames": timing.max_timing_error_frames,
            "max_envelope_error": timing.max_envelope_error,
        },
        "artifacts": artifacts,
    });
    fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write libretro session manifest: {e}");
        process::exit(1);
    });
    let absolute_dir = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let repo_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let asset_pack = repo_root.join("zelda3_assets.dat");
    let feature = "";
    let lane_flags = match (compare_video, compare_audio) {
        (true, true) => "",
        (false, true) => " --ignore-video",
        (true, false) => " --ignore-audio",
        (false, false) => " --ignore-video --ignore-audio",
    };
    let initial_state_flags = if start_frame == 0 && replay_save.is_none() {
        format!(
            "--load-sram {} --skip-oracle-frames {}",
            shell_single_quote(&absolute_dir.join("initial.srm").to_string_lossy()),
            skip_oracle_frames,
        )
    } else {
        format!(
            "--resume-rust-state {} --resume-oracle-state {}",
            shell_single_quote(&absolute_dir.join("rust_initial.z3state").to_string_lossy()),
            shell_single_quote(&absolute_dir.join("oracle_initial.state").to_string_lossy()),
        )
    };
    let rom_random_flags = if rom_random_script.is_some() {
        format!(
            " --rom-random-script {}",
            shell_single_quote(&absolute_dir.join("rom-random.txt").to_string_lossy()),
        )
    } else {
        String::new()
    };
    let live_oracle_rng_flag = if live_oracle_rng {
        " --live-oracle-rng"
    } else {
        ""
    };
    let scan_all_flag = if scan_all { " --scan-all" } else { "" };
    let engine_state_flag = match compare_engine_state_from_frame {
        Some(start) if start != compare_from_frame => {
            format!(" --compare-engine-state-from-frame {start}")
        }
        Some(_) => String::new(),
        None => " --ignore-engine-state".to_string(),
    };
    let replay = format!(
        "#!/bin/sh\nset -eu\ncd {}\nZELDA3_ASSET_PACK={} cargo run -q -p zelda3-bin{} -- --compare-snes9x-oracle {} {} {} --expected-core-sha256 {} --expected-rom-sha256 {} --input-script {}{}{} {} --compare-from-frame {}{}{} --audio-comparison {} --audio-window-ms {} --audio-silence-threshold {} --audio-timing-tolerance-ms {} --audio-envelope-tolerance {} --session-dir {}{}\n",
        shell_single_quote(&repo_root.to_string_lossy()),
        shell_single_quote(&asset_pack.to_string_lossy()),
        feature,
        shell_single_quote(core_path),
        shell_single_quote(rom_path),
        frames,
        core_sha256,
        rom_sha256,
        shell_single_quote(&absolute_dir.join("input.txt").to_string_lossy()),
        rom_random_flags,
        live_oracle_rng_flag,
        initial_state_flags,
        compare_from_frame,
        engine_state_flag,
        lane_flags,
        audio_comparison.as_str(),
        (timing.window_frames as f64 / oracle.av_info.timing.sample_rate) * 1000.0,
        timing.silence_threshold,
        (timing.max_timing_error_frames as f64 / oracle.av_info.timing.sample_rate) * 1000.0,
        timing.max_envelope_error,
        shell_single_quote(&absolute_dir.join("replay").to_string_lossy()),
        scan_all_flag,
    );
    fs::write(dir.join("replay.sh"), replay).unwrap_or_else(|e| {
        eprintln!("failed to write libretro session replay script: {e}");
        process::exit(1);
    });
    let file = fs::File::create(dir.join("frame_receipts.jsonl")).unwrap_or_else(|e| {
        eprintln!("failed to create libretro frame receipt: {e}");
        process::exit(1);
    });
    Some(BufWriter::new(file))
}

pub(crate) fn write_libretro_frame_receipt(
    writer: Option<&mut BufWriter<fs::File>>,
    frame: u32,
    input: u16,
    rust_audio_frames: usize,
    oracle_audio_frames: usize,
    video_width: u32,
    video_height: u32,
    rust_system_ram_before: &[u8],
    rust_system_ram: &[u8],
    rust_selected_game_load_remaining_before: u8,
    rust_selected_game_load_remaining: u8,
    rust_game_execution_scheduler: String,
    rust_poly_work: zelda3::zelda_rtl::PolyWorkMetrics,
    rust_poly_cycles: Option<u64>,
    rust_sfx_clock_checkpoint: (u32, u8, u8),
    rust_spc_driver_clock: Option<String>,
    rust_audio_events: &zelda3::game_output::AudioEventFrame,
    oracle_system_ram: Option<&[u8]>,
    rust_vram: &[u16],
    oracle_vram: Option<&[u8]>,
) {
    let Some(writer) = writer else {
        return;
    };
    let receipt = serde_json::json!({
        "frame": frame,
        "input": format!("0x{input:04x}"),
        "rust_audio_sample_frames": rust_audio_frames,
        "oracle_audio_sample_frames": oracle_audio_frames,
        "oracle_video_width": video_width,
        "oracle_video_height": video_height,
        "rust_engine_before": libretro_engine_state_receipt(rust_system_ram_before),
        "rust_engine": libretro_engine_state_receipt(rust_system_ram),
        "rust_selected_game_load_remaining_before": rust_selected_game_load_remaining_before,
        "rust_selected_game_load_remaining": rust_selected_game_load_remaining,
        "rust_game_execution_scheduler": rust_game_execution_scheduler,
        "rust_poly_work": rust_poly_work,
        "rust_poly_cycles": rust_poly_cycles,
        "rust_sfx_clock_checkpoint": {
            "epoch": rust_sfx_clock_checkpoint.0,
            "timer_cycles": rust_sfx_clock_checkpoint.1,
            "timer_accumulator": rust_sfx_clock_checkpoint.2,
        },
        "rust_spc_driver_clock": rust_spc_driver_clock,
        "rust_audio_command_queue": rust_audio_events.queue,
        "rust_audio_command_ports": rust_audio_events.queue.input,
        "rust_audio_event_count": rust_audio_events.events.len(),
        "rust_audio_event_hash": rust_audio_events.command_hash(),
        "rust_audio_events": rust_audio_events.events,
        "oracle_engine": oracle_system_ram.map(libretro_engine_state_receipt),
        "vram": vram_domain_receipt(rust_vram, oracle_vram),
    });
    serde_json::to_writer(&mut *writer, &receipt).unwrap_or_else(|e| {
        eprintln!("failed to write libretro frame receipt: {e}");
        process::exit(1);
    });
    writer.write_all(b"\n").unwrap_or_else(|e| {
        eprintln!("failed to terminate libretro frame receipt: {e}");
        process::exit(1);
    });
}

pub(crate) fn libretro_engine_state_receipt(ram: &[u8]) -> serde_json::Value {
    let byte = |address: usize| ram.get(address).copied().unwrap_or_default();
    let word =
        |address: usize| u16::from_le_bytes([byte(address), byte(address.saturating_add(1))]);
    let poly_buffer_nonzero_bytes = ram
        .get(0xe800..0xf000)
        .unwrap_or_default()
        .iter()
        .filter(|&&value| value != 0)
        .count();
    let ppu_oam_dma_shadow_hash = ram
        .get(0x0800..0x0a20)
        .unwrap_or_default()
        .iter()
        .fold(2_166_136_261u32, |hash, byte| {
            (hash ^ u32::from(*byte)).wrapping_mul(16_777_619)
        });
    let pending_tilemap_source_offset = word(0x0118);
    let pending_tilemap_source = ram
        .get(0x10000 + usize::from(pending_tilemap_source_offset)..)
        .and_then(|source| source.get(..0x200));
    let pending_tilemap_source_hash = pending_tilemap_source.map(|source| {
        source.iter().fold(2_166_136_261u32, |hash, byte| {
            (hash ^ u32::from(*byte)).wrapping_mul(16_777_619)
        })
    });
    let mut receipt = serde_json::json!({
        "main_module": byte(0x0010),
        "submodule": byte(0x0011),
        "subsubmodule": byte(0x00b0),
        "frame_counter": byte(0x001a),
        "screen_brightness": byte(0x0013),
        "attract_state": byte(0x0022),
        "attract_sequence": byte(0x0023),
        "attract_throne_fade_timer": byte(0x002c),
        "oam_priority_word": word(0x0064),
        "palette_filter_countdown": byte(0xc007),
        "vertical_irq_trigger": byte(0x00ff),
        "nmi_thread_active": byte(0x012a),
        "music_control": byte(0x012c),
        "last_music_control": byte(0x0133),
        "dialogue_message_index": word(0x1cf0),
        "messaging_module": byte(0x1cd8),
        "text_render_state": byte(0x1cd4),
        "vwf_line_speed_cur": byte(0x1cd5),
        "vwf_line_speed": byte(0x1cd6),
        "text_incremental_state": byte(0x1cd7),
        "dialogue_msg_read_pos": word(0x1cd9),
        "dialogue_msg_src_offs": word(0x1cdd),
        "dialogue_scroll_pixel": byte(0x1cdf),
        "text_wait_countdown": word(0x1ce0),
        "text_wait_countdown2": byte(0x1ce9),
        "dialogue_scroll_speed": byte(0x1cea),
        "shared_message_timer": word(0x02cd),
        "crystal_rotation_counter": byte(0x0649),
        "intro_step_index": byte(0x1e00),
        "intro_step_timer": byte(0x1e01),
        "intro_palette_flash_count": byte(0x0ff9),
        "intro_sword_sparkle_timer": byte(0x00ca),
        "intro_sword_sparkle_step": byte(0x00cb),
        "intro_sword_animation_step": byte(0x00cc),
        "intro_did_run_step": byte(0x1f00),
        "pending_polyhedral_update": byte(0x1f0c),
        "poly_config1": byte(0x1f02),
        "poly_angle_a": byte(0x1f04),
        "poly_angle_b": byte(0x1f05),
        "nmi_thread_stack": word(0x1f0a),
        "poly_buffer_nonzero_bytes": poly_buffer_nonzero_bytes,
    });
    receipt.as_object_mut().unwrap().insert(
        "resident_song_bank_kind".to_string(),
        serde_json::Value::from(byte(0x0136)),
    );
    if let Some(map) = receipt.as_object_mut() {
        for (name, value) in [
            ("link_animation_counter", u64::from(byte(0x002d))),
            ("link_animation_step", u64::from(byte(0x002e))),
            ("link_facing", u64::from(byte(0x002f))),
            ("link_last_direction", u64::from(byte(0x0026))),
            ("link_speed_setting", u64::from(byte(0x005e))),
            ("link_num_orthogonal_directions", u64::from(byte(0x006a))),
            ("link_direction_lock", u64::from(byte(0x0050))),
            ("link_direction_bits", u64::from(byte(0x0340))),
            ("link_flag_moving", u64::from(byte(0x034a))),
            ("link_dma_graphics_index", u64::from(word(0x0100))),
            ("link_dma_body_top", u64::from(word(0x0acc))),
            ("link_dma_head_top", u64::from(word(0x0ad0))),
        ] {
            map.insert(name.into(), value.into());
        }
        map.insert("sprite_graphics_index".into(), byte(0x0aa3).into());
        map.insert(
            "sprite_graphics_subsets".into(),
            serde_json::json!([byte(0xc2fc), byte(0xc2fd), byte(0xc2fe), byte(0xc2ff),]),
        );
        map.insert("bg_tile_animation_countdown".into(), word(0xc00d).into());
        map.insert("link_dma_source_offset".into(), word(0xc00f).into());
        map.insert("link_dma_countdown".into(), word(0xc013).into());
        map.insert("link_dma_tile_offset".into(), word(0xc015).into());
        map.insert("bg1_h_copy2".into(), word(0x00e0).into());
        map.insert("bg1_v_copy2".into(), word(0x00e6).into());
        map.insert("move_overlay_counter".into(), byte(0x0494).into());
        map.insert(
            "pending_tilemap_destination_page".into(),
            byte(0x0019).into(),
        );
        map.insert(
            "pending_tilemap_source_offset".into(),
            pending_tilemap_source_offset.into(),
        );
        map.insert(
            "incremental_vram_upload_counter".into(),
            byte(0x0412).into(),
        );
        map.insert(
            "pending_tilemap_source_hash".into(),
            pending_tilemap_source_hash.into(),
        );
        map.insert(
            "pending_tilemap_source_prefix".into(),
            pending_tilemap_source
                .map(|source| source.iter().take(8).copied().collect::<Vec<_>>())
                .into(),
        );
    }

    let object = receipt
        .as_object_mut()
        .expect("engine receipt is an object");
    object.insert(
        "player_equipment_oam_shadow".into(),
        (0x0800 + 112 * 4..0x0800 + 114 * 4)
            .map(byte)
            .collect::<Vec<_>>()
            .into(),
    );
    object.insert(
        "oam_slots".into(),
        (0..128)
            .map(|slot| {
                let base = 0x0800 + slot * 4;
                serde_json::json!({
                    "slot": slot,
                    "x": byte(base),
                    "y": byte(base + 1),
                    "tile": byte(base + 2),
                    "flags": byte(base + 3),
                })
            })
            .collect::<Vec<_>>()
            .into(),
    );
    object.insert(
        "oam_extended".into(),
        (0x0a00..0x0a20).map(byte).collect::<Vec<_>>().into(),
    );
    for (name, value) in [
        ("intro_sword_y", u64::from(word(0x00c8))),
        ("intro_sword_sparkle_y_offset", u64::from(byte(0x00cd))),
        ("nmi_update_latch", u64::from(byte(0x0012))),
        ("nmi_bg_vram_load_mode", u64::from(byte(0x0014))),
        ("nmi_subroutine_index", u64::from(byte(0x0017))),
        ("nmi_load_target_address", u64::from(word(0x0116))),
        ("nmi_core_update_disable", u64::from(byte(0x0710))),
        (
            "ppu_oam_dma_shadow_hash",
            u64::from(ppu_oam_dma_shadow_hash),
        ),
        ("ambient_sound_effect", u64::from(byte(0x012d))),
        ("sound_effect_1", u64::from(byte(0x012e))),
        ("sound_effect_2", u64::from(byte(0x012f))),
        ("queued_music_control", u64::from(byte(0x0132))),
        ("spotlight_window_radius", u64::from(word(0x067c))),
        ("spotlight_window_state", u64::from(word(0x067e))),
        ("dungeon_room_index", u64::from(word(0x00a0))),
        ("dungeon_staircase_index", u64::from(byte(0x0462))),
        ("dungeon_staircase_counter", u64::from(byte(0x0464))),
        ("joypad_high", u64::from(byte(0x00f0))),
        ("joypad_low", u64::from(byte(0x00f2))),
        ("joypad_high_filtered", u64::from(byte(0x00f4))),
        ("joypad_low_filtered", u64::from(byte(0x00f6))),
        ("link_x", u64::from(word(0x0022))),
        ("link_y", u64::from(word(0x0020))),
        ("link_last_direction", u64::from(byte(0x0026))),
        ("link_actual_y_velocity", u64::from(byte(0x0027))),
        ("link_actual_x_velocity", u64::from(byte(0x0028))),
        ("link_y_subpixel", u64::from(byte(0x002a))),
        ("link_x_subpixel", u64::from(byte(0x002b))),
        ("link_animation_counter", u64::from(byte(0x002d))),
        ("link_animation_step", u64::from(byte(0x002e))),
        ("link_direction", u64::from(byte(0x0067))),
        ("link_facing_direction", u64::from(byte(0x002f))),
        ("link_auxiliary_state", u64::from(byte(0x004d))),
        ("link_direction_lock", u64::from(byte(0x0050))),
        ("link_handler_state", u64::from(byte(0x005d))),
        ("link_speed_setting", u64::from(byte(0x005e))),
        ("link_orthogonal_direction_count", u64::from(byte(0x006a))),
        ("link_animation_timer_step", u64::from(byte(0x030a))),
        ("link_somaria_platform_state", u64::from(byte(0x02f5))),
        ("link_facing_direction_mirror", u64::from(byte(0x0323))),
        ("link_movement_direction_bits", u64::from(byte(0x0340))),
        ("link_moving_flag", u64::from(byte(0x034a))),
        ("link_water_ripple_or_grass", u64::from(byte(0x0351))),
        ("link_sort_sprites_offset", u64::from(word(0x0352))),
        ("link_player_oam_value", u64::from(byte(0x0354))),
        ("link_sprite_oam_state_timer", u64::from(byte(0x005c))),
        ("link_item_in_hand", u64::from(byte(0x0301))),
        ("link_state_bits", u64::from(byte(0x0308))),
        ("link_picking_throw_state", u64::from(byte(0x0309))),
        ("link_tile_action", u64::from(byte(0x036c))),
        ("link_lift_x_low", u64::from(byte(0x0368))),
        ("link_lift_x_high", u64::from(byte(0x036a))),
        ("sprite_pickup_slot", u64::from(byte(0x0fb2))),
        ("attract_scene_timer", u64::from(byte(0x0025))),
        ("attract_vram_destination", u64::from(word(0x0030))),
        ("attract_prison_soldier_x", u64::from(byte(0x0034))),
        ("attract_scene_frame_counter", u64::from(byte(0x0050))),
        ("attract_scene_substep", u64::from(byte(0x0060))),
    ] {
        object.insert(name.to_string(), serde_json::Value::from(value));
    }
    object.insert(
        "title_sword_oam_shadow".to_string(),
        serde_json::Value::Array(
            ram.get(0x0948..0x0970)
                .unwrap_or_default()
                .iter()
                .copied()
                .map(serde_json::Value::from)
                .collect(),
        ),
    );
    object.insert(
        "sprite_slots".to_string(),
        serde_json::Value::Array(
            (0..16)
                .map(|slot| {
                    serde_json::json!({
                        "slot": slot,
                        "state": byte(0x0dd0 + slot),
                        "type": byte(0x0e20 + slot),
                        "x": u16::from(byte(0x0d10 + slot)) | (u16::from(byte(0x0d30 + slot)) << 8),
                        "x_subpixel": byte(0x0d70 + slot),
                        "x_velocity": byte(0x0d50 + slot),
                        "y": u16::from(byte(0x0d00 + slot)) | (u16::from(byte(0x0d20 + slot)) << 8),
                        "y_subpixel": byte(0x0d60 + slot),
                        "y_velocity": byte(0x0d40 + slot),
                        "direction": byte(0x0de0 + slot),
                        "head_direction": byte(0x0eb0 + slot),
                        "graphics": byte(0x0dc0 + slot),
                        "ai_state": byte(0x0d80 + slot),
                        "wall_collision": byte(0x0e70 + slot),
                        "subtype": byte(0x0e30 + slot),
                        "subtype2": byte(0x0e80 + slot),
                        "delay_main": byte(0x0df0 + slot),
                        "delay_aux1": byte(0x0e00 + slot),
                    })
                })
                .collect(),
        ),
    );
    object.insert(
        "ancilla_slots".to_string(),
        serde_json::Value::Array(
            (0..10)
                .map(|slot| {
                    serde_json::json!({
                        "slot": slot,
                        "type": byte(0x0c4a + slot),
                        "x": u16::from(byte(0x0c04 + slot)) | (u16::from(byte(0x0c18 + slot)) << 8),
                        "x_subpixel": byte(0x0c40 + slot),
                        "x_velocity": byte(0x0c2c + slot),
                        "y": u16::from(byte(0x0bfa + slot)) | (u16::from(byte(0x0c0e + slot)) << 8),
                        "y_subpixel": byte(0x0c36 + slot),
                        "y_velocity": byte(0x0c22 + slot),
                        "item_to_link": byte(0x0c5e + slot),
                        "timer": byte(0x0c68 + slot),
                        "direction": byte(0x0c72 + slot),
                        "step": byte(0x0c54 + slot),
                        "aux_timer": byte(0x03b1 + slot),
                        "object_priority": byte(0x0280 + slot),
                        "num_sprites": byte(0x0c90 + slot),
                        "tile_attribute": byte(0x03e4 + slot),
                    })
                })
                .collect(),
        ),
    );
    receipt
}

pub(crate) fn oracle_music_route_state(ram: &[u8]) -> Option<[u8; 3]> {
    Some([
        *ram.get(ORACLE_MUSIC_CONTROL)?,
        *ram.get(ORACLE_QUEUED_MUSIC_CONTROL)?,
        *ram.get(ORACLE_LAST_MUSIC_CONTROL)?,
    ])
}

pub(crate) fn finalize_libretro_session(
    session_dir: Option<&Path>,
    writer: Option<&mut BufWriter<fs::File>>,
    av_hash_writer: Option<&mut BufWriter<fs::File>>,
    input_history: &[(u32, u16)],
    source_input_script: Option<&[u8]>,
    audio_report: Option<&libretro_timeline::AudioComparisonReport>,
    audio_frame_ends: &[u64],
    oracle_last_before: &[u8],
    oracle_last_before_frame: u32,
    oracle: &LibretroCore,
    game: &ZeldaState,
    frames: u32,
    video_mismatch_ranges: &[(u32, u32)],
    first_video_mismatch: Option<&str>,
    first_engine_state_mismatch: Option<&(u32, Vec<String>)>,
    diagnostic_probe: bool,
) {
    let Some(dir) = session_dir else {
        return;
    };
    if let Some(writer) = writer {
        writer.flush().unwrap_or_else(|e| {
            eprintln!("failed to flush libretro frame receipts: {e}");
            process::exit(1);
        });
    }
    if let Some(writer) = av_hash_writer {
        writer.flush().unwrap_or_else(|e| {
            eprintln!("failed to flush canonical A/V hash ledger: {e}");
            process::exit(1);
        });
    }
    let input_artifact = replayable_input_artifact(source_input_script, input_history);
    fs::write(dir.join("input.txt"), &input_artifact).unwrap_or_else(|e| {
        eprintln!("failed to write captured controller stream: {e}");
        process::exit(1);
    });
    fs::write(
        dir.join("audio_frame_ends.json"),
        serde_json::to_vec(audio_frame_ends).unwrap(),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write audio frame boundaries: {e}");
        process::exit(1);
    });
    if let Some(report) = audio_report {
        fs::write(
            dir.join("audio_report.json"),
            serde_json::to_vec_pretty(report).unwrap(),
        )
        .unwrap_or_else(|e| {
            eprintln!("failed to write continuous audio report: {e}");
            process::exit(1);
        });
    }
    fs::write(dir.join("oracle_last_before.state"), oracle_last_before).unwrap_or_else(|e| {
        eprintln!("failed to write last pre-frame oracle state: {e}");
        process::exit(1);
    });
    let oracle_final = oracle.serialize_state().unwrap_or_else(|e| {
        eprintln!("failed to serialize final libretro state: {e}");
        process::exit(1);
    });
    fs::write(dir.join("oracle_final.state"), oracle_final).unwrap_or_else(|e| {
        eprintln!("failed to write final libretro state: {e}");
        process::exit(1);
    });
    // Recorded chapter inputs are segment-local. This independently replayed
    // endpoint becomes frame zero when it is paired with the next boundary.
    let rust_final = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: 0,
        input: input_history.last().map(|(_, input)| *input).unwrap_or(0),
        run_what: select_run_what(&game.ram),
        game: game.clone(),
    };
    fs::write(
        dir.join("rust_final.z3state"),
        bincode::serialize(&rust_final).expect("serialize final Rust parity state"),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write final Rust parity state: {e}");
        process::exit(1);
    });
    // `ZELDA3_REPLAY_WRAM_DUMP=<path>` mirrors the replay-save diagnostic in
    // the compare harness: the full 128KB Rust WRAM at the final compared
    // frame, for byte-diffing against the oracle savestate's RAM block.
    if let Some(path) = std::env::var_os("ZELDA3_REPLAY_WRAM_DUMP") {
        if let Err(e) = fs::write(&path, &game.ram[..]) {
            eprintln!("failed to write WRAM dump to {path:?}: {e}");
        }
    }
    let matched = audio_report.map(|report| report.matched).unwrap_or(true)
        && video_mismatch_ranges.is_empty()
        && first_engine_state_mismatch.is_none();
    let parity_eligible = audio_report
        .map(|report| report.mode == AudioComparisonMode::Exact.as_str())
        .unwrap_or(true)
        // A live-RNG run resumed from a paired checkpoint is a fix-loop probe.
        && !diagnostic_probe;
    let status = if !matched {
        "failed"
    } else if parity_eligible {
        "passed"
    } else {
        "diagnostic_passed"
    };
    let result = serde_json::json!({
        "status": status,
        "parity_eligible": parity_eligible,
        "coverage_label": if parity_eligible {
            "exact parity for requested lanes"
        } else {
            "timing diagnostic only; not full parity"
        },
        "frames_completed": frames,
        "audio": audio_report,
        "video": {
            "matched": video_mismatch_ranges.is_empty(),
            "mismatch_ranges": video_mismatch_ranges,
            "first_mismatch": first_video_mismatch,
        },
        "engine_state": {
            "matched": first_engine_state_mismatch.is_none(),
            "first_mismatch": first_engine_state_mismatch.map(|(frame, mismatches)| serde_json::json!({
                "frame": frame,
                "mismatches": mismatches,
            })),
        },
        "dynamic_alignment": false,
        "rust_endpoint": "rust_final.z3state",
    });
    fs::write(
        dir.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write libretro session result: {e}");
        process::exit(1);
    });
    let manifest_path = dir.join("manifest.json");
    let bytes = fs::read(&manifest_path).unwrap_or_else(|error| {
        eprintln!("failed to read libretro session manifest: {error}");
        process::exit(1);
    });
    let mut manifest =
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_else(|error| {
            eprintln!("failed to parse libretro session manifest: {error}");
            process::exit(1);
        });
    manifest["status"] = serde_json::Value::String(status.to_string());
    manifest["parity_eligible"] = serde_json::Value::Bool(parity_eligible);
    manifest["frames_completed"] = serde_json::Value::from(frames);
    manifest["oracle_last_before"] = serde_json::json!({
        "artifact": "oracle_last_before.state",
        "frame": oracle_last_before_frame,
        "policy": "initial checkpoint only; authoritative execution performs no in-run serialization",
    });
    manifest["input_replay"] = serde_json::json!({
        "artifact": "input.txt",
        "mode": if source_input_script.is_some() {
            "source_script"
        } else {
            "captured_history"
        },
        "sha256": parity::evidence::sha256_bytes(&input_artifact),
    });
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap_or_else(
        |error| {
            eprintln!("failed to finalize libretro session manifest: {error}");
            process::exit(1);
        },
    );
}

pub(crate) fn replayable_input_artifact(
    source_input_script: Option<&[u8]>,
    input_history: &[(u32, u16)],
) -> Vec<u8> {
    source_input_script.map_or_else(
        || format_input_history(input_history).into_bytes(),
        <[u8]>::to_vec,
    )
}

pub(crate) fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn compare_libretro_video_frame(
    rust_frame: &[u8],
    rust_width: u32,
    rust_height: u32,
    libretro: &LibretroFrame,
    color_tolerance: u8,
    max_mismatched_pixels: usize,
) -> Option<String> {
    if libretro.video.is_empty() {
        return Some("missing libretro video frame".to_string());
    }
    if libretro.video_width != rust_width || libretro.video_height != rust_height {
        return Some(format!(
            "geometry rust={}x{} libretro={}x{} pitch={} pixel_format={}",
            rust_width,
            rust_height,
            libretro.video_width,
            libretro.video_height,
            libretro.video_pitch,
            libretro.pixel_format
        ));
    }
    let mut mismatched = 0usize;
    let mut first = None;
    // Keep a bounded set of samples in the receipt.  A pixel count and the
    // first coordinate alone cannot distinguish a broad timing failure from a
    // small compositor edge case (for example, a backdrop color-math pixel).
    let mut samples = Vec::with_capacity(4);
    for y in 0..rust_height as usize {
        for x in 0..rust_width as usize {
            let pixel_index = y * rust_width as usize + x;
            let rust_offset = pixel_index * 4;
            let snes9x_offset =
                y * libretro.video_pitch + x * snes9x_pixel_stride(libretro.pixel_format)?;
            let mine = rgba_pixel_at(rust_frame, rust_offset)?;
            let theirs = snes9x_rgba_pixel_at(libretro, snes9x_offset)?;
            if !rgb_within_tolerance(mine, theirs, color_tolerance) {
                mismatched += 1;
                first.get_or_insert((x, y, mine, theirs));
                if samples.len() < 4 {
                    samples.push((x, y, mine, theirs));
                }
            }
        }
    }
    if mismatched <= max_mismatched_pixels {
        return None;
    }
    first.map(|(x, y, mine, theirs)| {
        format!(
            "mismatched_pixels={mismatched}; allowed_mismatched_pixels={max_mismatched_pixels}; color_tolerance={color_tolerance}; first_mismatch=({x}, {y}) rust={mine:02x?} libretro={theirs:02x?}; samples={samples:02x?}; pixel_format={} pitch={}",
            libretro.pixel_format, libretro.video_pitch
        )
    })
}

#[derive(Serialize)]
pub(crate) struct ParityFailureReport {
    pub(crate) kind: String,
    pub(crate) frame: u32,
    pub(crate) input: String,
    pub(crate) run_what: Option<u8>,
    pub(crate) message: String,
    pub(crate) trace_mine: Option<String>,
    pub(crate) trace_theirs: Option<String>,
    pub(crate) ppu_mine: Option<String>,
    pub(crate) ppu_theirs: Option<String>,
    pub(crate) audio_mine: Option<String>,
    pub(crate) audio_theirs: Option<String>,
    pub(crate) artifacts: Vec<String>,
    pub(crate) notes: Vec<String>,
}

/// Seed a fresh translated state from the loaded oracle core's memory (WRAM,
/// VRAM, SRAM via the libretro memory map; CGRAM and OAM via the trace core's
/// debug PPU accessor). Used by `--seed-rust-from-oracle-state` to start a
/// route segment at a Snes9x boundary state without any Rust checkpoint.
pub(crate) fn seed_rust_game_from_oracle_memory(
    game: &mut ZeldaState,
    oracle: &LibretroCore,
    frame: u32,
) -> Result<(), String> {
    let wram = oracle
        .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
        .ok_or("oracle exposes no SYSTEM_RAM")?
        .to_vec();
    let vram = oracle
        .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
        .ok_or("oracle exposes no VIDEO_RAM")?
        .to_vec();
    let sram = oracle
        .memory_bytes(RETRO_MEMORY_SAVE_RAM)
        .ok_or("oracle exposes no SAVE_RAM")?
        .to_vec();
    let cgram = (0..256)
        .map(|index| {
            oracle
                .debug_ppu_value(2, index)
                .and_then(|value| u16::try_from(value).ok())
                .ok_or_else(|| {
                    format!("oracle CGRAM color {index} is unavailable (trace core required)")
                })
        })
        .collect::<Result<Vec<u16>, String>>()?;
    let oam = (0..544)
        .map(|index| {
            oracle
                .debug_ppu_value(15, index)
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| {
                    format!("oracle OAM byte {index} is unavailable (trace core required)")
                })
        })
        .collect::<Result<Vec<u8>, String>>()?;
    // `seed_from_snes9x_oracle_memory` re-arms the live timing owner itself;
    // `restore_live_rom_timing_after_checkpoint` would invalidate it again.
    game.seed_from_snes9x_oracle_memory(&wram, &vram, &sram, &cgram, &oam, frame)?;
    Ok(())
}

pub(crate) fn prune_parity_failure_dirs(root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
        .collect();
    // Directory names start with the unix-seconds timestamp, so the
    // lexicographic order is the chronological order.
    dirs.sort();
    while dirs.len() >= PARITY_FAILURE_DIRS_KEPT {
        let dir = dirs.remove(0);
        let _ = fs::remove_dir_all(&dir);
    }
}

pub(crate) fn create_parity_failure_dir() -> Result<PathBuf, Box<dyn Error>> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let root = PathBuf::from("target").join("parity-failures");
    prune_parity_failure_dirs(&root);
    let dir = root.join(format!("{seconds}-{}", process::id()));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub(crate) fn write_parity_diff(
    dir: &Path,
    report: &ParityFailureReport,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut artifacts = report.artifacts.clone();
    let diff = serde_json::to_string_pretty(report)?;
    fs::write(dir.join("diff.json"), diff)?;
    artifacts.push("diff.json".to_string());
    Ok(artifacts)
}

pub(crate) fn write_wav_i16_stereo(
    path: &Path,
    samples: &[i16],
    sample_rate: u32,
    channels: u16,
) -> Result<(), Box<dyn Error>> {
    let mut file = BufWriter::new(fs::File::create(path)?);
    let data_bytes = (samples.len() * 2) as u32;
    let byte_rate = sample_rate * channels as u32 * 2;
    let block_align = channels * 2;
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_bytes).to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;
    for &sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}

pub(crate) fn write_libretro_parity_failure_artifacts(
    pre_game: Option<&ZeldaState>,
    post_game: &ZeldaState,
    rust_frame_rgba: &[u8],
    rust_audio: &[i16],
    capture: &LibretroFrame,
    oracle_before_state: &[u8],
    oracle_after_state: &[u8],
    rust_before_ram: &[u8],
    input_history: &[(u32, u16)],
    frame: u32,
    input: u16,
    sample_rate: u32,
    oracle_name: &str,
    oracle_system_ram: Option<&[u8]>,
    oracle_before_vram: Option<&[u8]>,
    rendered_display: Option<&crate::gpu_capture::LiveGpuFrameCapture>,
    display_oracle_receipt: Option<&DisplayOracleReceipt>,
    oracle_presented_oam: Option<&[u8]>,
    message: String,
) -> Result<PathBuf, Box<dyn Error>> {
    let dir = create_parity_failure_dir()?;
    fs::write(dir.join("input.txt"), format_input_history(input_history))?;
    fs::write(dir.join("rust_before_ram.bin"), rust_before_ram)?;
    // The comparison loop no longer clones the full pre-frame state every
    // frame; the pre-state artifact exists only when the loop had one on hand
    // (poly frames). input.txt + the initial states reproduce it otherwise.
    if let Some(pre_game) = pre_game {
        let rust_checkpoint = PlayCrashCheckpoint {
            magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
            host_frame: frame,
            input,
            run_what: RUN_MAIN,
            game: pre_game.clone(),
        };
        fs::write(
            dir.join("rust_before.z3state"),
            bincode::serialize(&rust_checkpoint)?,
        )?;
    }
    fs::write(dir.join("oracle_before.state"), oracle_before_state)?;
    fs::write(dir.join("oracle_after.state"), oracle_after_state)?;
    if let Some(oracle_before_ram) = snes9x_state_section(oracle_before_state, b"RAM") {
        fs::write(dir.join("oracle_before_ram.bin"), oracle_before_ram)?;
    }
    if let Some(oracle_before_vram) = oracle_before_vram {
        fs::write(dir.join("oracle_before_vram.bin"), oracle_before_vram)?;
    }
    let rust_after_checkpoint = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: frame.saturating_add(1),
        input,
        run_what: RUN_MAIN,
        game: post_game.clone(),
    };
    fs::write(
        dir.join("rust_after.z3state"),
        bincode::serialize(&rust_after_checkpoint)?,
    )?;
    let rust_vram = post_game
        .ppu
        .vram
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    fs::write(dir.join("rust_after_vram.bin"), &rust_vram)?;
    fs::write(dir.join("rust_after_ram.bin"), &post_game.ram)?;

    // Preserve the exact composed state that both Rust renderers saw.  The
    // live post-frame PPU can already contain registers and memory authored for
    // the following frame, so it is not a reliable description of the failed
    // image by itself.
    let mut visible_obj_vram: Option<Vec<u8>> = None;
    let (visible_ppu_summary, visible_vram, visible_oam, visible_cgram) =
        if let Some(rendered) = rendered_display {
            let ppu = rendered.presented_ppu();
            visible_obj_vram = ppu.obj_vram_latch.as_deref().map(|latch| {
                latch
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>()
            });
            let mut visible_game = post_game.clone();
            visible_game.ppu = ppu.clone();
            (
                format_render_ppu_summary(&visible_game),
                ppu.vram
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>(),
                ppu.oam
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>(),
                ppu.cgram
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>(),
            )
        } else {
            let mut visible_game = post_game.clone();
            visible_game.with_display_snapshot(|display| {
                // OBJ tile fetches read `obj_vram_latch` (the presented OBJ
                // generation composed from receipts) when it is set; keep that
                // view beside the raw VRAM so the attribution tool sees the
                // tiles the scanout actually used (route frame 1155743 showed
                // a phantom Link-tile "presented divergence" without it).
                visible_obj_vram = display.ppu.obj_vram_latch.as_deref().map(|latch| {
                    latch
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .collect::<Vec<_>>()
                });
                (
                    format_render_ppu_summary(display),
                    display
                        .ppu
                        .vram
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .collect::<Vec<_>>(),
                    display
                        .ppu
                        .oam
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .collect::<Vec<_>>(),
                    display
                        .ppu
                        .cgram
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .collect::<Vec<_>>(),
                )
            })
        };
    fs::write(dir.join("rust_visible_vram.bin"), &visible_vram)?;
    if let Some(obj_vram) = visible_obj_vram.as_deref() {
        fs::write(dir.join("rust_visible_obj_vram.bin"), obj_vram)?;
    }
    fs::write(dir.join("rust_visible_oam.bin"), &visible_oam)?;
    if let Some(oracle_presented_oam) = oracle_presented_oam {
        fs::write(dir.join("oracle_presented_oam.bin"), oracle_presented_oam)?;
        fs::write(
            dir.join("oam_generations.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "visible_scanout": summarize_value_domain(&visible_oam, oracle_presented_oam),
            }))?,
        )?;
    }
    fs::write(dir.join("rust_visible_cgram.bin"), &visible_cgram)?;
    if let Some(receipt) = display_oracle_receipt {
        fs::write(
            dir.join("display_oracle.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "receipt": receipt,
                "differences": display_oracle_differences(receipt),
            }))?,
        )?;
    }

    let mut visible_game = post_game.clone();
    if let Some(rendered) = rendered_display {
        visible_game.ppu = rendered.presented_ppu().clone();
    }
    let vram_capture = gpu_capture::capture_gpu_frame_from_game(&mut visible_game);
    let vram_gpu_frame = vram_capture.gpu_frame();
    let vram_modern_frame_rgba =
        renderer::modern_extract::render_modern_frame_full_from_vram(&vram_gpu_frame);
    write_rgba_frame_png(
        &dir.join("rust_modern_vram_frame.png"),
        &vram_modern_frame_rgba,
        256,
        224,
    )?;
    let vram_modern_video_diff =
        compare_snes9x_video_frame(&vram_modern_frame_rgba, 256, 224, capture)
            .unwrap_or_else(|| "exact".to_string());
    fs::write(
        dir.join("modern_vram_video_diff.txt"),
        format!("{vram_modern_video_diff}\n"),
    )?;
    if let Some(oracle_ram) = snes9x_state_section(oracle_after_state, b"RAM") {
        fs::write(dir.join("oracle_after_ram.bin"), oracle_ram)?;
    }
    if let Some(oracle_vram) = snes9x_state_section(oracle_after_state, b"VRA") {
        fs::write(dir.join("oracle_after_vram.bin"), oracle_vram)?;
        let live_after = summarize_value_domain(&rust_vram, oracle_vram);
        let visible_scanout = oracle_before_vram
            .map(|oracle_vram| summarize_value_domain(&visible_vram, oracle_vram));
        fs::write(
            dir.join("vram_diff.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "rust_bytes": live_after.rust_values,
                "oracle_bytes": live_after.oracle_values,
                "mismatched_bytes": live_after.mismatched_values,
                "first_mismatch_byte": live_after.first_mismatch,
                "first_mismatch_word": live_after.first_mismatch.map(|offset| offset / 2),
            }))?,
        )?;
        fs::write(
            dir.join("vram_generations.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "live_after_frame": live_after,
                "visible_scanout": visible_scanout,
            }))?,
        )?;
    }

    write_rgba_frame_png(&dir.join("rust_frame.png"), rust_frame_rgba, 256, 224)?;
    let Some(stride) = snes9x_pixel_stride(capture.pixel_format) else {
        return Err(format!("unsupported libretro pixel format {}", capture.pixel_format).into());
    };
    let mut oracle_argb =
        vec![0u8; capture.video_width as usize * capture.video_height as usize * 4];
    for y in 0..capture.video_height as usize {
        for x in 0..capture.video_width as usize {
            let src = y * capture.video_pitch + x * stride;
            let Some([r, g, b, _]) = snes9x_rgba_pixel_at(capture, src) else {
                return Err(format!("failed to decode libretro pixel at {x},{y}").into());
            };
            let dst = (y * capture.video_width as usize + x) * 4;
            oracle_argb[dst] = b;
            oracle_argb[dst + 1] = g;
            oracle_argb[dst + 2] = r;
            oracle_argb[dst + 3] = 0xff;
        }
    }
    write_argb_frame_png(
        &dir.join("oracle_frame.png"),
        &oracle_argb,
        capture.video_width,
        capture.video_height,
    )?;
    write_wav_i16_stereo(&dir.join("rust_audio.wav"), rust_audio, sample_rate, 2)?;
    write_wav_i16_stereo(
        &dir.join("oracle_audio.wav"),
        &capture.audio,
        sample_rate,
        2,
    )?;

    let report = ParityFailureReport {
        kind: format!("libretro-{oracle_name}"),
        frame,
        input: format!("0x{input:04x}"),
        run_what: None,
        message,
        trace_mine: Some(TraceState::from_ram(&post_game.ram, input, RUN_MAIN).to_string()),
        trace_theirs: oracle_system_ram
            .map(|ram| TraceState::from_ram(ram, input, RUN_MAIN).to_string()),
        ppu_mine: Some(visible_ppu_summary),
        ppu_theirs: None,
        audio_mine: Some(summarize_audio_samples(rust_audio)),
        audio_theirs: Some(summarize_audio_samples(&capture.audio)),
        artifacts: vec![
            "input.txt".to_string(),
            "rust_before.z3state".to_string(),
            "rust_after.z3state".to_string(),
            "oracle_before.state".to_string(),
            "oracle_after.state".to_string(),
            "oracle_before_ram.bin".to_string(),
            "rust_before_ram.bin".to_string(),
            "oracle_before_vram.bin".to_string(),
            "rust_after_vram.bin".to_string(),
            "oracle_after_vram.bin".to_string(),
            "rust_after_ram.bin".to_string(),
            "oracle_after_ram.bin".to_string(),
            "rust_visible_vram.bin".to_string(),
            "rust_visible_oam.bin".to_string(),
            "rust_visible_cgram.bin".to_string(),
            "display_oracle.json".to_string(),
            "vram_diff.json".to_string(),
            "vram_generations.json".to_string(),
            "rust_frame.png".to_string(),
            "rust_classic_frame.png".to_string(),
            "classic_video_diff.txt".to_string(),
            "rust_modern_vram_frame.png".to_string(),
            "modern_vram_video_diff.txt".to_string(),
            "oracle_frame.png".to_string(),
            "rust_audio.wav".to_string(),
            "oracle_audio.wav".to_string(),
            "diff.json".to_string(),
        ],
        notes: vec![
            "oracle_before.state is the exact libretro state immediately before the failing frame"
                .to_string(),
            "rust_before_ram.bin is the translated runtime WRAM at that same pre-frame boundary"
                .to_string(),
            "oracle_before_vram.bin is the VRAM generation that produced the failing Snes9x scanout"
                .to_string(),
            "oracle_after.state and rust_after.z3state are the exact post-frame states used for the rendered comparison"
                .to_string(),
            "input.txt contains the complete controller stream from the synchronized start"
                .to_string(),
            "trace_theirs is decoded from the oracle core's exposed post-frame SNES WRAM"
                .to_string(),
            "ppu_mine and rust_visible_*.bin describe the composed display snapshot actually rendered, not the live post-frame state"
                .to_string(),
            "display_oracle.json classifies post-frame register, CGRAM, live OAM, presented OAM, and raster differences automatically"
                .to_string(),
            "vram_generations.json separates live post-frame VRAM from the generation that produced the failed scanout"
                .to_string(),
        ],
    };
    let _ = write_parity_diff(&dir, &report)?;
    Ok(dir)
}

pub(crate) fn snes9x_state_section<'a>(state: &'a [u8], tag: &[u8; 3]) -> Option<&'a [u8]> {
    let start = state
        .windows(4)
        .position(|window| window[..3] == tag[..] && window[3] == b':')?;
    let length_start = start + 4;
    let length_end = state[length_start..]
        .iter()
        .position(|byte| *byte == b':')?
        + length_start;
    let length = std::str::from_utf8(&state[length_start..length_end])
        .ok()?
        .parse::<usize>()
        .ok()?;
    let data_start = length_end + 1;
    state.get(data_start..data_start.checked_add(length)?)
}
