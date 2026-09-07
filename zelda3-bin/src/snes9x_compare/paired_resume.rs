//! Split out of `snes9x_compare.rs` by topic (paired_resume). Mechanical move:
//! bodies are unchanged; private items became `pub(crate)` and `super::`
//! paths became `crate::` (the parent's super is the crate root).

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PairedResumeCapture {
    pub(crate) frame: u32,
    pub(crate) dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RollingPairedResumeCapture {
    pub(crate) interval: u32,
    pub(crate) root: PathBuf,
}

pub(crate) fn parse_paired_resume_capture(
    frame: &str,
    dir: &str,
) -> Result<PairedResumeCapture, String> {
    let frame = frame
        .parse()
        .map_err(|error| format!("invalid paired-resume frame `{frame}`: {error}"))?;
    Ok(PairedResumeCapture {
        frame,
        dir: PathBuf::from(dir),
    })
}

pub(crate) fn parse_rolling_paired_resume_capture(
    interval: &str,
    root: &str,
) -> Result<RollingPairedResumeCapture, String> {
    let interval = interval
        .parse()
        .map_err(|error| format!("invalid rolling paired-resume interval `{interval}`: {error}"))?;
    if interval == 0 {
        return Err("rolling paired-resume interval must be greater than zero".to_string());
    }
    Ok(RollingPairedResumeCapture {
        interval,
        root: PathBuf::from(root),
    })
}

pub(crate) fn rolling_capture_frame_after(frame: u32, interval: u32) -> u32 {
    debug_assert_ne!(interval, 0);
    frame
        .checked_div(interval)
        .expect("rolling paired-resume interval is validated")
        .saturating_add(1)
        .saturating_mul(interval)
}

#[derive(Debug, Deserialize)]
pub(crate) struct PairedResumeManifest {
    pub(crate) schema: u32,
    pub(crate) boundary: String,
    pub(crate) frame: u32,
    pub(crate) rust_state: PairedResumeArtifact,
    /// Absent on a Rust-only checkpoint written by the cached A/V replay when
    /// its cache carries no oracle checkpoint at that boundary. Such a
    /// checkpoint resumes only the Rust-only cached replay (which never loads
    /// an oracle state); the live Snes9x compare rejects it.
    pub(crate) oracle_state: Option<PairedResumeArtifact>,
    pub(crate) original_timing_resume_checkpoint: PairedResumeArtifact,
    pub(crate) semantic_trace_checkpoint: Option<PairedResumeArtifact>,
    pub(crate) core: PairedResumeProvenance,
    pub(crate) rom: PairedResumeProvenance,
    pub(crate) input_script: Option<PairedResumeProvenance>,
    pub(crate) rom_random_script: Option<PairedResumeProvenance>,
    pub(crate) initial_sram: PairedResumeArtifact,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PairedResumeArtifact {
    pub(crate) artifact: String,
    pub(crate) sha256: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PairedResumeProvenance {
    pub(crate) sha256: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LatestPairedResume {
    pub(crate) schema: u32,
    pub(crate) frame: u32,
    pub(crate) checkpoint: String,
}

pub(crate) fn checkpoint_member(dir: &Path, member: &str) -> Result<PathBuf, String> {
    let mut components = Path::new(member).components();
    let Some(Component::Normal(name)) = components.next() else {
        return Err(format!("invalid paired-resume path member `{member}`"));
    };
    if components.next().is_some() {
        return Err(format!("invalid paired-resume path member `{member}`"));
    }
    Ok(dir.join(name))
}

pub(crate) fn resolve_paired_resume_dir(path: &Path) -> Result<(PathBuf, Option<u32>), String> {
    if path.join("manifest.json").is_file() {
        return Ok((path.to_path_buf(), None));
    }
    let latest_path = path.join("latest.json");
    let latest: LatestPairedResume = serde_json::from_slice(
        &fs::read(&latest_path)
            .map_err(|error| format!("failed to read {}: {error}", latest_path.display()))?,
    )
    .map_err(|error| format!("failed to parse {}: {error}", latest_path.display()))?;
    if latest.schema != PAIRED_RESUME_SCHEMA {
        return Err(format!(
            "{} has unsupported paired-resume schema {}; schema {} is required",
            latest_path.display(),
            latest.schema,
            PAIRED_RESUME_SCHEMA,
        ));
    }
    let checkpoint = checkpoint_member(path, &latest.checkpoint)?;
    Ok((checkpoint, Some(latest.frame)))
}

/// Verified member paths of a paired-resume checkpoint. `oracle_state` and
/// `semantic_trace` are `None` for a Rust-only cached A/V checkpoint.
pub(crate) struct PairedResumeArtifacts {
    pub(crate) rust_state: PathBuf,
    pub(crate) oracle_state: Option<PathBuf>,
    pub(crate) original_timing_resume: PathBuf,
    pub(crate) semantic_trace: Option<PathBuf>,
}

/// Paired-resume members for the live Snes9x compare, which needs the oracle
/// state and semantic-trace checkpoint; a Rust-only checkpoint is rejected.
pub(crate) fn paired_resume_paths(
    path: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf), String> {
    let artifacts = paired_resume_artifacts(path)?;
    let (Some(oracle_state), Some(semantic_trace)) =
        (artifacts.oracle_state, artifacts.semantic_trace)
    else {
        return Err(format!(
            "{} is a Rust-only cached A/V checkpoint (no oracle state); it can resume only --replay-cached-snes9x-av",
            path.display()
        ));
    };
    Ok((
        artifacts.rust_state,
        oracle_state,
        artifacts.original_timing_resume,
        semantic_trace,
    ))
}

pub(crate) fn paired_resume_artifacts(path: &Path) -> Result<PairedResumeArtifacts, String> {
    let (dir, expected_frame) = resolve_paired_resume_dir(path)?;
    let manifest_path = dir.join("manifest.json");
    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest_value: serde_json::Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    let schema = manifest_value
        .get("schema")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("{} has no paired-resume schema", manifest_path.display()))?;
    if schema != u64::from(PAIRED_RESUME_SCHEMA) {
        return Err(format!(
            "{} has unsupported paired-resume schema {schema}; schema {} is required",
            manifest_path.display(),
            PAIRED_RESUME_SCHEMA,
        ));
    }
    let manifest: PairedResumeManifest = serde_json::from_value(manifest_value)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    debug_assert_eq!(manifest.schema, PAIRED_RESUME_SCHEMA);
    if manifest.boundary != "pre-frame" {
        return Err(format!(
            "{} is not a supported pre-frame paired resume",
            manifest_path.display()
        ));
    }
    if let Some(expected_frame) = expected_frame {
        if expected_frame != manifest.frame {
            return Err(format!(
                "{} points to frame {expected_frame}, but its checkpoint records frame {}",
                path.join("latest.json").display(),
                manifest.frame
            ));
        }
    }
    let verify_artifact =
        |label: &str, artifact: &PairedResumeArtifact| -> Result<PathBuf, String> {
            let artifact_path = checkpoint_member(&dir, &artifact.artifact)?;
            if !artifact_path.is_file() {
                return Err(format!(
                    "{} paired-resume {label} artifact is missing",
                    artifact_path.display()
                ));
            }
            let actual_sha256 = parity::evidence::sha256_file(&artifact_path)
                .map_err(|error| format!("failed to hash {}: {error}", artifact_path.display()))?;
            if actual_sha256 != artifact.sha256 {
                return Err(format!(
                    "{} paired-resume {label} hash mismatch: expected {}, got {}",
                    artifact_path.display(),
                    artifact.sha256,
                    actual_sha256,
                ));
            }
            Ok(artifact_path)
        };
    let rust_state = verify_artifact("Rust state", &manifest.rust_state)?;
    let oracle_state = manifest
        .oracle_state
        .as_ref()
        .map(|artifact| verify_artifact("oracle state", artifact))
        .transpose()?;
    let original_timing_resume = verify_artifact(
        "original-timing checkpoint",
        &manifest.original_timing_resume_checkpoint,
    )?;
    let semantic_trace = manifest
        .semantic_trace_checkpoint
        .as_ref()
        .map(|artifact| verify_artifact("semantic-trace checkpoint", artifact))
        .transpose()?;
    if oracle_state.is_some() != semantic_trace.is_some() {
        return Err(format!(
            "{} records only one of the oracle state and semantic-trace checkpoint",
            manifest_path.display()
        ));
    }
    verify_artifact("initial SRAM", &manifest.initial_sram)?;
    let rust_checkpoint = load_play_crash_checkpoint(&rust_state)
        .map_err(|error| format!("failed to validate {}: {error}", rust_state.display()))?;
    if rust_checkpoint.host_frame != manifest.frame {
        return Err(format!(
            "{} records frame {}, but its Rust checkpoint records frame {}",
            manifest_path.display(),
            manifest.frame,
            rust_checkpoint.host_frame,
        ));
    }
    Ok(PairedResumeArtifacts {
        rust_state,
        oracle_state,
        original_timing_resume,
        semantic_trace,
    })
}

pub(crate) fn validate_paired_resume_provenance(
    path: &Path,
    core: &Path,
    rom: &Path,
    input_script: Option<&Path>,
    rom_random_script: Option<&Path>,
    allow_mixed_replay_provenance: bool,
) -> Result<(), String> {
    let (dir, _) = resolve_paired_resume_dir(path)?;
    let manifest_path = dir.join("manifest.json");
    let manifest: PairedResumeManifest = serde_json::from_slice(
        &fs::read(&manifest_path)
            .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?,
    )
    .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    if manifest.schema != PAIRED_RESUME_SCHEMA {
        return Err(format!(
            "{} has unsupported paired-resume schema {}; schema {} is required",
            manifest_path.display(),
            manifest.schema,
            PAIRED_RESUME_SCHEMA,
        ));
    }

    let verify_required =
        |label: &str, selected: &Path, expected: &PairedResumeProvenance| -> Result<(), String> {
            let actual = parity::evidence::sha256_file(selected).map_err(|error| {
                format!(
                    "failed to hash selected {label} {}: {error}",
                    selected.display()
                )
            })?;
            if actual != expected.sha256 && !allow_mixed_replay_provenance {
                return Err(format!(
                    "paired-resume {label} provenance mismatch: manifest={}, selected={actual}",
                    expected.sha256,
                ));
            }
            Ok(())
        };
    let verify_optional = |label: &str,
                           selected: Option<&Path>,
                           expected: Option<&PairedResumeProvenance>|
     -> Result<(), String> {
        match (selected, expected) {
            (None, None) => Ok(()),
            (Some(selected), Some(expected)) => verify_required(label, selected, expected),
            (Some(_), None) => Err(format!(
                "paired-resume {label} provenance is absent, but a source was selected"
            )),
            (None, Some(_)) => Err(format!(
                "paired-resume {label} provenance requires the recorded source"
            )),
        }
    };

    verify_required("core", core, &manifest.core)?;
    verify_required("ROM", rom, &manifest.rom)?;
    verify_optional("input script", input_script, manifest.input_script.as_ref())?;
    verify_optional(
        "ROM-random script",
        rom_random_script,
        manifest.rom_random_script.as_ref(),
    )?;
    Ok(())
}

pub(crate) fn validate_paired_resume_sram_selection(
    resume_paired: bool,
    load_sram: bool,
) -> Result<(), String> {
    if resume_paired && load_sram {
        return Err(
            "--resume-paired is self-contained and cannot be combined with --load-sram; the paired initial.srm is provenance, not a progressed-state replacement"
                .to_string(),
        );
    }
    Ok(())
}

pub(crate) fn restore_original_timing_resume_checkpoint(
    game: &mut ZeldaState,
    path: &Path,
) -> Result<(), String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let checkpoint: zelda3::OriginalTimingResumeCheckpoint = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to decode {}: {error}", path.display()))?;
    game.restore_original_timing_resume_checkpoint(checkpoint)
        .map_err(|error| format!("failed to restore {}: {error}", path.display()))
}

pub(crate) fn write_cached_av_paired_resume_from_sources(
    cache: &Path,
    cache_manifest: &serde_json::Value,
    cache_manifest_bytes: &[u8],
    rom: &Path,
    rom_sha256: &str,
    frame: u32,
    game: &ZeldaState,
    oracle_sources: Option<(&Path, &Path)>,
    final_dir: &Path,
) -> Result<PathBuf, String> {
    if !game.paired_resume_cpu_boundary_is_quiescent() {
        return Err(
            "cached A/V replay ended inside an unserialized ROM-call continuation".to_string(),
        );
    }
    let initial_sram_source = cache.join("initial.srm");
    // Rust-only checkpoint: the cache has no oracle state at this boundary.
    // Everything the Rust-only cached replay resumes from is still written and
    // hashed; the oracle artifacts are recorded as absent so the live compare
    // refuses the checkpoint instead of guessing.
    let oracle_hashes = oracle_sources
        .map(|(oracle_source, semantic_trace_source)| -> Result<(String, String), String> {
    let oracle_relative = oracle_source.strip_prefix(cache).map_err(|_| {
        format!(
            "oracle checkpoint {} is outside cache {}",
            oracle_source.display(),
            cache.display()
        )
    })?;
    let semantic_relative = semantic_trace_source.strip_prefix(cache).map_err(|_| {
        format!(
            "semantic checkpoint {} is outside cache {}",
            semantic_trace_source.display(),
            cache.display()
        )
    })?;
    let expected_oracle_sha256 = cache_manifest
        .get("artifact_sha256")
        .and_then(|artifacts| artifacts.get(oracle_relative.to_string_lossy().as_ref()))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            format!(
                "cached A/V manifest has no oracle-state hash for {}",
                oracle_relative.display()
            )
        })?;
    let actual_oracle_sha256 = parity::evidence::sha256_file(&oracle_source)
        .map_err(|error| format!("failed to hash {}: {error}", oracle_source.display()))?;
    if actual_oracle_sha256 != expected_oracle_sha256 {
        return Err(format!(
            "cached final oracle state hash mismatch: expected {expected_oracle_sha256}, got {actual_oracle_sha256}"
        ));
    }
    if !semantic_trace_source.is_file() {
        return Err(format!(
            "cached A/V frontier has no semantic trace checkpoint: {}",
            semantic_trace_source.display()
        ));
    }
    let expected_semantic_trace_sha256 = cache_manifest
        .get("artifact_sha256")
        .and_then(|artifacts| artifacts.get(semantic_relative.to_string_lossy().as_ref()))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            "cached semantic trace checkpoint is absent from the artifact inventory".to_string()
        })?;
    let semantic_trace_sha256 =
        parity::evidence::sha256_file(&semantic_trace_source).map_err(|error| {
            format!(
                "failed to hash {}: {error}",
                semantic_trace_source.display()
            )
        })?;
    if semantic_trace_sha256 != expected_semantic_trace_sha256 {
        return Err(format!(
            "cached semantic trace checkpoint hash mismatch: expected {expected_semantic_trace_sha256}, got {semantic_trace_sha256}"
        ));
    }
            Ok((actual_oracle_sha256, semantic_trace_sha256))
        })
        .transpose()?;
    let cache_identity = cache_manifest
        .get("cache_identity")
        .ok_or_else(|| "cached A/V manifest has no cache identity".to_string())?;
    let core_sha256 = cache_identity
        .get("core_sha256")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cached A/V identity has no core hash".to_string())?;
    let source_artifacts = cache_identity
        .get("source_artifact_sha256")
        .ok_or_else(|| "cached A/V identity has no source artifact hashes".to_string())?;
    let input_sha256 = source_artifacts
        .get("input.txt")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cached A/V identity has no input-script hash".to_string())?;
    let rom_random_sha256 = source_artifacts
        .get("rom-random.txt")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cached A/V identity has no ROM-random-script hash".to_string())?;
    let initial_sram_sha256 = source_artifacts
        .get("initial.srm")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cached A/V identity has no initial-SRAM hash".to_string())?;

    if final_dir.exists() {
        return Err(format!(
            "refusing to replace existing paired frontier {}",
            final_dir.display()
        ));
    }
    fs::create_dir_all(
        final_dir
            .parent()
            .ok_or_else(|| format!("paired frontier has no parent: {}", final_dir.display()))?,
    )
    .map_err(|error| format!("create paired frontier parent: {error}"))?;
    let temporary_dir = final_dir.with_file_name(format!(
        ".{}.tmp-{}",
        final_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("paired"),
        process::id()
    ));
    if temporary_dir.exists() {
        return Err(format!(
            "stale paired-frontier temporary directory exists: {}",
            temporary_dir.display()
        ));
    }

    let result = (|| -> Result<(), String> {
        fs::create_dir(&temporary_dir).map_err(|error| {
            format!(
                "failed to create paired frontier {}: {error}",
                temporary_dir.display()
            )
        })?;
        let original_timing_resume =
            game.capture_original_timing_resume_checkpoint()
                .map_err(|error| {
                    format!("cached Rust frontier is not an exact timing boundary: {error}")
                })?;
        let original_timing_resume_bytes = serde_json::to_vec_pretty(&original_timing_resume)
            .map_err(|error| {
                format!("failed to encode original-timing resume checkpoint: {error}")
            })?;
        let rust_state = PlayCrashCheckpoint {
            magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
            host_frame: frame,
            input: 0,
            run_what: select_run_what(&game.ram),
            game: game.clone(),
        };
        let rust_bytes = bincode::serialize(&rust_state)
            .map_err(|error| format!("failed to serialize cached Rust frontier: {error}"))?;
        fs::write(temporary_dir.join("rust.z3state"), &rust_bytes)
            .map_err(|error| format!("failed to write cached Rust frontier: {error}"))?;
        fs::write(
            temporary_dir.join("original-timing.resume.json"),
            &original_timing_resume_bytes,
        )
        .map_err(|error| format!("failed to write original-timing resume checkpoint: {error}"))?;
        fs::copy(&initial_sram_source, temporary_dir.join("initial.srm"))
            .map_err(|error| format!("failed to copy cached initial SRAM: {error}"))?;
        if let Some((oracle_source, semantic_trace_source)) = oracle_sources {
            fs::copy(oracle_source, temporary_dir.join("oracle.state"))
                .map_err(|error| format!("failed to copy cached final oracle state: {error}"))?;
            fs::copy(
                semantic_trace_source,
                temporary_dir.join("semantic-trace.checkpoint.json"),
            )
            .map_err(|error| format!("failed to copy cached semantic trace checkpoint: {error}"))?;
        }
        let (oracle_state, semantic_trace_checkpoint) = match &oracle_hashes {
            Some((oracle_sha256, semantic_trace_sha256)) => (
                serde_json::json!({"artifact": "oracle.state", "sha256": oracle_sha256}),
                serde_json::json!({
                    "artifact": "semantic-trace.checkpoint.json",
                    "sha256": semantic_trace_sha256,
                }),
            ),
            None => (serde_json::Value::Null, serde_json::Value::Null),
        };
        let manifest = serde_json::json!({
            "schema": PAIRED_RESUME_SCHEMA,
            "boundary": "pre-frame",
            "frame": frame,
            "cpu_boundary": "quiescent",
            "renderer_warmup_required": true,
            "rust_only": oracle_hashes.is_none(),
            "rust_state": {
                "artifact": "rust.z3state",
                "sha256": parity::evidence::sha256_bytes(&rust_bytes),
            },
            "oracle_state": oracle_state,
            "original_timing_resume_checkpoint": {
                "artifact": "original-timing.resume.json",
                "sha256": parity::evidence::sha256_bytes(&original_timing_resume_bytes),
            },
            "semantic_trace_checkpoint": semantic_trace_checkpoint,
            "source": {
                "kind": "matched-rust-only-cached-snes9x-av-replay",
                "cache": cache,
                "cache_key": cache_manifest.get("cache_key"),
                "cache_manifest_sha256": parity::evidence::sha256_bytes(cache_manifest_bytes),
            },
            "core": {"sha256": core_sha256},
            "rom": {"path": rom, "sha256": rom_sha256},
            "input_script": {"sha256": input_sha256},
            "rom_random_script": {"sha256": rom_random_sha256},
            "initial_sram": {"artifact": "initial.srm", "sha256": initial_sram_sha256},
        });
        fs::write(
            temporary_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)
                .map_err(|error| format!("failed to encode paired frontier manifest: {error}"))?,
        )
        .map_err(|error| format!("failed to write paired frontier manifest: {error}"))?;
        fs::rename(&temporary_dir, &final_dir).map_err(|error| {
            format!(
                "failed to install paired frontier {}: {error}",
                final_dir.display()
            )
        })?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&temporary_dir);
    }
    result?;
    Ok(final_dir.to_path_buf())
}

pub(crate) fn write_cached_av_final_paired_resume(
    cache: &Path,
    output: &Path,
    cache_manifest: &serde_json::Value,
    cache_manifest_bytes: &[u8],
    rom: &Path,
    rom_sha256: &str,
    frame: u32,
    game: &ZeldaState,
) -> Result<PathBuf, String> {
    let oracle_source = cache.join("oracle_final.state");
    let semantic_trace_source = cache.join("semantic-trace-final.checkpoint.json");
    let final_dir = output.join("paired-final");
    write_cached_av_paired_resume_from_sources(
        cache,
        cache_manifest,
        cache_manifest_bytes,
        rom,
        rom_sha256,
        frame,
        game,
        Some((&oracle_source, &semantic_trace_source)),
        &final_dir,
    )
}

pub(crate) fn cached_oracle_checkpoint_sources(
    cache: &Path,
    cache_manifest: &serde_json::Value,
    frame: u32,
) -> Result<(PathBuf, PathBuf), String> {
    let relative_dir = PathBuf::from(format!("oracle-checkpoints/frame-{frame:08}"));
    let checkpoint_dir = cache.join(&relative_dir);
    let manifest_path = checkpoint_dir.join("manifest.json");
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest_path)
            .map_err(|error| format!("read {}: {error}", manifest_path.display()))?,
    )
    .map_err(|error| format!("decode {}: {error}", manifest_path.display()))?;
    if manifest.get("schema").and_then(serde_json::Value::as_u64) != Some(1)
        || manifest.get("boundary").and_then(serde_json::Value::as_str) != Some("pre-frame")
        || manifest.get("frame").and_then(serde_json::Value::as_u64) != Some(u64::from(frame))
    {
        return Err(format!(
            "unsupported oracle checkpoint boundary: {}",
            manifest_path.display()
        ));
    }
    let resolve = |key: &str, expected_name: &str| -> Result<PathBuf, String> {
        let record = manifest
            .get(key)
            .ok_or_else(|| format!("oracle checkpoint has no {key}"))?;
        let artifact = record
            .get("artifact")
            .and_then(serde_json::Value::as_str)
            .filter(|value| *value == expected_name)
            .ok_or_else(|| format!("oracle checkpoint has unsafe {key} artifact"))?;
        let path = checkpoint_dir.join(artifact);
        let actual = parity::evidence::sha256_file(&path)
            .map_err(|error| format!("hash {}: {error}", path.display()))?;
        let expected = record
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("oracle checkpoint has no {key} hash"))?;
        if actual != expected {
            return Err(format!(
                "oracle checkpoint {key} hash mismatch: expected {expected}, got {actual}"
            ));
        }
        let relative = path
            .strip_prefix(cache)
            .expect("checkpoint path was constructed beneath cache")
            .to_string_lossy();
        let inventory = cache_manifest
            .get("artifact_sha256")
            .and_then(|artifacts| artifacts.get(relative.as_ref()))
            .and_then(serde_json::Value::as_str);
        if inventory != Some(actual.as_str()) {
            return Err(format!(
                "oracle checkpoint {key} is not bound by the immutable cache inventory"
            ));
        }
        Ok(path)
    };
    let oracle = resolve("oracle_state", "oracle.state")?;
    let semantic = resolve(
        "semantic_trace_checkpoint",
        "semantic-trace.checkpoint.json",
    )?;
    let cache_identity = cache_manifest
        .get("cache_identity")
        .ok_or_else(|| "cached A/V manifest has no cache identity".to_string())?;
    let provenance = manifest
        .get("provenance")
        .ok_or_else(|| "oracle checkpoint has no source provenance".to_string())?;
    for (checkpoint_key, cache_key) in
        [("core_sha256", "core_sha256"), ("rom_sha256", "rom_sha256")]
    {
        if provenance.get(checkpoint_key) != cache_identity.get(cache_key) {
            return Err(format!(
                "oracle checkpoint {checkpoint_key} does not match cache identity"
            ));
        }
    }
    let cache_sources = cache_identity
        .get("source_artifact_sha256")
        .ok_or_else(|| "cached A/V identity has no source artifacts".to_string())?;
    for (checkpoint_key, cache_name) in [
        ("input_sha256", "input.txt"),
        ("rom_random_sha256", "rom-random.txt"),
        ("initial_sram_sha256", "initial.srm"),
    ] {
        if provenance.get(checkpoint_key) != cache_sources.get(cache_name) {
            return Err(format!(
                "oracle checkpoint {checkpoint_key} does not match cache source identity"
            ));
        }
    }
    Ok((oracle, semantic))
}

pub(crate) fn write_cached_av_periodic_paired_resume(
    cache: &Path,
    output: &Path,
    cache_manifest: &serde_json::Value,
    cache_manifest_bytes: &[u8],
    rom: &Path,
    rom_sha256: &str,
    frame: u32,
    game: &ZeldaState,
) -> Result<PathBuf, String> {
    // A cache captured without oracle checkpoints at this boundary yields a
    // Rust-only checkpoint (see `PairedResumeManifest::oracle_state`).
    let oracle_sources = cache
        .join(format!("oracle-checkpoints/frame-{frame:08}/manifest.json"))
        .is_file()
        .then(|| cached_oracle_checkpoint_sources(cache, cache_manifest, frame))
        .transpose()?;
    let paired_root = output.join("paired");
    let final_dir = paired_root.join(format!("frame-{frame:08}"));
    let checkpoint = write_cached_av_paired_resume_from_sources(
        cache,
        cache_manifest,
        cache_manifest_bytes,
        rom,
        rom_sha256,
        frame,
        game,
        oracle_sources
            .as_ref()
            .map(|(oracle, semantic)| (oracle.as_path(), semantic.as_path())),
        &final_dir,
    )?;
    let latest = serde_json::json!({
        "schema": PAIRED_RESUME_SCHEMA,
        "frame": frame,
        "checkpoint": checkpoint
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("invalid paired checkpoint path: {}", checkpoint.display()))?,
    });
    let latest_path = paired_root.join("latest.json");
    let temporary = paired_root.join(format!(".latest.tmp-{}", process::id()));
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&latest)
            .map_err(|error| format!("encode paired latest manifest: {error}"))?,
    )
    .map_err(|error| format!("write paired latest manifest: {error}"))?;
    fs::rename(&temporary, &latest_path)
        .map_err(|error| format!("install paired latest manifest: {error}"))?;
    Ok(checkpoint)
}

pub(crate) fn install_directory_atomically(
    final_dir: &Path,
    write_contents: impl FnOnce(&Path) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    if final_dir.exists() {
        return Err(format!(
            "refusing to replace existing paired-resume generation {}",
            final_dir.display()
        )
        .into());
    }
    let parent = final_dir
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let name = final_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "paired-resume generation has no file name: {}",
                final_dir.display()
            )
        })?;
    let temporary = parent.join(format!(".{name}.tmp-{}", process::id()));
    if temporary.exists() {
        return Err(format!(
            "stale paired-resume temporary generation exists: {}",
            temporary.display()
        )
        .into());
    }
    fs::create_dir(&temporary)?;
    let result = write_contents(&temporary).and_then(|()| {
        fs::rename(&temporary, final_dir).map_err(|error| {
            format!(
                "install paired-resume generation {}: {error}",
                final_dir.display()
            )
            .into()
        })
    });
    if result.is_err() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}

pub(crate) fn write_file_atomically(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("atomic file has no file name: {}", path.display()))?;
    let temporary = parent.join(format!(".{name}.tmp-{}", process::id()));
    if temporary.exists() {
        return Err(format!(
            "stale atomic temporary file exists: {}",
            temporary.display()
        )
        .into());
    }
    let result = fs::write(&temporary, bytes)
        .map_err(|error| -> Box<dyn Error> { Box::new(error) })
        .and_then(|()| {
            fs::rename(&temporary, path)
                .map_err(|error| format!("install atomic file {}: {error}", path.display()).into())
        });
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub(crate) fn write_paired_resume_capture(
    capture: &PairedResumeCapture,
    core_path: &str,
    rom_path: &str,
    input_script_path: Option<&Path>,
    rom_random_script: Option<&Path>,
    initial_sram: &[u8],
    game: &ZeldaState,
    oracle: &LibretroCore,
    semantic_trace: Option<&Snes9xOracleSemanticTrace>,
) -> Result<(), Box<dyn Error>> {
    if !game.paired_resume_cpu_boundary_is_quiescent() {
        return Err(
            "frame is inside an unserialized ROM-call continuation; choose a quiescent pre-frame boundary"
                .into(),
        );
    }
    let original_timing_resume = game.capture_original_timing_resume_checkpoint()?;
    let original_timing_resume_bytes = serde_json::to_vec_pretty(&original_timing_resume)?;
    let rust_state = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: capture.frame,
        input: 0,
        run_what: select_run_what(&game.ram),
        game: game.clone(),
    };
    let rust_bytes = bincode::serialize(&rust_state)?;
    let trace = semantic_trace
        .ok_or("paired resume requires an authoritative Snes9x semantic trace checkpoint")?;
    let semantic_trace_bytes = serde_json::to_vec_pretty(&trace.checkpoint())?;
    let source_manifest = |path: Option<&Path>| -> Result<serde_json::Value, Box<dyn Error>> {
        Ok(match path {
            Some(path) => serde_json::json!({
                "path": path,
                "sha256": parity::runner::sha256_file(path)?,
            }),
            None => serde_json::Value::Null,
        })
    };
    let core_sha256 = parity::runner::sha256_file(Path::new(core_path))?;
    let rom_sha256 = parity::runner::sha256_file(Path::new(rom_path))?;
    let input_script = source_manifest(input_script_path)?;
    let rom_random_script = source_manifest(rom_random_script)?;

    install_directory_atomically(&capture.dir, |temporary| {
        let oracle_bytes = oracle.serialize_state()?;
        fs::write(temporary.join("rust.z3state"), &rust_bytes)?;
        fs::write(temporary.join("oracle.state"), &oracle_bytes)?;
        fs::write(temporary.join("initial.srm"), initial_sram)?;
        fs::write(
            temporary.join("original-timing.resume.json"),
            &original_timing_resume_bytes,
        )?;
        fs::write(
            temporary.join("semantic-trace.checkpoint.json"),
            &semantic_trace_bytes,
        )?;
        let manifest = serde_json::json!({
            "schema": PAIRED_RESUME_SCHEMA,
            "boundary": "pre-frame",
            "frame": capture.frame,
            "cpu_boundary": "quiescent",
            "renderer_warmup_required": true,
            "rust_state": {
                "artifact": "rust.z3state",
                "sha256": parity::evidence::sha256_bytes(&rust_bytes),
            },
            "oracle_state": {
                "artifact": "oracle.state",
                "sha256": parity::evidence::sha256_bytes(&oracle_bytes),
            },
            "original_timing_resume_checkpoint": {
                "artifact": "original-timing.resume.json",
                "sha256": parity::evidence::sha256_bytes(&original_timing_resume_bytes),
            },
            "semantic_trace_checkpoint": {
                "artifact": "semantic-trace.checkpoint.json",
                "sha256": parity::evidence::sha256_bytes(&semantic_trace_bytes),
            },
            "core": {"path": core_path, "sha256": core_sha256},
            "rom": {"path": rom_path, "sha256": rom_sha256},
            "input_script": input_script,
            "rom_random_script": rom_random_script,
            "initial_sram": {
                "artifact": "initial.srm",
                "sha256": parity::evidence::sha256_bytes(initial_sram),
            },
        });
        fs::write(
            temporary.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        Ok(())
    })
}

pub(crate) fn write_rolling_paired_resume_capture(
    rolling: &RollingPairedResumeCapture,
    frame: u32,
    core_path: &str,
    rom_path: &str,
    input_script_path: Option<&Path>,
    rom_random_script: Option<&Path>,
    initial_sram: &[u8],
    game: &ZeldaState,
    oracle: &LibretroCore,
    semantic_trace: Option<&Snes9xOracleSemanticTrace>,
) -> Result<PathBuf, Box<dyn Error>> {
    fs::create_dir_all(&rolling.root)?;
    let checkpoint_name = format!("frame-{frame:08}");
    let capture = PairedResumeCapture {
        frame,
        dir: rolling.root.join(&checkpoint_name),
    };
    write_paired_resume_capture(
        &capture,
        core_path,
        rom_path,
        input_script_path,
        rom_random_script,
        initial_sram,
        game,
        oracle,
        semantic_trace,
    )?;
    write_file_atomically(
        &rolling.root.join("latest.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({
            "schema": PAIRED_RESUME_SCHEMA,
            "frame": frame,
            "checkpoint": checkpoint_name,
        }))?,
    )?;
    // ZELDA3_ROLLING_RESUME_KEEP=<n> keeps more generations for bisection
    // (diagnostic only; the default bounds the disk cost of long routes).
    let keep = env::var("ZELDA3_ROLLING_RESUME_KEEP")
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|keep| *keep >= 1)
        .unwrap_or(ROLLING_PAIRED_RESUME_GENERATIONS_KEPT);
    prune_rolling_paired_resume_captures(&rolling.root, keep, &capture.dir);
    Ok(capture.dir)
}

pub(crate) fn prune_rolling_paired_resume_captures(root: &Path, keep: usize, current: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut captures = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            name.strip_prefix("frame-")?.parse::<u32>().ok()?;
            let path = entry.path();
            let modified = entry.metadata().ok()?.modified().ok()?;
            path.join("manifest.json")
                .is_file()
                .then_some((modified, path))
        })
        .collect::<Vec<_>>();
    // A restarted run may capture a lower frame into an existing root. Keep
    // the generation named by latest.json plus the most recently written
    // fallback; numeric frame order alone could immediately delete the new
    // capture and leave latest.json dangling.
    captures.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
    captures.sort_by_key(|(_, path)| path != current);
    for (_, path) in captures.into_iter().skip(keep.max(1)) {
        if let Err(error) = fs::remove_dir_all(&path) {
            eprintln!(
                "failed to prune generated rolling checkpoint {}: {error}",
                path.display()
            );
        }
    }
}
