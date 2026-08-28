use crate::evidence::sha256_file;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const REPORT_SCHEMA: u32 = 1;
const LEDGER_SCHEMA: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct VideoDigest {
    pub width: u32,
    pub height: u32,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AudioDigest {
    pub sample_frames: u64,
    pub channels: u32,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
struct CandidateVideo {
    rust: VideoDigest,
}

#[derive(Debug, Deserialize)]
struct CandidateAudio {
    rust: AudioDigest,
}

#[derive(Debug, Deserialize)]
struct CandidateRecord {
    schema: u32,
    frame: u32,
    input: String,
    #[serde(default)]
    video: Option<CandidateVideo>,
    #[serde(default)]
    audio: Option<CandidateAudio>,
}

#[derive(Debug, Deserialize)]
struct OracleRecord {
    schema: u32,
    frame: u32,
    input: String,
    #[serde(default)]
    video: Option<VideoDigest>,
    #[serde(default)]
    audio: Option<AudioDigest>,
}

trait FramedRecord {
    fn frame(&self) -> u32;
    fn schema(&self) -> u32;
}

impl FramedRecord for CandidateRecord {
    fn frame(&self) -> u32 {
        self.frame
    }

    fn schema(&self) -> u32 {
        self.schema
    }
}

impl FramedRecord for OracleRecord {
    fn frame(&self) -> u32 {
        self.frame
    }

    fn schema(&self) -> u32 {
        self.schema
    }
}

struct JsonlRecords<T> {
    path: String,
    reader: BufReader<File>,
    line: String,
    line_number: u64,
    previous_frame: Option<u32>,
    _record: std::marker::PhantomData<T>,
}

impl<T> JsonlRecords<T>
where
    T: for<'de> Deserialize<'de> + FramedRecord,
{
    fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path)
            .map_err(|error| format!("cannot open A/V ledger {}: {error}", path.display()))?;
        Ok(Self {
            path: path.display().to_string(),
            reader: BufReader::new(file),
            line: String::new(),
            line_number: 0,
            previous_frame: None,
            _record: std::marker::PhantomData,
        })
    }

    fn next(&mut self) -> Result<Option<T>, String> {
        self.line.clear();
        let bytes = self
            .reader
            .read_line(&mut self.line)
            .map_err(|error| format!("cannot read A/V ledger {}: {error}", self.path))?;
        if bytes == 0 {
            return Ok(None);
        }
        self.line_number += 1;
        let record: T = serde_json::from_str(&self.line).map_err(|error| {
            format!(
                "invalid A/V ledger {}:{}: {error}",
                self.path, self.line_number
            )
        })?;
        if record.schema() != LEDGER_SCHEMA {
            return Err(format!(
                "unsupported A/V ledger schema {} at {}:{}",
                record.schema(),
                self.path,
                self.line_number
            ));
        }
        if self
            .previous_frame
            .is_some_and(|previous| record.frame() <= previous)
        {
            return Err(format!(
                "A/V ledger frames are not strictly increasing at {}:{}",
                self.path, self.line_number
            ));
        }
        self.previous_frame = Some(record.frame());
        Ok(Some(record))
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AvDifference {
    pub path: String,
    pub rust: Value,
    pub oracle: Value,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AvFrameDifference {
    pub frame: u32,
    pub differences: Vec<AvDifference>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AvCoverage {
    pub paired_frames: u64,
    pub video_frames: u64,
    pub audio_frames: u64,
    pub frames_without_enabled_lanes: u64,
    pub first_frame: Option<u32>,
    pub last_frame: Option<u32>,
    pub contiguous: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AvComparisonReport {
    pub schema: u32,
    pub kind: String,
    pub status: String,
    pub matched: bool,
    pub complete: bool,
    pub candidate_sha256: String,
    pub oracle_sha256: String,
    pub candidate_records: u64,
    pub oracle_records: u64,
    pub coverage: AvCoverage,
    pub first_mismatch_frame: Option<u32>,
    pub differing_frames: Vec<AvFrameDifference>,
    pub differing_frames_truncated: bool,
}

fn difference(path: &str, rust: impl Serialize, oracle: impl Serialize) -> AvDifference {
    AvDifference {
        path: path.to_string(),
        rust: serde_json::to_value(rust).unwrap_or_else(|_| json!("<unserializable>")),
        oracle: serde_json::to_value(oracle).unwrap_or_else(|_| json!("<unserializable>")),
    }
}

fn compare_pair(
    candidate: &CandidateRecord,
    oracle: &OracleRecord,
    coverage: &mut AvCoverage,
) -> Vec<AvDifference> {
    let mut differences = Vec::new();
    coverage.paired_frames += 1;
    coverage.first_frame.get_or_insert(candidate.frame);
    if let Some(last) = coverage.last_frame {
        coverage.contiguous &= candidate.frame == last.saturating_add(1);
    }
    coverage.last_frame = Some(candidate.frame);
    if candidate.input != oracle.input {
        differences.push(difference("input", &candidate.input, &oracle.input));
    }
    match (&candidate.video, &oracle.video) {
        (Some(candidate), Some(oracle)) => {
            coverage.video_frames += 1;
            if candidate.rust != *oracle {
                if candidate.rust.width != oracle.width {
                    differences.push(difference(
                        "video.width",
                        candidate.rust.width,
                        oracle.width,
                    ));
                }
                if candidate.rust.height != oracle.height {
                    differences.push(difference(
                        "video.height",
                        candidate.rust.height,
                        oracle.height,
                    ));
                }
                if candidate.rust.sha256 != oracle.sha256 {
                    differences.push(difference(
                        "video.sha256",
                        &candidate.rust.sha256,
                        &oracle.sha256,
                    ));
                }
            }
        }
        (Some(_), None) => differences.push(difference("video", "present", "<missing>")),
        (None, Some(_)) => differences.push(difference("video", "<missing>", "present")),
        (None, None) => {}
    }
    match (&candidate.audio, &oracle.audio) {
        (Some(candidate), Some(oracle)) => {
            coverage.audio_frames += 1;
            if candidate.rust != *oracle {
                if candidate.rust.sample_frames != oracle.sample_frames {
                    differences.push(difference(
                        "audio.sample_frames",
                        candidate.rust.sample_frames,
                        oracle.sample_frames,
                    ));
                }
                if candidate.rust.channels != oracle.channels {
                    differences.push(difference(
                        "audio.channels",
                        candidate.rust.channels,
                        oracle.channels,
                    ));
                }
                if candidate.rust.sha256 != oracle.sha256 {
                    differences.push(difference(
                        "audio.sha256",
                        &candidate.rust.sha256,
                        &oracle.sha256,
                    ));
                }
            }
        }
        (Some(_), None) => differences.push(difference("audio", "present", "<missing>")),
        (None, Some(_)) => differences.push(difference("audio", "<missing>", "present")),
        (None, None) => {}
    }
    if candidate.video.is_none() && candidate.audio.is_none() {
        coverage.frames_without_enabled_lanes += 1;
    }
    differences
}

pub fn compare_av_ledgers(
    candidate_path: &Path,
    oracle_path: &Path,
    max_differing_frames: usize,
    allow_stopped_candidate_prefix: bool,
) -> Result<AvComparisonReport, String> {
    if max_differing_frames == 0 {
        return Err("A/V comparison limit must be greater than zero".to_string());
    }
    let mut candidates = JsonlRecords::<CandidateRecord>::open(candidate_path)?;
    let mut oracles = JsonlRecords::<OracleRecord>::open(oracle_path)?;
    let mut candidate = candidates.next()?;
    let mut oracle = oracles.next()?;
    let mut candidate_records = 0_u64;
    let mut oracle_records = 0_u64;
    let mut coverage = AvCoverage {
        paired_frames: 0,
        video_frames: 0,
        audio_frames: 0,
        frames_without_enabled_lanes: 0,
        first_frame: None,
        last_frame: None,
        contiguous: true,
    };
    let mut differing_frames = Vec::new();
    let mut differing_frames_truncated = false;
    let mut first_mismatch_frame = None;
    while candidate.is_some() || oracle.is_some() {
        let suppress_expected_tail =
            allow_stopped_candidate_prefix && candidate.is_none() && first_mismatch_frame.is_some();
        let candidate_frame = candidate.as_ref().map(FramedRecord::frame);
        let oracle_frame = oracle.as_ref().map(FramedRecord::frame);
        let frame = match (candidate_frame, oracle_frame) {
            (Some(candidate), Some(oracle)) => candidate.min(oracle),
            (Some(frame), None) | (None, Some(frame)) => frame,
            (None, None) => break,
        };
        let differences = if suppress_expected_tail {
            Vec::new()
        } else {
            match (candidate.as_ref(), oracle.as_ref()) {
                (Some(candidate), Some(oracle)) if candidate.frame == oracle.frame => {
                    compare_pair(candidate, oracle, &mut coverage)
                }
                (Some(candidate), _) if candidate.frame == frame => {
                    vec![difference("record", "present", "<missing>")]
                }
                _ => vec![difference("record", "<missing>", "present")],
            }
        };
        if !differences.is_empty() {
            first_mismatch_frame.get_or_insert(frame);
            if differing_frames.len() < max_differing_frames {
                differing_frames.push(AvFrameDifference { frame, differences });
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
        && coverage.frames_without_enabled_lanes == 0
        && candidate_records == oracle_records
        && candidate_records == coverage.paired_frames;
    let status = if !matched {
        "mismatch"
    } else if complete {
        "matched_exact_av_hashes"
    } else {
        "matched_available_av_hashes"
    };
    Ok(AvComparisonReport {
        schema: REPORT_SCHEMA,
        kind: "zelda3-canonical-av-hash-comparison".to_string(),
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
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "zparity-av-{}-{nonce}-{sequence}",
                std::process::id()
            ));
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
    fn compares_complete_av_hash_ledgers() {
        let temporary = TestDirectory::new();
        let candidate = temporary.0.join("candidate.jsonl");
        let oracle = temporary.0.join("oracle.jsonl");
        fs::write(&candidate, "{\"schema\":1,\"frame\":7,\"input\":\"0x0001\",\"video\":{\"rust\":{\"width\":256,\"height\":224,\"sha256\":\"v\"},\"oracle\":{\"width\":256,\"height\":224,\"sha256\":\"ignored\"}},\"audio\":{\"rust\":{\"sample_frames\":534,\"channels\":2,\"sha256\":\"a\"},\"oracle\":{\"sample_frames\":534,\"channels\":2,\"sha256\":\"ignored\"}}}\n").unwrap();
        fs::write(&oracle, "{\"schema\":1,\"frame\":7,\"input\":\"0x0001\",\"video\":{\"width\":256,\"height\":224,\"sha256\":\"v\"},\"audio\":{\"sample_frames\":534,\"channels\":2,\"sha256\":\"a\"}}\n").unwrap();
        let report = compare_av_ledgers(&candidate, &oracle, 8, false).unwrap();
        assert!(report.matched);
        assert!(report.complete);
        assert_eq!(report.status, "matched_exact_av_hashes");
    }

    #[test]
    fn localizes_av_hash_difference() {
        let temporary = TestDirectory::new();
        let candidate = temporary.0.join("candidate.jsonl");
        let oracle = temporary.0.join("oracle.jsonl");
        fs::write(&candidate, "{\"schema\":1,\"frame\":7,\"input\":\"0x0000\",\"video\":{\"rust\":{\"width\":256,\"height\":224,\"sha256\":\"rust\"}},\"audio\":null}\n").unwrap();
        fs::write(&oracle, "{\"schema\":1,\"frame\":7,\"input\":\"0x0000\",\"video\":{\"width\":256,\"height\":224,\"sha256\":\"oracle\"},\"audio\":null}\n").unwrap();
        let report = compare_av_ledgers(&candidate, &oracle, 8, false).unwrap();
        assert!(!report.matched);
        assert_eq!(report.first_mismatch_frame, Some(7));
        assert_eq!(
            report.differing_frames[0].differences[0].path,
            "video.sha256"
        );
    }

    #[test]
    fn stopped_candidate_prefix_does_not_expand_one_mismatch_into_missing_tail_frames() {
        let temporary = TestDirectory::new();
        let candidate = temporary.0.join("candidate.jsonl");
        let oracle = temporary.0.join("oracle.jsonl");
        fs::write(&candidate, "{\"schema\":1,\"frame\":0,\"input\":\"0x0000\",\"video\":{\"rust\":{\"width\":1,\"height\":1,\"sha256\":\"bad\"}},\"audio\":null}\n").unwrap();
        fs::write(&oracle, concat!(
            "{\"schema\":1,\"frame\":0,\"input\":\"0x0000\",\"video\":{\"width\":1,\"height\":1,\"sha256\":\"good\"},\"audio\":null}\n",
            "{\"schema\":1,\"frame\":1,\"input\":\"0x0000\",\"video\":{\"width\":1,\"height\":1,\"sha256\":\"later\"},\"audio\":null}\n"
        )).unwrap();
        let report = compare_av_ledgers(&candidate, &oracle, 8, true).unwrap();
        assert!(!report.matched);
        assert_eq!(report.oracle_records, 2);
        assert_eq!(report.differing_frames.len(), 1);
        assert_eq!(report.differing_frames[0].frame, 0);
    }
}
