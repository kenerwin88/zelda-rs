use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::image_output::write_rgba_frame_png;

const RECORDING_KIND: &str = "zelda3_snes9x_route_recording_v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecorderIdentity {
    pub(crate) core_name: String,
    pub(crate) core_version: String,
    pub(crate) core_sha256: String,
    pub(crate) rom_sha256: String,
}

pub(crate) struct BoundaryCapture<'a> {
    pub(crate) state: &'a [u8],
    pub(crate) wram: &'a [u8],
    pub(crate) vram: &'a [u8],
    pub(crate) sram: &'a [u8],
    pub(crate) screenshot_rgba: &'a [u8],
    pub(crate) screenshot_width: u32,
    pub(crate) screenshot_height: u32,
    pub(crate) telemetry: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OracleFrameReceipt {
    pub(crate) frame: u32,
    pub(crate) input: String,
    pub(crate) video_fnv64: String,
    pub(crate) audio_fnv64: String,
    pub(crate) audio_sample_frames: usize,
    pub(crate) telemetry: serde_json::Value,
}

impl OracleFrameReceipt {
    pub(crate) fn new(
        frame: u32,
        input: u16,
        video_fnv64: u64,
        audio_fnv64: u64,
        audio_sample_frames: usize,
        telemetry: serde_json::Value,
    ) -> Self {
        Self {
            frame,
            input: format!("0x{input:04x}"),
            video_fnv64: format!("{video_fnv64:016x}"),
            audio_fnv64: format!("{audio_fnv64:016x}"),
            audio_sample_frames,
            telemetry,
        }
    }

    fn input_value(&self) -> Result<u16, String> {
        u16::from_str_radix(self.input.trim_start_matches("0x"), 16)
            .map_err(|error| format!("invalid recorded input {}: {error}", self.input))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BoundaryEntry {
    id: usize,
    #[serde(default)]
    oracle_generation: usize,
    #[serde(default)]
    reset_start: bool,
    state_path: String,
    state_sha256: String,
    wram_path: String,
    wram_sha256: String,
    #[serde(default)]
    vram_path: String,
    #[serde(default)]
    vram_sha256: String,
    sram_path: String,
    sram_sha256: String,
    screenshot_path: String,
    #[serde(default)]
    screenshot_sha256: String,
    created_by: String,
    converted_from_rust: bool,
    telemetry: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TakeEntry {
    id: usize,
    #[serde(default)]
    oracle_generation: usize,
    start_boundary: usize,
    end_boundary: Option<usize>,
    frames: usize,
    input_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rom_random_path: Option<String>,
    #[serde(default)]
    receipts_path: String,
    status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    merged_from_takes: Option<Vec<usize>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    merged_across_boundary: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OracleGeneration {
    id: usize,
    first_take: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resumed_from_boundary: Option<usize>,
    identity: RecorderIdentity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProjectManifest {
    kind: String,
    oracle_only: bool,
    converted_from_rust: bool,
    identity: RecorderIdentity,
    #[serde(default)]
    oracle_generations: Vec<OracleGeneration>,
    boundaries: Vec<BoundaryEntry>,
    takes: Vec<TakeEntry>,
}

#[derive(Debug)]
struct ActiveTake {
    id: usize,
    oracle_generation: usize,
    start_boundary: usize,
    dir: PathBuf,
    receipts: BufWriter<File>,
    input_history: Vec<(u32, u16)>,
}

#[derive(Debug)]
pub(crate) struct RecorderProject {
    root: PathBuf,
    manifest: ProjectManifest,
    active_oracle_generation: usize,
    pending_rollover_identity: Option<RecorderIdentity>,
    active_take: Option<ActiveTake>,
}

fn validate_oracle_generations(manifest: &ProjectManifest) -> Result<(), String> {
    for (expected_id, generation) in manifest.oracle_generations.iter().enumerate() {
        if generation.id != expected_id {
            return Err(format!(
                "recorder oracle generation {} is out of sequence; expected {expected_id}",
                generation.id
            ));
        }
        if generation.identity.rom_sha256 != manifest.identity.rom_sha256 {
            return Err(format!(
                "recorder oracle generation {} ROM SHA-256 does not match the project",
                generation.id
            ));
        }
        if generation.id == 0 {
            if generation.identity != manifest.identity
                || generation.first_take != 0
                || generation.resumed_from_boundary.is_some()
            {
                return Err(
                    "recorder oracle generation 0 does not match the original project identity"
                        .to_string(),
                );
            }
        } else if generation
            .resumed_from_boundary
            .is_none_or(|boundary| boundary >= manifest.boundaries.len())
        {
            return Err(format!(
                "recorder oracle generation {} has no valid resume boundary",
                generation.id
            ));
        }
    }
    Ok(())
}

impl RecorderProject {
    pub(crate) fn open(
        root: &Path,
        identity: RecorderIdentity,
        allow_core_rollover: bool,
    ) -> Result<Self, String> {
        fs::create_dir_all(root)
            .map_err(|error| format!("create recorder project {}: {error}", root.display()))?;
        let manifest_path = root.join("manifest.json");
        let (manifest, pending_rollover_identity) = if manifest_path.exists() {
            let mut manifest: ProjectManifest = serde_json::from_slice(
                &fs::read(&manifest_path)
                    .map_err(|error| format!("read {}: {error}", manifest_path.display()))?,
            )
            .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
            if manifest.kind != RECORDING_KIND {
                return Err(format!(
                    "{} has an unsupported recorder kind",
                    manifest_path.display()
                ));
            }
            if manifest.identity.rom_sha256 != identity.rom_sha256 {
                return Err("recorder project ROM SHA-256 does not match".to_string());
            }
            if manifest.oracle_generations.is_empty() {
                manifest.oracle_generations.push(OracleGeneration {
                    id: 0,
                    first_take: 0,
                    resumed_from_boundary: None,
                    identity: manifest.identity.clone(),
                });
            }
            validate_oracle_generations(&manifest)?;
            let active_identity = &manifest
                .oracle_generations
                .last()
                .expect("oracle generation was initialized")
                .identity;
            if active_identity.core_sha256 == identity.core_sha256 {
                if active_identity.rom_sha256 != identity.rom_sha256 {
                    return Err(
                        "recorder project oracle generation ROM SHA-256 does not match".to_string(),
                    );
                }
                (manifest, None)
            } else if allow_core_rollover {
                (manifest, Some(identity))
            } else {
                return Err(
                    "recorder project Snes9x core SHA-256 does not match; an explicit oracle generation rollover is required"
                        .to_string(),
                );
            }
        } else {
            let generation = OracleGeneration {
                id: 0,
                first_take: 0,
                resumed_from_boundary: None,
                identity: identity.clone(),
            };
            (
                ProjectManifest {
                    kind: RECORDING_KIND.to_string(),
                    oracle_only: true,
                    converted_from_rust: false,
                    identity,
                    oracle_generations: vec![generation],
                    boundaries: Vec::new(),
                    takes: Vec::new(),
                },
                None,
            )
        };
        let active_oracle_generation = manifest
            .oracle_generations
            .last()
            .expect("oracle generation was initialized")
            .id;
        let mut project = Self {
            root: root.to_path_buf(),
            manifest,
            active_oracle_generation,
            pending_rollover_identity,
            active_take: None,
        };
        project.recover_interrupted_takes()?;
        project.write_manifest()?;
        Ok(project)
    }

    pub(crate) fn has_pending_oracle_rollover(&self) -> bool {
        self.pending_rollover_identity.is_some()
    }

    pub(crate) fn commit_oracle_rollover(
        &mut self,
        resumed_from_boundary: usize,
    ) -> Result<(), String> {
        let Some(identity) = self.pending_rollover_identity.take() else {
            return Ok(());
        };
        if resumed_from_boundary >= self.boundary_count() {
            self.pending_rollover_identity = Some(identity);
            return Err(format!(
                "unknown recorder rollover boundary {resumed_from_boundary}"
            ));
        }
        let id = self.manifest.oracle_generations.len();
        let first_take = self
            .manifest
            .takes
            .iter()
            .map(|take| take.id)
            .max()
            .map_or(0, |take| take + 1);
        self.manifest.oracle_generations.push(OracleGeneration {
            id,
            first_take,
            resumed_from_boundary: Some(resumed_from_boundary),
            identity,
        });
        self.active_oracle_generation = id;
        self.write_manifest()
    }

    pub(crate) fn boundary_count(&self) -> usize {
        self.manifest.boundaries.len()
    }

    pub(crate) fn take_count(&self) -> usize {
        self.manifest.takes.len()
    }

    #[cfg(test)]
    pub(crate) fn take_start_boundary(&self, take: usize) -> Option<usize> {
        self.manifest
            .takes
            .get(take)
            .map(|entry| entry.start_boundary)
    }

    pub(crate) fn capture_boundary(
        &mut self,
        capture: BoundaryCapture<'_>,
    ) -> Result<usize, String> {
        let expected_rgba =
            capture.screenshot_width as usize * capture.screenshot_height as usize * 4;
        if capture.screenshot_rgba.len() != expected_rgba {
            return Err(format!(
                "boundary screenshot has {} bytes, expected {expected_rgba}",
                capture.screenshot_rgba.len()
            ));
        }
        let id = self.manifest.boundaries.len();
        let relative_dir = format!("boundaries/{id:04}");
        let dir = self.root.join(&relative_dir);
        fs::create_dir_all(&dir)
            .map_err(|error| format!("create boundary directory {}: {error}", dir.display()))?;
        let state_path = dir.join("oracle.state");
        let wram_path = dir.join("wram.bin");
        let vram_path = dir.join("vram.bin");
        let sram_path = dir.join("sram.bin");
        let screenshot_path = dir.join("frame.png");
        fs::write(&state_path, capture.state)
            .map_err(|error| format!("write {}: {error}", state_path.display()))?;
        fs::write(&wram_path, capture.wram)
            .map_err(|error| format!("write {}: {error}", wram_path.display()))?;
        fs::write(&vram_path, capture.vram)
            .map_err(|error| format!("write {}: {error}", vram_path.display()))?;
        fs::write(&sram_path, capture.sram)
            .map_err(|error| format!("write {}: {error}", sram_path.display()))?;
        write_rgba_frame_png(
            &screenshot_path,
            capture.screenshot_rgba,
            capture.screenshot_width,
            capture.screenshot_height,
        )
        .map_err(|error| format!("write {}: {error}", screenshot_path.display()))?;
        let entry = BoundaryEntry {
            id,
            oracle_generation: self.active_oracle_generation,
            reset_start: id == 0,
            state_path: format!("{relative_dir}/oracle.state"),
            state_sha256: hash_file(&state_path)?,
            wram_path: format!("{relative_dir}/wram.bin"),
            wram_sha256: hash_file(&wram_path)?,
            vram_path: format!("{relative_dir}/vram.bin"),
            vram_sha256: hash_file(&vram_path)?,
            sram_path: format!("{relative_dir}/sram.bin"),
            sram_sha256: hash_file(&sram_path)?,
            screenshot_path: format!("{relative_dir}/frame.png"),
            screenshot_sha256: hash_file(&screenshot_path)?,
            created_by: "Snes9x libretro retro_serialize".to_string(),
            converted_from_rust: false,
            telemetry: capture.telemetry,
        };
        self.manifest.boundaries.push(entry);
        self.write_manifest()?;
        Ok(id)
    }

    pub(crate) fn begin_take(&mut self, start_boundary: usize) -> Result<usize, String> {
        if self.active_take.is_some() {
            return Err("a recorder take is already active".to_string());
        }
        if start_boundary >= self.boundary_count() {
            return Err(format!("unknown recorder boundary {start_boundary}"));
        }
        let id = self
            .manifest
            .takes
            .iter()
            .map(|take| take.id)
            .max()
            .map_or(0, |id| id + 1);
        let dir = self.root.join(format!("takes/{id:04}"));
        fs::create_dir_all(&dir)
            .map_err(|error| format!("create take directory {}: {error}", dir.display()))?;
        let receipts = BufWriter::new(
            File::create(dir.join("frame_receipts.jsonl"))
                .map_err(|error| format!("create take receipts: {error}"))?,
        );
        fs::write(
            dir.join("status.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "status": "recording",
                "id": id,
                "oracle_generation": self.active_oracle_generation,
                "start_boundary": start_boundary,
            }))
            .unwrap(),
        )
        .map_err(|error| format!("write take recovery status: {error}"))?;
        self.active_take = Some(ActiveTake {
            id,
            oracle_generation: self.active_oracle_generation,
            start_boundary,
            dir,
            receipts,
            input_history: Vec::new(),
        });
        Ok(id)
    }

    pub(crate) fn record_frame(&mut self, receipt: OracleFrameReceipt) -> Result<(), String> {
        let active = self
            .active_take
            .as_mut()
            .ok_or_else(|| "no recorder take is active".to_string())?;
        let expected_frame = active.input_history.len() as u32;
        if receipt.frame != expected_frame {
            return Err(format!(
                "recorder receipt frame {} is not the next frame {expected_frame}",
                receipt.frame
            ));
        }
        active
            .input_history
            .push((receipt.frame, receipt.input_value()?));
        serde_json::to_writer(&mut active.receipts, &receipt)
            .map_err(|error| format!("write frame receipt: {error}"))?;
        active
            .receipts
            .write_all(b"\n")
            .map_err(|error| format!("terminate frame receipt: {error}"))?;
        if receipt.frame % 60 == 59 {
            active
                .receipts
                .flush()
                .map_err(|error| format!("flush frame receipts: {error}"))?;
        }
        Ok(())
    }

    pub(crate) fn active_take_frames(&self) -> usize {
        self.active_take
            .as_ref()
            .map(|take| take.input_history.len())
            .unwrap_or(0)
    }

    pub(crate) fn finish_take(&mut self, end_boundary: Option<usize>) -> Result<usize, String> {
        let mut active = self
            .active_take
            .take()
            .ok_or_else(|| "no recorder take is active".to_string())?;
        if end_boundary.is_some_and(|boundary| boundary >= self.boundary_count()) {
            return Err(format!("unknown recorder end boundary {end_boundary:?}"));
        }
        active
            .receipts
            .flush()
            .map_err(|error| format!("flush frame receipts: {error}"))?;
        fs::write(
            active.dir.join("input.txt"),
            format_input_history(&active.input_history),
        )
        .map_err(|error| format!("write take input script: {error}"))?;
        let entry = TakeEntry {
            id: active.id,
            oracle_generation: active.oracle_generation,
            start_boundary: active.start_boundary,
            end_boundary,
            frames: active.input_history.len(),
            input_path: format!("takes/{:04}/input.txt", active.id),
            rom_random_path: None,
            receipts_path: format!("takes/{:04}/frame_receipts.jsonl", active.id),
            status: "complete".to_string(),
            merged_from_takes: None,
            merged_across_boundary: None,
        };
        self.manifest.takes.push(entry);
        fs::write(
            active.dir.join("status.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "status": "complete",
                "id": active.id,
                "oracle_generation": active.oracle_generation,
                "start_boundary": active.start_boundary,
                "end_boundary": end_boundary,
                "frames": active.input_history.len(),
            }))
            .unwrap(),
        )
        .map_err(|error| format!("finalize take recovery status: {error}"))?;
        self.write_manifest()?;
        Ok(active.id)
    }

    pub(crate) fn load_boundary_state(&self, boundary: usize) -> Result<Vec<u8>, String> {
        let entry = self
            .manifest
            .boundaries
            .get(boundary)
            .ok_or_else(|| format!("unknown recorder boundary {boundary}"))?;
        let path = self.root.join(&entry.state_path);
        let bytes = fs::read(&path)
            .map_err(|error| format!("read boundary state {}: {error}", path.display()))?;
        let actual = hash_file(&path)?;
        if actual != entry.state_sha256 {
            return Err(format!(
                "boundary {boundary} state SHA-256 does not match provenance"
            ));
        }
        Ok(bytes)
    }

    pub(crate) fn load_boundary_sram(&self, boundary: usize) -> Result<Vec<u8>, String> {
        let entry = self
            .manifest
            .boundaries
            .get(boundary)
            .ok_or_else(|| format!("unknown recorder boundary {boundary}"))?;
        let path = self.root.join(&entry.sram_path);
        let bytes = fs::read(&path)
            .map_err(|error| format!("read boundary SRAM {}: {error}", path.display()))?;
        let actual = hash_file(&path)?;
        if actual != entry.sram_sha256 {
            return Err(format!(
                "boundary {boundary} SRAM SHA-256 does not match provenance"
            ));
        }
        Ok(bytes)
    }

    fn write_manifest(&self) -> Result<(), String> {
        let path = self.root.join("manifest.json");
        let temp = self.root.join("manifest.json.tmp");
        fs::write(
            &temp,
            serde_json::to_vec_pretty(&self.manifest)
                .map_err(|error| format!("serialize recorder manifest: {error}"))?,
        )
        .map_err(|error| format!("write {}: {error}", temp.display()))?;
        fs::rename(&temp, &path).map_err(|error| format!("replace {}: {error}", path.display()))
    }

    fn recover_interrupted_takes(&mut self) -> Result<(), String> {
        let takes_dir = self.root.join("takes");
        if !takes_dir.exists() {
            return Ok(());
        }
        let mut directories = fs::read_dir(&takes_dir)
            .map_err(|error| format!("scan {}: {error}", takes_dir.display()))?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .collect::<Vec<_>>();
        directories.sort_by_key(|entry| entry.file_name());
        for directory in directories {
            let Some(id) = directory
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<usize>().ok())
            else {
                continue;
            };
            if self.manifest.takes.iter().any(|take| take.id == id) {
                continue;
            }
            let dir = directory.path();
            let status_path = dir.join("status.json");
            let status: serde_json::Value = serde_json::from_slice(
                &fs::read(&status_path)
                    .map_err(|error| format!("read {}: {error}", status_path.display()))?,
            )
            .map_err(|error| format!("parse {}: {error}", status_path.display()))?;
            let start_boundary = status["start_boundary"]
                .as_u64()
                .ok_or_else(|| format!("{} has no start_boundary", status_path.display()))?
                as usize;
            let oracle_generation = status["oracle_generation"]
                .as_u64()
                .map_or(self.active_oracle_generation, |generation| {
                    generation as usize
                });
            if oracle_generation >= self.manifest.oracle_generations.len() {
                return Err(format!(
                    "{} references unknown oracle generation {oracle_generation}",
                    status_path.display()
                ));
            }
            let receipts_path = dir.join("frame_receipts.jsonl");
            let text = fs::read_to_string(&receipts_path)
                .map_err(|error| format!("read {}: {error}", receipts_path.display()))?;
            let lines = text.lines().collect::<Vec<_>>();
            let mut history = Vec::new();
            for (line_index, line) in lines.iter().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<OracleFrameReceipt>(line) {
                    Ok(receipt) => history.push((receipt.frame, receipt.input_value()?)),
                    Err(error) if line_index + 1 == lines.len() => {
                        eprintln!(
                            "ignoring truncated final receipt in recovered take {id}: {error}"
                        );
                    }
                    Err(error) => {
                        return Err(format!(
                            "parse receipt {}:{}: {error}",
                            receipts_path.display(),
                            line_index + 1
                        ));
                    }
                }
            }
            fs::write(dir.join("input.txt"), format_input_history(&history))
                .map_err(|error| format!("recover take {id} input script: {error}"))?;
            self.manifest.takes.push(TakeEntry {
                id,
                oracle_generation,
                start_boundary,
                end_boundary: None,
                frames: history.len(),
                input_path: format!("takes/{id:04}/input.txt"),
                rom_random_path: None,
                receipts_path: format!("takes/{id:04}/frame_receipts.jsonl"),
                status: "recovered_after_interruption".to_string(),
                merged_from_takes: None,
                merged_across_boundary: None,
            });
            fs::write(
                status_path,
                serde_json::to_vec_pretty(&serde_json::json!({
                    "status": "recovered_after_interruption",
                    "id": id,
                    "oracle_generation": oracle_generation,
                    "start_boundary": start_boundary,
                    "frames": history.len(),
                }))
                .unwrap(),
            )
            .map_err(|error| format!("write recovered take {id} status: {error}"))?;
        }
        self.manifest.takes.sort_by_key(|take| take.id);
        Ok(())
    }
}

fn hash_file(path: &Path) -> Result<String, String> {
    parity::runner::sha256_file(path).map_err(|error| format!("hash {}: {error}", path.display()))
}

fn format_input_history(history: &[(u32, u16)]) -> String {
    let mut output = String::from("# Snes9x controller stream captured once per oracle frame.\n");
    let mut index = 0usize;
    while index < history.len() {
        let (start_frame, value) = history[index];
        let mut end_frame = start_frame;
        index += 1;
        while index < history.len()
            && history[index].0 == end_frame + 1
            && history[index].1 == value
        {
            end_frame = history[index].0;
            index += 1;
        }
        if value == 0 {
            continue;
        }
        if start_frame == end_frame {
            output.push_str(&format!("{start_frame} 0x{value:04x}\n"));
        } else {
            output.push_str(&format!("{start_frame}..{end_frame} 0x{value:04x}\n"));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> RecorderIdentity {
        RecorderIdentity {
            core_name: "Snes9x".to_string(),
            core_version: "1.63 test".to_string(),
            core_sha256: "11".repeat(32),
            rom_sha256: "22".repeat(32),
        }
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "zelda3-snes9x-recorder-{name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn boundaries_and_takes_are_restartable_and_branchable() {
        let dir = temp_dir("branch");
        let _ = std::fs::remove_dir_all(&dir);
        let mut project = RecorderProject::open(&dir, identity(), false).unwrap();
        let start = project
            .capture_boundary(BoundaryCapture {
                state: b"native-state-zero",
                wram: b"wram-zero",
                vram: b"vram-zero",
                sram: b"sram-zero",
                screenshot_rgba: &[0, 1, 2, 0xff],
                screenshot_width: 1,
                screenshot_height: 1,
                telemetry: serde_json::json!({"main": 0}),
            })
            .unwrap();
        assert_eq!(start, 0);
        assert_eq!(project.load_boundary_sram(start).unwrap(), b"sram-zero");

        project.begin_take(start).unwrap();
        project
            .record_frame(OracleFrameReceipt::new(
                0,
                0,
                0x1111,
                0x2222,
                534,
                serde_json::json!({"health": 24}),
            ))
            .unwrap();
        project
            .record_frame(OracleFrameReceipt::new(
                1,
                0x80,
                0x3333,
                0x4444,
                534,
                serde_json::json!({"health": 24}),
            ))
            .unwrap();
        let next = project
            .capture_boundary(BoundaryCapture {
                state: b"native-state-one",
                wram: b"wram-one",
                vram: b"vram-one",
                sram: b"sram-one",
                screenshot_rgba: &[3, 4, 5, 0xff],
                screenshot_width: 1,
                screenshot_height: 1,
                telemetry: serde_json::json!({"main": 7}),
            })
            .unwrap();
        project.finish_take(Some(next)).unwrap();
        assert_eq!(project.load_boundary_sram(next).unwrap(), b"sram-one");

        project.begin_take(start).unwrap();
        project
            .record_frame(OracleFrameReceipt::new(
                0,
                0x10,
                5,
                6,
                533,
                serde_json::json!({"health": 24}),
            ))
            .unwrap();
        project.finish_take(None).unwrap();

        assert_eq!(
            project.load_boundary_state(start).unwrap(),
            b"native-state-zero"
        );
        assert_eq!(project.boundary_count(), 2);
        assert_eq!(project.take_count(), 2);
        assert_eq!(project.take_start_boundary(0), Some(0));
        assert_eq!(project.take_start_boundary(1), Some(0));
        assert_eq!(
            std::fs::read_to_string(dir.join("takes/0000/input.txt")).unwrap(),
            "# Snes9x controller stream captured once per oracle frame.\n1 0x0080\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("takes/0001/input.txt")).unwrap(),
            "# Snes9x controller stream captured once per oracle frame.\n0 0x0010\n"
        );
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["kind"], "zelda3_snes9x_route_recording_v1");
        assert_eq!(manifest["oracle_only"], true);
        assert_eq!(manifest["converted_from_rust"], false);
        assert_eq!(manifest["boundaries"].as_array().unwrap().len(), 2);
        assert_eq!(manifest["boundaries"][0]["reset_start"], true);
        assert_eq!(manifest["boundaries"][1]["reset_start"], false);
        assert_eq!(manifest["takes"].as_array().unwrap().len(), 2);
        drop(project);
        let mut legacy_manifest = manifest;
        legacy_manifest["takes"][0]
            .as_object_mut()
            .unwrap()
            .remove("receipts_path");
        legacy_manifest["takes"][0]["rom_random_path"] =
            serde_json::json!("takes/0000/rom-random.txt");
        legacy_manifest["takes"][0]["merged_from_takes"] = serde_json::json!([7, 8]);
        legacy_manifest["takes"][0]["merged_across_boundary"] = serde_json::json!(3);
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec_pretty(&legacy_manifest).unwrap(),
        )
        .unwrap();
        drop(RecorderProject::open(&dir, identity(), false).unwrap());
        let rewritten: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(rewritten["takes"][0]["receipts_path"], "");
        assert_eq!(
            rewritten["takes"][0]["rom_random_path"],
            "takes/0000/rom-random.txt"
        );
        assert_eq!(
            rewritten["takes"][0]["merged_from_takes"],
            serde_json::json!([7, 8])
        );
        assert_eq!(rewritten["takes"][0]["merged_across_boundary"], 3);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn reopening_rejects_a_different_core_or_rom_identity() {
        let dir = temp_dir("identity");
        let _ = std::fs::remove_dir_all(&dir);
        drop(RecorderProject::open(&dir, identity(), false).unwrap());

        let mut different = identity();
        different.rom_sha256 = "33".repeat(32);
        let error = RecorderProject::open(&dir, different, false).unwrap_err();
        assert!(error.contains("ROM SHA-256"));

        let mut different = identity();
        different.core_sha256 = "44".repeat(32);
        let error = RecorderProject::open(&dir, different, false).unwrap_err();
        assert!(error.contains("explicit oracle generation rollover"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn core_rollover_is_recorded_only_after_the_boundary_is_restored() {
        let dir = temp_dir("core-rollover");
        let _ = std::fs::remove_dir_all(&dir);
        let mut project = RecorderProject::open(&dir, identity(), false).unwrap();
        let boundary = project
            .capture_boundary(BoundaryCapture {
                state: b"native-state-zero",
                wram: b"wram-zero",
                vram: b"vram-zero",
                sram: b"sram-zero",
                screenshot_rgba: &[0, 1, 2, 0xff],
                screenshot_width: 1,
                screenshot_height: 1,
                telemetry: serde_json::json!({"main": 0}),
            })
            .unwrap();
        drop(project);

        let mut next_identity = identity();
        next_identity.core_sha256 = "44".repeat(32);
        let mut project = RecorderProject::open(&dir, next_identity.clone(), true).unwrap();
        assert!(project.has_pending_oracle_rollover());
        let before: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(before["oracle_generations"].as_array().unwrap().len(), 1);

        project.commit_oracle_rollover(boundary).unwrap();
        assert!(!project.has_pending_oracle_rollover());
        project.begin_take(boundary).unwrap();
        project.finish_take(None).unwrap();

        let after: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(after["identity"]["core_sha256"], "11".repeat(32));
        assert_eq!(after["oracle_generations"].as_array().unwrap().len(), 2);
        assert_eq!(
            after["oracle_generations"][1]["identity"]["core_sha256"],
            next_identity.core_sha256
        );
        assert_eq!(after["oracle_generations"][1]["resumed_from_boundary"], 0);
        assert_eq!(after["takes"][0]["oracle_generation"], 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn reopening_recovers_a_take_interrupted_before_manifest_finalization() {
        let dir = temp_dir("recover");
        let _ = std::fs::remove_dir_all(&dir);
        let mut project = RecorderProject::open(&dir, identity(), false).unwrap();
        let boundary = project
            .capture_boundary(BoundaryCapture {
                state: b"native-state-zero",
                wram: b"wram-zero",
                vram: b"vram-zero",
                sram: b"sram-zero",
                screenshot_rgba: &[0, 1, 2, 0xff],
                screenshot_width: 1,
                screenshot_height: 1,
                telemetry: serde_json::json!({"main": 0}),
            })
            .unwrap();
        project.begin_take(boundary).unwrap();
        project
            .record_frame(OracleFrameReceipt::new(
                0,
                0x80,
                1,
                2,
                534,
                serde_json::json!({"main": 0}),
            ))
            .unwrap();
        drop(project);

        let recovered = RecorderProject::open(&dir, identity(), false).unwrap();

        assert_eq!(recovered.take_count(), 1);
        assert_eq!(recovered.take_start_boundary(0), Some(0));
        assert_eq!(
            std::fs::read_to_string(dir.join("takes/0000/input.txt")).unwrap(),
            "# Snes9x controller stream captured once per oracle frame.\n0 0x0080\n"
        );
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(
            manifest["takes"][0]["status"],
            "recovered_after_interruption"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
