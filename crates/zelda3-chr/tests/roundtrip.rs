//! Integration round-trip against the real generated CHR assets. Skips
//! gracefully when the generated tree is absent (fresh checkout / no ROM).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn container_round_trips_real_bins() {
    let assets = repo_root().join("generated/zelda3_assets/assets");
    let spr_path = assets.join("064-kSprGfx.bin");
    let bg_path = assets.join("065-kBgGfx.bin");
    if !spr_path.is_file() || !bg_path.is_file() {
        eprintln!("skipping: generated CHR bins absent");
        return;
    }
    for path in [spr_path, bg_path] {
        let bytes = std::fs::read(&path).unwrap();
        let items = zelda3_chr::unpack_packed_arrays(&bytes).unwrap();
        let repacked = zelda3_chr::pack_arrays(&items);
        assert_eq!(repacked, bytes, "{} must round-trip", path.display());
    }
}

#[test]
fn unedited_sheets_compile_to_donor_bins() {
    let root = repo_root();
    let assets = root.join("generated/zelda3_assets/assets");
    let chr_dir = root.join("assets/chr");
    let spr_path = assets.join("064-kSprGfx.bin");
    let bg_path = assets.join("065-kBgGfx.bin");
    if !spr_path.is_file() || !bg_path.is_file() || !chr_dir.join("1w-2d.json").is_file() {
        eprintln!("skipping: generated CHR bins or sheets absent");
        return;
    }
    let donor_spr = std::fs::read(&spr_path).unwrap();
    let donor_bg = std::fs::read(&bg_path).unwrap();
    let sheets = zelda3_chr::read_sheets_dir(&chr_dir).unwrap();

    // The generated sheets are unedited, so they must reproduce the donor bins.
    let (spr, bg) = zelda3_chr::compile_chr_packs(&sheets, &donor_spr, &donor_bg).unwrap();
    assert_eq!(spr, donor_spr, "kSprGfx must compile byte-identically");
    assert_eq!(bg, donor_bg, "kBgGfx must compile byte-identically");

    // The generated lock (if we build one) must verify clean against those sheets.
    let lock = zelda3_chr::generate_sha_lock(&sheets);
    zelda3_chr::verify_against_lock(&sheets, &lock).unwrap();
}
