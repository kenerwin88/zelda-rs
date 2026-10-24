use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use zelda3::ZeldaState;
use zelda3::debug_env;

use crate::{PlayCrashCheckpoint, PLAY_CRASH_CHECKPOINT_MAGIC};
use zelda3::RUN_MAIN;

pub(crate) struct LiveInputRecording {
    dir: PathBuf,
    events: BufWriter<File>,
    input_history: Vec<(u32, u16)>,
    output: RecordingOutput,
}

// Native movies stay available in the shipped build; cartridge scripts are tooling.
enum RecordingOutput {
    Lagless(crate::lagless_replay::MovieWriter),
    #[cfg(feature = "rom-timing")]
    Cartridge(crate::cartridge_input_recording::CartridgeInputRecording),
}

impl RecordingOutput {
    fn start(dir: &Path, game: &ZeldaState, samples: usize, channels: usize) -> Result<Self, String> {
        if game.lag_free() {
            return crate::lagless_replay::MovieWriter::create(&dir.join("input.z3lag"), game, samples, channels)
                .map(Self::Lagless).map_err(|e| format!("create lagless recording: {e}"));
        }
        Self::cartridge()
    }

    #[cfg(feature = "rom-timing")]
    fn cartridge() -> Result<Self, String> {
        Ok(Self::Cartridge(crate::cartridge_input_recording::CartridgeInputRecording))
    }

    #[cfg(not(feature = "rom-timing"))]
    fn cartridge() -> Result<Self, String> {
        Err("cartridge input recording requires the rom-timing Cargo feature".into())
    }

    fn record(&mut self, frame: u32, input: u16, count: usize) -> Result<(), String> {
        match self {
            Self::Lagless(movie) => {
                if frame as usize != count {
                    return Err("lagless recording requires contiguous iterations from boot".into());
                }
                movie.record(input).map_err(|e| format!("write lagless input: {e}"))
            }
            #[cfg(feature = "rom-timing")]
            Self::Cartridge(_) => Ok(()),
        }
    }

    fn artifact(&self) -> &'static str {
        match self {
            Self::Lagless(_) => "input.z3lag",
            #[cfg(feature = "rom-timing")]
            Self::Cartridge(_) => "input.txt",
        }
    }

    fn finish(&self, _dir: &Path, _history: &[(u32, u16)]) -> Result<(), String> {
        match self {
            Self::Lagless(_) => Ok(()),
            #[cfg(feature = "rom-timing")]
            Self::Cartridge(script) => script.finish(_dir, _history),
        }
    }
}

#[derive(Serialize)]
struct LiveInputEvent {
    frame: u32,
    input: String,
}

impl LiveInputRecording {
    pub(crate) fn start_from_env(game: &ZeldaState, samples: usize, channels: usize) -> Result<Option<Self>, String> {
        let Some(dir) = debug_env::var_os!("ZELDA3_RECORD_INPUT_SESSION") else {
            return Ok(None);
        };
        Self::start(Path::new(&dir), game, samples, channels).map(Some)
    }

    pub(crate) fn start(dir: &Path, game: &ZeldaState, samples: usize, channels: usize) -> Result<Self, String> {
        fs::create_dir_all(dir)
            .map_err(|e| format!("create live input session {}: {e}", dir.display()))?;
        for stale in ["input.txt", "input.recovered.txt", "result.json", "input.z3lag"] {
            match fs::remove_file(dir.join(stale)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(format!("remove stale live input {stale}: {error}")),
            }
        }
        let output = RecordingOutput::start(dir, game, samples, channels)?;
        fs::write(dir.join("initial.srm"), &game.sram)
            .map_err(|e| format!("write live input initial.srm: {e}"))?;
        fs::write(
            dir.join("rust_initial.z3state"),
            bincode::serialize(&PlayCrashCheckpoint {
                magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
                host_frame: 0,
                input: 0,
                run_what: RUN_MAIN,
                game: game.clone(),
            })
            .map_err(|e| format!("serialize initial game state: {e}"))?,
        )
        .map_err(|e| format!("write live input rust_initial.z3state: {e}"))?;
        let manifest = serde_json::json!({
            "schema": 1,
            "status": "recording",
            "controller_polling": if game.lag_free() { "once-per-main-iteration" } else { "once-per-game-frame" },
            "lag_free": game.lag_free(),
            "native_movie": game.lag_free().then_some("input.z3lag"),
            "transfer_boundary": if game.lag_free() { "embedded-sram-plus-clean-native-boot" } else { "initial-sram-plus-clean-rom-boot" },
            "artifacts": [
                "initial.srm",
                "rust_initial.z3state",
                "live_inputs.jsonl",
                output.artifact(),
                "result.json"
            ]
        });
        fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .map_err(|e| format!("write live input manifest: {e}"))?;
        let events = BufWriter::new(
            File::create(dir.join("live_inputs.jsonl"))
                .map_err(|e| format!("create live input event stream: {e}"))?,
        );
        Ok(Self {
            dir: dir.to_path_buf(),
            events,
            input_history: Vec::new(),
            output,
        })
    }

    pub(crate) fn record(&mut self, frame: u32, input: u16) -> Result<(), String> {
        self.output.record(frame, input, self.input_history.len())?;
        self.input_history.push((frame, input));
        serde_json::to_writer(
            &mut self.events,
            &LiveInputEvent {
                frame,
                input: format!("0x{input:04x}"),
            },
        )
        .map_err(|e| format!("write live input event: {e}"))?;
        self.events
            .write_all(b"\n")
            .map_err(|e| format!("terminate live input event: {e}"))?;
        if frame % 60 == 59 {
            self.events
                .flush()
                .map_err(|e| format!("flush live input events: {e}"))?;
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<PathBuf, String> {
        self.events
            .flush()
            .map_err(|e| format!("flush live input events: {e}"))?;
        self.output.finish(&self.dir, &self.input_history)?;
        let result = serde_json::json!({
            "status": "complete",
            "frames": self.input_history.len(),
            "last_frame": self.input_history.last().map(|entry| entry.0),
        });
        fs::write(
            self.dir.join("result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .map_err(|e| format!("write live input result: {e}"))?;
        let manifest_path = self.dir.join("manifest.json");
        let mut manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(&manifest_path).map_err(|e| format!("read live input manifest: {e}"))?,
        )
        .map_err(|e| format!("parse live input manifest: {e}"))?;
        manifest["status"] = serde_json::Value::String("complete".to_string());
        manifest["frames"] = serde_json::Value::from(self.input_history.len());
        fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap())
            .map_err(|e| format!("finalize live input manifest: {e}"))?;
        Ok(self.dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "rom-timing"))]
    #[test]
    fn cartridge_recording_requires_rom_timing() {
        assert!(matches!(RecordingOutput::cartridge(), Err(error) if error.contains("rom-timing")));
    }

    #[test]
    fn lagless_recording_writes_native_movie_from_shipped_boot() {
        let dir = std::env::temp_dir().join(format!("zelda3-native-record-{}", std::process::id()));
        let mut game = crate::load_romless_play_state();
        game.set_lag_free(true);
        let mut recording = LiveInputRecording::start(&dir, &game, 735, 2).unwrap();
        recording.record(0, 0x80).unwrap();
        assert!(recording.record(2, 0).is_err());
        recording.finish().unwrap();
        let movie = fs::read(dir.join("input.z3lag")).unwrap();
        assert_eq!(&movie[..8], b"Z3LGIN01");
        assert_eq!(&movie[movie.len() - 2..], &0x80u16.to_le_bytes());
        let manifest: serde_json::Value = serde_json::from_slice(&fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["lag_free"], true);
        assert!(manifest["artifacts"].as_array().unwrap().iter().any(|value| value == "input.z3lag"));
        assert!(!dir.join("input.txt").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(feature = "rom-timing")]
    #[test]
    fn recording_writes_crash_resilient_events_and_deterministic_script() {
        let dir = std::env::temp_dir().join(format!(
            "zelda3-live-input-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = fs::remove_dir_all(&dir);
        let game = ZeldaState::new();
        let mut recording = LiveInputRecording::start(&dir, &game, 735, 2).unwrap();
        recording.record(0, 0).unwrap();
        recording.record(1, 0x80).unwrap();
        recording.record(2, 0x80).unwrap();
        recording.finish().unwrap();

        assert_eq!(
            fs::read_to_string(dir.join("input.txt")).unwrap(),
            "# Deterministic controller stream captured once per game frame.\n1..2 0x0080\n"
        );
        assert_eq!(
            fs::read_to_string(dir.join("live_inputs.jsonl"))
                .unwrap()
                .lines()
                .count(),
            3
        );
        assert!(dir.join("initial.srm").exists());
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["status"], "complete");
        assert_eq!(manifest["frames"], 3);
        let _ = fs::remove_dir_all(dir);
    }
}
