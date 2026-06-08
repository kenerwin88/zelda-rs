use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.parent().unwrap();
    let generated_dir = repo_root.join("generated").join("zelda3_assets");
    let generated_dir = env::var_os("ZELDA3_ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or(generated_dir);
    let asset_dir = generated_dir.join("assets");

    println!("cargo:rerun-if-env-changed=ZELDA3_ASSETS_DIR");
    println!("cargo:rerun-if-env-changed=ZELDA3_ROM");
    println!("cargo:rerun-if-env-changed=ZELDA3_C_SOURCE");
    println!(
        "cargo:rerun-if-changed={}",
        generated_dir.join("manifest.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        generated_dir.join("asset_signature.bin").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        generated_dir.join("asset_key_signature.bin").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo_root
            .join("scripts")
            .join("extract_assets.py")
            .display()
    );

    if !asset_dir.is_dir() {
        let Some(rom) = find_rom(repo_root) else {
            panic!(
                "missing generated assets at {}\n\
                 Provide a USA Zelda 3 ROM and run:\n\
                 ZELDA3_ROM=/path/to/zelda3.sfc cargo build -p zelda3-bin\n\
                 or pre-generate assets with:\n\
                 python3 scripts/extract_assets.py --rom /path/to/zelda3.sfc",
                generated_dir.display()
            );
        };
        extract_assets(repo_root, &rom, &generated_dir);
    }

    let asset_pack = pack_assets(&generated_dir);
    println!(
        "cargo:rustc-env=ZELDA3_EMBEDDED_ASSETS={}",
        asset_pack.display()
    );
}

fn find_rom(repo_root: &Path) -> Option<PathBuf> {
    env::var_os("ZELDA3_ROM")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| {
            ["zelda3.sfc", "zelda3.smc"]
                .into_iter()
                .map(|name| repo_root.join(name))
                .find(|path| path.is_file())
        })
}

fn extract_assets(repo_root: &Path, rom: &Path, out_dir: &Path) {
    let status = Command::new("python3")
        .arg(repo_root.join("scripts").join("extract_assets.py"))
        .arg("--rom")
        .arg(rom)
        .arg("--out-dir")
        .arg(out_dir)
        .status()
        .expect("failed to run scripts/extract_assets.py");
    if !status.success() || !out_dir.join("assets").is_dir() {
        panic!(
            "asset extraction failed for {}; expected split assets in {}",
            rom.display(),
            out_dir.display()
        );
    }
}

fn pack_assets(generated_dir: &Path) -> PathBuf {
    let signature = fs::read(generated_dir.join("asset_signature.bin"))
        .expect("failed to read generated asset_signature.bin");
    assert_eq!(
        signature.len(),
        48,
        "generated asset_signature.bin must be 48 bytes"
    );
    let key_signature = fs::read(generated_dir.join("asset_key_signature.bin"))
        .expect("failed to read generated asset_key_signature.bin");
    let names = key_signature
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| String::from_utf8(name.to_vec()).expect("asset name is not utf8"))
        .collect::<Vec<_>>();

    let asset_dir = generated_dir.join("assets");
    let mut assets = Vec::with_capacity(names.len());
    for (index, name) in names.iter().enumerate() {
        let path = asset_dir.join(format!("{index:03}-{name}.bin"));
        println!("cargo:rerun-if-changed={}", path.display());
        assets.push(fs::read(&path).unwrap_or_else(|err| {
            panic!("failed to read generated asset {}: {err}", path.display())
        }));
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let asset_pack = out_dir.join("zelda3_assets.dat");
    let mut file_data = Vec::new();
    file_data.extend_from_slice(&signature);
    file_data.extend_from_slice(&[0u8; 32]);
    file_data.extend_from_slice(&(assets.len() as u32).to_le_bytes());
    file_data.extend_from_slice(&(key_signature.len() as u32).to_le_bytes());
    for asset in &assets {
        file_data.extend_from_slice(&(asset.len() as u32).to_le_bytes());
    }
    file_data.extend_from_slice(&key_signature);
    for asset in &assets {
        while file_data.len() & 3 != 0 {
            file_data.push(0);
        }
        file_data.extend_from_slice(asset);
    }
    fs::write(&asset_pack, file_data)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", asset_pack.display()));
    asset_pack
}
