//! Shard invariance: 1-shard and 4-shard `check` must produce the IDENTICAL
//! verdict (same MATCH root, or same DIVERGE frame), whether or not the binary
//! currently matches the golden. `check` is RED against the current code by
//! design (real Rust-side parity bugs); this test asserts only the sharding
//! guarantee — checkpoint-resume yields byte-identical fingerprints, so the
//! verdict cannot depend on how the frame range is split. Ignored by default
//! (needs a captured golden + built parity binaries).
use std::path::Path;
use std::process::Command;

fn zparity(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

/// Extract the shard-count-independent verdict from `check` stdout: for a
/// divergence this is `DIVERGE at frame <N>`; for a match it is `root=0x...`.
/// (The full lines also carry the shard count + duration, which legitimately
/// differ between runs, so we normalize those away.)
fn verdict(stdout: &str) -> String {
    for line in stdout.lines() {
        if let Some(after) = line.strip_prefix("DIVERGE at frame ") {
            let n: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            return format!("DIVERGE at frame {n}");
        }
        if let Some(idx) = line.find("root=") {
            return format!("MATCH {}", line[idx..].trim());
        }
    }
    String::new()
}

#[test]
#[ignore]
fn shard_invariant() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    // Requires a golden already captured for >= 30000 frames.
    let a = zparity(root, &["check", "--frames", "30000", "--shards", "1"]);
    let b = zparity(root, &["check", "--frames", "30000", "--shards", "4"]);
    // NOTE: do NOT assert status.success() — DIVERGE exits 1 by design.
    let sa = String::from_utf8_lossy(&a.stdout);
    let sb = String::from_utf8_lossy(&b.stdout);
    let va = verdict(&sa);
    let vb = verdict(&sb);
    assert!(
        !va.is_empty(),
        "1-shard produced no verdict\nstdout: {sa}\nstderr: {}",
        String::from_utf8_lossy(&a.stderr)
    );
    assert!(
        !vb.is_empty(),
        "4-shard produced no verdict\nstdout: {sb}\nstderr: {}",
        String::from_utf8_lossy(&b.stderr)
    );
    // Core correctness guarantee: the verdict is independent of how the frame
    // range is sharded (checkpoint-resume == from-scratch, byte-for-byte).
    assert_eq!(
        va, vb,
        "shard-invariance violated:\n  1-shard: {va}\n  4-shard: {vb}"
    );
}
