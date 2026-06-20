//! Smoke: capture a tiny route, assert artifacts + manifest sanity.
//! Ignored by default (needs built binaries + ROM); run with --ignored.
use std::path::Path;
use std::process::Command;

#[test]
#[ignore]
fn capture_smoke() {
    let repo = env!("CARGO_MANIFEST_DIR"); // crates/parity
    let root = Path::new(repo).parent().unwrap().parent().unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(root)
        .args(["capture", "--frames", "500"])
        .status()
        .unwrap();
    assert!(status.success());
    let gd = root.join("parity-golden");
    assert!(gd.join("rollup.bin").exists());
    assert!(gd.join("merkle.bin").exists());
    let man: serde_json::Value =
        serde_json::from_slice(&std::fs::read(gd.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(man["frames"], 500);
    assert_eq!(man["block_size"], 8192);
    assert!(
        man["rom_sha256"].as_str().is_some_and(|s| !s.is_empty()),
        "rom_sha256 must be a non-empty string"
    );
    assert!(
        man["save_sha256"].as_str().is_some_and(|s| !s.is_empty()),
        "save_sha256 must be a non-empty string"
    );
}
