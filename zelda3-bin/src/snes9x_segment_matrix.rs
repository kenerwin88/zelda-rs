use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

const PROOF_KIND: &str = "zelda3_combined_checkpoint_route_v1";

#[derive(Clone, Debug, Deserialize)]
struct CombinedSaveProof {
    input_frames: u32,
    segment_frame_counts: Vec<u32>,
    cumulative_frames: Vec<u32>,
    snapshot_boundaries: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct MilestoneProof {
    cumulative_frames: u32,
    expected: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
struct RawMatrixProof {
    kind: String,
    source_segments: Vec<String>,
    combined_save: CombinedSaveProof,
    milestones: Vec<MilestoneProof>,
}

#[derive(Clone, Debug)]
pub(crate) struct MatrixProof {
    source_segments: Vec<String>,
    segment_frame_counts: Vec<u32>,
    cumulative_frames: Vec<u32>,
    milestones: Vec<BTreeMap<String, String>>,
    total_frames: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MatrixSegment<'a> {
    pub index: usize,
    pub source_name: &'a str,
    pub frames: u32,
    pub cumulative_frames: u32,
    pub expected: &'a BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct NativeStateProof {
    pub segment: usize,
    pub path: String,
    pub sha256: String,
    created_by: String,
    converted_from_rust: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct RawNativeStateSetProof {
    kind: String,
    core_sha256: String,
    rom_sha256: String,
    states: Vec<NativeStateProof>,
}

#[derive(Clone, Debug)]
pub(crate) struct NativeStateSetProof {
    core_sha256: String,
    rom_sha256: String,
    states: Vec<NativeStateProof>,
}

impl NativeStateSetProof {
    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self, String> {
        let raw: RawNativeStateSetProof = serde_json::from_slice(bytes)
            .map_err(|error| format!("failed to parse native state provenance JSON: {error}"))?;
        if raw.kind != "zelda3_snes9x_native_state_set_v1" {
            return Err("native state provenance has the wrong kind".to_string());
        }
        if raw.states.len() != 12 {
            return Err(format!(
                "native state provenance contains {} states, expected 12",
                raw.states.len()
            ));
        }
        for (index, state) in raw.states.iter().enumerate() {
            let expected_segment = index + 2;
            if state.segment != expected_segment {
                return Err(format!(
                    "native state {} is for segment {}, expected {expected_segment}",
                    index + 1,
                    state.segment
                ));
            }
            let path = Path::new(&state.path);
            if path.is_absolute() || path.components().count() != 1 || path.file_name().is_none() {
                return Err(format!(
                    "native state segment {} path must be a plain file name",
                    state.segment
                ));
            }
            if state.sha256.len() != 64
                || !state.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(format!(
                    "native state segment {} has an invalid SHA-256",
                    state.segment
                ));
            }
            if state.created_by != "Snes9x libretro retro_serialize" {
                return Err(format!(
                    "native state segment {} was not created by the required Snes9x libretro capture path",
                    state.segment
                ));
            }
            if state.converted_from_rust {
                return Err(format!(
                    "native state segment {} declares that it was converted from Rust",
                    state.segment
                ));
            }
        }
        if raw.core_sha256.is_empty() || raw.rom_sha256.is_empty() {
            return Err("native state provenance is missing core or ROM identity".to_string());
        }
        Ok(Self {
            core_sha256: raw.core_sha256,
            rom_sha256: raw.rom_sha256,
            states: raw.states,
        })
    }

    pub(crate) fn validate_engine_hashes(
        &self,
        core_sha256: &str,
        rom_sha256: &str,
    ) -> Result<(), String> {
        if self.core_sha256 != core_sha256 {
            return Err("native state set Snes9x core hash does not match".to_string());
        }
        if self.rom_sha256 != rom_sha256 {
            return Err("native state set ROM hash does not match".to_string());
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn states(&self) -> &[NativeStateProof] {
        &self.states
    }

    pub(crate) fn state_for_segment(&self, segment: usize) -> Option<&NativeStateProof> {
        self.states.iter().find(|state| state.segment == segment)
    }
}

impl MatrixProof {
    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self, String> {
        let raw: RawMatrixProof = serde_json::from_slice(bytes)
            .map_err(|error| format!("failed to parse route proof JSON: {error}"))?;
        if raw.kind != PROOF_KIND {
            return Err(format!(
                "route proof kind must be {PROOF_KIND}, got {}",
                raw.kind
            ));
        }
        let count = raw.source_segments.len();
        if count == 0 {
            return Err("route proof contains no source segments".to_string());
        }
        if raw.combined_save.segment_frame_counts.len() != count
            || raw.combined_save.cumulative_frames.len() != count
            || raw.milestones.len() != count
        {
            return Err(format!(
                "route proof arrays disagree: sources={count} segment_frames={} cumulative={} milestones={}",
                raw.combined_save.segment_frame_counts.len(),
                raw.combined_save.cumulative_frames.len(),
                raw.milestones.len()
            ));
        }
        if raw.combined_save.snapshot_boundaries != count.saturating_sub(1) {
            return Err(format!(
                "route proof snapshot boundary count is {}, expected {}",
                raw.combined_save.snapshot_boundaries,
                count.saturating_sub(1)
            ));
        }

        let mut cumulative = 0u32;
        for index in 0..count {
            let source = &raw.source_segments[index];
            let source_path = Path::new(source);
            if source_path.is_absolute()
                || source_path.components().count() != 1
                || source_path.file_name().is_none()
            {
                return Err(format!(
                    "source segment {} must be a plain file name, got {source}",
                    index + 1
                ));
            }
            let frames = raw.combined_save.segment_frame_counts[index];
            if frames == 0 {
                return Err(format!("source segment {} contains no frames", index + 1));
            }
            cumulative = cumulative
                .checked_add(frames)
                .ok_or_else(|| "aggregate frame count overflowed u32".to_string())?;
            if raw.combined_save.cumulative_frames[index] != cumulative {
                return Err(format!(
                    "cumulative frame count {} is {}, expected {cumulative}",
                    index + 1,
                    raw.combined_save.cumulative_frames[index]
                ));
            }
            if raw.milestones[index].cumulative_frames != cumulative {
                return Err(format!(
                    "milestone cumulative frame count {} is {}, expected {cumulative}",
                    index + 1,
                    raw.milestones[index].cumulative_frames
                ));
            }
            if raw.milestones[index].expected.is_empty() {
                return Err(format!("milestone {} has no expected values", index + 1));
            }
        }
        if raw.combined_save.input_frames != cumulative {
            return Err(format!(
                "aggregate input frame count is {}, expected {cumulative}",
                raw.combined_save.input_frames
            ));
        }

        Ok(Self {
            source_segments: raw.source_segments,
            segment_frame_counts: raw.combined_save.segment_frame_counts,
            cumulative_frames: raw.combined_save.cumulative_frames,
            milestones: raw
                .milestones
                .into_iter()
                .map(|milestone| milestone.expected)
                .collect(),
            total_frames: cumulative,
        })
    }

    pub(crate) fn require_segment_count(&self, expected: usize) -> Result<(), String> {
        if self.source_segments.len() == expected {
            Ok(())
        } else {
            Err(format!(
                "segmented oracle matrix requires exactly {expected} chapters, proof contains {}",
                self.source_segments.len()
            ))
        }
    }

    pub(crate) fn total_frames(&self) -> u32 {
        self.total_frames
    }

    pub(crate) fn segments(&self) -> Vec<MatrixSegment<'_>> {
        (0..self.source_segments.len())
            .map(|index| MatrixSegment {
                index,
                source_name: &self.source_segments[index],
                frames: self.segment_frame_counts[index],
                cumulative_frames: self.cumulative_frames[index],
                expected: &self.milestones[index],
            })
            .collect()
    }
}

pub(crate) fn milestone_values(ram: &[u8]) -> Result<BTreeMap<String, String>, String> {
    let byte = |address: usize| {
        ram.get(address)
            .copied()
            .ok_or_else(|| format!("Snes9x WRAM is missing address ${address:05x}"))
    };
    let word = |address: usize| -> Result<u16, String> {
        Ok(u16::from_le_bytes([byte(address)?, byte(address + 1)?]))
    };
    let main = byte(0x10)?;
    Ok(BTreeMap::from([
        (
            "ending".to_string(),
            u8::from((24..=26).contains(&main)).to_string(),
        ),
        ("main".to_string(), main.to_string()),
        ("saved".to_string(), byte(0x10c)?.to_string()),
        ("item".to_string(), format!("0x{:02x}", byte(0x202)?)),
        ("big".to_string(), format!("0x{:04x}", word(0xf366)?)),
        ("hp".to_string(), format!("0x{:02x}", byte(0xf36d)?)),
        ("state".to_string(), format!("0x{:02x}", byte(0x5d)?)),
    ]))
}

pub(crate) fn milestone_mismatches(
    ram: &[u8],
    expected: &BTreeMap<String, String>,
) -> Result<Vec<String>, String> {
    let actual = milestone_values(ram)?;
    expected
        .iter()
        .map(|(key, expected_value)| {
            let actual_value = actual
                .get(key)
                .ok_or_else(|| format!("unsupported milestone field {key}"))?;
            Ok((actual_value != expected_value)
                .then(|| format!("{key}: expected {expected_value}, got {actual_value}")))
        })
        .filter_map(|result| match result {
            Ok(Some(mismatch)) => Some(Ok(mismatch)),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{milestone_mismatches, MatrixProof, NativeStateSetProof};

    const VALID_PROOF: &str = r#"{
      "kind": "zelda3_combined_checkpoint_route_v1",
      "source_segments": ["one.sav", "two.sav"],
      "combined_save": {
        "input_frames": 30,
        "segment_frame_counts": [10, 20],
        "cumulative_frames": [10, 30],
        "snapshot_boundaries": 1
      },
      "milestones": [
        {"cumulative_frames": 10, "expected": {"main": "9", "hp": "0x28"}},
        {"cumulative_frames": 30, "expected": {"ending": "1", "main": "26"}}
      ]
    }"#;

    #[test]
    fn proof_requires_consistent_segment_and_cumulative_frame_counts() {
        let proof = MatrixProof::from_slice(VALID_PROOF.as_bytes()).unwrap();
        assert_eq!(proof.total_frames(), 30);
        assert_eq!(proof.segments().len(), 2);

        let invalid = VALID_PROOF.replace("[10, 30]", "[10, 31]");
        assert!(MatrixProof::from_slice(invalid.as_bytes())
            .unwrap_err()
            .contains("cumulative"));
    }

    #[test]
    fn milestone_values_are_read_from_snes_wram_addresses() {
        let mut ram = vec![0; 0x20_000];
        ram[0x10] = 26;
        ram[0x10c] = 25;
        ram[0x202] = 1;
        ram[0x5d] = 0;
        ram[0xf366..0xf368].copy_from_slice(&0x77fcu16.to_le_bytes());
        ram[0xf36d] = 0x50;
        let expected = serde_json::from_str(
            r#"{"ending":"1","main":"26","saved":"25","item":"0x01","big":"0x77fc","hp":"0x50","state":"0x00"}"#,
        )
        .unwrap();

        assert!(milestone_mismatches(&ram, &expected).unwrap().is_empty());
        ram[0x10] = 14;
        assert_eq!(
            milestone_mismatches(&ram, &expected).unwrap()[0],
            "ending: expected 1, got 0"
        );
    }

    #[test]
    fn native_state_set_requires_twelve_unconverted_snes9x_boundaries() {
        let states = (2..=13)
            .map(|segment| {
                serde_json::json!({
                    "segment": segment,
                    "path": format!("segment-{segment:02}.state"),
                    "sha256": format!("{segment:064x}"),
                    "created_by": "Snes9x libretro retro_serialize",
                    "converted_from_rust": false,
                })
            })
            .collect::<Vec<_>>();
        let proof = serde_json::json!({
            "kind": "zelda3_snes9x_native_state_set_v1",
            "core_sha256": "c".repeat(64),
            "rom_sha256": "d".repeat(64),
            "states": states,
        });
        let parsed = NativeStateSetProof::from_slice(&serde_json::to_vec(&proof).unwrap()).unwrap();
        assert_eq!(parsed.states().len(), 12);

        let mut converted = proof;
        converted["states"][0]["converted_from_rust"] = serde_json::Value::Bool(true);
        assert!(
            NativeStateSetProof::from_slice(&serde_json::to_vec(&converted).unwrap())
                .unwrap_err()
                .contains("converted")
        );
    }
}
