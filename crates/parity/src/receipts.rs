use crate::evidence::sha256_file;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const REPORT_SCHEMA: u32 = 1;

#[derive(Debug, Deserialize)]
struct CandidateVram {
    rust_words: u64,
    #[serde(default)]
    rust_sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CandidateReceipt {
    frame: u32,
    input: String,
    rust_audio_sample_frames: u64,
    rust_engine: Value,
    #[serde(default)]
    vram: Option<CandidateVram>,
}

#[derive(Debug, Deserialize)]
struct OracleReceipt {
    frame: u32,
    input: String,
    oracle_audio_sample_frames: u64,
    oracle_engine: Value,
    #[serde(default)]
    oracle_vram_words: Option<u64>,
    #[serde(default)]
    oracle_vram_sha256: Option<String>,
}

trait FramedReceipt {
    fn frame(&self) -> u32;
}

impl FramedReceipt for CandidateReceipt {
    fn frame(&self) -> u32 {
        self.frame
    }
}

impl FramedReceipt for OracleReceipt {
    fn frame(&self) -> u32 {
        self.frame
    }
}

struct JsonlReceipts<T> {
    path: String,
    reader: BufReader<File>,
    line: String,
    line_number: u64,
    previous_frame: Option<u32>,
    _receipt: std::marker::PhantomData<T>,
}

impl<T> JsonlReceipts<T>
where
    T: for<'de> Deserialize<'de> + FramedReceipt,
{
    fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path)
            .map_err(|error| format!("cannot open receipt stream {}: {error}", path.display()))?;
        Ok(Self {
            path: path.display().to_string(),
            reader: BufReader::new(file),
            line: String::new(),
            line_number: 0,
            previous_frame: None,
            _receipt: std::marker::PhantomData,
        })
    }

    fn next(&mut self) -> Result<Option<T>, String> {
        self.line.clear();
        let bytes = self
            .reader
            .read_line(&mut self.line)
            .map_err(|error| format!("cannot read receipt stream {}: {error}", self.path))?;
        if bytes == 0 {
            return Ok(None);
        }
        self.line_number += 1;
        let receipt: T = serde_json::from_str(&self.line).map_err(|error| {
            format!(
                "invalid receipt {}:{}: {error}",
                self.path, self.line_number
            )
        })?;
        if self
            .previous_frame
            .is_some_and(|previous| receipt.frame() <= previous)
        {
            return Err(format!(
                "receipt frames are not strictly increasing at {}:{}",
                self.path, self.line_number
            ));
        }
        self.previous_frame = Some(receipt.frame());
        Ok(Some(receipt))
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ReceiptDifference {
    pub path: String,
    pub rust: Value,
    pub oracle: Value,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ReceiptFrameDifference {
    pub frame: u32,
    pub differences: Vec<ReceiptDifference>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReceiptDomainCoverage {
    pub paired_frames: u64,
    pub first_frame: Option<u32>,
    pub last_frame: Option<u32>,
    pub contiguous: bool,
    pub engine_frames: u64,
    pub input_frames: u64,
    pub audio_boundary_frames: u64,
    pub vram_word_count_frames: u64,
    pub vram_hash_frames: u64,
    pub vram_hash_unavailable_frames: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ReceiptComparisonReport {
    pub schema: u32,
    pub kind: String,
    pub status: String,
    pub matched: bool,
    pub complete: bool,
    pub candidate_sha256: String,
    pub oracle_sha256: String,
    pub candidate_records: u64,
    pub oracle_records: u64,
    pub coverage: ReceiptDomainCoverage,
    pub first_mismatch_frame: Option<u32>,
    pub differing_frames: Vec<ReceiptFrameDifference>,
    pub differing_frames_truncated: bool,
}

fn summarized(value: &Value) -> Value {
    match value {
        Value::Array(values) => json!({"type": "array", "len": values.len()}),
        Value::Object(values) => json!({"type": "object", "keys": values.len()}),
        _ => value.clone(),
    }
}

fn push_difference(
    differences: &mut Vec<ReceiptDifference>,
    max_differences: usize,
    path: String,
    rust: Value,
    oracle: Value,
) {
    if differences.len() < max_differences {
        differences.push(ReceiptDifference {
            path,
            rust: summarized(&rust),
            oracle: summarized(&oracle),
        });
    }
}

fn diff_values(
    rust: &Value,
    oracle: &Value,
    path: &str,
    differences: &mut Vec<ReceiptDifference>,
    max_differences: usize,
) {
    if rust == oracle || differences.len() >= max_differences {
        return;
    }
    match (rust, oracle) {
        (Value::Object(rust), Value::Object(oracle)) => {
            let keys = rust
                .keys()
                .chain(oracle.keys())
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            for key in keys {
                let child = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                match (rust.get(key), oracle.get(key)) {
                    (Some(rust), Some(oracle)) => {
                        diff_values(rust, oracle, &child, differences, max_differences)
                    }
                    (rust, oracle) => push_difference(
                        differences,
                        max_differences,
                        child,
                        rust.cloned().unwrap_or_else(|| json!("<missing>")),
                        oracle.cloned().unwrap_or_else(|| json!("<missing>")),
                    ),
                }
                if differences.len() >= max_differences {
                    break;
                }
            }
        }
        (Value::Array(rust), Value::Array(oracle)) => {
            if rust.len() != oracle.len() {
                push_difference(
                    differences,
                    max_differences,
                    format!("{path}.length"),
                    json!(rust.len()),
                    json!(oracle.len()),
                );
            }
            for (index, (rust, oracle)) in rust.iter().zip(oracle).enumerate() {
                diff_values(
                    rust,
                    oracle,
                    &format!("{path}[{index}]"),
                    differences,
                    max_differences,
                );
                if differences.len() >= max_differences {
                    break;
                }
            }
        }
        _ => push_difference(
            differences,
            max_differences,
            path.to_string(),
            rust.clone(),
            oracle.clone(),
        ),
    }
}

fn compare_pair(
    candidate: &CandidateReceipt,
    oracle: &OracleReceipt,
    coverage: &mut ReceiptDomainCoverage,
    max_differences: usize,
) -> Vec<ReceiptDifference> {
    let mut differences = Vec::new();
    coverage.paired_frames += 1;
    coverage.first_frame.get_or_insert(candidate.frame);
    if let Some(last) = coverage.last_frame {
        coverage.contiguous &= candidate.frame == last.saturating_add(1);
    }
    coverage.last_frame = Some(candidate.frame);
    coverage.input_frames += 1;
    if candidate.input != oracle.input {
        push_difference(
            &mut differences,
            max_differences,
            "input".to_string(),
            json!(candidate.input),
            json!(oracle.input),
        );
    }
    coverage.audio_boundary_frames += 1;
    if candidate.rust_audio_sample_frames != oracle.oracle_audio_sample_frames {
        push_difference(
            &mut differences,
            max_differences,
            "audio_sample_frames".to_string(),
            json!(candidate.rust_audio_sample_frames),
            json!(oracle.oracle_audio_sample_frames),
        );
    }
    coverage.engine_frames += 1;
    diff_values(
        &candidate.rust_engine,
        &oracle.oracle_engine,
        "engine",
        &mut differences,
        max_differences,
    );
    match (&candidate.vram, oracle.oracle_vram_words) {
        (Some(candidate_vram), Some(oracle_words)) => {
            coverage.vram_word_count_frames += 1;
            if candidate_vram.rust_words != oracle_words {
                push_difference(
                    &mut differences,
                    max_differences,
                    "vram.words".to_string(),
                    json!(candidate_vram.rust_words),
                    json!(oracle_words),
                );
            }
            match (
                candidate_vram.rust_sha256.as_deref(),
                oracle.oracle_vram_sha256.as_deref(),
            ) {
                (Some(rust), Some(oracle)) => {
                    coverage.vram_hash_frames += 1;
                    if rust != oracle {
                        push_difference(
                            &mut differences,
                            max_differences,
                            "vram.sha256".to_string(),
                            json!(rust),
                            json!(oracle),
                        );
                    }
                }
                _ => coverage.vram_hash_unavailable_frames += 1,
            }
        }
        _ => coverage.vram_hash_unavailable_frames += 1,
    }
    differences
}

pub fn compare_receipts(
    candidate_path: &Path,
    oracle_path: &Path,
    max_differing_frames: usize,
    max_differences_per_frame: usize,
) -> Result<ReceiptComparisonReport, String> {
    if max_differing_frames == 0 || max_differences_per_frame == 0 {
        return Err("receipt comparison limits must be greater than zero".to_string());
    }
    let mut candidates = JsonlReceipts::<CandidateReceipt>::open(candidate_path)?;
    let mut oracles = JsonlReceipts::<OracleReceipt>::open(oracle_path)?;
    let mut candidate = candidates.next()?;
    let mut oracle = oracles.next()?;
    let mut candidate_records = 0_u64;
    let mut oracle_records = 0_u64;
    let mut coverage = ReceiptDomainCoverage {
        paired_frames: 0,
        first_frame: None,
        last_frame: None,
        contiguous: true,
        engine_frames: 0,
        input_frames: 0,
        audio_boundary_frames: 0,
        vram_word_count_frames: 0,
        vram_hash_frames: 0,
        vram_hash_unavailable_frames: 0,
    };
    let mut differing_frames = Vec::new();
    let mut differing_frames_truncated = false;
    let mut first_mismatch_frame = None;
    while candidate.is_some() || oracle.is_some() {
        let candidate_frame = candidate.as_ref().map(FramedReceipt::frame);
        let oracle_frame = oracle.as_ref().map(FramedReceipt::frame);
        let frame = match (candidate_frame, oracle_frame) {
            (Some(candidate_frame), Some(oracle_frame)) => candidate_frame.min(oracle_frame),
            (Some(frame), None) | (None, Some(frame)) => frame,
            (None, None) => break,
        };
        let differences = match (candidate.as_ref(), oracle.as_ref()) {
            (Some(candidate), Some(oracle)) if candidate.frame == oracle.frame => {
                compare_pair(candidate, oracle, &mut coverage, max_differences_per_frame)
            }
            (Some(candidate), _) if candidate.frame == frame => {
                vec![ReceiptDifference {
                    path: "receipt".to_string(),
                    rust: json!("present"),
                    oracle: json!("<missing>"),
                }]
            }
            _ => vec![ReceiptDifference {
                path: "receipt".to_string(),
                rust: json!("<missing>"),
                oracle: json!("present"),
            }],
        };
        if !differences.is_empty() {
            first_mismatch_frame.get_or_insert(frame);
            if differing_frames.len() < max_differing_frames {
                differing_frames.push(ReceiptFrameDifference { frame, differences });
            } else {
                differing_frames_truncated = true;
            }
        }
        if candidate_frame == Some(frame) {
            candidate_records += 1;
            candidate = candidates.next()?;
        }
        if oracle_frame == Some(frame) {
            oracle_records += 1;
            oracle = oracles.next()?;
        }
    }
    let matched = first_mismatch_frame.is_none();
    let complete = coverage.paired_frames != 0
        && coverage.vram_hash_unavailable_frames == 0
        && candidate_records == oracle_records
        && candidate_records == coverage.paired_frames;
    let status = if !matched {
        "mismatch"
    } else if complete {
        "matched_recorded_receipts"
    } else {
        "matched_available_domains"
    };
    Ok(ReceiptComparisonReport {
        schema: REPORT_SCHEMA,
        kind: "zelda3-semantic-receipt-comparison".to_string(),
        status: status.to_string(),
        matched,
        complete,
        candidate_sha256: sha256_file(candidate_path)?,
        oracle_sha256: sha256_file(oracle_path)?,
        candidate_records,
        oracle_records,
        coverage,
        first_mismatch_frame,
        differing_frames,
        differing_frames_truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("zparity-receipts-{}-{nonce}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn compares_complete_semantic_receipts() {
        let temporary = TestDirectory::new();
        let candidate = temporary.0.join("candidate.jsonl");
        let oracle = temporary.0.join("oracle.jsonl");
        fs::write(
            &candidate,
            "{\"frame\":10,\"input\":\"0x0001\",\"rust_audio_sample_frames\":534,\"rust_engine\":{\"main\":7,\"slots\":[1,2]},\"vram\":{\"rust_words\":32768,\"rust_sha256\":\"abc\"}}\n",
        )
        .unwrap();
        fs::write(
            &oracle,
            "{\"frame\":10,\"input\":\"0x0001\",\"oracle_audio_sample_frames\":534,\"oracle_engine\":{\"main\":7,\"slots\":[1,2]},\"oracle_vram_words\":32768,\"oracle_vram_sha256\":\"abc\"}\n",
        )
        .unwrap();
        let report = compare_receipts(&candidate, &oracle, 8, 16).unwrap();
        assert!(report.matched);
        assert!(report.complete);
        assert_eq!(report.status, "matched_recorded_receipts");
    }

    #[test]
    fn localizes_nested_semantic_difference_and_marks_missing_hash_incomplete() {
        let temporary = TestDirectory::new();
        let candidate = temporary.0.join("candidate.jsonl");
        let oracle = temporary.0.join("oracle.jsonl");
        fs::write(
            &candidate,
            "{\"frame\":10,\"input\":\"0x0001\",\"rust_audio_sample_frames\":534,\"rust_engine\":{\"slots\":[{\"x\":4}]},\"vram\":{\"rust_words\":32768}}\n",
        )
        .unwrap();
        fs::write(
            &oracle,
            "{\"frame\":10,\"input\":\"0x0001\",\"oracle_audio_sample_frames\":534,\"oracle_engine\":{\"slots\":[{\"x\":5}]},\"oracle_vram_words\":32768}\n",
        )
        .unwrap();
        let report = compare_receipts(&candidate, &oracle, 8, 16).unwrap();
        assert!(!report.matched);
        assert!(!report.complete);
        assert_eq!(report.first_mismatch_frame, Some(10));
        assert_eq!(
            report.differing_frames[0].differences[0].path,
            "engine.slots[0].x"
        );
    }
}
