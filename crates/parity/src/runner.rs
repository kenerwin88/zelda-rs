use std::path::{Path, PathBuf};
use std::process::Command;

pub const HACK_KEYS: &[&str] = &[
    "SELECT_FILE",
    "LOADFILE",
    "DUNGEON",
    "OVERWORLD",
    "MESSAGING",
    "DEATH_INTRO",
    "DEATH_RELOAD",
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
        Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()?
    } else {
        Command::new("sha256sum").arg(path).output()?
    };
    let s = String::from_utf8_lossy(&out.stdout);
    let hash = s.split_whitespace().next().unwrap_or("").to_string();
    if !out.status.success() || hash.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("sha256 of {} failed", path.display()),
        ));
    }
    Ok(hash)
}

#[derive(Clone)]
pub struct Paths {
    pub repo: PathBuf,
    pub c_root: PathBuf,
    pub rom: PathBuf,
    pub save: PathBuf,
    pub rust_bin: PathBuf,
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
            rom: std::env::var_os("ZELDA3_ROM")
                .map(PathBuf::from)
                .unwrap_or_else(|| repo.join("saves/zelda3.sfc")),
            save: std::env::var_os("ZELDA3_REPLAY_SAVE")
                .map(PathBuf::from)
                .unwrap_or_else(|| repo.join("saves/zelda3-combined-route.sav")),
            rust_bin: std::env::var_os("ZELDA3_NEW_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| repo.join("target/parity/zelda3")),
            cache_dir: repo.join(".cache/parity-golden"),
            c_root,
            repo,
        }
    }
}




/// Build a Rust replay command that records route-surface coverage.
pub fn rust_coverage_cmd(p: &Paths, frames: u32, coverage_out: &Path) -> Command {
    rust_coverage_cmd_with_options(p, frames, coverage_out, &CoverageRunOptions::default())
}

#[derive(Default)]
pub struct CoverageRunOptions<'a> {
    pub input_script: Option<&'a Path>,
    pub input_script_overlay: Option<&'a Path>,
    pub load_state: Option<&'a Path>,
    pub load_sram: Option<&'a Path>,
    pub stop_replay_after_load: bool,
}

/// Build a Rust replay command that records route-surface coverage.
pub fn rust_coverage_cmd_with_options(
    p: &Paths,
    frames: u32,
    coverage_out: &Path,
    options: &CoverageRunOptions<'_>,
) -> Command {
    let mut c = Command::new(&p.rust_bin);
    c.current_dir(&p.repo)
        .args(["--replay-save"])
        .arg(&p.rom)
        .arg(&p.save)
        .arg(frames.to_string())
        .args(["--audio-trace-log", "100000000"])
        .args(["--coverage-log"])
        .arg(coverage_out);
    if let Some(input_script) = options.input_script {
        c.args(["--input-script"]).arg(input_script);
    }
    if let Some(input_script_overlay) = options.input_script_overlay {
        c.args(["--input-script-overlay"]).arg(input_script_overlay);
    }
    if let Some(load_state) = options.load_state {
        c.args(["--load-state"]).arg(load_state);
    }
    if let Some(load_sram) = options.load_sram {
        c.args(["--load-sram"]).arg(load_sram);
    }
    if options.stop_replay_after_load {
        c.arg("--stop-replay-after-load");
    }
    for (k, v) in hack_env() {
        c.env(k, v);
    }
    c
}

/// Build a Rust command that runs targeted route-surface probes and records coverage.
pub fn rust_direct_entrance_probe_cmd(
    p: &Paths,
    coverage_out: &Path,
    entrance_indices: &[u16],
    dungeon_rooms: &[u16],
    overworld_screens: &[u16],
) -> Command {
    let mut c = Command::new(&p.rust_bin);
    c.current_dir(&p.repo)
        .args(["--coverage-probe"])
        .arg(&p.rom)
        .args(["--coverage-log"])
        .arg(coverage_out);
    for entrance in entrance_indices {
        c.args(["--direct-entrance", &entrance.to_string()]);
    }
    for room in dungeon_rooms {
        c.args(["--dungeon-room", &room.to_string()]);
    }
    for screen in overworld_screens {
        c.args(["--overworld-screen", &screen.to_string()]);
    }
    for (k, v) in hack_env() {
        c.env(k, v);
    }
    c
}
