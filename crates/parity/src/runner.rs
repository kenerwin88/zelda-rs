use std::path::{Path, PathBuf};
use std::process::Command;

pub const HACK_KEYS: &[&str] = &[
    "SELECT_FILE", "LOADFILE", "DUNGEON", "OVERWORLD", "MESSAGING", "DEATH_INTRO", "DEATH_RELOAD",
];

pub fn hack_env() -> Vec<(String, String)> {
    HACK_KEYS
        .iter()
        .map(|k| (format!("ZELDA3_SMV_{k}_TIMING_HACKS"), "1".to_string()))
        .collect()
}

pub fn sdl_dummy_env() -> Vec<(String, String)> {
    vec![
        ("SDL_VIDEODRIVER".into(), "dummy".into()),
        ("SDL_AUDIODRIVER".into(), "dummy".into()),
        ("SDL_RENDER_DRIVER".into(), "software".into()),
    ]
}

pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    // Shell out to the platform sha256 to avoid a crypto dependency.
    let out = if cfg!(target_os = "macos") {
        Command::new("shasum").args(["-a", "256"]).arg(path).output()?
    } else {
        Command::new("sha256sum").arg(path).output()?
    };
    let s = String::from_utf8_lossy(&out.stdout);
    Ok(s.split_whitespace().next().unwrap_or("").to_string())
}

pub struct Paths {
    pub repo: PathBuf,
    pub c_root: PathBuf,
    pub rom: PathBuf,
    pub save: PathBuf,
    pub rust_bin: PathBuf,
    pub golden_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl Paths {
    pub fn discover() -> Self {
        let repo = std::env::var_os("ZELDA3_REPO")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        let c_root = std::env::var_os("ZELDA3_C_REPO")
            .map(PathBuf::from)
            .unwrap_or_else(|| repo.parent().unwrap().join("zelda3"));
        Paths {
            rom: std::env::var_os("ZELDA3_ROM").map(PathBuf::from)
                .unwrap_or_else(|| repo.join("saves/zelda3.sfc")),
            save: std::env::var_os("ZELDA3_REPLAY_SAVE").map(PathBuf::from)
                .unwrap_or_else(|| repo.join("saves/zelda3-combined-route.sav")),
            rust_bin: std::env::var_os("ZELDA3_NEW_BIN").map(PathBuf::from)
                .unwrap_or_else(|| repo.join("target/parity/zelda3")),
            golden_dir: repo.join("parity-golden"),
            cache_dir: repo.join(".cache/parity-golden"),
            c_root,
            repo,
        }
    }
}

/// Build the C-oracle capture command writing a fingerprint stream.
pub fn c_capture_cmd(p: &Paths, frames: u32, fp_out: &Path) -> Command {
    let mut c = Command::new(p.c_root.join("zelda3"));
    c.current_dir(&p.c_root)
        .args(["--config"]).arg(p.c_root.join("other/headless_replay.ini"))
        .args(["--replay-save"]).arg(&p.save)
        .args(["--smv-test-frames", &frames.to_string()])
        .args(["--fingerprint-log"]).arg(fp_out);
    for (k, v) in sdl_dummy_env() {
        c.env(k, v);
    }
    c
}

/// Build a Rust replay shard command: [start checkpoint?] -> end_frame, writing fingerprints.
pub fn rust_shard_cmd(p: &Paths, end_frame: u32, fp_out: &Path, load_state: Option<&Path>) -> Command {
    let mut c = Command::new(&p.rust_bin);
    c.current_dir(&p.repo)
        .args(["--replay-save"]).arg(&p.rom).arg(&p.save).arg(end_frame.to_string())
        .args(["--fingerprint-log"]).arg(fp_out);
    if let Some(ls) = load_state {
        c.args(["--load-state"]).arg(ls);
    }
    for (k, v) in hack_env() {
        c.env(k, v);
    }
    c
}
