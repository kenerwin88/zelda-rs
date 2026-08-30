//! Read-only verification of reusable cold parity evidence.
//!
//! Schema 2 deliberately binds the request and receipt to one exact authority
//! object.  A receipt is not reusable merely because it says that a run
//! passed: the verifier reopens the session manifest, result, and replay-source
//! artifacts and reconstructs their identity before returning it.

use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::evidence::sha256_bytes;

pub const COLD_EVIDENCE_SCHEMA: u32 = 2;
pub const COLD_EVIDENCE_KIND: &str = "zelda3-cold-parity-pass";
pub const COLD_EVIDENCE_REQUEST_KIND: &str = "zelda3-cold-parity-reuse-request";
pub const CLEAN_ENV_EXECUTION_POLICY: &str = "clean_env_v1";
pub const EMPTY_RUNTIME_CONFIG_SHA256: &str =
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ColdEvidenceRequest {
    pub schema: u32,
    pub kind: String,
    pub authority: ColdEvidenceAuthority,
}

/// The complete identity of a requested cold run.
///
/// Python producers must copy this object byte-for-value into the receipt.
/// Equality is structural JSON equality, not a subset or compatibility test.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ColdEvidenceAuthority {
    pub target_frames: u64,
    pub route_signature: Value,
    pub route_signature_sha256: String,
    pub binary: BinaryAuthority,
    pub staged_source: StagedSourceAuthority,
    pub invocation: InvocationAuthority,
    pub core_sha256: String,
    pub rom_sha256: String,
    pub source_artifact_sha256: SourceArtifactHashes,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryAuthority {
    pub sha256: String,
    pub size: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StagedSourceAuthority {
    pub identity: Value,
    pub identity_sha256: String,
    pub build_binding: Value,
    pub build_binding_sha256: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationAuthority {
    /// Fail-closed child-process environment normalization contract.
    pub execution_policy: String,
    /// Normalized command/option policy, excluding incidental output paths.
    pub policy: Value,
    pub policy_sha256: String,
    /// Digest of the normalized environment allowlist used by the invocation.
    pub environment_sha256: String,
    /// Hash of the runtime timing configuration. Reusable cold proof requires
    /// the empty configuration; route-specific timing rules are diagnostics.
    pub runtime_config_sha256: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceArtifactHashes {
    #[serde(rename = "input.txt")]
    pub input: String,
    #[serde(rename = "rom-random.txt")]
    pub rom_random: String,
    #[serde(rename = "initial.srm")]
    pub initial_sram: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ColdSessionIdentity {
    pub manifest_sha256: String,
    pub result_sha256: String,
    pub source_artifact_sha256: SourceArtifactHashes,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ColdEvidenceReceipt {
    schema: u32,
    kind: String,
    created_unix_ns: u64,
    invocation_id: String,
    run_nonce: String,
    authority: ColdEvidenceAuthority,
    session: PathBuf,
    session_identity: ColdSessionIdentity,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VerifiedColdReceipt {
    pub invocation_id: String,
    pub run_nonce: String,
    pub receipt_path: PathBuf,
    pub receipt_sha256: String,
    pub session_path: PathBuf,
    pub target_frames: u64,
    pub authority: ColdEvidenceAuthority,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RejectedColdReceipt {
    pub receipt_path: PathBuf,
    pub error: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FindColdEvidenceOutput {
    pub schema: u32,
    pub mode: &'static str,
    pub reusable: bool,
    pub receipts: Vec<VerifiedColdReceipt>,
    pub rejected: Vec<RejectedColdReceipt>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ListColdEvidenceOutput {
    pub schema: u32,
    pub mode: &'static str,
    pub receipts: Vec<VerifiedColdReceipt>,
    pub rejected: Vec<RejectedColdReceipt>,
}

pub fn load_request(path: &Path) -> Result<ColdEvidenceRequest, String> {
    let bytes = read_regular_file(path, "cold-evidence request")?;
    let request: ColdEvidenceRequest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
    validate_request(&request)?;
    Ok(request)
}

pub fn find_reusable_cold_evidence(
    pass_root: &Path,
    request: &ColdEvidenceRequest,
) -> Result<FindColdEvidenceOutput, String> {
    validate_request(request)?;
    let (mut receipts, rejected) = scan_receipts(pass_root, Some(&request.authority))?;
    receipts.sort_by(|left, right| left.receipt_path.cmp(&right.receipt_path));
    Ok(FindColdEvidenceOutput {
        schema: COLD_EVIDENCE_SCHEMA,
        mode: "find",
        reusable: !receipts.is_empty(),
        receipts,
        rejected,
    })
}

pub fn list_verified_cold_evidence(pass_root: &Path) -> Result<ListColdEvidenceOutput, String> {
    let (mut receipts, rejected) = scan_receipts(pass_root, None)?;
    receipts.sort_by(|left, right| left.receipt_path.cmp(&right.receipt_path));
    Ok(ListColdEvidenceOutput {
        schema: COLD_EVIDENCE_SCHEMA,
        mode: "list",
        receipts,
        rejected,
    })
}

fn validate_request(request: &ColdEvidenceRequest) -> Result<(), String> {
    require_equal("request schema", request.schema, COLD_EVIDENCE_SCHEMA)?;
    require_equal(
        "request kind",
        request.kind.as_str(),
        COLD_EVIDENCE_REQUEST_KIND,
    )?;
    validate_authority(&request.authority)
}

fn validate_authority(authority: &ColdEvidenceAuthority) -> Result<(), String> {
    if authority.target_frames == 0 {
        return Err("authority target_frames must be greater than zero".into());
    }
    require_nonempty_object("route_signature", &authority.route_signature)?;
    require_stable_hash(
        "route_signature_sha256",
        &authority.route_signature,
        &authority.route_signature_sha256,
    )?;
    require_sha256("binary.sha256", &authority.binary.sha256)?;
    if authority.binary.size == 0 {
        return Err("authority binary.size must be greater than zero".into());
    }
    require_nonempty_object("staged_source.identity", &authority.staged_source.identity)?;
    require_stable_hash(
        "staged_source.identity_sha256",
        &authority.staged_source.identity,
        &authority.staged_source.identity_sha256,
    )?;
    require_nonempty_object(
        "staged_source.build_binding",
        &authority.staged_source.build_binding,
    )?;
    require_stable_hash(
        "staged_source.build_binding_sha256",
        &authority.staged_source.build_binding,
        &authority.staged_source.build_binding_sha256,
    )?;
    require_nonempty_object("invocation.policy", &authority.invocation.policy)?;
    require_equal(
        "invocation.execution_policy",
        authority.invocation.execution_policy.as_str(),
        CLEAN_ENV_EXECUTION_POLICY,
    )?;
    require_stable_hash(
        "invocation.policy_sha256",
        &authority.invocation.policy,
        &authority.invocation.policy_sha256,
    )?;
    require_sha256(
        "invocation.environment_sha256",
        &authority.invocation.environment_sha256,
    )?;
    require_equal(
        "invocation.runtime_config_sha256",
        authority.invocation.runtime_config_sha256.as_str(),
        EMPTY_RUNTIME_CONFIG_SHA256,
    )?;
    require_sha256("core_sha256", &authority.core_sha256)?;
    require_sha256("rom_sha256", &authority.rom_sha256)?;
    validate_source_hashes(&authority.source_artifact_sha256)
}

fn scan_receipts(
    pass_root: &Path,
    requested_authority: Option<&ColdEvidenceAuthority>,
) -> Result<(Vec<VerifiedColdReceipt>, Vec<RejectedColdReceipt>), String> {
    let metadata = match fs::symlink_metadata(pass_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((Vec::new(), Vec::new()));
        }
        Err(error) => {
            return Err(format!(
                "cannot inspect cold-evidence root {}: {error}",
                pass_root.display()
            ));
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "cold-evidence root must be a real directory, not a symlink: {}",
            pass_root.display()
        ));
    }

    let mut paths = fs::read_dir(pass_root)
        .map_err(|error| format!("cannot read {}: {error}", pass_root.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot enumerate {}: {error}", pass_root.display()))?;
    paths.sort();

    let mut verified = Vec::new();
    let mut rejected = Vec::new();
    for path in paths {
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        match verify_receipt(&path, requested_authority) {
            Ok(Some(receipt)) => verified.push(receipt),
            Ok(None) => {}
            Err(error) => rejected.push(RejectedColdReceipt {
                receipt_path: path,
                error,
            }),
        }
    }
    Ok((verified, rejected))
}

fn verify_receipt(
    path: &Path,
    requested_authority: Option<&ColdEvidenceAuthority>,
) -> Result<Option<VerifiedColdReceipt>, String> {
    let bytes = read_regular_file(path, "cold-evidence receipt")?;
    let raw: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("cannot parse receipt JSON: {error}"))?;
    let schema = raw
        .get("schema")
        .and_then(Value::as_u64)
        .ok_or_else(|| "receipt schema is missing or not an integer".to_string())?;
    if schema != u64::from(COLD_EVIDENCE_SCHEMA) {
        return Err(format!(
            "receipt schema {schema} is not reusable; schema 2 is required"
        ));
    }
    let receipt: ColdEvidenceReceipt = serde_json::from_value(raw.clone())
        .map_err(|error| format!("receipt does not satisfy schema 2: {error}"))?;
    require_equal("receipt kind", receipt.kind.as_str(), COLD_EVIDENCE_KIND)?;
    validate_invocation_id(&receipt.invocation_id)?;
    require_sha256("receipt run_nonce", &receipt.run_nonce)?;
    validate_authority(&receipt.authority)?;
    validate_receipt_filename(path, &raw, &receipt)?;

    if requested_authority.is_some_and(|requested| requested != &receipt.authority) {
        return Ok(None);
    }

    let computed_identity = validate_session(
        &receipt.session,
        &receipt.authority,
        &receipt.invocation_id,
        &receipt.run_nonce,
    )?;
    require_equal(
        "receipt session_identity",
        &receipt.session_identity,
        &computed_identity,
    )?;

    Ok(Some(VerifiedColdReceipt {
        invocation_id: receipt.invocation_id,
        run_nonce: receipt.run_nonce,
        receipt_path: path.to_path_buf(),
        receipt_sha256: sha256_bytes(&bytes),
        session_path: receipt.session,
        target_frames: receipt.authority.target_frames,
        authority: receipt.authority,
    }))
}

fn validate_receipt_filename(
    path: &Path,
    raw: &Value,
    receipt: &ColdEvidenceReceipt,
) -> Result<(), String> {
    let content_hash = stable_hash(raw)?;
    let expected = format!(
        "{}-{}-{}.json",
        receipt.created_unix_ns,
        receipt.authority.target_frames,
        &content_hash[..12]
    );
    let actual = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "receipt filename is not valid UTF-8".to_string())?;
    require_equal("receipt content-hash filename", actual, expected.as_str())
}

fn validate_session(
    session: &Path,
    authority: &ColdEvidenceAuthority,
    invocation_id: &str,
    run_nonce: &str,
) -> Result<ColdSessionIdentity, String> {
    if !session.is_absolute()
        || session
            .components()
            .any(|part| part == Component::ParentDir)
    {
        return Err(format!(
            "receipt session path must be absolute and normalized: {}",
            session.display()
        ));
    }
    reject_symlink_components(session)?;
    let metadata = fs::symlink_metadata(session)
        .map_err(|error| format!("cannot inspect session {}: {error}", session.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "receipt session must be a real directory: {}",
            session.display()
        ));
    }

    let manifest_path = session.join("manifest.json");
    let result_path = session.join("result.json");
    let input_path = session.join("input.txt");
    let random_path = session.join("rom-random.txt");
    let sram_path = session.join("initial.srm");
    let manifest_bytes = read_regular_file(&manifest_path, "session manifest")?;
    let result_bytes = read_regular_file(&result_path, "session result")?;
    let input_bytes = read_regular_file(&input_path, "session input replay")?;
    let random_bytes = read_regular_file(&random_path, "session ROM-random replay")?;
    let sram_bytes = read_regular_file(&sram_path, "session initial SRAM")?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("cannot parse {}: {error}", manifest_path.display()))?;
    let result: Value = serde_json::from_slice(&result_bytes)
        .map_err(|error| format!("cannot parse {}: {error}", result_path.display()))?;

    let source_hashes = SourceArtifactHashes {
        input: sha256_bytes(&input_bytes),
        rom_random: sha256_bytes(&random_bytes),
        initial_sram: sha256_bytes(&sram_bytes),
    };
    require_equal(
        "authority source_artifact_sha256",
        &authority.source_artifact_sha256,
        &source_hashes,
    )?;
    validate_manifest_and_result(
        &manifest,
        &result,
        authority,
        &source_hashes,
        invocation_id,
        run_nonce,
    )?;

    Ok(ColdSessionIdentity {
        manifest_sha256: sha256_bytes(&manifest_bytes),
        result_sha256: sha256_bytes(&result_bytes),
        source_artifact_sha256: source_hashes,
    })
}

fn validate_manifest_and_result(
    manifest: &Value,
    result: &Value,
    authority: &ColdEvidenceAuthority,
    source_hashes: &SourceArtifactHashes,
    invocation_id: &str,
    run_nonce: &str,
) -> Result<(), String> {
    let target = authority.target_frames;
    require_json_string(manifest, "/cold_evidence_invocation_id", invocation_id)?;
    require_json_string(manifest, "/cold_evidence_run_nonce", run_nonce)?;
    require_json_u64(manifest, "/timing/start_frame", 0)?;
    require_json_u64(manifest, "/timing/compare_from_frame", 0)?;
    require_json_u64(manifest, "/timing/frames_requested", target)?;
    require_json_u64(manifest, "/timing/fixed_oracle_startup_skip_frames", 0)?;
    require_json_bool(manifest, "/timing/dynamic_alignment", false)?;
    require_json_u64(manifest, "/frames_completed", target)?;
    require_json_string(manifest, "/status", "passed")?;
    require_json_bool(manifest, "/parity_eligible", true)?;
    require_json_bool(manifest, "/comparison_lanes/video", true)?;
    require_json_bool(manifest, "/comparison_lanes/audio", true)?;
    require_json_string(manifest, "/audio/comparison", "exact")?;
    require_explicit_null(manifest, "/replay_save")?;
    require_explicit_null(manifest, "/replay_bundle")?;
    require_json_string(manifest, "/rom_random_authority/mode", "replay_script")?;
    require_json_string(manifest, "/rom_random_replay/artifact", "rom-random.txt")?;
    require_json_string(
        manifest,
        "/rom_random_replay/sha256",
        &source_hashes.rom_random,
    )?;
    require_json_string(manifest, "/input_replay/artifact", "input.txt")?;
    require_json_string(manifest, "/input_replay/sha256", &source_hashes.input)?;
    require_json_string(manifest, "/core/sha256", &authority.core_sha256)?;
    require_json_string(manifest, "/rom/sha256", &authority.rom_sha256)?;

    require_json_string(result, "/status", "passed")?;
    require_json_bool(result, "/parity_eligible", true)?;
    require_json_u64(result, "/frames_completed", target)?;
    require_json_bool(result, "/dynamic_alignment", false)?;
    require_json_bool(result, "/video/matched", true)?;
    require_json_empty_array(result, "/video/mismatch_ranges")?;
    require_explicit_null(result, "/video/first_mismatch")?;
    require_json_bool(result, "/audio/matched", true)?;
    require_json_string(result, "/audio/mode", "exact")?;
    let rust_sample_frames = required_json_u64(result, "/audio/rust_sample_frames")?;
    let oracle_sample_frames = required_json_u64(result, "/audio/oracle_sample_frames")?;
    require_equal(
        "exact audio sample-frame count",
        rust_sample_frames,
        oracle_sample_frames,
    )?;
    require_json_u64(result, "/audio/mismatched_interleaved_samples", 0)?;
    require_explicit_null(result, "/audio/first_mismatch_interleaved")?;
    require_explicit_null(result, "/audio/first_mismatch_sample_frame")?;
    require_explicit_null(result, "/audio/first_mismatch_channel")?;
    Ok(())
}

fn read_regular_file(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {label} {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{label} must be a regular non-symlink file: {}",
            path.display()
        ));
    }
    let file = File::open(path)
        .map_err(|error| format!("cannot open {label} {}: {error}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read {label} {}: {error}", path.display()))?;
    Ok(bytes)
}

fn reject_symlink_components(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if matches!(component, Component::RootDir | Component::Prefix(_)) {
            continue;
        }
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            format!(
                "cannot inspect session path component {}: {error}",
                current.display()
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "session path contains a symlink component: {}",
                current.display()
            ));
        }
    }
    Ok(())
}

fn stable_hash(value: &Value) -> Result<String, String> {
    let canonical = canonicalize_json(value);
    serde_json::to_vec(&canonical)
        .map(|bytes| sha256_bytes(&bytes))
        .map_err(|error| format!("cannot canonicalize JSON for hashing: {error}"))
}

fn canonicalize_json(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            Value::Object(
                keys.into_iter()
                    .map(|key| (key.clone(), canonicalize_json(&object[key])))
                    .collect::<Map<_, _>>(),
            )
        }
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_json).collect()),
        other => other.clone(),
    }
}

fn require_stable_hash(label: &str, value: &Value, expected: &str) -> Result<(), String> {
    require_sha256(label, expected)?;
    let actual = stable_hash(value)?;
    require_equal(label, expected, actual.as_str())
}

fn require_sha256(label: &str, value: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(format!("{label} must be a lowercase SHA-256 digest"));
    }
    Ok(())
}

fn validate_source_hashes(hashes: &SourceArtifactHashes) -> Result<(), String> {
    require_sha256("source_artifact_sha256.input.txt", &hashes.input)?;
    require_sha256("source_artifact_sha256.rom-random.txt", &hashes.rom_random)?;
    require_sha256("source_artifact_sha256.initial.srm", &hashes.initial_sram)
}

fn validate_invocation_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err("receipt invocation_id is empty or contains unsafe characters".into());
    }
    Ok(())
}

fn require_nonempty_object(label: &str, value: &Value) -> Result<(), String> {
    if value.as_object().is_none_or(Map::is_empty) {
        return Err(format!("authority {label} must be a nonempty JSON object"));
    }
    Ok(())
}

fn require_json_u64(value: &Value, pointer: &str, expected: u64) -> Result<(), String> {
    let actual = required_json_u64(value, pointer)?;
    if actual != expected {
        return Err(format!("{pointer} must be {expected}, got {actual}",));
    }
    Ok(())
}

fn required_json_u64(value: &Value, pointer: &str) -> Result<u64, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            format!(
                "{pointer} must be an unsigned integer, got {}",
                value.pointer(pointer).unwrap_or(&Value::Null)
            )
        })
}

fn require_json_bool(value: &Value, pointer: &str, expected: bool) -> Result<(), String> {
    let actual = value.pointer(pointer).and_then(Value::as_bool);
    if actual != Some(expected) {
        return Err(format!(
            "{pointer} must be {expected}, got {}",
            value.pointer(pointer).unwrap_or(&Value::Null)
        ));
    }
    Ok(())
}

fn require_json_string(value: &Value, pointer: &str, expected: &str) -> Result<(), String> {
    let actual = value.pointer(pointer).and_then(Value::as_str);
    if actual != Some(expected) {
        return Err(format!(
            "{pointer} must be {expected:?}, got {}",
            value.pointer(pointer).unwrap_or(&Value::Null)
        ));
    }
    Ok(())
}

fn require_explicit_null(value: &Value, pointer: &str) -> Result<(), String> {
    match value.pointer(pointer) {
        Some(Value::Null) => Ok(()),
        Some(other) => Err(format!("{pointer} must be null, got {other}")),
        None => Err(format!("{pointer} must be present and null")),
    }
}

fn require_json_empty_array(value: &Value, pointer: &str) -> Result<(), String> {
    match value.pointer(pointer) {
        Some(Value::Array(items)) if items.is_empty() => Ok(()),
        Some(other) => Err(format!("{pointer} must be an empty array, got {other}")),
        None => Err(format!("{pointer} must be present and an empty array")),
    }
}

fn require_equal<T>(label: &str, actual: T, expected: T) -> Result<(), String>
where
    T: PartialEq + std::fmt::Debug,
{
    if actual != expected {
        return Err(format!(
            "{label} mismatch: got {actual:?}, expected {expected:?}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::sha256_file;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "zparity-cold-evidence-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(fs::canonicalize(path).unwrap())
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn object(entries: &[(&str, Value)]) -> Value {
        Value::Object(
            entries
                .iter()
                .map(|(key, value)| ((*key).to_string(), value.clone()))
                .collect(),
        )
    }

    fn hashed(value: &Value) -> String {
        stable_hash(value).unwrap()
    }

    fn write_fixture(root: &Path) -> (ColdEvidenceRequest, PathBuf, PathBuf) {
        let pass_root = root.join("passes");
        let session = root.join("session");
        fs::create_dir(&pass_root).unwrap();
        fs::create_dir(&session).unwrap();
        fs::write(session.join("input.txt"), b"0 0x0000\n").unwrap();
        fs::write(session.join("rom-random.txt"), b"9 0xa5 carry=1\n").unwrap();
        fs::write(session.join("initial.srm"), b"sram").unwrap();
        let source_hashes = SourceArtifactHashes {
            input: sha256_file(&session.join("input.txt")).unwrap(),
            rom_random: sha256_file(&session.join("rom-random.txt")).unwrap(),
            initial_sram: sha256_file(&session.join("initial.srm")).unwrap(),
        };
        let route = object(&[("name", Value::String("standard".into()))]);
        let staged_identity = object(&[("head", Value::String("a".repeat(40)))]);
        let build_binding = object(&[("profile", Value::String("parity".into()))]);
        let policy = object(&[("cold", Value::Bool(true))]);
        let authority = ColdEvidenceAuthority {
            target_frames: 10_000,
            route_signature_sha256: hashed(&route),
            route_signature: route,
            binary: BinaryAuthority {
                sha256: "b".repeat(64),
                size: 123,
            },
            staged_source: StagedSourceAuthority {
                identity_sha256: hashed(&staged_identity),
                identity: staged_identity,
                build_binding_sha256: hashed(&build_binding),
                build_binding,
            },
            invocation: InvocationAuthority {
                execution_policy: CLEAN_ENV_EXECUTION_POLICY.into(),
                policy_sha256: hashed(&policy),
                policy,
                environment_sha256: "e".repeat(64),
                runtime_config_sha256: EMPTY_RUNTIME_CONFIG_SHA256.into(),
            },
            core_sha256: "c".repeat(64),
            rom_sha256: "d".repeat(64),
            source_artifact_sha256: source_hashes.clone(),
        };
        let manifest = serde_json::json!({
            "schema": 1,
            "status": "passed",
            "cold_evidence_invocation_id": "invocation-42",
            "cold_evidence_run_nonce": "f".repeat(64),
            "parity_eligible": true,
            "frames_completed": 10_000,
            "core": {"sha256": authority.core_sha256},
            "rom": {"sha256": authority.rom_sha256},
            "replay_save": null,
            "replay_bundle": null,
            "rom_random_replay": {
                "artifact": "rom-random.txt",
                "sha256": source_hashes.rom_random,
            },
            "rom_random_authority": {"mode": "replay_script"},
            "input_replay": {
                "artifact": "input.txt",
                "sha256": source_hashes.input,
            },
            "timing": {
                "frames_requested": 10_000,
                "start_frame": 0,
                "compare_from_frame": 0,
                "fixed_oracle_startup_skip_frames": 0,
                "dynamic_alignment": false,
            },
            "comparison_lanes": {"video": true, "audio": true},
            "audio": {"comparison": "exact"},
        });
        let result = serde_json::json!({
            "status": "passed",
            "parity_eligible": true,
            "frames_completed": 10_000,
            "dynamic_alignment": false,
            "video": {
                "matched": true,
                "mismatch_ranges": [],
                "first_mismatch": null,
            },
            "audio": {
                "mode": "exact",
                "matched": true,
                "rust_sample_frames": 5_333_333,
                "oracle_sample_frames": 5_333_333,
                "mismatched_interleaved_samples": 0,
                "first_mismatch_interleaved": null,
                "first_mismatch_sample_frame": null,
                "first_mismatch_channel": null,
            },
        });
        fs::write(
            session.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(
            session.join("result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        let session_identity = ColdSessionIdentity {
            manifest_sha256: sha256_file(&session.join("manifest.json")).unwrap(),
            result_sha256: sha256_file(&session.join("result.json")).unwrap(),
            source_artifact_sha256: source_hashes,
        };
        let receipt = ColdEvidenceReceipt {
            schema: 2,
            kind: COLD_EVIDENCE_KIND.into(),
            created_unix_ns: 42,
            invocation_id: "invocation-42".into(),
            run_nonce: "f".repeat(64),
            authority: authority.clone(),
            session: session.clone(),
            session_identity,
        };
        let raw = serde_json::to_value(&receipt).unwrap();
        let receipt_path = pass_root.join(format!(
            "42-10000-{}.json",
            &stable_hash(&raw).unwrap()[..12]
        ));
        fs::write(&receipt_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
        (
            ColdEvidenceRequest {
                schema: 2,
                kind: COLD_EVIDENCE_REQUEST_KIND.into(),
                authority,
            },
            pass_root,
            receipt_path,
        )
    }

    #[test]
    fn schema2_receipt_is_reusable_only_after_rehashing_its_session() {
        let root = TestRoot::new();
        let (request, pass_root, receipt_path) = write_fixture(&root.0);
        let output = find_reusable_cold_evidence(&pass_root, &request).unwrap();
        assert!(output.reusable, "{:?}", output.rejected);
        assert_eq!(output.receipts.len(), 1, "{:?}", output.rejected);
        assert_eq!(output.receipts[0].receipt_path, receipt_path);
        assert_eq!(output.receipts[0].invocation_id, "invocation-42");
        assert_eq!(output.receipts[0].run_nonce, "f".repeat(64));
        assert_eq!(output.receipts[0].receipt_sha256.len(), 64);
        assert!(output.rejected.is_empty());

        fs::write(root.0.join("session/input.txt"), b"tampered\n").unwrap();
        let output = find_reusable_cold_evidence(&pass_root, &request).unwrap();
        assert!(!output.reusable);
        assert_eq!(output.rejected.len(), 1);
        assert!(output.rejected[0].error.contains("source_artifact_sha256"));
    }

    #[test]
    fn schema1_and_bad_content_hash_are_never_reusable() {
        let root = TestRoot::new();
        let (request, pass_root, receipt_path) = write_fixture(&root.0);
        let schema1 = pass_root.join("old.json");
        fs::write(
            &schema1,
            br#"{"schema":1,"kind":"zelda3-cold-parity-pass"}"#,
        )
        .unwrap();
        let bad_name = pass_root.join("42-10000-000000000000.json");
        fs::rename(receipt_path, bad_name).unwrap();

        let output = find_reusable_cold_evidence(&pass_root, &request).unwrap();
        assert!(!output.reusable);
        assert_eq!(output.rejected.len(), 2);
        assert!(output
            .rejected
            .iter()
            .any(|item| item.error.contains("schema 1 is not reusable")));
        assert!(output
            .rejected
            .iter()
            .any(|item| item.error.contains("content-hash filename")));
    }

    #[test]
    fn authority_must_match_as_one_exact_object() {
        let root = TestRoot::new();
        let (mut request, pass_root, _) = write_fixture(&root.0);
        request.authority.binary.size += 1;
        let output = find_reusable_cold_evidence(&pass_root, &request).unwrap();
        assert!(!output.reusable);
        assert!(output.receipts.is_empty());
        assert!(output.rejected.is_empty());
    }

    #[test]
    fn receipt_invocation_must_match_the_session_manifest() {
        let root = TestRoot::new();
        let (request, pass_root, receipt_path) = write_fixture(&root.0);
        let mut receipt: Value = serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
        receipt["invocation_id"] = Value::String("different-invocation".into());
        fs::remove_file(&receipt_path).unwrap();
        let mismatched_path = pass_root.join(format!(
            "42-10000-{}.json",
            &stable_hash(&receipt).unwrap()[..12]
        ));
        fs::write(
            &mismatched_path,
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();

        let output = find_reusable_cold_evidence(&pass_root, &request).unwrap();
        assert!(!output.reusable);
        assert_eq!(output.rejected.len(), 1);
        assert!(
            output.rejected[0]
                .error
                .contains("/cold_evidence_invocation_id must be"),
            "{:?}",
            output.rejected
        );
    }

    #[test]
    fn invocation_only_mutation_retains_the_runner_nonce() {
        let root = TestRoot::new();
        let (request, pass_root, receipt_path) = write_fixture(&root.0);
        let manifest_path = root.0.join("session/manifest.json");
        let mut manifest: Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["cold_evidence_invocation_id"] = Value::String("invocation-99".into());
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let mut receipt: Value = serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
        receipt["invocation_id"] = Value::String("invocation-99".into());
        receipt["session_identity"]["manifest_sha256"] =
            Value::String(sha256_file(&manifest_path).unwrap());
        fs::remove_file(&receipt_path).unwrap();
        let updated_path = pass_root.join(format!(
            "42-10000-{}.json",
            &stable_hash(&receipt).unwrap()[..12]
        ));
        fs::write(&updated_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();

        let output = find_reusable_cold_evidence(&pass_root, &request).unwrap();
        assert!(output.reusable, "{:?}", output.rejected);
        assert_eq!(output.receipts[0].invocation_id, "invocation-99");
        assert_eq!(output.receipts[0].run_nonce, "f".repeat(64));
    }

    #[test]
    fn receipt_run_nonce_must_match_the_session_manifest() {
        let root = TestRoot::new();
        let (request, pass_root, receipt_path) = write_fixture(&root.0);
        let mut receipt: Value = serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
        receipt["run_nonce"] = Value::String("e".repeat(64));
        fs::remove_file(&receipt_path).unwrap();
        let mismatched_path = pass_root.join(format!(
            "42-10000-{}.json",
            &stable_hash(&receipt).unwrap()[..12]
        ));
        fs::write(
            &mismatched_path,
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();

        let output = find_reusable_cold_evidence(&pass_root, &request).unwrap();
        assert!(!output.reusable);
        assert_eq!(output.rejected.len(), 1);
        assert!(
            output.rejected[0]
                .error
                .contains("/cold_evidence_run_nonce must be"),
            "{:?}",
            output.rejected
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_source_artifact_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = TestRoot::new();
        let (request, pass_root, _) = write_fixture(&root.0);
        let source = root.0.join("session/input.txt");
        let replacement = root.0.join("replacement-input.txt");
        fs::rename(&source, &replacement).unwrap();
        symlink(&replacement, &source).unwrap();
        let output = find_reusable_cold_evidence(&pass_root, &request).unwrap();
        assert!(!output.reusable);
        assert!(
            output.rejected[0].error.contains("non-symlink"),
            "{:?}",
            output.rejected
        );
    }

    #[test]
    fn list_returns_only_fully_verified_invocation_ids() {
        let root = TestRoot::new();
        let (_, pass_root, _) = write_fixture(&root.0);
        let output = list_verified_cold_evidence(&pass_root).unwrap();
        assert_eq!(output.mode, "list");
        assert_eq!(output.receipts.len(), 1);
        assert_eq!(output.receipts[0].invocation_id, "invocation-42");
        assert_eq!(output.receipts[0].run_nonce, "f".repeat(64));
    }

    #[test]
    fn missing_pass_root_is_an_empty_reuse_inventory() {
        let root = TestRoot::new();
        let (request, _, _) = write_fixture(&root.0);
        let missing = root.0.join("not-created");
        let output = find_reusable_cold_evidence(&missing, &request).unwrap();
        assert!(!output.reusable);
        assert!(output.receipts.is_empty());
        assert!(output.rejected.is_empty());
    }

    #[test]
    fn hot_runtime_configuration_and_unclean_environment_are_not_authority() {
        let root = TestRoot::new();
        let (mut request, _, _) = write_fixture(&root.0);
        request.authority.invocation.runtime_config_sha256 = "a".repeat(64);
        assert!(validate_request(&request)
            .unwrap_err()
            .contains("runtime_config_sha256"));

        request.authority.invocation.runtime_config_sha256 = EMPTY_RUNTIME_CONFIG_SHA256.into();
        request.authority.invocation.execution_policy = "inherited_env".into();
        assert!(validate_request(&request)
            .unwrap_err()
            .contains("execution_policy"));
    }
}
