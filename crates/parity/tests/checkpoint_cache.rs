use parity::checkpoint_cache::CheckpointCacheManifest;

fn manifest() -> CheckpointCacheManifest {
    CheckpointCacheManifest {
        schema: 1,
        checkpoint_format_version: 1,
        seed_command_version: 1,
        rust_bin_sha256: "rust-bin".to_string(),
        rom_sha256: "rom".to_string(),
        save_sha256: "save".to_string(),
        timing_hacks: vec!["SELECT_FILE".to_string(), "LOADFILE".to_string()],
    }
}

#[test]
fn checkpoint_cache_manifest_accepts_identical_seed_inputs() {
    let actual = manifest();
    let expected = manifest();

    assert_eq!(actual.incompatibility_reason(&expected), None);
    assert!(actual.is_compatible_with(&expected));
}

#[test]
fn checkpoint_cache_manifest_rejects_checkpoint_format_changes() {
    let actual = manifest();
    let mut expected = manifest();
    expected.checkpoint_format_version += 1;

    assert_eq!(
        actual.incompatibility_reason(&expected),
        Some("checkpoint format changed")
    );
    assert!(!actual.is_compatible_with(&expected));
}

#[test]
fn checkpoint_cache_manifest_rejects_rust_binary_changes() {
    let actual = manifest();
    let mut expected = manifest();
    expected.rust_bin_sha256 = "new-rust-bin".to_string();

    assert_eq!(
        actual.incompatibility_reason(&expected),
        Some("rust binary changed")
    );
    assert!(!actual.is_compatible_with(&expected));
}

#[test]
fn checkpoint_cache_manifest_does_not_depend_on_shard_boundaries() {
    let actual = manifest();
    let expected = manifest();

    assert!(
        actual.is_compatible_with(&expected),
        "shard boundary changes should be handled by missing checkpoint files, not by invalidating the seed identity"
    );
}
