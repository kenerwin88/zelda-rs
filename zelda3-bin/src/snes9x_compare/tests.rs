use super::{
    append_smp_instruction_frame, cached_ledger_input, cached_oracle_checkpoint_sources,
    canonical_audio_digest, canonical_oracle_video_digest, canonical_rust_video_digest,
    checkpoint_member, cold_evidence_run_nonce, compact_byte_snapshot,
    compact_delta_integer_sequence, compact_delta_integer_sequence_with_zstd, compact_dma_ledger,
    compact_engine_state_mismatches, compact_framed_smp_instructions,
    compact_ordinal_cpu_apu_accesses, decode_snes9x_presented_obj_tiles,
    first_dsp_write_timing_mismatch, first_nmi_apui_anchor_indices, first_nmi_dma_ledger_slice,
    first_nmi_dma_setup_initial_sram_provenance, first_nmi_dma_setup_stop_index,
    first_nmi_dma_transaction_slice, first_nmi_return_start_index, fnv1a32,
    install_directory_atomically, last_spc_clock_witness, libretro_engine_state_receipt,
    oracle_preframe_snapshot_required, oracle_rng_sample_from_trace_line, paired_resume_paths,
    parse_debug_frame_selection, parse_paired_resume_capture, parse_rolling_paired_resume_capture,
    presented_video_rows_match_prior_surface, prune_rolling_paired_resume_captures,
    read_snes9x_retro_run_trace, replayable_input_artifact, resolve_engine_state_compare_start,
    resolve_replay_bundle, rolling_capture_frame_after, scan_all_policy,
    semantic_receipts_from_dma_ledger, semantic_trace_authority_available,
    should_render_video_frame, should_stop_after_first_mismatch, should_write_frame_receipt,
    smp_bootstrap_handoff_index, snes9x_presented_scanline_for_video_y,
    summarize_presented_obj_cache, summarize_value_domain, trace_events_with_rom_rng,
    validate_cold_evidence_invocation_id, validate_first_nmi_return_cpu_slice,
    validate_oracle_av_checkpoint_interval, validate_oracle_rng_samples_for_run,
    validate_paired_resume_provenance, validate_paired_resume_sram_selection,
    validate_replay_source_parents, vram_domain_receipt, write_cached_av_final_paired_resume,
    write_file_atomically, BootBoundaryState, FramedApuPortAccess, FramedCpuTimingTransaction,
    FramedSmpInstruction, OrdinalApuPortAccess, PairedResumeCapture, PendingFirstNmiReturnFixture,
    PlayCrashCheckpoint, PresentedOracleVideo, RollingPairedResumeCapture, ValueDomainDiff,
    VramDomainReceipt, PAIRED_RESUME_SCHEMA, PLAY_CRASH_CHECKPOINT_MAGIC,
};
use crate::libretro_core::{
    LibretroApuPortWrite, LibretroCpuTimingTransaction, LibretroDmaLedgerEvent,
    LibretroDspRegisterWrite, LibretroFrame, LibretroSmpInstruction,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zelda3::{game_output::DspWriteEvent, OriginalTimingSemanticReceipt, RomRandomSample};

#[test]
fn obsolete_native_bank_checkpoints_are_rejected_before_positional_decode() {
    let root = std::env::temp_dir().join(format!(
        "zelda3-native-bank-checkpoint-layout-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    for old_magic in [b"Z3RSPC01", b"Z3RSPC02", b"Z3RSPC03", b"Z3RSPC04", b"Z3RSPC05"] {
        let path = root.join(String::from_utf8_lossy(old_magic).as_ref());
        // A header alone cannot deserialize as a checkpoint. The useful layout
        // error must be returned before attempting the obsolete positional body.
        fs::write(&path, old_magic).unwrap();
        let error = crate::snes9x_apu_tools::load_play_crash_checkpoint(&path)
            .err()
            .expect("old checkpoint layout must be rejected")
            .to_string();
        assert!(error.contains("does not match this binary"), "{error}");
        assert!(
            error.contains("re-create it with the current binary"),
            "{error}"
        );
    }
    fs::remove_dir_all(root).unwrap();
}

fn write_paired_resume_test_generation(
    root: &Path,
    directory: &str,
    manifest_frame: u32,
    rust_checkpoint_frame: u32,
) -> PathBuf {
    let checkpoint_dir = root.join(directory);
    fs::create_dir_all(&checkpoint_dir).unwrap();
    let rust_checkpoint = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: rust_checkpoint_frame,
        input: 0,
        run_what: 0,
        game: zelda3::ZeldaState::new(),
    };
    let rust_state = bincode::serialize(&rust_checkpoint).unwrap();
    let oracle_state = b"oracle state";
    let original_timing = b"original timing sidecar";
    let semantic_trace = b"semantic trace sidecar";
    let initial_sram = b"initial SRAM";
    for (name, bytes) in [
        ("rust.z3state", rust_state.as_slice()),
        ("oracle.state", oracle_state.as_slice()),
        ("original-timing.resume.json", original_timing.as_slice()),
        ("semantic-trace.checkpoint.json", semantic_trace.as_slice()),
        ("initial.srm", initial_sram.as_slice()),
    ] {
        fs::write(checkpoint_dir.join(name), bytes).unwrap();
    }
    let manifest = serde_json::json!({
        "schema": PAIRED_RESUME_SCHEMA,
        "boundary": "pre-frame",
        "frame": manifest_frame,
        "rust_state": {
            "artifact": "rust.z3state",
            "sha256": parity::evidence::sha256_bytes(&rust_state),
        },
        "oracle_state": {
            "artifact": "oracle.state",
            "sha256": parity::evidence::sha256_bytes(oracle_state),
        },
        "original_timing_resume_checkpoint": {
            "artifact": "original-timing.resume.json",
            "sha256": parity::evidence::sha256_bytes(original_timing),
        },
        "semantic_trace_checkpoint": {
            "artifact": "semantic-trace.checkpoint.json",
            "sha256": parity::evidence::sha256_bytes(semantic_trace),
        },
        "core": {"sha256": "core-sha"},
        "rom": {"sha256": "rom-sha"},
        "input_script": null,
        "rom_random_script": null,
        "initial_sram": {
            "artifact": "initial.srm",
            "sha256": parity::evidence::sha256_bytes(initial_sram),
        },
    });
    fs::write(
        checkpoint_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    checkpoint_dir
}

#[test]
fn matched_cached_av_replay_writes_a_quiescent_paired_frontier() {
    let root = std::env::temp_dir().join(format!(
        "zelda3-cached-av-paired-frontier-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let cache = root.join("cache");
    let output = root.join("output");
    fs::create_dir_all(&cache).unwrap();
    fs::create_dir_all(&output).unwrap();
    let oracle = b"canonical final oracle state";
    let semantic_trace = b"canonical semantic trace checkpoint";
    fs::write(cache.join("oracle_final.state"), oracle).unwrap();
    fs::write(
        cache.join("semantic-trace-final.checkpoint.json"),
        semantic_trace,
    )
    .unwrap();
    let initial_sram = b"canonical startup SRAM";
    let initial_sram_sha256 = parity::evidence::sha256_bytes(initial_sram);
    fs::write(cache.join("initial.srm"), initial_sram).unwrap();
    let manifest = serde_json::json!({
        "cache_key": "fixture-key",
        "cache_identity": {
            "core_sha256": "core-sha",
            "source_artifact_sha256": {
                "input.txt": "input-sha",
                "rom-random.txt": "rng-sha",
                "initial.srm": initial_sram_sha256.clone(),
            },
        },
        "artifact_sha256": {
            "oracle_final.state": parity::evidence::sha256_bytes(oracle),
            "semantic-trace-final.checkpoint.json": parity::evidence::sha256_bytes(semantic_trace),
        },
    });
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    let game = super::load_default_play_state();

    let frontier = write_cached_av_final_paired_resume(
        &cache,
        &output,
        &manifest,
        &manifest_bytes,
        Path::new("zelda3.sfc"),
        "rom-sha",
        52_000,
        &game,
    )
    .unwrap();
    let (rust_state, oracle_state, original_timing_resume, semantic_trace_checkpoint) =
        paired_resume_paths(&frontier).unwrap();
    let checkpoint: crate::PlayCrashCheckpoint =
        bincode::deserialize(&fs::read(rust_state).unwrap()).unwrap();
    assert_eq!(checkpoint.host_frame, 52_000);
    assert_eq!(checkpoint.run_what, super::select_run_what(&game.ram));
    assert_eq!(fs::read(oracle_state).unwrap(), oracle);
    assert_eq!(fs::read(semantic_trace_checkpoint).unwrap(), semantic_trace);
    let original_timing: zelda3::OriginalTimingResumeCheckpoint =
        serde_json::from_slice(&fs::read(original_timing_resume).unwrap()).unwrap();
    assert_eq!(original_timing.schema(), 2);
    let paired_manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(frontier.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(paired_manifest["schema"], PAIRED_RESUME_SCHEMA);
    assert_eq!(paired_manifest["boundary"], "pre-frame");
    assert_eq!(paired_manifest["frame"], 52_000);
    assert_eq!(paired_manifest["rust_state"]["artifact"], "rust.z3state");
    assert_eq!(paired_manifest["oracle_state"]["artifact"], "oracle.state");
    assert!(paired_manifest["rust_state"]["sha256"].is_string());
    assert_eq!(
        paired_manifest["oracle_state"]["sha256"],
        parity::evidence::sha256_bytes(oracle)
    );
    assert_eq!(paired_manifest["source"]["cache_key"], "fixture-key");
    assert_eq!(paired_manifest["core"]["sha256"], "core-sha");
    assert_eq!(paired_manifest["input_script"]["sha256"], "input-sha");
    assert_eq!(paired_manifest["rom_random_script"]["sha256"], "rng-sha");
    assert_eq!(
        paired_manifest["initial_sram"]["sha256"],
        initial_sram_sha256
    );
    assert_eq!(
        fs::read(frontier.join("initial.srm")).unwrap(),
        b"canonical startup SRAM"
    );
    assert!(!output.join(".paired-final.tmp").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cached_oracle_checkpoint_is_bound_to_nested_artifacts_and_route_sources() {
    let root = std::env::temp_dir().join(format!(
        "zelda3-cached-oracle-checkpoint-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let checkpoint = root.join("oracle-checkpoints/frame-00002500");
    fs::create_dir_all(&checkpoint).unwrap();
    let oracle = b"oracle at frame 2500";
    let semantic = b"semantic trace at frame 2500";
    fs::write(checkpoint.join("oracle.state"), oracle).unwrap();
    fs::write(checkpoint.join("semantic-trace.checkpoint.json"), semantic).unwrap();
    let checkpoint_manifest = serde_json::json!({
        "schema": 1,
        "boundary": "pre-frame",
        "frame": 2500,
        "oracle_state": {
            "artifact": "oracle.state",
            "sha256": parity::evidence::sha256_bytes(oracle),
        },
        "semantic_trace_checkpoint": {
            "artifact": "semantic-trace.checkpoint.json",
            "sha256": parity::evidence::sha256_bytes(semantic),
        },
        "provenance": {
            "core_sha256": "core",
            "rom_sha256": "rom",
            "input_sha256": "input",
            "rom_random_sha256": "rng",
            "initial_sram_sha256": "sram",
        },
    });
    fs::write(
        checkpoint.join("manifest.json"),
        serde_json::to_vec_pretty(&checkpoint_manifest).unwrap(),
    )
    .unwrap();
    let cache_manifest = serde_json::json!({
        "cache_identity": {
            "core_sha256": "core",
            "rom_sha256": "rom",
            "source_artifact_sha256": {
                "input.txt": "input",
                "rom-random.txt": "rng",
                "initial.srm": "sram",
            },
        },
        "artifact_sha256": {
            "oracle-checkpoints/frame-00002500/oracle.state": parity::evidence::sha256_bytes(oracle),
            "oracle-checkpoints/frame-00002500/semantic-trace.checkpoint.json": parity::evidence::sha256_bytes(semantic),
        },
    });

    let (actual_oracle, actual_semantic) =
        cached_oracle_checkpoint_sources(&root, &cache_manifest, 2500).unwrap();
    assert_eq!(actual_oracle, checkpoint.join("oracle.state"));
    assert_eq!(
        actual_semantic,
        checkpoint.join("semantic-trace.checkpoint.json")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn animated_bg_receipt_decodes_complete_post_nmi_vram_without_cache_validity() {
    let mut vram = vec![0; 0x10000];
    let base = 0x3b00 * 2;
    vram[base] = 0x80;
    vram[base + 1] = 0x40;
    vram[base + 16] = 0x20;
    vram[base + 17] = 0x10;

    let actual = super::decode_presented_animated_bg_tiles(
        zelda3::PresentedAnimatedBgDestination::Dungeon,
        &vram,
    )
    .unwrap();
    let mut pixels = vec![0; zelda3::PresentedAnimatedBgTiles::TILE_COUNT * 64];
    pixels[..4].copy_from_slice(&[1, 2, 4, 8]);
    let expected = zelda3::PresentedAnimatedBgTiles::new(
        zelda3::PresentedAnimatedBgDestination::Dungeon,
        pixels,
    )
    .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn animated_bg_receipt_rejects_incomplete_post_nmi_vram() {
    let error = super::decode_presented_animated_bg_tiles(
        zelda3::PresentedAnimatedBgDestination::Overworld,
        &[0; 0x7800],
    )
    .unwrap_err();
    assert!(error.contains("omits the animated-BG publication"));
}

#[test]
fn animated_bg_destination_is_absent_until_source_graphics_setup() {
    for value in [None, Some(-1), Some(0), Some(0x5555)] {
        assert_eq!(
            super::decode_presented_animated_bg_destination(value).unwrap(),
            None,
        );
    }
    assert_eq!(
        super::decode_presented_animated_bg_destination(Some(0x3b00)).unwrap(),
        Some(zelda3::PresentedAnimatedBgDestination::Dungeon),
    );
    assert_eq!(
        super::decode_presented_animated_bg_destination(Some(0x3c00)).unwrap(),
        Some(zelda3::PresentedAnimatedBgDestination::Overworld),
    );
    assert!(super::decode_presented_animated_bg_destination(Some(1)).is_err());
}

#[test]
fn semantic_receipts_require_a_loaded_generic_trace_authority() {
    assert!(semantic_trace_authority_available(true, true));
    assert!(!semantic_trace_authority_available(true, false));
    assert!(!semantic_trace_authority_available(false, true));
    assert!(!semantic_trace_authority_available(false, false));
}

fn decode_base64_bytes(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 4, 0);
    let value = |byte: u8| match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        b'=' => 0,
        _ => panic!("invalid fixture base64 digit"),
    };
    let mut decoded = Vec::with_capacity(encoded.len() / 4 * 3);
    for chunk in encoded.as_bytes().chunks_exact(4) {
        let bits = u32::from(value(chunk[0])) << 18
            | u32::from(value(chunk[1])) << 12
            | u32::from(value(chunk[2])) << 6
            | u32::from(value(chunk[3]));
        decoded.push((bits >> 16) as u8);
        if chunk[2] != b'=' {
            decoded.push((bits >> 8) as u8);
        }
        if chunk[3] != b'=' {
            decoded.push(bits as u8);
        }
    }
    decoded
}

fn read_unsigned_varint(bytes: &[u8], index: &mut usize) -> u64 {
    let mut value = 0u64;
    let mut shift = 0;
    loop {
        let byte = bytes[*index];
        *index += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        assert!(shift < 64);
    }
    value
}

pub(crate) fn expand_delta_sequence(sequence: &serde_json::Value) -> Vec<Vec<i64>> {
    let encoding = sequence["encoding"].as_str().unwrap();
    assert!(matches!(
        encoding,
        "columnar-signed-delta-zero-rle-varint-base64-v1"
            | "columnar-signed-delta-zero-rle-varint-zstd-base64-v1"
    ));
    let field_count = sequence["fields"].as_array().unwrap().len();
    let record_count = sequence["record_count"].as_u64().unwrap() as usize;
    let mut bytes = decode_base64_bytes(sequence["data_base64"].as_str().unwrap());
    if encoding == "columnar-signed-delta-zero-rle-varint-zstd-base64-v1" {
        bytes = zstd::stream::decode_all(bytes.as_slice()).unwrap();
    }
    let mut columns = Vec::with_capacity(field_count);
    let mut byte_index = 0;
    for _ in 0..field_count {
        let column_length = read_unsigned_varint(&bytes, &mut byte_index) as usize;
        let column_end = byte_index + column_length;
        let mut column = Vec::with_capacity(record_count);
        let mut previous = 0i64;
        while byte_index < column_end {
            let code = read_unsigned_varint(&bytes, &mut byte_index);
            if code == 0 {
                let run_length = read_unsigned_varint(&bytes, &mut byte_index) as usize;
                column.extend(std::iter::repeat_n(previous, run_length));
            } else {
                let zigzag = code - 1;
                let delta = ((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64);
                previous += delta;
                column.push(previous);
            }
        }
        assert_eq!(byte_index, column_end);
        assert_eq!(column.len(), record_count);
        columns.push(column);
    }
    assert_eq!(byte_index, bytes.len());
    let mut expanded_bytes = Vec::new();
    let rows = (0..record_count)
        .map(|row| {
            (0..field_count)
                .map(|field| {
                    let value = columns[field][row];
                    expanded_bytes.extend_from_slice(&value.to_le_bytes());
                    value
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        sequence["expanded_sha256"],
        parity::evidence::sha256_bytes(&expanded_bytes)
    );
    rows
}

#[test]
fn bootstrap_delta_sequence_round_trips_signed_field_changes() {
    let sequence =
        compact_delta_integer_sequence(["a", "b", "c"], [[0, -3, 130], [1, -3, 129], [1, 400, -2]]);
    let value = serde_json::to_value(sequence).unwrap();
    assert_eq!(
        expand_delta_sequence(&value),
        vec![vec![0, -3, 130], vec![1, -3, 129], vec![1, 400, -2]]
    );
}

#[test]
fn presented_video_history_receipt_requires_byte_exact_consecutive_rows() {
    let previous_frame = LibretroFrame {
        audio: Vec::new(),
        video: vec![1, 2, 3, 4, 5, 6, 7, 8],
        video_width: 2,
        video_height: 2,
        video_pitch: 4,
        pixel_format: 2,
    };
    let previous = PresentedOracleVideo::from(&previous_frame);
    let current = LibretroFrame {
        audio: Vec::new(),
        video: vec![1, 2, 3, 4, 5, 6, 7, 9],
        video_width: 2,
        video_height: 2,
        video_pitch: 4,
        pixel_format: 2,
    };

    assert!(presented_video_rows_match_prior_surface(&current, Some(&previous), 0, 1,).unwrap());
    assert!(!presented_video_rows_match_prior_surface(&current, Some(&previous), 0, 2,).unwrap());
    assert!(!presented_video_rows_match_prior_surface(&current, None, 0, 1).unwrap());
}

#[test]
fn first_nmi_return_selector_requires_exact_committed_successor() {
    let mut transaction = LibretroCpuTimingTransaction {
        kind: 0,
        duration: 8,
        origin_pc: 0x008a38,
        opcode: 0x8c,
        start_v_counter: 227,
        start_cpu_cycle: 742,
        end_v_counter: 227,
        end_cpu_cycle: 750,
        cpu_model_identity: 1,
        cpu_model_5a22: 2,
        start_wram_refresh_position: 534,
        end_wram_refresh_position: 534,
    };
    assert_eq!(
        first_nmi_return_start_index(&[transaction]).unwrap(),
        Some(0)
    );
    transaction.start_cpu_cycle = 740;
    assert!(first_nmi_return_start_index(&[transaction])
        .unwrap_err()
        .contains("committed V227:H742->H750"));
}

#[test]
fn first_nmi_return_core_trace_requires_direct_entry_return_and_paired_hdma() {
    let path = std::env::temp_dir().join(format!(
        "zelda3-first-nmi-return-trace-{}-{}.jsonl",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    // The core writes Z3TRACE1 records with `hdma-start`/`hdma-end`
    // kinds; the reader renders them as the `hdma` domain with a stage.
    let mut bytes = parity::trace_format::MAGIC.to_vec();
    for line in [
        r#"{"event":"frame","stage":"entry","run":81,"v":225,"cycles":6,"pc":32822}"#,
        r#"{"event":"video","stage":"presented","run":81,"v":225,"cycles":16,"pc":836060}"#,
        r#"{"event":"hdma-start","run":81,"v":0,"cycles":1112,"pc":34000}"#,
        r#"{"event":"hdma-end","run":81,"v":0,"cycles":1140,"pc":34000}"#,
        r#"{"event":"frame","stage":"return","run":81,"v":225,"cycles":94,"pc":32969}"#,
    ] {
        let value: serde_json::Value = serde_json::from_str(line).unwrap();
        bytes.extend(
            parity::trace_format::TraceRecord::from_json(&value)
                .unwrap()
                .encode_framed(),
        );
    }
    fs::write(&path, bytes).unwrap();
    let trace = read_snes9x_retro_run_trace(&path, 81).unwrap().unwrap();
    let transaction = |start_v, start_h, end_v, end_h| LibretroCpuTimingTransaction {
        kind: 1,
        duration: 8,
        origin_pc: 0x008a38,
        opcode: 0x8c,
        start_v_counter: start_v,
        start_cpu_cycle: start_h,
        end_v_counter: end_v,
        end_cpu_cycle: end_h,
        cpu_model_identity: 1,
        cpu_model_5a22: 2,
        start_wram_refresh_position: 534,
        end_wram_refresh_position: 534,
    };
    let transactions = [
        transaction(227, 742, 227, 750),
        transaction(227, 750, 227, 1162),
        // `$008a59 STA $420b`: the memory operand receipt ends before
        // source-owned DMA/event processing, and the semantic receipt
        // resumes on the following scanline. The CPU timing ledger is
        // ordered evidence, not an exhaustive partition of raster time.
        transaction(228, 1160, 261, 1360),
        transaction(261, 1360, 0, 4),
        transaction(0, 4, 225, 94),
    ];
    let gaps = validate_first_nmi_return_cpu_slice(&trace, &transactions).unwrap();
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].previous_transaction_ordinal, 1);
    assert_eq!(gaps[0].next_transaction_ordinal, 2);
    assert_eq!(gaps[0].previous_end_v_counter, 227);
    assert_eq!(gaps[0].previous_end_cpu_cycle, 1162);
    assert_eq!(gaps[0].next_start_v_counter, 228);
    assert_eq!(gaps[0].next_start_cpu_cycle, 1160);
    assert_eq!(gaps[0].elapsed_master_cycles, 1362);
    assert_eq!(trace.hdma_events.len(), 2);
    assert_eq!(trace.video_events.len(), 1);
    assert_eq!(trace.return_event["cycles"], 94);
    assert_eq!(
        trace.raw_sha256,
        parity::evidence::sha256_file(&path).unwrap()
    );
    fs::remove_file(&path).unwrap();
}

#[test]
fn first_nmi_return_fixture_is_installed_only_after_explicit_success() {
    let path = std::env::temp_dir().join(format!(
        "zelda3-first-nmi-return-fixture-{}-{}.jsonl",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut pending = PendingFirstNmiReturnFixture::create(&path).unwrap();
    pending
        .writer
        .write_all(b"{\"kind\":\"synthetic\"}\n")
        .unwrap();
    assert!(!path.exists());
    assert!(pending.temporary_path.exists());
    pending.install().unwrap();
    assert!(pending.installed);
    assert!(!pending.temporary_path.exists());
    assert_eq!(fs::read(&path).unwrap(), b"{\"kind\":\"synthetic\"}\n");
    fs::remove_file(path).unwrap();
}

#[test]
fn continuation_apui_compaction_preserves_all_four_dsp_fields() {
    let access = OrdinalApuPortAccess {
        cpu_transaction_ordinal: 7,
        access: LibretroApuPortWrite {
            port: 2,
            value: 0x55,
            output_sample: 3,
            v_counter: 4,
            cpu_cycle: 5,
            program_counter: 0x008123,
            apu_cycle_before: 6,
            apu_cycle_after: 7,
            smp_clock_before: 8,
            smp_clock_after: 9,
            dsp_clock_before: 10,
            dsp_clock_after: 11,
            dsp_phase_before: 12,
            dsp_phase_after: 13,
            smp_pc_before: 14,
            smp_pc_after: 15,
            smp_opcode_before: 16,
            smp_opcode_after: 17,
            smp_opcode_cycle_before: 18,
            smp_opcode_cycle_after: 19,
            is_read: true,
            cpu_model_5a22: 2,
            wram_refresh_position: 534,
            cpu_model_identity: 1,
        },
    };
    let sequence = serde_json::to_value(compact_ordinal_cpu_apu_accesses(&[access])).unwrap();
    assert_eq!(
        sequence["fields"],
        serde_json::json!([
            "cpu_transaction_ordinal",
            "port",
            "value",
            "output_sample",
            "v_counter",
            "cpu_cycle",
            "program_counter",
            "apu_cycle_before",
            "apu_cycle_after",
            "smp_clock_before",
            "smp_clock_after",
            "dsp_clock_before",
            "dsp_clock_after",
            "dsp_phase_before",
            "dsp_phase_after",
            "smp_pc_before",
            "smp_pc_after",
            "smp_opcode_before",
            "smp_opcode_after",
            "smp_opcode_cycle_before",
            "smp_opcode_cycle_after",
            "is_read",
            "cpu_model_5a22",
            "wram_refresh_position",
            "cpu_model_identity"
        ])
    );
    let rows = expand_delta_sequence(&sequence);
    assert_eq!(&rows[0][11..15], &[10, 11, 12, 13]);
}

#[test]
fn first_nmi_dma_setup_slice_starts_after_apui_completion_and_excludes_dma_start() {
    let apui = FramedApuPortAccess {
        frame: 81,
        access: LibretroApuPortWrite {
            port: 0,
            value: 0x8b,
            output_sample: 0,
            v_counter: 225,
            cpu_cycle: 480,
            program_counter: 0x0080e4,
            apu_cycle_before: 19,
            apu_cycle_after: 43,
            smp_clock_before: 0,
            smp_clock_after: 1,
            dsp_clock_before: 12,
            dsp_clock_after: 13,
            dsp_phase_before: 14,
            dsp_phase_after: 15,
            smp_pc_before: 0x0873,
            smp_pc_after: 0x0873,
            smp_opcode_before: 0xf0,
            smp_opcode_after: 0xf0,
            smp_opcode_cycle_before: 0,
            smp_opcode_cycle_after: 0,
            is_read: true,
            cpu_model_5a22: 2,
            wram_refresh_position: 534,
            cpu_model_identity: 1,
        },
    };
    let transaction = |kind, origin_pc, opcode, start_cpu_cycle| FramedCpuTimingTransaction {
        frame: 81,
        transaction: LibretroCpuTimingTransaction {
            kind,
            duration: if matches!(kind, 0 | 1) { 8 } else { 6 },
            origin_pc,
            opcode,
            start_v_counter: 225,
            start_cpu_cycle,
            end_v_counter: 225,
            end_cpu_cycle: start_cpu_cycle + if matches!(kind, 0 | 1) { 8 } else { 6 },
            cpu_model_identity: 1,
            cpu_model_5a22: 2,
            start_wram_refresh_position: 534,
            end_wram_refresh_position: 534,
        },
    };
    let transactions = vec![
        transaction(2, 0x0080e1, 0xad, 480),
        transaction(0, 0x008a33, 0xa9, 486),
        transaction(1, 0x008a33, 0xa9, 494),
        transaction(0, 0x008a35, 0x8d, 500),
        transaction(2, 0x008a35, 0x8d, 508),
    ];

    assert_eq!(
        first_nmi_apui_anchor_indices(&[apui], &transactions).unwrap(),
        Some((0, 0))
    );
    let post_anchor = &transactions[1..];
    let stop = first_nmi_dma_setup_stop_index(post_anchor)
        .unwrap()
        .unwrap();
    assert_eq!(stop, 2);
    assert_eq!(post_anchor[..stop].last().unwrap().transaction.kind, 1);
    assert_eq!(
        post_anchor[..stop].last().unwrap().transaction.origin_pc,
        0x008a33
    );
    assert!(post_anchor[..stop]
        .iter()
        .all(|framed| { (framed.transaction.origin_pc & 0x00ff_ffff) != 0x008a35 }));
    assert_eq!(post_anchor[stop].transaction.kind, 0);
}

#[test]
#[ignore = "requires routes/full_run/comparisons/continuous-audio/initial.srm, an untracked 8 KiB saved-game SRAM (sha256 d8a02e6e…) that the first-NMI DMA-setup fixture was captured with"]
fn first_nmi_dma_setup_requires_the_route_initial_sram_loaded_into_snes9x() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../routes/full_run/comparisons/continuous-audio/initial.srm");
    let sram = fs::read(&path).unwrap();
    let provenance = first_nmi_dma_setup_initial_sram_provenance(Some(&path), &sram).unwrap();
    assert_eq!(
        provenance["sha256"],
        "d8a02e6e08a22377f919c7350e5bfa6117c6408db9a728815af7ad6e4e6b83bc"
    );
    assert_eq!(provenance["bytes"], 8_192);
    assert_eq!(provenance["valid_slot_marker"]["offset"], 0x03e5);
    assert_eq!(
        provenance["valid_slot_marker"]["bytes"],
        serde_json::json!([0xaa, 0x55])
    );

    let missing = first_nmi_dma_setup_initial_sram_provenance(None, &sram)
        .unwrap_err()
        .to_string();
    assert!(missing.contains("requires --load-sram"));

    let mut different_loaded_sram = sram.clone();
    different_loaded_sram[0] ^= 1;
    let mismatch = first_nmi_dma_setup_initial_sram_provenance(Some(&path), &different_loaded_sram)
        .unwrap_err()
        .to_string();
    assert!(mismatch.contains("does not match the bytes loaded into Snes9x"));
}

#[test]
fn bootstrap_zstd_delta_sequence_round_trips_signed_field_changes() {
    let sequence = compact_delta_integer_sequence_with_zstd(
        ["a", "b", "c"],
        [[0, -3, 130], [1, -3, 129], [1, 400, -2]],
        true,
    );
    let value = serde_json::to_value(sequence).unwrap();
    assert_eq!(
        expand_delta_sequence(&value),
        vec![vec![0, -3, 130], vec![1, -3, 129], vec![1, 400, -2]]
    );
}

fn bootstrap_instruction(
    absolute_cycle: u64,
    program_counter: i32,
    opcode: i32,
    op_step_calls: i32,
) -> LibretroSmpInstruction {
    LibretroSmpInstruction {
        absolute_cycle,
        program_counter,
        opcode,
        a: 0,
        x: 0,
        y: 0,
        stack_pointer: 0xef,
        status: 2,
        timer0_stage1: 0,
        timer0_stage2: 0,
        timer0_stage3: 0,
        output_sample: 0,
        dsp_phase: 0,
        smp_clock: 0,
        direct_page_0_11: [0; 12],
        boundary_opcode_cycle: 0,
        op_step_calls,
        max_continuation_opcode_cycle: op_step_calls - 1,
    }
}

#[test]
fn framed_smp_boundary_sequence_round_trips_all_recorded_fields() {
    let mut first = bootstrap_instruction(1_000, 0x0800, 0x20, 2);
    first.a = 0x11;
    first.x = 0x22;
    first.y = 0x33;
    first.stack_pointer = 0xcc;
    first.status = 0x81;
    first.timer0_stage1 = 4;
    first.timer0_stage2 = 5;
    first.timer0_stage3 = 6;
    first.output_sample = 7;
    first.dsp_phase = 8;
    first.smp_clock = -9;
    first.direct_page_0_11 = [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21];
    first.boundary_opcode_cycle = 1;
    first.max_continuation_opcode_cycle = 3;

    let mut second = bootstrap_instruction(1_006, 0x0801, 0xcd, 1);
    second.a = -1;
    second.direct_page_0_11[11] = 0x7f;
    let framed = [
        FramedSmpInstruction {
            frame: 79,
            instruction: first,
        },
        FramedSmpInstruction {
            frame: 80,
            instruction: second,
        },
    ];

    let sequence = serde_json::to_value(compact_framed_smp_instructions(&framed)).unwrap();
    assert_eq!(sequence["fields"].as_array().unwrap().len(), 30);
    assert_eq!(
        expand_delta_sequence(&sequence),
        vec![
            vec![
                79, 1_000, 0x0800, 0x20, 0x11, 0x22, 0x33, 0xcc, 0x81, 4, 5, 6, 7, 8, -9, 10, 11,
                12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3,
            ],
            vec![
                80, 1_006, 0x0801, 0xcd, -1, 0, 0, 0xef, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0x7f, 0, 1, 0,
            ],
        ]
    );
}

#[test]
fn bootstrap_instruction_frames_replace_straddling_pseudo_op_record() {
    let mut accumulated = vec![
        bootstrap_instruction(0, 0xfff9, 0xd0, 1),
        bootstrap_instruction(4, 0xfffb, 0x1f, 1),
    ];
    append_smp_instruction_frame(
        &mut accumulated,
        vec![
            bootstrap_instruction(4, 0xfffb, 0x1f, 2),
            bootstrap_instruction(10, 0x0800, 0xcd, 1),
        ],
    );

    assert_eq!(accumulated.len(), 3);
    assert_eq!(accumulated[1].op_step_calls, 2);
    assert_eq!(smp_bootstrap_handoff_index(&accumulated), Some(1));
}

#[test]
fn pinned_snes9x_cold_apu_bootstrap_fixture_reaches_final_ipl_handoff() {
    const FIXTURE: &str =
        include_str!("../../../external/snes9x-libretro/fixtures/zelda3-cold-apu-bootstrap.jsonl");
    assert!(
        FIXTURE.len() < 9_000_000,
        "bootstrap fixture lost compact encoding"
    );
    let records = FIXTURE
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 3);

    let provenance = &records[0];
    assert_eq!(provenance["kind"], "provenance");
    assert_eq!(provenance["schema"], 1);
    assert_eq!(provenance["core"]["library_name"], "Snes9x");
    assert_eq!(provenance["core"]["library_version"], "1.63 921f9f7b");
    assert_eq!(
        provenance["core"]["sha256"],
        "7d4fa577dd2e0a79ace97ebec2630929ffcd6622d46ae5b23c4700514c7169cb"
    );
    assert_eq!(
        provenance["source"]["revision"],
        "921f9f7b83660eb44ad263022a57a4a029057c37"
    );
    assert_eq!(
        provenance["rom"]["sha256"],
        "66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb"
    );
    let trace_patch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/patches/zelda3-trace.patch");
    assert_eq!(
        provenance["source"]["trace_patch_sha256"],
        parity::evidence::sha256_file(&trace_patch).unwrap()
    );
    assert_eq!(provenance["cpu_to_smp_ratio"]["numerator"], 15_664);
    assert_eq!(provenance["cpu_to_smp_ratio"]["denominator"], 328_125);

    let reset = &records[1];
    assert_eq!(reset["kind"], "reset-state");
    assert_eq!(reset["absolute_cycle"], 0);
    assert_eq!(reset["pc"], 0xffc0);
    assert_eq!(reset["sp"], 0xef);
    assert_eq!(reset["status"], 0x02);
    assert_eq!(reset["ipl_rom_enabled"], true);
    assert_eq!(reset["ram_nonzero_bytes"], 0);
    assert_eq!(reset["output_ports"], serde_json::json!([0, 0, 0, 0]));
    // globals.cpp selects the M1SNES object, whose _5A22 field is 2.
    // cpu.cpp therefore uses the v2 refresh schedule; the object name is
    // not evidence that WRAM refresh occurs at the v1 position.
    assert_eq!(reset["cpu_model_identity"], 1);
    assert_eq!(reset["cpu_model_5a22"], 2);
    assert_eq!(reset["wram_refresh_position"], 538);

    let events = &records[2];
    assert_eq!(events["kind"], "bootstrap-events");
    assert_eq!(events["first_cc_acknowledged"], true);
    assert_eq!(events["final_frame"], 79);
    assert_eq!(events["final_ipl_handoff"]["absolute_cycle"], 1_355_567);
    assert_eq!(events["final_ipl_handoff"]["origin_pc"], 0xfffb);
    assert_eq!(events["final_ipl_handoff"]["opcode"], 0x1f);
    assert_eq!(events["final_ipl_handoff"]["target_pc"], 0x0800);

    let cpu = expand_delta_sequence(&events["cpu_apu_access_sequence"]);
    assert_eq!(cpu.len(), 356_174);
    assert_eq!(
        cpu[0],
        vec![0, 0, 0, 0, 0, 326, 0x800d, 0, 16, 0, 1, 0xffc0, 0xffc5, 0, 0xd0, 0, 2, 538, 1,]
    );
    assert!(cpu.iter().all(|access| access[16] == 2));
    assert!(cpu.iter().all(|access| matches!(access[17], 534 | 538)));
    assert!(cpu.iter().all(|access| access[18] == 1));

    let before_refresh_pair = cpu
        .windows(2)
        .find(|pair| {
            pair[0][0] == 0
                && pair[0][4] == 44
                && pair[0][5] == 528
                && pair[1][4] == 44
                && pair[1][5] == 534
        })
        .expect("fixture lost the H528/H534 APUI pair");
    // Both semantics precede this scanline's live H538 refresh event, so
    // their six-master-cycle separation is not evidence of an H530 model.
    for access in before_refresh_pair {
        assert_eq!(access[16], 2);
        assert_eq!(access[17], 538);
        assert_eq!(access[18], 1);
    }
    let initial_cc = cpu
        .iter()
        .position(|access| {
            access[1] == 0 && access[2] == 0xcc && access[6] == 0x88ef && access[15] == 1
        })
        .unwrap();
    assert_eq!(cpu[initial_cc][0], 0);
    assert_eq!(cpu[initial_cc][4], 37);
    assert_eq!(cpu[initial_cc][5], 1_102);
    assert_eq!(cpu[initial_cc][8], 2_461);
    assert_eq!(
        cpu.last().unwrap(),
        &vec![
            79, 3, 0, 319, 120, 384, 0x88ff, 10_255, 10_255, 4, 3, 0x0800, 0x0800, 0x1f, 0x1f, 0,
            2, 534, 1,
        ]
    );

    assert_eq!(
        events["cpu_timing_transaction_kinds"],
        serde_json::json!({
            "0": "fast_pcbase_opcode_fetch_non_draining",
            "1": "cpuops_add_cycles_draining",
            "2": "getset_memory_access_after_semantic_draining",
            "3": "getset_memory_access_x2_after_semantic_draining",
        })
    );
    let cpu_timing = expand_delta_sequence(&events["cpu_timing_transaction_sequence"]);
    assert_eq!(cpu_timing.len(), 3_213_852);
    assert_eq!(
        &cpu_timing[..5],
        &[
            vec![0, 0, 8, 0x8000, 0x78, 0, 198, 0, 206, 1, 2, 538, 538],
            vec![0, 1, 6, 0x8000, 0x78, 0, 206, 0, 212, 1, 2, 538, 538],
            vec![0, 0, 8, 0x8001, 0x9c, 0, 212, 0, 220, 1, 2, 538, 538],
            vec![0, 1, 16, 0x8001, 0x9c, 0, 220, 0, 236, 1, 2, 538, 538],
            vec![0, 2, 6, 0x8001, 0x9c, 0, 236, 0, 242, 1, 2, 538, 538],
        ]
    );
    assert_eq!(
        [13, 16, 19, 22].map(|index| cpu_timing[index].clone()),
        [
            vec![0, 2, 6, 0x800a, 0x9c, 0, 326, 0, 332, 1, 2, 538, 538],
            vec![0, 2, 6, 0x800d, 0x9c, 0, 356, 0, 362, 1, 2, 538, 538],
            vec![0, 2, 6, 0x8010, 0x9c, 0, 386, 0, 392, 1, 2, 538, 538],
            vec![0, 2, 6, 0x8013, 0x9c, 0, 416, 0, 422, 1, 2, 538, 538],
        ]
    );
    assert!(cpu_timing.iter().all(|transaction| transaction[9] == 1));
    assert!(cpu_timing.iter().all(|transaction| transaction[10] == 2));
    assert!(cpu_timing
        .iter()
        .all(|transaction| matches!(transaction[11], 534 | 538)));
    assert!(cpu_timing
        .iter()
        .all(|transaction| matches!(transaction[12], 534 | 538)));
    assert_eq!(
        cpu_timing
            .iter()
            .fold([0usize; 4], |mut counts, transaction| {
                counts[transaction[1] as usize] += 1;
                counts
            }),
        [1_143_074, 1_552_419, 464_286, 54_073]
    );
    assert!(cpu_timing.iter().any(|transaction| {
        transaction == &vec![0, 1, 8, 0x8894, 0xd0, 0, 1_366, 1, 10, 1, 2, 538, 534]
    }));
    assert_eq!(
        cpu_timing.last().unwrap(),
        &vec![79, 2, 6, 0x88fc, 0x9c, 120, 384, 120, 390, 1, 2, 534, 534]
    );

    let output = expand_delta_sequence(&events["smp_output_port_write_sequence"]);
    assert_eq!(output.len(), 54_043);
    assert_eq!(
        &output[..3],
        &[
            vec![
                2_399, 0, 0xaa, 0xffc9, 0x8f, 3, 36, 1_184, 0x8894, 1_132, 52_954, -1, 0xffcc, 53,
                10, 73
            ],
            vec![
                2_404, 1, 0xbb, 0xffcc, 0x8f, 3, 36, 1_300, 0x8894, 1_248, 229_353, -2, 0xffcf, 58,
                10, 73
            ],
            vec![
                2_461, 0, 0xcc, 0xfff5, 0xc4, 3, 37, 1_102, 0x88ef, 1_050, 118_577, 0, 0xfff7, 52,
                9, 75
            ],
        ]
    );
    assert_eq!(output.last().unwrap()[0], 1_355_555);
    assert_eq!(output.last().unwrap()[2], 0x8b);
    assert_eq!(output.last().unwrap()[3], 0xfff5);

    let sequence = &events["smp_instruction_boundary_sequence"];
    assert_eq!(sequence["encoding"], "repeated-span-v1");
    assert_eq!(sequence["instruction_count"], 379_743);
    assert_eq!(sequence["absolute_start_cycle"], 0);
    assert_eq!(sequence["absolute_end_cycle"], 1_355_567);
    assert_eq!(sequence["spans"].as_array().unwrap().len(), 437);
    let mut expanded = Vec::new();
    let mut expected_cycle = 0;
    for span in sequence["spans"].as_array().unwrap() {
        assert_eq!(span["absolute_start_cycle"].as_u64(), Some(expected_cycle));
        let span_start = span["absolute_start_cycle"].as_u64().unwrap();
        let stride = span["repeat_cycle_stride"].as_u64().unwrap();
        let repeat_count = span["repeat_count"].as_u64().unwrap();
        for repeat in 0..repeat_count {
            let repeat_start = span_start + repeat * stride;
            for instruction in span["instructions"].as_array().unwrap() {
                let start = repeat_start + instruction["start_cycle_offset"].as_u64().unwrap();
                let end = repeat_start + instruction["end_cycle_offset"].as_u64().unwrap();
                assert_eq!(start, expected_cycle);
                assert!(end > start);
                let calls = instruction["op_step_calls"].as_u64().unwrap();
                let continuation = instruction["max_continuation_opcode_cycle"]
                    .as_u64()
                    .unwrap();
                assert_eq!(continuation, calls - 1);
                expanded.push((
                    start,
                    end,
                    instruction["origin_pc"].as_u64().unwrap(),
                    instruction["opcode"].as_u64().unwrap(),
                    calls,
                    continuation,
                ));
                expected_cycle = end;
            }
        }
        assert_eq!(span["absolute_end_cycle"].as_u64(), Some(expected_cycle));
    }
    assert_eq!(expanded.len(), 379_743);
    assert_eq!(expanded[0], (0, 2, 0xffc0, 0xcd, 1, 0));
    assert_eq!(
        expanded.iter().find(|step| step.0 == 2_461).copied(),
        Some((2_461, 2_463, 0xfff7, 0xdd, 1, 0))
    );
    assert!(expanded
        .iter()
        .any(|step| step.2 == 0xffe7 && step.3 == 0xab));
    assert_eq!(
        expanded.last().copied(),
        Some((1_355_561, 1_355_567, 0xfffb, 0x1f, 1, 0))
    );
}

#[test]
fn pinned_snes9x_post_handoff_fixture_reaches_first_nmi_apui_sync() {
    const FIXTURE: &str =
        include_str!("../../../external/snes9x-libretro/fixtures/zelda3-cold-apu-first-nmi.jsonl");
    assert!(
        FIXTURE.len() < 20_000,
        "post-handoff fixture lost compact encoding"
    );
    let records = FIXTURE
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);

    let provenance = &records[0];
    assert_eq!(provenance["kind"], "provenance");
    assert_eq!(provenance["schema"], 1);
    assert_eq!(provenance["core"]["library_name"], "Snes9x");
    assert_eq!(provenance["core"]["library_version"], "1.63 921f9f7b");
    assert_eq!(
        provenance["core"]["sha256"],
        "7d4fa577dd2e0a79ace97ebec2630929ffcd6622d46ae5b23c4700514c7169cb"
    );
    assert_eq!(
        provenance["source"]["revision"],
        "921f9f7b83660eb44ad263022a57a4a029057c37"
    );
    assert_eq!(
        provenance["rom"]["sha256"],
        "66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb"
    );
    let trace_patch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/patches/zelda3-trace.patch");
    assert_eq!(
        provenance["source"]["trace_patch_sha256"],
        parity::evidence::sha256_file(&trace_patch).unwrap()
    );
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-apu-bootstrap.jsonl");
    assert_eq!(
        provenance["prefix_fixture"]["sha256"],
        parity::evidence::sha256_file(&prefix_fixture).unwrap()
    );

    let events = &records[1];
    assert_eq!(events["kind"], "post-handoff-first-nmi");
    assert_eq!(
        events["stop_reason"],
        "first_$8080e1_apui0_read_semantic_and_kind2_timing_complete"
    );
    assert_eq!(
        events["start_anchor"]["final_ipl_handoff"]["absolute_cycle"],
        1_355_567
    );
    assert_eq!(
        events["start_anchor"]["final_cpu_timing_transaction"]["transaction"]["origin_pc"],
        0x88fc
    );
    assert_eq!(
        events["start_anchor"]["final_cpu_timing_transaction"]["transaction"]["kind"],
        2
    );
    assert_eq!(
        events["nmi_enable_source"]["rom_bytes"],
        serde_json::json!([0xa9, 0x81, 0x8d, 0x00, 0x42])
    );

    let cpu_timing = expand_delta_sequence(&events["cpu_timing_transaction_sequence"]);
    assert_eq!(cpu_timing.len(), 55_405);
    assert_eq!(
        cpu_timing.first().unwrap(),
        &vec![79, 0, 8, 0x88ff, 0x28, 120, 390, 120, 398, 1, 2, 534, 534]
    );
    assert_eq!(
        cpu_timing.last().unwrap(),
        &vec![81, 2, 6, 0x80e1, 0xad, 225, 480, 225, 486, 1, 2, 534, 534]
    );
    assert!(cpu_timing.iter().all(|transaction| transaction[9] == 1));
    assert!(cpu_timing.iter().all(|transaction| transaction[10] == 2));
    assert!(cpu_timing
        .iter()
        .all(|transaction| matches!(transaction[11], 534 | 538)));
    assert!(cpu_timing
        .iter()
        .all(|transaction| matches!(transaction[12], 534 | 538)));
    assert_eq!(
        events["first_hmax_crossing_transaction"],
        serde_json::json!({
            "frame": 79,
            "transaction": {
                "kind": 1,
                "duration": 16,
                "origin_pc": 0x87da,
                "opcode": 0x9d,
                "start_v_counter": 120,
                "start_cpu_cycle": 1368,
                "end_v_counter": 121,
                "end_cpu_cycle": 20,
                "cpu_model_identity": 1,
                "cpu_model_5a22": 2,
                "start_wram_refresh_position": 534,
                "end_wram_refresh_position": 538,
            },
        })
    );
    assert_eq!(
        events["nmi_enable_source"]["timing_transaction"]["transaction"]["origin_pc"],
        0x8031
    );
    assert_eq!(
        events["first_nmi_entry_transaction"]["transaction"]["origin_pc"],
        0x80c9
    );

    let apui = expand_delta_sequence(&events["cpu_apu_access_sequence"]);
    assert_eq!(
        apui,
        vec![vec![
            81, 0, 0x8b, 0, 225, 480, 0x80e4, 19, 43, 0, 1, 0x0873, 0x0873, 0xf0, 0xf0, 0, 0, 1, 2,
            534, 1,
        ]]
    );
    let output = expand_delta_sequence(&events["smp_output_port_write_sequence"]);
    assert_eq!(output.len(), 3);
    assert_eq!(
        output.iter().map(|write| &write[..3]).collect::<Vec<_>>(),
        vec![
            &[1_374_569, 1, 0][..],
            &[1_374_676, 2, 0][..],
            &[1_374_783, 3, 0][..]
        ]
    );

    let boundaries = &events["smp_instruction_boundaries"];
    assert_eq!(boundaries["count"], 6_146);
    assert_eq!(
        boundaries["expanded_sha256"],
        "c7b7a58df7141c741a13e858532f317bd7e8c54e0056d15d79fb8c88fdb959d8"
    );
    let smp = expand_delta_sequence(&events["smp_instruction_boundary_sequence"]);
    assert_eq!(
        events["smp_instruction_boundary_sequence"]["fields"]
            .as_array()
            .unwrap()
            .len(),
        30
    );
    assert_eq!(smp.len(), 6_146);
    assert_eq!(
        smp.first().unwrap(),
        &vec![
            79, 1_355_567, 0x0800, 0x20, 0, 0, 0, 0xef, 2, 47, 0, 0, 319, 26, -45, 0, 8, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
        ]
    );
    assert_eq!(
        smp.last().unwrap(),
        &vec![
            81, 1_379_527, 0x0876, 0xf0, 63, 69, 0, 207, 2, 71, 7, 0, 2, 22, -46, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 0,
        ]
    );
    let mut digest_bytes = Vec::with_capacity(smp.len() * 128);
    for row in &smp {
        digest_bytes.extend_from_slice(&(row[0] as u32).to_le_bytes());
        digest_bytes.extend_from_slice(&(row[1] as u64).to_le_bytes());
        for value in &row[2..] {
            digest_bytes.extend_from_slice(&(*value as i32).to_le_bytes());
        }
    }
    assert_eq!(
        boundaries["expanded_sha256"],
        parity::evidence::sha256_bytes(&digest_bytes)
    );
    assert_eq!(
        boundaries["before_sync"]["instruction"]["absolute_cycle"],
        1_379_507
    );
    assert_eq!(
        boundaries["before_sync"]["instruction"]["program_counter"],
        0x0873
    );
    assert_eq!(
        boundaries["after_sync"]["instruction"]["absolute_cycle"],
        1_379_527
    );
    assert_eq!(
        boundaries["after_sync"]["instruction"]["program_counter"],
        0x0876
    );
    assert_eq!(boundaries["absolute_end_cycle"], 1_379_531);
    assert_eq!(
        boundaries["terminal_successor"]["instruction"]["absolute_cycle"],
        1_379_531
    );
    assert_eq!(
        boundaries["terminal_successor"]["instruction"]["program_counter"],
        0x0873
    );
}

#[test]
#[ignore = "requires routes/full_run/comparisons/continuous-audio/initial.srm, an untracked 8 KiB saved-game SRAM (sha256 d8a02e6e…) that the first-NMI DMA-setup fixture was captured with"]
fn pinned_snes9x_first_nmi_dma_setup_fixture_stops_before_dma() {
    const FIXTURE: &str = include_str!(
        "../../../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma-setup.jsonl"
    );
    assert!(
        FIXTURE.len() < 10_000,
        "DMA-setup fixture lost compact encoding"
    );
    let records = FIXTURE
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);

    let provenance = &records[0];
    assert_eq!(provenance["kind"], "provenance");
    assert_eq!(provenance["schema"], 1);
    assert_eq!(
        provenance["core"]["sha256"],
        "7d4fa577dd2e0a79ace97ebec2630929ffcd6622d46ae5b23c4700514c7169cb"
    );
    assert_eq!(
        provenance["source"]["revision"],
        "921f9f7b83660eb44ad263022a57a4a029057c37"
    );
    assert_eq!(
        provenance["rom"]["sha256"],
        "66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb"
    );
    assert_eq!(
        provenance["core_build_receipt"]["patch_sha256s"],
        serde_json::json!([
            "a285f684f69b58959367877e173759baad0bc7f6b152d6644c5b6a00f691f5a3",
            "669b7767888be19f99623135ec00cc04913908fb74f19944da752fe863044dd2",
        ])
    );
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-apu-first-nmi.jsonl");
    assert_eq!(
        provenance["prefix_fixture"]["sha256"],
        parity::evidence::sha256_file(&prefix_fixture).unwrap()
    );
    let initial_sram = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../routes/full_run/comparisons/continuous-audio/initial.srm");
    assert_eq!(
        provenance["initial_sram"],
        serde_json::json!({
            "source": "routes/full_run/comparisons/continuous-audio/initial.srm",
            "sha256": parity::evidence::sha256_file(&initial_sram).unwrap(),
            "bytes": 8_192,
            "valid_slot_marker": {"offset": 0x03e5, "bytes": [0xaa, 0x55]},
        })
    );

    let events = &records[1];
    assert_eq!(events["kind"], "first-nmi-dma-setup");
    assert_eq!(
        events["stop_reason"],
        "before_$008a35_sta_$420b_raw_fetch_transaction"
    );
    let anchor = &events["start_anchor"]["first_nmi_apui_read"];
    assert_eq!(anchor["instruction_origin_pc"], 0x0080e1);
    assert_eq!(anchor["access"]["cpu_cycle"], 480);
    assert_eq!(anchor["access"]["smp_clock_before"], 0);
    assert_eq!(anchor["access"]["smp_clock_after"], 1);
    assert_eq!(anchor["completed_timing_transaction"]["kind"], 2);
    assert_eq!(anchor["completed_timing_transaction"]["end_cpu_cycle"], 486);

    let cpu_timing = expand_delta_sequence(&events["cpu_timing_transaction_sequence"]);
    assert_eq!(cpu_timing.len(), 157);
    assert_eq!(
        cpu_timing.first().unwrap(),
        &vec![81, 0, 8, 0x80e4, 0xcd, 225, 486, 225, 494, 1, 2, 534, 534]
    );
    assert_eq!(
        &cpu_timing[cpu_timing.len() - 2..],
        &[
            vec![81, 0, 8, 0x8a33, 0xa9, 226, 698, 226, 706, 1, 2, 538, 538],
            vec![81, 1, 8, 0x8a33, 0xa9, 226, 706, 226, 714, 1, 2, 538, 538],
        ]
    );
    assert!(cpu_timing
        .iter()
        .all(|transaction| transaction[3] != 0x8a35));

    assert_eq!(
        events["final_completed_setup_instruction"]["rom_bytes"],
        serde_json::json!([0xa9, 0x07])
    );
    let excluded =
        &events["stop_before_instruction"]["excluded_raw_fetch_transaction"]["transaction"];
    assert_eq!(excluded["origin_pc"], 0x8a35);
    assert_eq!(excluded["opcode"], 0x8d);
    assert_eq!(excluded["kind"], 0);
    assert_eq!(excluded["start_v_counter"], 226);
    assert_eq!(excluded["start_cpu_cycle"], 714);
    assert_eq!(excluded["end_cpu_cycle"], 722);
}

#[test]
fn first_nmi_dma_host_selector_and_compaction_are_exact_and_route_external() {
    let transaction = |kind, pc, opcode, start, end| LibretroCpuTimingTransaction {
        kind,
        duration: end - start,
        origin_pc: pc,
        opcode,
        start_v_counter: 226,
        start_cpu_cycle: start,
        end_v_counter: 226,
        end_cpu_cycle: end,
        cpu_model_identity: 1,
        cpu_model_5a22: 2,
        start_wram_refresh_position: 538,
        end_wram_refresh_position: 538,
    };
    let transactions = vec![
        transaction(0, 0x8a35, 0x8d, 714, 722),
        transaction(1, 0x8a35, 0x8d, 722, 730),
        transaction(1, 0x8a35, 0x8d, 730, 738),
        transaction(2, 0x8a35, 0x8d, 738, 900),
        transaction(0, 0x8a38, 0xa9, 900, 908),
    ];
    assert_eq!(
        first_nmi_dma_transaction_slice(&transactions).unwrap(),
        Some((0, 3, 4))
    );

    let event = |kind: i32, owner: i32, global: i32, channel: i32| {
        let mut fields = [-1; 72];
        fields[0] = kind;
        fields[1] = 3;
        fields[2] = owner;
        fields[3] = global;
        fields[4] = channel;
        fields[9] = 1;
        fields[27] = 0x8a38;
        if kind == 2 {
            fields[5] = 0;
            fields[50] = 0x1000 + global;
            fields[51] = 0x7e;
            fields[6] = (fields[51] << 16) | fields[50];
            fields[7] = 0x2118;
            fields[8] = 0x40 + global;
            fields[47] = 0;
            fields[48] = 0;
            fields[49] = 1;
            fields[63] = 0;
            fields[64] = fields[50] + 1;
        }
        LibretroDmaLedgerEvent { fields }
    };
    let mut events = vec![event(0, 7, -1, -1)];
    for (global, channel) in [0, 1, 2].into_iter().enumerate() {
        events.push(event(1, channel, -1, -1));
        events.push(event(2, channel, global as i32, 0));
        events.push(event(3, channel, -1, -1));
    }
    events.push(event(4, 7, -1, -1));
    let (outer, selected) = first_nmi_dma_ledger_slice(&events).unwrap().unwrap();
    assert_eq!(outer, 3);
    assert_eq!(selected, events);
    assert_eq!(
        semantic_receipts_from_dma_ledger(&events).unwrap(),
        vec![OriginalTimingSemanticReceipt::DmaPublicationCompleted { channel_mask: 7 }],
    );

    let zero_mask_events = vec![event(0, 0, -1, -1), event(4, 0, -1, -1)];
    assert_eq!(
        semantic_receipts_from_dma_ledger(&zero_mask_events).unwrap(),
        Vec::<OriginalTimingSemanticReceipt>::new(),
    );

    let compact = serde_json::to_value(compact_dma_ledger(&selected)).unwrap();
    assert_eq!(expand_delta_sequence(&compact).len(), selected.len());
    assert_eq!(
        expand_delta_sequence(&compact),
        selected
            .iter()
            .map(|event| event.fields.map(i64::from).to_vec())
            .collect::<Vec<_>>()
    );
    let snapshot = (0..=255).cycle().take(0x10000).collect::<Vec<u8>>();
    let compact = serde_json::to_value(compact_byte_snapshot(&snapshot)).unwrap();
    assert_eq!(
        expand_delta_sequence(&compact),
        snapshot
            .iter()
            .map(|byte| vec![i64::from(*byte)])
            .collect::<Vec<_>>()
    );
}

#[test]
fn pinned_snes9x_first_nmi_dma_fixture_is_fully_reconstructable() {
    const FIXTURE: &str =
        include_str!("../../../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma.jsonl");
    assert!(FIXTURE.len() < 12_000, "DMA fixture lost compact encoding");
    let records = FIXTURE
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 2);

    let provenance = &records[0];
    assert_eq!(provenance["kind"], "provenance");
    assert_eq!(
        provenance["core"]["sha256"],
        "425a2e8b451970dd3719a165259af265a53b468447c7f6a3b4e614d322204cc6"
    );
    assert_eq!(
        provenance["source"]["revision"],
        "921f9f7b83660eb44ad263022a57a4a029057c37"
    );
    assert_eq!(
        provenance["core_build_receipt"]["patch_sha256s"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    let patch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/patches/zelda3-dma-ledger.patch");
    assert_eq!(
        provenance["dma_ledger_patch"]["sha256"],
        parity::evidence::sha256_file(&patch).unwrap()
    );
    assert_eq!(provenance["dma_ledger_patch"]["core_route_selector"], false);
    let prefix = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma-setup.jsonl");
    assert_eq!(
        provenance["prefix_fixture"]["sha256"],
        parity::evidence::sha256_file(&prefix).unwrap()
    );

    let capture = &records[1];
    assert_eq!(capture["kind"], "first-nmi-dma");
    assert_eq!(capture["frame"], 81);
    assert_eq!(capture["dma"]["byte_count"], 160);
    assert_eq!(
        capture["stop_reason"],
        "completed_$008a35_sta_$420b_07_and_observed_$008a38_raw_fetch"
    );
    let source = &capture["source_instruction"];
    assert_eq!(source["raw_fetch_anchor"]["origin_pc"], 0x8a35);
    assert_eq!(source["raw_fetch_anchor"]["start_v_counter"], 226);
    assert_eq!(source["raw_fetch_anchor"]["start_cpu_cycle"], 714);
    assert_eq!(source["raw_fetch_anchor"]["end_cpu_cycle"], 722);
    assert_eq!(source["completed_outer_write_transaction"]["kind"], 2);
    assert_eq!(source["successor_raw_fetch"]["origin_pc"], 0x8a38);
    assert_eq!(
        source["completed_outer_write_transaction"]["end_cpu_cycle"],
        source["successor_raw_fetch"]["start_cpu_cycle"]
    );

    let cpu = expand_delta_sequence(&capture["cpu_timing_transaction_sequence"]);
    assert_eq!(cpu.len(), 4);
    assert_eq!(&cpu[0][1..9], &[0, 8, 0x8a35, 0x8d, 226, 714, 226, 722]);
    assert_eq!(cpu.last().unwrap()[1], 0);
    assert_eq!(cpu.last().unwrap()[3], 0x8a38);

    assert_eq!(
        capture["dma"]["ordered_event_sequence"]["fields"],
        serde_json::json!(super::DMA_LEDGER_FIELDS.to_vec())
    );
    let ledger = expand_delta_sequence(&capture["dma"]["ordered_event_sequence"]);
    assert_eq!(ledger.len(), 168);
    let events = ledger
        .into_iter()
        .map(|row| LibretroDmaLedgerEvent {
            fields: row
                .into_iter()
                .map(|value| i32::try_from(value).unwrap())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        })
        .collect::<Vec<_>>();
    let (_, selected) = first_nmi_dma_ledger_slice(&events).unwrap().unwrap();
    assert_eq!(selected, events);
    let channel_byte_counts = [0, 1, 2].map(|channel| {
        events
            .iter()
            .filter(|event| event.fields[0] == 2 && event.fields[2] == channel)
            .count()
    });
    assert_eq!(channel_byte_counts, [64, 64, 32]);
    assert_eq!(
        capture["dma"]["hmax_crossing_receipts"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let before = expand_delta_sequence(&capture["vram"]["before_sequence"])
        .into_iter()
        .map(|row| u8::try_from(row[0]).unwrap())
        .collect::<Vec<_>>();
    let after = expand_delta_sequence(&capture["vram"]["after_sequence"])
        .into_iter()
        .map(|row| u8::try_from(row[0]).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(before.len(), 0x10000);
    assert_eq!(after.len(), 0x10000);
    assert_eq!(
        capture["vram"]["before_sha256"],
        parity::evidence::sha256_bytes(&before)
    );
    assert_eq!(
        capture["vram"]["after_sha256"],
        parity::evidence::sha256_bytes(&after)
    );
    assert_ne!(before, after);
}

#[test]
fn failed_session_keeps_the_complete_authoritative_input_script() {
    let source = b"0..5722 0x0000\n5723..21442 0x0080\n";
    let completed_prefix = [(0, 0), (1, 0), (2, 0)];

    assert_eq!(
        replayable_input_artifact(Some(source), &completed_prefix),
        source
    );
}

#[test]
fn session_without_an_input_script_persists_its_captured_history() {
    let captured = [(0, 0), (1, 0x80), (2, 0x80)];

    assert_eq!(
        replayable_input_artifact(None, &captured),
        super::format_input_history(&captured).into_bytes()
    );
}

#[test]
fn maps_cropped_libretro_video_rows_back_to_snes9x_presented_scanlines() {
    assert_eq!(snes9x_presented_scanline_for_video_y(224, 133), 140);
    assert_eq!(snes9x_presented_scanline_for_video_y(448, 266), 280);
    assert_eq!(snes9x_presented_scanline_for_video_y(239, 133), 133);
}

struct PresentedObjCacheFixture {
    meta: [i32; 5],
    validity: Vec<i32>,
    word_addresses: Vec<i32>,
    pixels: Vec<i32>,
}

impl PresentedObjCacheFixture {
    fn new(page_0_base: i32, page_1_base: i32) -> Self {
        Self {
            meta: [1, 512, 64, page_0_base, page_1_base],
            validity: vec![0; 512],
            word_addresses: vec![-1; 512],
            pixels: vec![0; 512 * 64],
        }
    }

    fn publish(&mut self, slot: usize) {
        let (page_base, page_slot) = if slot < 256 {
            (self.meta[3], slot)
        } else {
            (self.meta[4], slot - 256)
        };
        self.validity[slot] = 1;
        self.word_addresses[slot] = (page_base + (page_slot * 16) as i32) & 0x7fff;
        for pixel in 0..64 {
            self.pixels[slot * 64 + pixel] = (pixel & 0x0f) as i32;
        }
    }

    fn decode(&self) -> Result<Option<zelda3::PresentedObjTiles>, String> {
        decode_snes9x_presented_obj_tiles(|field, index| {
            let index = usize::try_from(index).ok()?;
            match field {
                29 => self.pixels.get(index).copied(),
                30 => self.validity.get(index).copied(),
                45 => self.word_addresses.get(index).copied(),
                46 => self.meta.get(index).copied(),
                _ => None,
            }
        })
    }
}

#[test]
fn address_bearing_obj_cache_maps_the_second_obsel_page() {
    let mut fixture = PresentedObjCacheFixture::new(0x4000, 0x5800);
    fixture.publish(256);

    let receipt = fixture.decode().unwrap().unwrap();
    let receipt = serde_json::to_value(receipt).unwrap();
    assert_eq!(receipt["tile_word_addresses"], serde_json::json!([0x5800]));
    assert_eq!(receipt["tile_pixels"].as_array().unwrap().len(), 64);
    assert_eq!(receipt["tile_pixels"][0], 0);
    assert_eq!(receipt["tile_pixels"][15], 15);
    assert_eq!(receipt["tile_pixels"][63], 15);
}

#[test]
fn address_bearing_obj_cache_rejects_duplicate_physical_tiles() {
    let mut fixture = PresentedObjCacheFixture::new(0x4000, 0x4000);
    fixture.publish(0);
    fixture.publish(256);

    let error = fixture.decode().unwrap_err();
    assert!(
        error.contains("repeats physical word address 0x4000"),
        "{error}"
    );
}

#[test]
fn address_bearing_obj_cache_requires_exact_validity_and_address_shape() {
    let mut fixture = PresentedObjCacheFixture::new(0x4000, 0x5800);
    fixture.validity[0] = 3;
    assert!(fixture
        .decode()
        .unwrap_err()
        .contains("validity 0 is invalid: 3"));

    fixture.validity[0] = 0;
    fixture.word_addresses[0] = 0x4000;
    assert!(fixture
        .decode()
        .unwrap_err()
        .contains("invalid presented OBJ cache slot 0 has word address 16384"));

    fixture.word_addresses[0] = -1;
    fixture.publish(256);
    fixture.word_addresses[256] = 0x5801;
    assert!(fixture
        .decode()
        .unwrap_err()
        .contains("word address 256 is invalid: 22529"));
}

#[test]
fn address_bearing_obj_cache_rejects_old_or_malformed_abi() {
    assert_eq!(super::ORIGINAL_TIMING_HOST_RECEIPT_SCHEMA, 138);
    assert_eq!(
        decode_snes9x_presented_obj_tiles(|_, _| None).unwrap(),
        None
    );

    let mut fixture = PresentedObjCacheFixture::new(0x4000, 0x5800);
    fixture.meta[0] = -1;
    assert!(fixture
        .decode()
        .unwrap_err()
        .contains("unsupported presented OBJ cache ABI -1"));

    fixture.meta[0] = 1;
    fixture.meta[1] = 64;
    assert!(fixture
        .decode()
        .unwrap_err()
        .contains("slot count is invalid: 64"));
}

#[test]
fn song_end_poll_receipt_keeps_only_exact_source_read_timing() {
    assert_eq!(
        super::song_end_poll_native_sample_offset(0x08_c400, 0, true, 75, 534)
            .unwrap()
            .unwrap(),
        75,
    );
    assert_eq!(
        super::song_end_poll_native_sample_offset(0x08_c609, 0, true, 96, 534)
            .unwrap()
            .unwrap(),
        96,
    );
    assert!(super::song_end_poll_native_sample_offset(0x00_80e4, 0, true, 75, 534).is_none());
    assert!(super::song_end_poll_native_sample_offset(0x08_c400, 0, false, 75, 534).is_none());
    assert!(super::song_end_poll_native_sample_offset(0x08_c400, 1, true, 75, 534).is_none());
}

#[test]
fn song_end_poll_receipt_rejects_offsets_outside_the_host_audio_window() {
    assert!(
        super::song_end_poll_native_sample_offset(0x08_c609, 0, true, -1, 534)
            .unwrap()
            .unwrap_err()
            .contains("negative sample offset")
    );
    assert!(
        super::song_end_poll_native_sample_offset(0x08_c609, 0, true, 535, 534)
            .unwrap()
            .unwrap_err()
            .contains("beyond the 534-sample host window")
    );
}

#[test]
fn obj_cache_comparison_ignores_invalid_snes9x_tiles() {
    let mut rust = vec![0; 64 * 64];
    let mut oracle = rust.clone();
    let mut valid = vec![0; 64];
    rust[3] = 1;
    oracle[3] = 2;
    rust[64 + 7] = 3;
    oracle[64 + 7] = 4;
    valid[1] = 1;

    assert_eq!(
        summarize_presented_obj_cache(Some(&rust), Some(&oracle), Some(&valid)),
        Some(ValueDomainDiff {
            rust_values: 64,
            oracle_values: 64,
            mismatched_values: 1,
            first_mismatch: Some(71),
        })
    );
}

#[test]
fn obj_state_ledger_hash_is_stable_fnv1a() {
    assert_eq!(fnv1a32([0, 1, 2, 3]), 0xc3aa_51b1);
}

#[test]
fn compact_engine_state_reports_the_first_semantic_scheduler_drift() {
    let mut rust = vec![0; 0x1000];
    let mut oracle = rust.clone();
    rust[0x10] = 7;
    oracle[0x10] = 7;
    rust[0x11] = 0x0e;
    oracle[0x11] = 0x0e;
    rust[0xb0] = 3;
    oracle[0xb0] = 4;
    rust[0x22..0x24].copy_from_slice(&0x05a9u16.to_le_bytes());
    oracle[0x22..0x24].copy_from_slice(&0x05a8u16.to_le_bytes());
    rust[0xe2..0xe4].copy_from_slice(&0x034eu16.to_le_bytes());
    oracle[0xe2..0xe4].copy_from_slice(&0x028eu16.to_le_bytes());
    rust[0x0df1] = 0x58;
    oracle[0x0df1] = 0x59;
    rust[0x0eb1] = 2;
    oracle[0x0eb1] = 3;

    assert_eq!(
        compact_engine_state_mismatches(&rust, &oracle),
        [
            "subsubmodule rust=0x03 oracle=0x04",
            "link_x rust=0x05a9 oracle=0x05a8",
            "bg2_h rust=0x034e oracle=0x028e",
            "sprite[1].head_direction rust=0x02 oracle=0x03",
            "sprite[1].delay_main rust=0x58 oracle=0x59",
        ]
    );
}

#[test]
fn engine_state_comparison_fails_closed_with_an_explicit_diagnostic_opt_out() {
    assert_eq!(
        resolve_engine_state_compare_start(123, None, false),
        Ok(Some(123))
    );
    assert_eq!(
        resolve_engine_state_compare_start(123, Some(456), false),
        Ok(Some(456))
    );
    assert_eq!(
        resolve_engine_state_compare_start(123, None, true),
        Ok(None)
    );
    assert_eq!(
        resolve_engine_state_compare_start(123, Some(456), true),
        Err(
            "--ignore-engine-state cannot be combined with --compare-engine-state-from-frame"
                .to_string()
        )
    );
}

#[test]
fn live_oracle_rng_accepts_only_the_cartridge_store_site() {
    let sample = oracle_rng_sample_from_trace_line(
        r#"{"event":"rng-write","run":24377,"pc":899711,"value":134,"carry":1}"#,
        24377,
        24377,
    )
    .unwrap()
    .unwrap();
    assert_eq!(sample, RomRandomSample::with_carry(24377, 134, true));

    assert!(oracle_rng_sample_from_trace_line(
        r#"{"event":"rng-write","run":24377,"pc":57291,"value":36,"carry":0}"#,
        24377,
        24377,
    )
    .unwrap()
    .is_none());
    assert!(oracle_rng_sample_from_trace_line(
        r#"{"event":"rng-ppu-read","run":24377,"pc":899700,"value":1,"carry":0}"#,
        24377,
        24377,
    )
    .unwrap()
    .is_none());
    assert!(oracle_rng_sample_from_trace_line(
        r#"{"event":"dma","run":24377,"frame":24377,"v":228,"cycles":24}"#,
        24377,
        24377,
    )
    .unwrap()
    .is_none());
}

#[test]
fn live_oracle_rng_preserves_requested_hardware_trace_domains() {
    assert_eq!(trace_events_with_rom_rng(None), "rom-rng");
    assert_eq!(
        trace_events_with_rom_rng(Some("frame,nmi,dma")),
        "frame,nmi,dma,rom-rng"
    );
    assert_eq!(
        trace_events_with_rom_rng(Some("nmi,rom-rng,dma")),
        "nmi,rom-rng,dma"
    );
}

#[test]
fn live_oracle_rng_rejects_cross_frame_samples() {
    let error = oracle_rng_sample_from_trace_line(
        r#"{"event":"rng-write","run":24486,"pc":899711,"value":125,"carry":0}"#,
        24458,
        24458,
    )
    .unwrap_err();
    assert!(error.contains("run 24486"));
    assert!(error.contains("frame 24458"));
}

#[test]
fn live_oracle_rng_maps_resumed_trace_run_to_absolute_execution_frame() {
    let sample = oracle_rng_sample_from_trace_line(
        r#"{"event":"rng-write","run":119,"pc":899711,"value":32,"carry":1}"#,
        119,
        56_119,
    )
    .unwrap()
    .unwrap();

    assert_eq!(sample, RomRandomSample::with_carry(56_119, 32, true));
}

#[test]
fn oracle_capture_rejects_stale_or_mismatched_rng_scripts() {
    let expected = [
        RomRandomSample::with_carry(7, 0x22, false),
        RomRandomSample::with_carry(7, 0x45, true),
        RomRandomSample::with_carry(9, 0x88, false),
    ];
    let mut cursor = 0;
    assert!(validate_oracle_rng_samples_for_run(&expected, &mut cursor, 6, &[]).is_ok());
    assert!(
        validate_oracle_rng_samples_for_run(&expected, &mut cursor, 7, &expected[..2],).is_ok()
    );
    assert_eq!(cursor, 2);
    let mismatch = validate_oracle_rng_samples_for_run(
        &expected,
        &mut cursor,
        9,
        &[RomRandomSample::with_carry(9, 0x89, false)],
    )
    .unwrap_err();
    assert!(mismatch.contains("frame 9"));

    let mut stale_cursor = 0;
    let stale =
        validate_oracle_rng_samples_for_run(&expected, &mut stale_cursor, 8, &[]).unwrap_err();
    assert!(stale.contains("frame 7"));
}

#[test]
fn parse_debug_frame_selection_expands_and_deduplicates_ranges() {
    assert_eq!(
        parse_debug_frame_selection("81,79-81,84..=85,invalid,8-3"),
        vec![79, 80, 81, 84, 85]
    );
}

#[test]
fn session_receipts_do_not_force_a_frontier_probe_to_scan_past_its_first_mismatch() {
    assert!(!scan_all_policy(false, true));
    assert!(scan_all_policy(true, true));
    assert!(should_stop_after_first_mismatch(false, true, false));
    assert!(should_stop_after_first_mismatch(false, false, true));
    assert!(!should_stop_after_first_mismatch(true, true, true));
}

#[test]
fn late_probe_receipts_skip_the_uncompared_warmup() {
    assert!(!should_write_frame_receipt(10_000, 10_000, 10_100, false));
    assert!(should_write_frame_receipt(10_000, 10_000, 10_100, true));
}

#[test]
fn long_cold_sweeps_sample_receipts_without_losing_the_boundary() {
    assert!(should_write_frame_receipt(0, 0, 20_000, true));
    assert!(!should_write_frame_receipt(1, 0, 20_000, true));
    assert!(should_write_frame_receipt(60, 0, 20_000, true));
    assert!(should_write_frame_receipt(12_345, 12_345, 30_000, true));
    assert!(!should_write_frame_receipt(12_346, 12_345, 30_000, true));
}

#[test]
fn late_video_windows_render_only_the_priming_tail_and_comparison() {
    assert!(!should_render_video_frame(10_000, 24_427, true));
    assert!(!should_render_video_frame(24_366, 24_427, true));
    assert!(should_render_video_frame(24_367, 24_427, true));
    assert!(should_render_video_frame(24_427, 24_427, true));
    assert!(!should_render_video_frame(24_427, 24_427, false));
}

#[test]
fn engine_receipts_retain_full_sprite_motion_witnesses() {
    let mut ram = vec![0; 0x20_000];
    ram[0x0aa3] = 0x2a;
    ram[0xc2fc..0xc300].copy_from_slice(&[0x10, 0x20, 0x30, 0x40]);
    ram[0x0d10] = 0x68;
    ram[0x0d30] = 0x04;
    ram[0x0d70] = 0x80;
    ram[0x0d50] = 0x08;
    ram[0x0e70] = 0x02;

    let receipt = libretro_engine_state_receipt(&ram);
    assert_eq!(receipt["sprite_graphics_index"], 0x2a);
    assert_eq!(
        receipt["sprite_graphics_subsets"],
        serde_json::json!([0x10, 0x20, 0x30, 0x40])
    );
    let slot = &receipt["sprite_slots"][0];
    assert_eq!(slot["x"], 0x0468);
    assert_eq!(slot["x_subpixel"], 0x80);
    assert_eq!(slot["x_velocity"], 0x08);
    assert_eq!(slot["wall_collision"], 0x02);
}

#[test]
fn engine_receipts_retain_full_ancilla_motion_witnesses() {
    let mut ram = vec![0; 0x20_000];
    ram[0x0c04] = 0x68;
    ram[0x0c18] = 0x04;
    ram[0x0c40] = 0x80;
    ram[0x0c2c] = 0x08;
    ram[0x0c4a] = 0x02;
    ram[0x0c90] = 0x04;

    let receipt = libretro_engine_state_receipt(&ram);
    let slot = &receipt["ancilla_slots"][0];
    assert_eq!(slot["x"], 0x0468);
    assert_eq!(slot["x_subpixel"], 0x80);
    assert_eq!(slot["x_velocity"], 0x08);
    assert_eq!(slot["type"], 0x02);
    assert_eq!(slot["num_sprites"], 0x04);
}

#[test]
fn engine_receipts_retain_each_oam_shadow_entry() {
    let mut ram = vec![0; 0x20_000];
    ram[0x0800 + 37 * 4..0x0800 + 38 * 4].copy_from_slice(&[0x68, 0x57, 0x40, 0x3c]);

    let receipt = libretro_engine_state_receipt(&ram);

    assert_eq!(receipt["oam_slots"].as_array().unwrap().len(), 128);
    assert_eq!(
        receipt["oam_slots"][37],
        serde_json::json!({
            "slot": 37,
            "x": 0x68,
            "y": 0x57,
            "tile": 0x40,
            "flags": 0x3c,
        })
    );
    assert_eq!(receipt["oam_extended"].as_array().unwrap().len(), 32);
}

#[test]
fn dsp_event_timing_comparison_reports_phase_and_length_drift() {
    let rust = [DspWriteEvent::new(0x5c, 8, 228, 15)];
    let exact = [LibretroDspRegisterWrite {
        register: 0x5c,
        value: 8,
        output_sample: 228,
        dsp_phase: 15,
    }];
    assert_eq!(first_dsp_write_timing_mismatch(&rust, &exact), None);

    let phase_drift = [LibretroDspRegisterWrite {
        dsp_phase: 16,
        ..exact[0]
    }];
    assert_eq!(
        first_dsp_write_timing_mismatch(&rust, &phase_drift),
        Some(0)
    );
    assert_eq!(first_dsp_write_timing_mismatch(&rust, &[]), Some(0));
}

#[test]
fn spc_clock_witness_aligns_different_frame_boundary_instructions() {
    let rust_instruction = snes::apu::SpcInstructionTrace {
        cycle: 0,
        pc: 0x1234,
        opcode: 0xe8,
        operands: [0; 2],
        a: 1,
        x: 2,
        y: 3,
        sp: 4,
        p: false,
        direct_page_0_3: [0; 4],
        direct_page_4_7: [0; 4],
        direct_page_8_11: [0; 4],
        input_ports: [0; 4],
        timer0_cycles: 100,
        timer0_divider: 6,
        timer0_counter: 0,
    };
    let oracle_instruction = crate::libretro_core::LibretroSmpInstruction {
        absolute_cycle: 0,
        program_counter: 0x1234,
        opcode: 0xe8,
        a: 1,
        x: 2,
        y: 3,
        stack_pointer: 4,
        status: 0,
        direct_page_0_11: [0; 12],
        timer0_stage1: 27,
        timer0_stage2: 6,
        timer0_stage3: 0,
        output_sample: 0,
        dsp_phase: 0,
        smp_clock: 0,
        boundary_opcode_cycle: 0,
        op_step_calls: 1,
        max_continuation_opcode_cycle: 0,
    };

    let witness =
        last_spc_clock_witness(&[rust_instruction, rust_instruction], &[oracle_instruction])
            .unwrap();

    assert_eq!(witness.phase_delta, 1);
    assert_eq!(witness.rust_tail, 0);
    assert_eq!(witness.oracle_tail, 0);
}

#[test]
fn authoritative_oracle_never_serializes_during_frame_execution() {
    assert!(!oracle_preframe_snapshot_required(10_000, 23_005, false));
    assert!(!oracle_preframe_snapshot_required(23_000, 23_005, true));
    assert!(!oracle_preframe_snapshot_required(23_004, 23_005, false));
}

#[test]
fn replay_bundle_rejects_requests_past_its_recorded_coverage() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-replay-bundle-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    let rom = root.join("test.sfc");
    fs::write(&rom, b"rom").unwrap();
    fs::write(root.join("input.txt"), b"0 0\n").unwrap();
    fs::write(root.join("rom-random.txt"), b"").unwrap();
    fs::write(root.join("initial.srm"), b"sram").unwrap();
    let rom_sha256 = parity::runner::sha256_file(&rom).unwrap();
    let rng_sha256 = parity::runner::sha256_file(&root.join("rom-random.txt")).unwrap();
    fs::write(
        root.join("manifest.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema": 1,
            "frames_completed": 2298,
            "rom": { "sha256": rom_sha256 },
            "rom_random_replay": { "sha256": rng_sha256 },
        }))
        .unwrap(),
    )
    .unwrap();

    let error = resolve_replay_bundle(&root, 15_000, &rom).unwrap_err();
    assert!(error.contains("proven through frame 2298"), "{error}");
    assert!(resolve_replay_bundle(&root, 2_298, &rom).is_ok());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn replay_sources_from_different_directories_require_an_explicit_unsafe_opt_in() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-mixed-replay-{}-{unique}",
        std::process::id()
    ));
    let first = root.join("first");
    let second = root.join("second");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    let input = first.join("input.txt");
    let rng = second.join("rom-random.txt");
    fs::write(&input, b"").unwrap();
    fs::write(&rng, b"").unwrap();

    let sources = [
        ("--input-script", Some(input.as_path())),
        ("--rom-random-script", Some(rng.as_path())),
        ("--load-sram", None),
    ];
    let error = validate_replay_source_parents(&sources, false).unwrap_err();
    assert!(
        error.contains("mixed replay provenance is unsafe"),
        "{error}"
    );
    assert!(validate_replay_source_parents(&sources, true).is_ok());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn paired_resume_capture_keeps_an_absolute_pre_frame_boundary() {
    assert_eq!(
        parse_paired_resume_capture("12120", "target/parity-checkpoints/frontier").unwrap(),
        PairedResumeCapture {
            frame: 12120,
            dir: PathBuf::from("target/parity-checkpoints/frontier"),
        }
    );
    assert!(parse_paired_resume_capture("not-a-frame", "unused").is_err());
}

#[test]
fn rolling_paired_resume_schedules_the_next_interval_boundary() {
    assert_eq!(
        parse_rolling_paired_resume_capture("256", "target/parity-checkpoints/frontier").unwrap(),
        RollingPairedResumeCapture {
            interval: 256,
            root: PathBuf::from("target/parity-checkpoints/frontier"),
        }
    );
    assert!(parse_rolling_paired_resume_capture("0", "unused").is_err());
    assert_eq!(rolling_capture_frame_after(0, 256), 256);
    assert_eq!(rolling_capture_frame_after(255, 256), 256);
    assert_eq!(rolling_capture_frame_after(256, 256), 512);
    assert_eq!(rolling_capture_frame_after(3400, 256), 3584);
}

#[test]
fn paired_resume_manifest_paths_cannot_escape_the_checkpoint() {
    let root = Path::new("target/parity-checkpoints/frontier");
    assert_eq!(
        checkpoint_member(root, "rust.z3state").unwrap(),
        root.join("rust.z3state")
    );
    assert!(checkpoint_member(root, "../rust.z3state").is_err());
    assert!(checkpoint_member(root, "/tmp/rust.z3state").is_err());
}

#[test]
fn paired_resume_root_resolves_its_latest_complete_generation() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-paired-resume-{}-{unique}",
        std::process::id()
    ));
    let checkpoint = write_paired_resume_test_generation(&root, "frame-00003700", 3700, 3700);
    fs::write(
        root.join("latest.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema": PAIRED_RESUME_SCHEMA,
            "frame": 3700,
            "checkpoint": "frame-00003700",
        }))
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        paired_resume_paths(&root).unwrap(),
        (
            checkpoint.join("rust.z3state"),
            checkpoint.join("oracle.state"),
            checkpoint.join("original-timing.resume.json"),
            checkpoint.join("semantic-trace.checkpoint.json")
        )
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn paired_resume_v2_verifies_every_bound_artifact() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-paired-resume-hashes-{}-{unique}",
        std::process::id()
    ));
    let checkpoint = write_paired_resume_test_generation(&root, "checkpoint", 3700, 3700);
    for (artifact, expected_label) in [
        ("rust.z3state", "Rust state"),
        ("oracle.state", "oracle state"),
        ("original-timing.resume.json", "original-timing checkpoint"),
        (
            "semantic-trace.checkpoint.json",
            "semantic-trace checkpoint",
        ),
        ("initial.srm", "initial SRAM"),
    ] {
        let path = checkpoint.join(artifact);
        let original = fs::read(&path).unwrap();
        fs::write(&path, b"mixed checkpoint artifact").unwrap();
        let error = paired_resume_paths(&checkpoint).unwrap_err();
        assert!(
            error.contains(expected_label) && error.contains("hash mismatch"),
            "unexpected {artifact} validation error: {error}"
        );
        fs::write(path, original).unwrap();
    }
    assert!(paired_resume_paths(&checkpoint).is_ok());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn paired_resume_provenance_binds_every_selected_causal_source() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-paired-resume-provenance-{}-{unique}",
        std::process::id()
    ));
    let checkpoint = write_paired_resume_test_generation(&root, "checkpoint", 3700, 3700);
    let core = root.join("core.dylib");
    let rom = root.join("zelda3.sfc");
    let input = root.join("input.txt");
    let rng = root.join("rom-random.txt");
    for (path, bytes) in [
        (&core, b"core".as_slice()),
        (&rom, b"rom".as_slice()),
        (&input, b"input".as_slice()),
        (&rng, b"rng".as_slice()),
    ] {
        fs::write(path, bytes).unwrap();
    }
    let manifest_path = checkpoint.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    for (key, path) in [
        ("core", core.as_path()),
        ("rom", rom.as_path()),
        ("input_script", input.as_path()),
        ("rom_random_script", rng.as_path()),
    ] {
        manifest[key] = serde_json::json!({
            "sha256": parity::evidence::sha256_file(path).unwrap(),
        });
    }
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();

    assert!(validate_paired_resume_provenance(
        &checkpoint,
        &core,
        &rom,
        Some(&input),
        Some(&rng),
        false,
    )
    .is_ok());
    for (path, label) in [
        (&core, "core"),
        (&rom, "ROM"),
        (&input, "input script"),
        (&rng, "ROM-random script"),
    ] {
        let original = fs::read(path).unwrap();
        fs::write(path, b"different selected source").unwrap();
        let error = validate_paired_resume_provenance(
            &checkpoint,
            &core,
            &rom,
            Some(&input),
            Some(&rng),
            false,
        )
        .unwrap_err();
        assert!(
            error.contains(label) && error.contains("mismatch"),
            "{error}"
        );
        assert!(validate_paired_resume_provenance(
            &checkpoint,
            &core,
            &rom,
            Some(&input),
            Some(&rng),
            true,
        )
        .is_ok());
        fs::write(path, original).unwrap();
    }
    let error =
        validate_paired_resume_provenance(&checkpoint, &core, &rom, None, Some(&rng), false)
            .unwrap_err();
    assert!(
        error.contains("input script provenance requires"),
        "{error}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn paired_resume_rejects_progressed_state_sram_replacement() {
    assert!(validate_paired_resume_sram_selection(false, false).is_ok());
    assert!(validate_paired_resume_sram_selection(false, true).is_ok());
    assert!(validate_paired_resume_sram_selection(true, false).is_ok());
    let error = validate_paired_resume_sram_selection(true, true).unwrap_err();
    assert!(
        error.contains("cannot be combined with --load-sram"),
        "{error}"
    );
    assert!(error.contains("provenance"), "{error}");
}

#[test]
fn paired_generation_install_is_atomic_and_never_replaces_same_frame() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-paired-atomic-{}-{unique}",
        std::process::id()
    ));
    let generation = root.join("frame-00003700");
    install_directory_atomically(&generation, |temporary| {
        fs::write(temporary.join("manifest.json"), b"first")?;
        Ok(())
    })
    .unwrap();
    let error = install_directory_atomically(&generation, |temporary| {
        fs::write(temporary.join("manifest.json"), b"replacement")?;
        Ok(())
    })
    .unwrap_err();
    assert!(error.to_string().contains("refusing to replace"), "{error}");
    assert_eq!(
        fs::read(generation.join("manifest.json")).unwrap(),
        b"first"
    );

    let failed = root.join("frame-00003800");
    let error = install_directory_atomically(&failed, |temporary| {
        fs::write(temporary.join("partial"), b"partial")?;
        Err("injected write failure".into())
    })
    .unwrap_err();
    assert!(error.to_string().contains("injected write failure"));
    assert!(!failed.exists());
    assert!(!root
        .join(format!(".frame-00003800.tmp-{}", std::process::id()))
        .exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rolling_latest_pointer_is_replaced_as_one_complete_file() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-paired-latest-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    let latest = root.join("latest.json");
    fs::write(&latest, br#"{"schema":2,"frame":100}"#).unwrap();
    let replacement = br#"{"schema":2,"frame":200}"#;
    write_file_atomically(&latest, replacement).unwrap();
    assert_eq!(fs::read(&latest).unwrap(), replacement);
    assert!(!root
        .join(format!(".latest.json.tmp-{}", std::process::id()))
        .exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn oracle_av_capture_rejects_every_mid_run_checkpoint_interval() {
    assert!(validate_oracle_av_checkpoint_interval(None).is_ok());
    for interval in [1, 5_000, u32::MAX] {
        let error = validate_oracle_av_checkpoint_interval(Some(interval)).unwrap_err();
        assert!(
            error.contains("serialization mutates live DSP state"),
            "{error}"
        );
    }
}

#[test]
fn cold_evidence_invocation_id_has_the_receipt_safe_grammar() {
    for value in ["run-10000-123.abc", "A_b-9"] {
        assert!(validate_cold_evidence_invocation_id(value).is_ok());
    }
    for value in ["", "contains/slash", "contains space", "contains:colon"] {
        assert!(
            validate_cold_evidence_invocation_id(value).is_err(),
            "{value:?}"
        );
    }
    assert!(validate_cold_evidence_invocation_id(&"a".repeat(129)).is_err());
}

#[test]
fn cold_evidence_run_nonce_is_runner_authored_and_source_bound() {
    let session = Path::new("/tmp/zelda3-cold-session");
    let nonce = cold_evidence_run_nonce(session, "invocation-1", 42, 7);
    assert_eq!(nonce.len(), 64);
    assert!(nonce.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(
        nonce,
        cold_evidence_run_nonce(session, "invocation-1", 42, 7)
    );
    assert_ne!(
        nonce,
        cold_evidence_run_nonce(session, "invocation-1", 43, 7)
    );
    assert_ne!(
        nonce,
        cold_evidence_run_nonce(session, "invocation-2", 42, 7)
    );
}

#[test]
fn paired_resume_direct_directory_rejects_host_frame_mismatch_before_restore() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-paired-resume-frame-{}-{unique}",
        std::process::id()
    ));
    let checkpoint = write_paired_resume_test_generation(&root, "checkpoint", 3700, 3699);
    let rust_state_before = fs::read(checkpoint.join("rust.z3state")).unwrap();
    let timing_sidecar_before = fs::read(checkpoint.join("original-timing.resume.json")).unwrap();
    let error = paired_resume_paths(&checkpoint).unwrap_err();
    assert!(error.contains("records frame 3700"), "{error}");
    assert!(
        error.contains("Rust checkpoint records frame 3699"),
        "{error}"
    );
    assert_eq!(
        fs::read(checkpoint.join("rust.z3state")).unwrap(),
        rust_state_before,
        "validation must not rewrite the restored Rust state"
    );
    assert_eq!(
        fs::read(checkpoint.join("original-timing.resume.json")).unwrap(),
        timing_sidecar_before,
        "validation must fail before any sidecar restoration or rewrite"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn paired_resume_schema_one_is_rejected_before_artifact_access() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-paired-resume-v1-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("manifest.json"),
        br#"{"schema":1,"boundary":"pre-frame","frame":3700}"#,
    )
    .unwrap();
    let error = paired_resume_paths(&root).unwrap_err();
    assert!(
        error.contains("unsupported paired-resume schema 1"),
        "{error}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rolling_prune_preserves_the_new_generation_after_a_restart() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zelda3-rolling-prune-{}-{unique}",
        std::process::id()
    ));
    for frame in [3600, 3700, 100] {
        let dir = root.join(format!("frame-{frame:08}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.json"), b"{}").unwrap();
    }
    let current = root.join("frame-00000100");

    prune_rolling_paired_resume_captures(&root, 2, &current);

    assert!(current.is_dir());
    assert_eq!(
        fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .count(),
        2
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn boot_boundary_reports_the_first_named_semantic_difference() {
    let mut rust_ram = vec![0; 0x20];
    let mut oracle_ram = vec![0; 0x20];
    rust_ram[0x13] = 0x0f;
    oracle_ram[0x13] = 0x0e;
    oracle_ram[0x17] = 3;

    let rust = BootBoundaryState::from_ram(82, "after", &rust_ram);
    let oracle = BootBoundaryState::from_ram(82, "after", &oracle_ram);

    assert_eq!(
        rust.first_difference(&oracle),
        Some(("inidisp", 0x0f, 0x0e))
    );
}

#[test]
fn value_domain_diff_reports_content_and_generation_length_skew() {
    assert_eq!(
        summarize_value_domain(&[1, 2, 3], &[1, 4, 3, 5]),
        ValueDomainDiff {
            rust_values: 3,
            oracle_values: 4,
            mismatched_values: 2,
            first_mismatch: Some(1),
        }
    );
    assert!(summarize_value_domain(&[1, 2], &[1, 2]).is_exact());
}

#[test]
fn vram_receipt_reports_first_word_and_compact_ranges() {
    let rust = [0x1111, 0x2222, 0x3333, 0x4444, 0x5555];
    let oracle_words = [0x1111u16, 0xaaaa, 0xbbbb, 0x4444, 0xcccc];
    let oracle = oracle_words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();

    assert_eq!(
        vram_domain_receipt(&rust, Some(&oracle)),
        Some(VramDomainReceipt {
            rust_words: 5,
            oracle_words: 5,
            rust_sha256: parity::evidence::sha256_bytes(
                &rust
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>(),
            ),
            oracle_sha256: parity::evidence::sha256_bytes(&oracle),
            mismatched_words: 3,
            first_mismatch_word: Some(1),
            first_rust_word: Some(0x2222),
            first_oracle_word: Some(0xaaaa),
            mismatch_ranges: vec![[1, 3], [4, 5]],
            mismatch_ranges_truncated: false,
            mismatch_blocks: vec![[0, 3]],
        })
    );
}

#[test]
fn canonical_video_hash_ignores_alpha_and_libretro_pitch_padding() {
    let rust = [10, 20, 30, 0, 40, 50, 60, 1];
    let oracle = LibretroFrame {
        audio: Vec::new(),
        // Libretro XRGB8888 is exposed as B,G,R,X bytes here. The final
        // four bytes are row padding and must not enter the digest.
        video: vec![30, 20, 10, 255, 60, 50, 40, 128, 9, 9, 9, 9],
        video_width: 2,
        video_height: 1,
        video_pitch: 12,
        pixel_format: 1,
    };
    let rust_digest = canonical_rust_video_digest(&rust, 2, 1).unwrap();
    let oracle_digest = canonical_oracle_video_digest(&oracle).unwrap();
    assert_eq!(rust_digest["sha256"], oracle_digest["sha256"]);
}

#[test]
fn canonical_audio_hash_is_little_endian_interleaved_i16() {
    let samples = [0x1234_i16, -2_i16];
    let digest = canonical_audio_digest(&samples);
    assert_eq!(digest["sample_frames"], 1);
    assert_eq!(digest["channels"], 2);
    assert_eq!(
        digest["sha256"],
        parity::evidence::sha256_bytes(&[0x34, 0x12, 0xfe, 0xff])
    );
}

#[test]
fn cached_av_inputs_require_an_explicit_hexadecimal_receipt() {
    assert_eq!(cached_ledger_input("0x8080").unwrap(), 0x8080);
    assert!(cached_ledger_input("32896").is_err());
    assert!(cached_ledger_input("0x10000").is_err());
}
