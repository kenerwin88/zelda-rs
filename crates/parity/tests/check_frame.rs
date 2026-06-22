use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use parity::checkpoint_cache::CheckpointCacheManifest;
use parity::golden::{self, Manifest, SCHEMA};
use parity::runner::{self, Paths};

#[test]
#[cfg(unix)]
fn check_frame_uses_nearest_compatible_checkpoint_for_one_rollup() {
    let root = temp_root("check-frame");
    let golden_dir = root.join("parity-golden");
    let cache_dir = root.join(".cache/parity-golden");
    let ck_dir = cache_dir.join("ck");
    std::fs::create_dir_all(&golden_dir).unwrap();
    std::fs::create_dir_all(&ck_dir).unwrap();

    let rom = root.join("rom.sfc");
    let save = root.join("route.sav");
    let fake_bin = root.join("target/parity/zelda3");
    let load_state_log = root.join("used-load-state.txt");
    std::fs::create_dir_all(fake_bin.parent().unwrap()).unwrap();
    std::fs::write(&rom, b"test rom").unwrap();
    std::fs::write(&save, b"test save").unwrap();
    write_fake_zelda3(&fake_bin);

    let rollups: Vec<u32> = (1..=16).map(|frame| frame * 17).collect();
    golden::write_rollup(&golden_dir.join("rollup.bin"), &rollups).unwrap();
    Manifest {
        schema: SCHEMA,
        frames: rollups.len() as u32,
        rom_sha256: runner::sha256_file(&rom).unwrap(),
        save_sha256: runner::sha256_file(&save).unwrap(),
        c_oracle_rev: "fake".into(),
        timing_hacks: vec![],
        mask: vec![],
        block_size: 8192,
        page_kb: 1,
    }
    .save(&golden_dir.join("manifest.json"))
    .unwrap();

    std::fs::write(ck_dir.join("8.sav"), b"checkpoint").unwrap();
    CheckpointCacheManifest::current(&Paths {
        repo: root.clone(),
        c_root: root.join("c"),
        rom: rom.clone(),
        save: save.clone(),
        rust_bin: fake_bin.clone(),
        golden_dir: golden_dir.clone(),
        cache_dir: cache_dir.clone(),
    })
    .unwrap()
    .save(&ck_dir.join("manifest.json"))
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(&root)
        .args(["check", "--frame", "9"])
        .env("ZELDA3_ROM", &rom)
        .env("ZELDA3_REPLAY_SAVE", &save)
        .env("ZELDA3_NEW_BIN", &fake_bin)
        .env("FAKE_ZELDA3_USED_STATE_LOG", &load_state_log)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MATCH frame 9"), "{stdout}");
    assert!(std::fs::read_to_string(&load_state_log)
        .unwrap()
        .contains("8.sav"));

    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
fn write_fake_zelda3(path: &Path) {
    std::fs::write(
        path,
        format!(
            r#"#!/bin/sh
python3 - "$@" <<'PY'
import os
import struct
import sys

record_len = {record_len}
args = sys.argv[1:]
end_frame = None
out_path = None
load_state = None
i = 0
while i < len(args):
    if args[i] == "--fingerprint-log":
        out_path = args[i + 1]
        i += 2
    elif args[i] == "--load-state":
        load_state = args[i + 1]
        i += 2
    else:
        try:
            end_frame = int(args[i])
        except ValueError:
            pass
        i += 1

if end_frame is None or out_path is None:
    sys.exit(2)

start = 0
if load_state:
    start = int(os.path.splitext(os.path.basename(load_state))[0])
    log_path = os.environ.get("FAKE_ZELDA3_USED_STATE_LOG")
    if log_path:
        with open(log_path, "w") as f:
            f.write(load_state)

with open(out_path, "wb") as f:
    for frame in range(start + 1, end_frame + 1):
        rollup = frame * 17
        f.write(struct.pack("<I", frame))
        f.write(bytes(record_len - 8))
        f.write(struct.pack("<I", rollup))
PY
"#,
            record_len = parity::RECORD_LEN
        ),
    )
    .unwrap();

    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).unwrap();
}

fn temp_root(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zelda3-rs-{name}-{}-{nanos}", std::process::id()))
}
