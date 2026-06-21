//! `zparity drill` integration tests. Ignored by default; require built binaries,
//! ROM, golden detail (run `zparity capture --detail`).

use std::path::Path;
use std::process::Command;

#[test]
#[ignore]
fn drill_reports_match_before_divergence() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    // Frame 1000 is before the first known divergence; drill must say MATCH.
    let out = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(root)
        .args(["drill", "1000"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("MATCH"),
        "expected MATCH at frame 1000:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(out.status.success(), "zparity drill 1000 exited non-zero:\n{stdout}{stderr}");
}
