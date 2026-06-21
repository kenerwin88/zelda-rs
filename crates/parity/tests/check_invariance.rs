//! Shard invariance + MATCH on the current (passing) binary. Ignored by default.
use std::path::Path;
use std::process::Command;

fn zparity(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zparity")).current_dir(root).args(args).output().unwrap()
}

#[test]
#[ignore]
fn match_and_shard_invariant() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    // Requires a golden already captured for >= 30000 frames.
    let a = zparity(root, &["check", "--frames", "30000", "--shards", "1"]);
    let b = zparity(root, &["check", "--frames", "30000", "--shards", "4"]);
    assert!(a.status.success(), "1-shard: {}", String::from_utf8_lossy(&a.stderr));
    assert!(b.status.success(), "4-shard: {}", String::from_utf8_lossy(&b.stderr));
    let sa = String::from_utf8_lossy(&a.stdout);
    let sb = String::from_utf8_lossy(&b.stdout);
    assert!(sa.contains("MATCH") && sb.contains("MATCH"), "{sa}||{sb}");
    // roots reported identically
    let root_of = |s: &str| s.split("root=").nth(1).map(|x| x.trim().to_string());
    assert_eq!(root_of(&sa), root_of(&sb));
}
