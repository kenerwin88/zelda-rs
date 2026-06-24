use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const FORMAT_BYTE_TILEMAP: &str = "zelda3_byte_tilemap_v1";
const FORMAT_BYTE_STREAM_TILEMAP: &str = "zelda3_byte_stream_tilemap_v1";
const FORMAT_SNES_PALETTE: &str = "zelda3_snes_palette_v1";

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
        repo_root.join("scripts").join("tilemap_json.py").display()
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

    let manifest = read_manifest(generated_dir);
    let manifest_assets = manifest
        .get("assets")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "{} assets must be an array",
                generated_dir.join("manifest.json").display()
            )
        });
    assert_eq!(
        manifest_assets.len(),
        names.len(),
        "manifest asset count must match asset key signature"
    );

    let mut assets = Vec::with_capacity(names.len());
    for (index, name) in names.iter().enumerate() {
        assets.push(read_asset(
            generated_dir,
            index,
            name,
            &manifest_assets[index],
        ));
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

fn read_manifest(generated_dir: &Path) -> serde_json::Value {
    let manifest_path = generated_dir.join("manifest.json");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", manifest_path.display()));
    serde_json::from_str(&manifest)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", manifest_path.display()))
}

fn read_asset(
    generated_dir: &Path,
    index: usize,
    name: &str,
    manifest_asset: &serde_json::Value,
) -> Vec<u8> {
    if manifest_asset
        .get("name")
        .and_then(serde_json::Value::as_str)
        != Some(name)
    {
        panic!("manifest asset {index:03} name does not match key signature {name}");
    }
    if let (Some(source_format), Some(source_file)) = (
        manifest_asset
            .get("source_format")
            .and_then(serde_json::Value::as_str),
        manifest_asset
            .get("source_file")
            .and_then(serde_json::Value::as_str),
    ) {
        return read_source_asset(generated_dir, source_format, source_file);
    }

    let bin_path = generated_dir
        .join("assets")
        .join(format!("{index:03}-{name}.bin"));
    println!("cargo:rerun-if-changed={}", bin_path.display());
    match fs::read(&bin_path) {
        Ok(asset) => asset,
        Err(bin_err) => {
            if let Some((source_format, source_file)) = known_source(name) {
                read_source_asset(generated_dir, source_format, source_file)
            } else {
                panic!(
                    "failed to read generated asset {}: {bin_err}",
                    bin_path.display()
                );
            }
        }
    }
}

fn known_source(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "kLightOverworldTilemap" => Some((
            FORMAT_BYTE_TILEMAP,
            "assets_src/tilemaps/light_overworld_tilemap.json",
        )),
        "kDarkOverworldTilemap" => Some((
            FORMAT_BYTE_TILEMAP,
            "assets_src/tilemaps/dark_overworld_tilemap.json",
        )),
        "kBgTilemap_0" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets_src/tilemaps/bg_tilemap_0.json",
        )),
        "kBgTilemap_1" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets_src/tilemaps/bg_tilemap_1.json",
        )),
        "kBgTilemap_2" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets_src/tilemaps/bg_tilemap_2.json",
        )),
        "kBgTilemap_3" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets_src/tilemaps/bg_tilemap_3.json",
        )),
        "kBgTilemap_4" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets_src/tilemaps/bg_tilemap_4.json",
        )),
        "kBgTilemap_5" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets_src/tilemaps/bg_tilemap_5.json",
        )),
        "kPalette_DungBgMain" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_dung_bg_main.json",
        )),
        "kPalette_MainSpr" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_main_spr.json",
        )),
        "kPalette_ArmorAndGloves" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_armor_and_gloves.json",
        )),
        "kPalette_Sword" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_sword.json",
        )),
        "kPalette_Shield" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_shield.json",
        )),
        "kPalette_SpriteAux3" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_sprite_aux3.json",
        )),
        "kPalette_MiscSprite_Indoors" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_misc_sprite_indoors.json",
        )),
        "kPalette_SpriteAux1" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_sprite_aux1.json",
        )),
        "kPalette_OverworldBgMain" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_overworld_bg_main.json",
        )),
        "kPalette_OverworldBgAux12" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_overworld_bg_aux12.json",
        )),
        "kPalette_OverworldBgAux3" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_overworld_bg_aux3.json",
        )),
        "kPalette_PalaceMapBg" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_palace_map_bg.json",
        )),
        "kPalette_PalaceMapSpr" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/palette_palace_map_spr.json",
        )),
        "kHudPalData" => Some((FORMAT_SNES_PALETTE, "assets_src/palettes/hud_pal_data.json")),
        "kOverworldMapPaletteData" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/overworld_map_palette_data.json",
        )),
        "kOverworldBgPalettes" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/overworld_bg_palettes.json",
        )),
        "kOverworldSpritePalettes" => Some((
            FORMAT_SNES_PALETTE,
            "assets_src/palettes/overworld_sprite_palettes.json",
        )),
        _ => None,
    }
}

fn read_source_asset(generated_dir: &Path, source_format: &str, source_file: &str) -> Vec<u8> {
    let source_path = generated_dir.join(source_file);
    println!("cargo:rerun-if-changed={}", source_path.display());
    match source_format {
        FORMAT_BYTE_TILEMAP => read_byte_tilemap_json(&source_path),
        FORMAT_BYTE_STREAM_TILEMAP => read_byte_stream_tilemap_json(&source_path),
        FORMAT_SNES_PALETTE => read_snes_palette_json(&source_path),
        _ => panic!(
            "unsupported readable asset format {source_format} for {}",
            source_path.display()
        ),
    }
}

fn read_byte_tilemap_json(source_path: &Path) -> Vec<u8> {
    let text = fs::read_to_string(source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let json: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", source_path.display()));
    if json.get("format").and_then(serde_json::Value::as_str) != Some(FORMAT_BYTE_TILEMAP) {
        panic!("{} is not a {FORMAT_BYTE_TILEMAP}", source_path.display());
    }
    let width = positive_usize(&json, "width", source_path);
    let height = positive_usize(&json, "height", source_path);
    let rows = json
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{} rows must be an array", source_path.display()));
    if rows.len() != height {
        panic!(
            "{} has {} rows, expected {height}",
            source_path.display(),
            rows.len()
        );
    }

    let mut data = Vec::with_capacity(width * height);
    for (y, row) in rows.iter().enumerate() {
        let row = row
            .as_array()
            .unwrap_or_else(|| panic!("{} row {y} must be an array", source_path.display()));
        if row.len() != width {
            panic!(
                "{} row {y} has {} entries, expected {width}",
                source_path.display(),
                row.len()
            );
        }
        for (x, value) in row.iter().enumerate() {
            let value = value.as_u64().unwrap_or_else(|| {
                panic!(
                    "{} row {y} column {x} must be 0..255",
                    source_path.display()
                )
            });
            let value = u8::try_from(value).unwrap_or_else(|_| {
                panic!(
                    "{} row {y} column {x} must be 0..255",
                    source_path.display()
                )
            });
            data.push(value);
        }
    }
    data
}

fn read_snes_palette_json(source_path: &Path) -> Vec<u8> {
    let text = fs::read_to_string(source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let json: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", source_path.display()));
    if json.get("format").and_then(serde_json::Value::as_str) != Some(FORMAT_SNES_PALETTE) {
        panic!("{} is not a {FORMAT_SNES_PALETTE}", source_path.display());
    }
    let colors = json
        .get("colors")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{} colors must be an array", source_path.display()));
    let mut data = Vec::with_capacity(colors.len() * 2);
    for (expected_index, color) in colors.iter().enumerate() {
        let color = color.as_object().unwrap_or_else(|| {
            panic!(
                "{} color {expected_index} must be an object",
                source_path.display()
            )
        });
        let actual_index = color
            .get("index")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| {
                panic!(
                    "{} color {expected_index} missing index",
                    source_path.display()
                )
            });
        assert_eq!(
            actual_index as usize,
            expected_index,
            "{} color index mismatch",
            source_path.display()
        );
        let word = color
            .get("snes_bgr15")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| {
                panic!(
                    "{} color {expected_index} missing snes_bgr15",
                    source_path.display()
                )
            });
        let word = parse_snes_palette_word(source_path, expected_index, word);
        data.extend_from_slice(&word.to_le_bytes());
    }
    data
}

fn parse_snes_palette_word(source_path: &Path, index: usize, word: &str) -> u16 {
    let word = u16::from_str_radix(word.strip_prefix("0x").unwrap_or(word), 16)
        .unwrap_or_else(|_| panic!("{} color {index} must be hex", source_path.display()));
    assert!(
        word <= 0x7fff,
        "{} color {index} must be 0x0000..0x7fff",
        source_path.display()
    );
    word
}

fn read_byte_stream_tilemap_json(source_path: &Path) -> Vec<u8> {
    let text = fs::read_to_string(source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let json: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", source_path.display()));
    if json.get("format").and_then(serde_json::Value::as_str) != Some(FORMAT_BYTE_STREAM_TILEMAP) {
        panic!(
            "{} is not a {FORMAT_BYTE_STREAM_TILEMAP}",
            source_path.display()
        );
    }
    let values = json
        .get("values")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{} values must be an array", source_path.display()));
    let mut data = Vec::new();
    for value in values {
        if let Some(chunk) = value.as_array() {
            for value in chunk {
                data.push(byte_stream_value(source_path, data.len(), value));
            }
        } else {
            data.push(byte_stream_value(source_path, data.len(), value));
        }
    }
    data
}

fn byte_stream_value(source_path: &Path, index: usize, value: &serde_json::Value) -> u8 {
    let value = value
        .as_u64()
        .unwrap_or_else(|| panic!("{} value {index} must be 0..255", source_path.display()));
    u8::try_from(value)
        .unwrap_or_else(|_| panic!("{} value {index} must be 0..255", source_path.display()))
}

fn positive_usize(json: &serde_json::Value, key: &str, source_path: &Path) -> usize {
    let value = json
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| panic!("{} {key} must be a positive integer", source_path.display()));
    let value = usize::try_from(value)
        .unwrap_or_else(|_| panic!("{} {key} is too large", source_path.display()));
    assert!(
        value > 0,
        "{} {key} must be a positive integer",
        source_path.display()
    );
    value
}
