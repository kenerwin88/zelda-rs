use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const FORMAT_BYTE_TILEMAP: &str = "zelda3_byte_tilemap_v1";
const FORMAT_BYTE_STREAM_TILEMAP: &str = "zelda3_byte_stream_tilemap_v1";
const FORMAT_SNES_PALETTE: &str = "zelda3_snes_palette_v1";
const FORMAT_DUNGEON_ENTRANCES: &str = "zelda3_dungeon_entrances_v1";
const FORMAT_STARTING_POINTS: &str = "zelda3_starting_points_v1";
const FORMAT_OVERWORLD_EXITS: &str = "zelda3_overworld_exits_v1";
const FORMAT_SPECIAL_EXITS: &str = "zelda3_special_exits_v1";
const DIALOGUE_SOURCE_SIDECAR_ASSET_NAME: &str = "kDialogueSourceSemantic";
const DIALOGUE_SOURCE_SIDECAR_MAGIC: &[u8; 16] = b"Z3DLGSRCv1\0\0\0\0\0\0";

struct NavigationField {
    asset: &'static str,
    field: &'static str,
    value_type: &'static str,
    values_per_record: usize,
}

const ENTRANCE_FIELDS: &[NavigationField] = &[
    NavigationField {
        asset: "kEntranceData_rooms",
        field: "room",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_relativeCoords",
        field: "relative_coords",
        value_type: "u8",
        values_per_record: 8,
    },
    NavigationField {
        asset: "kEntranceData_scrollX",
        field: "scroll_x",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_scrollY",
        field: "scroll_y",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_playerX",
        field: "player_x",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_playerY",
        field: "player_y",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_cameraX",
        field: "camera_x",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_cameraY",
        field: "camera_y",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_blockset",
        field: "blockset",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_floor",
        field: "floor",
        value_type: "i8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_palace",
        field: "palace",
        value_type: "i8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_doorwayOrientation",
        field: "doorway_orientation",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_startingBg",
        field: "starting_bg",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_quadrant1",
        field: "quadrant1",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_quadrant2",
        field: "quadrant2",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_doorSettings",
        field: "door_settings",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kEntranceData_musicTrack",
        field: "music_track",
        value_type: "u8",
        values_per_record: 1,
    },
];

const STARTING_POINT_FIELDS: &[NavigationField] = &[
    NavigationField {
        asset: "kStartingPoint_rooms",
        field: "room",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_relativeCoords",
        field: "relative_coords",
        value_type: "u8",
        values_per_record: 8,
    },
    NavigationField {
        asset: "kStartingPoint_scrollX",
        field: "scroll_x",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_scrollY",
        field: "scroll_y",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_playerX",
        field: "player_x",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_playerY",
        field: "player_y",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_cameraX",
        field: "camera_x",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_cameraY",
        field: "camera_y",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_blockset",
        field: "blockset",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_floor",
        field: "floor",
        value_type: "i8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_palace",
        field: "palace",
        value_type: "i8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_doorwayOrientation",
        field: "doorway_orientation",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_startingBg",
        field: "starting_bg",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_quadrant1",
        field: "quadrant1",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_quadrant2",
        field: "quadrant2",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_doorSettings",
        field: "door_settings",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_entrance",
        field: "entrance",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kStartingPoint_musicTrack",
        field: "music_track",
        value_type: "u8",
        values_per_record: 1,
    },
];

const EXIT_FIELDS: &[NavigationField] = &[
    NavigationField {
        asset: "kExitData_ScreenIndex",
        field: "screen_index",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitDataRooms",
        field: "room",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_Map16LoadSrcOff",
        field: "map16_load_src_off",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_ScrollX",
        field: "scroll_x",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_ScrollY",
        field: "scroll_y",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_XCoord",
        field: "x_coord",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_YCoord",
        field: "y_coord",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_CameraXScroll",
        field: "camera_x_scroll",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_CameraYScroll",
        field: "camera_y_scroll",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_NormalDoor",
        field: "normal_door",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_FancyDoor",
        field: "fancy_door",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_Unk1",
        field: "unk1",
        value_type: "i8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kExitData_Unk3",
        field: "unk3",
        value_type: "i8",
        values_per_record: 1,
    },
];

const SPECIAL_EXIT_FIELDS: &[NavigationField] = &[
    NavigationField {
        asset: "kSpExit_Top",
        field: "top",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_Bottom",
        field: "bottom",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_Left",
        field: "left",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_Right",
        field: "right",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_Tab4",
        field: "tab4",
        value_type: "i16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_Tab5",
        field: "tab5",
        value_type: "i16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_Tab6",
        field: "tab6",
        value_type: "i16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_Tab7",
        field: "tab7",
        value_type: "i16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_LeftEdgeOfMap",
        field: "left_edge_of_map",
        value_type: "u16",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_Dir",
        field: "dir",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_SprGfx",
        field: "spr_gfx",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_AuxGfx",
        field: "aux_gfx",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_PalBg",
        field: "pal_bg",
        value_type: "u8",
        values_per_record: 1,
    },
    NavigationField {
        asset: "kSpExit_PalSpr",
        field: "pal_spr",
        value_type: "u8",
        values_per_record: 1,
    },
];

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
    println!("cargo:rerun-if-env-changed=ZELDA3_DIALOGUE_MESSAGES");
    println!("cargo:rerun-if-env-changed=ZELDA3_CHR_SHEETS");
    println!("cargo:rerun-if-env-changed=ZELDA3_CHR_SHA_LOCK");
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
            .join("navigation_json.py")
            .display()
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
    let mut key_signature = fs::read(generated_dir.join("asset_key_signature.bin"))
        .expect("failed to read generated asset_key_signature.bin");
    let mut names = key_signature
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
    if let Some(sidecar) = read_dialogue_source_sidecar(generated_dir) {
        key_signature.extend_from_slice(DIALOGUE_SOURCE_SIDECAR_ASSET_NAME.as_bytes());
        key_signature.push(0);
        names.push(DIALOGUE_SOURCE_SIDECAR_ASSET_NAME.to_string());
        assets.push(sidecar);
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

/// Repo root (parent of this crate's manifest dir). The authored dialogue source
/// lives in the tracked repo tree, independent of the (possibly redirected)
/// generated asset dir.
fn dialogue_repo_root() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Load the authored dialogue messages, in precedence order:
///   1. `ZELDA3_DIALOGUE_MESSAGES` env override (alt-dialogue / tests, never mutates
///      the tracked authority),
///   2. the tracked `assets/dialogue/messages.toml` (the sole authority; it is
///      committed, so every clone has it — there is no generated fallback).
/// `kDialogue` is always source-built (never the stale `.bin`), matching the runtime's
/// required `kDialogueSourceSemantic` sidecar.
fn read_dialogue_messages_document(_generated_dir: &Path) -> zelda3_dialogue::DialogueMessagesDocument {
    if let Some(override_path) = env::var_os("ZELDA3_DIALOGUE_MESSAGES") {
        let override_path = PathBuf::from(override_path);
        println!("cargo:rerun-if-changed={}", override_path.display());
        let text = fs::read_to_string(&override_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", override_path.display()));
        return zelda3_dialogue::parse_messages_document(&text)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", override_path.display()));
    }
    let messages_path = dialogue_repo_root().join("assets/dialogue/messages.toml");
    if messages_path.is_file() {
        println!("cargo:rerun-if-changed={}", messages_path.display());
        let text = fs::read_to_string(&messages_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", messages_path.display()));
        return zelda3_dialogue::parse_messages_document(&text)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", messages_path.display()));
    }
    panic!(
        "no dialogue source: the tracked authority {} is missing \
         (set ZELDA3_DIALOGUE_MESSAGES to override, or restore the file from git)",
        messages_path.display()
    );
}

fn read_dialogue_asset(generated_dir: &Path) -> Vec<u8> {
    let doc = read_dialogue_messages_document(generated_dir);
    zelda3_dialogue::compile_messages_document(&doc)
        .unwrap_or_else(|err| panic!("failed to compile dialogue messages: {err}"))
}

/// Return the source-authoritative `kSprGfx`/`kBgGfx` payload. The tracked
/// editable CHR sheets (when present) are the authority: they compile to the
/// packed containers, and an unedited sheet tree reproduces the donor `.bin`
/// bytes exactly. With no sheets, the donor payload is embedded unchanged.
fn read_chr_gfx_asset(generated_dir: &Path, name: &str) -> Vec<u8> {
    let (spr, bg) = chr_gfx_payloads(generated_dir);
    match name {
        "kSprGfx" => spr.clone(),
        "kBgGfx" => bg.clone(),
        other => panic!("read_chr_gfx_asset called for non-CHR asset {other}"),
    }
}

/// Compile the CHR packs once per build and memoize, so `kSprGfx` and `kBgGfx`
/// don't each pay the full compile.
fn chr_gfx_payloads(generated_dir: &Path) -> &'static (Vec<u8>, Vec<u8>) {
    static CELL: OnceLock<(Vec<u8>, Vec<u8>)> = OnceLock::new();
    CELL.get_or_init(|| compute_chr_gfx_payloads(generated_dir))
}

/// Sheet-directory precedence: `ZELDA3_CHR_SHEETS` env override, then the tracked
/// `<repo>/assets/chr` directory. `None` means "no editable sheets" -> donor
/// pass-through.
fn resolve_chr_sheets_dir() -> Option<PathBuf> {
    if let Some(env_dir) = env::var_os("ZELDA3_CHR_SHEETS") {
        return Some(PathBuf::from(env_dir));
    }
    let tracked = dialogue_repo_root().join("assets/chr");
    tracked.is_dir().then_some(tracked)
}

/// Lock-file path: `ZELDA3_CHR_SHA_LOCK` env override, else `<sheets-dir>/chr.sha1`.
fn chr_lock_path(sheets_dir: &Path) -> PathBuf {
    env::var_os("ZELDA3_CHR_SHA_LOCK")
        .map(PathBuf::from)
        .unwrap_or_else(|| sheets_dir.join("chr.sha1"))
}

fn compute_chr_gfx_payloads(generated_dir: &Path) -> (Vec<u8>, Vec<u8>) {
    let assets_dir = generated_dir.join("assets");
    let spr_path = assets_dir.join("064-kSprGfx.bin");
    let bg_path = assets_dir.join("065-kBgGfx.bin");
    println!("cargo:rerun-if-changed={}", spr_path.display());
    println!("cargo:rerun-if-changed={}", bg_path.display());
    let donor_spr = fs::read(&spr_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", spr_path.display()));
    let donor_bg = fs::read(&bg_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", bg_path.display()));

    let Some(sheets_dir) = resolve_chr_sheets_dir() else {
        // No editable CHR sheets: embed the donor payloads unchanged.
        return (donor_spr, donor_bg);
    };

    for name in zelda3_chr::SHEET_NAMES {
        println!(
            "cargo:rerun-if-changed={}",
            sheets_dir.join(format!("{name}.png")).display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            sheets_dir.join(format!("{name}.json")).display()
        );
    }
    let lock_path = chr_lock_path(&sheets_dir);
    println!("cargo:rerun-if-changed={}", lock_path.display());

    let sheets = zelda3_chr::read_sheets_dir(&sheets_dir).unwrap_or_else(|err| {
        panic!(
            "failed to read CHR sheets from {}: {err}",
            sheets_dir.display()
        )
    });
    let lock_toml = fs::read_to_string(&lock_path).unwrap_or_else(|err| {
        panic!(
            "missing CHR parity lock {} ({err}); run `zelda3 --bless-chr` to generate it",
            lock_path.display()
        )
    });
    zelda3_chr::verify_against_lock(&sheets, &lock_toml)
        .unwrap_or_else(|err| panic!("CHR sheet parity check failed: {err}"));
    zelda3_chr::compile_chr_packs(&sheets, &donor_spr, &donor_bg)
        .unwrap_or_else(|err| panic!("failed to compile CHR packs: {err}"))
}

fn read_dialogue_source_sidecar(generated_dir: &Path) -> Option<Vec<u8>> {
    let doc = read_dialogue_messages_document(generated_dir);
    let table = zelda3_dialogue::compile_messages_ir_table(&doc)
        .unwrap_or_else(|err| panic!("failed to compile dialogue IR table: {err}"));
    let mut sidecar = DIALOGUE_SOURCE_SIDECAR_MAGIC.to_vec();
    sidecar.extend(
        bincode::serialize(&table)
            .unwrap_or_else(|err| panic!("failed to serialize dialogue IR: {err}")),
    );
    Some(sidecar)
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
    if name == "kDialogue" {
        return read_dialogue_asset(generated_dir);
    }
    if name == "kSprGfx" || name == "kBgGfx" {
        return read_chr_gfx_asset(generated_dir, name);
    }
    if let (Some(source_format), Some(source_file)) = (
        manifest_asset
            .get("source_format")
            .and_then(serde_json::Value::as_str),
        manifest_asset
            .get("source_file")
            .and_then(serde_json::Value::as_str),
    ) {
        return read_source_asset(generated_dir, source_format, source_file, name);
    }
    if let Some((source_format, source_file)) = known_source(name) {
        if resolve_source_path(generated_dir, source_file).is_file() {
            return read_source_asset(generated_dir, source_format, source_file, name);
        }
    }

    let bin_path = generated_dir
        .join("assets")
        .join(format!("{index:03}-{name}.bin"));
    println!("cargo:rerun-if-changed={}", bin_path.display());
    match fs::read(&bin_path) {
        Ok(asset) => asset,
        Err(bin_err) => {
            if let Some((source_format, source_file)) = known_source(name) {
                read_source_asset(generated_dir, source_format, source_file, name)
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
    if ENTRANCE_FIELDS.iter().any(|field| field.asset == name) {
        return Some((
            FORMAT_DUNGEON_ENTRANCES,
            "assets/navigation/dungeon_entrances.json",
        ));
    }
    if STARTING_POINT_FIELDS
        .iter()
        .any(|field| field.asset == name)
    {
        return Some((
            FORMAT_STARTING_POINTS,
            "assets/navigation/starting_points.json",
        ));
    }
    if EXIT_FIELDS.iter().any(|field| field.asset == name) {
        return Some((
            FORMAT_OVERWORLD_EXITS,
            "assets/navigation/overworld_exits.json",
        ));
    }
    if SPECIAL_EXIT_FIELDS.iter().any(|field| field.asset == name) {
        return Some((
            FORMAT_SPECIAL_EXITS,
            "assets/navigation/special_exits.json",
        ));
    }
    match name {
        "kLightOverworldTilemap" => Some((
            FORMAT_BYTE_TILEMAP,
            "assets/tilemaps/light_overworld_tilemap.json",
        )),
        "kDarkOverworldTilemap" => Some((
            FORMAT_BYTE_TILEMAP,
            "assets/tilemaps/dark_overworld_tilemap.json",
        )),
        "kBgTilemap_0" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets/tilemaps/bg_tilemap_0.json",
        )),
        "kBgTilemap_1" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets/tilemaps/bg_tilemap_1.json",
        )),
        "kBgTilemap_2" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets/tilemaps/bg_tilemap_2.json",
        )),
        "kBgTilemap_3" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets/tilemaps/bg_tilemap_3.json",
        )),
        "kBgTilemap_4" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets/tilemaps/bg_tilemap_4.json",
        )),
        "kBgTilemap_5" => Some((
            FORMAT_BYTE_STREAM_TILEMAP,
            "assets/tilemaps/bg_tilemap_5.json",
        )),
        "kPalette_DungBgMain" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_dung_bg_main.json",
        )),
        "kPalette_MainSpr" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_main_spr.json",
        )),
        "kPalette_ArmorAndGloves" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_armor_and_gloves.json",
        )),
        "kPalette_Sword" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_sword.json",
        )),
        "kPalette_Shield" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_shield.json",
        )),
        "kPalette_SpriteAux3" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_sprite_aux3.json",
        )),
        "kPalette_MiscSprite_Indoors" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_misc_sprite_indoors.json",
        )),
        "kPalette_SpriteAux1" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_sprite_aux1.json",
        )),
        "kPalette_OverworldBgMain" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_overworld_bg_main.json",
        )),
        "kPalette_OverworldBgAux12" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_overworld_bg_aux12.json",
        )),
        "kPalette_OverworldBgAux3" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_overworld_bg_aux3.json",
        )),
        "kPalette_PalaceMapBg" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_palace_map_bg.json",
        )),
        "kPalette_PalaceMapSpr" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/palette_palace_map_spr.json",
        )),
        "kHudPalData" => Some((FORMAT_SNES_PALETTE, "assets/palettes/hud_pal_data.json")),
        "kOverworldMapPaletteData" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/overworld_map_palette_data.json",
        )),
        "kOverworldBgPalettes" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/overworld_bg_palettes.json",
        )),
        "kOverworldSpritePalettes" => Some((
            FORMAT_SNES_PALETTE,
            "assets/palettes/overworld_sprite_palettes.json",
        )),
        _ => None,
    }
}

/// Resolve a manifest/known `source_file` to an on-disk path. Tracked readable
/// authorities live in the repo tree (`assets/...`) and are resolved against the
/// repo root, independent of the (possibly redirected) generated asset dir;
/// everything else resolves against `generated_dir`.
fn resolve_source_path(generated_dir: &Path, source_file: &str) -> PathBuf {
    if source_file.starts_with("assets/") {
        dialogue_repo_root().join(source_file)
    } else {
        generated_dir.join(source_file)
    }
}

fn read_source_asset(
    generated_dir: &Path,
    source_format: &str,
    source_file: &str,
    asset_name: &str,
) -> Vec<u8> {
    let source_path = resolve_source_path(generated_dir, source_file);
    println!("cargo:rerun-if-changed={}", source_path.display());
    match source_format {
        FORMAT_BYTE_TILEMAP => read_byte_tilemap_json(&source_path),
        FORMAT_BYTE_STREAM_TILEMAP => read_byte_stream_tilemap_json(&source_path),
        FORMAT_SNES_PALETTE => read_snes_palette_json(&source_path),
        zelda3_dialogue::FORMAT_DIALOGUE_SOURCE => read_dialogue_source_json(&source_path),
        FORMAT_DUNGEON_ENTRANCES
        | FORMAT_STARTING_POINTS
        | FORMAT_OVERWORLD_EXITS
        | FORMAT_SPECIAL_EXITS => read_navigation_json(&source_path, source_format, asset_name),
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

fn read_navigation_json(source_path: &Path, expected_format: &str, asset_name: &str) -> Vec<u8> {
    let text = fs::read_to_string(source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let json: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", source_path.display()));
    if json.get("format").and_then(serde_json::Value::as_str) != Some(expected_format) {
        panic!("{} is not a {expected_format}", source_path.display());
    }
    let field = navigation_field(expected_format, asset_name).unwrap_or_else(|| {
        panic!(
            "{} does not contain legacy asset {asset_name}",
            source_path.display()
        )
    });
    let records = json
        .get("records")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{} records must be an array", source_path.display()));
    let mut data = Vec::new();
    for (expected_index, record) in records.iter().enumerate() {
        let record = record.as_object().unwrap_or_else(|| {
            panic!(
                "{} record {expected_index} must be an object",
                source_path.display()
            )
        });
        let actual_index = record
            .get("index")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| {
                panic!(
                    "{} record {expected_index} missing index",
                    source_path.display()
                )
            });
        assert_eq!(
            actual_index as usize,
            expected_index,
            "{} record index mismatch",
            source_path.display()
        );
        let value = record.get(field.field).unwrap_or_else(|| {
            panic!(
                "{} record {expected_index} missing {}",
                source_path.display(),
                field.field
            )
        });
        if field.values_per_record == 1 {
            push_navigation_value(&mut data, source_path, field, expected_index, 0, value);
        } else {
            let values = value.as_array().unwrap_or_else(|| {
                panic!(
                    "{} record {expected_index} {} must be an array",
                    source_path.display(),
                    field.field
                )
            });
            if values.len() != field.values_per_record {
                panic!(
                    "{} record {expected_index} {} has {} values, expected {}",
                    source_path.display(),
                    field.field,
                    values.len(),
                    field.values_per_record
                );
            }
            for (value_index, value) in values.iter().enumerate() {
                push_navigation_value(
                    &mut data,
                    source_path,
                    field,
                    expected_index,
                    value_index,
                    value,
                );
            }
        }
    }
    data
}

fn navigation_field(source_format: &str, asset_name: &str) -> Option<&'static NavigationField> {
    let fields = match source_format {
        FORMAT_DUNGEON_ENTRANCES => ENTRANCE_FIELDS,
        FORMAT_STARTING_POINTS => STARTING_POINT_FIELDS,
        FORMAT_OVERWORLD_EXITS => EXIT_FIELDS,
        FORMAT_SPECIAL_EXITS => SPECIAL_EXIT_FIELDS,
        _ => return None,
    };
    fields.iter().find(|field| field.asset == asset_name)
}

fn push_navigation_value(
    data: &mut Vec<u8>,
    source_path: &Path,
    field: &NavigationField,
    record_index: usize,
    value_index: usize,
    value: &serde_json::Value,
) {
    let value = value.as_i64().unwrap_or_else(|| {
        panic!(
            "{} record {record_index} {}[{value_index}] must be an integer",
            source_path.display(),
            field.field
        )
    });
    match field.value_type {
        "u8" => {
            let value = u8::try_from(value).unwrap_or_else(|_| {
                panic!(
                    "{} record {record_index} {}[{value_index}] must be 0..255",
                    source_path.display(),
                    field.field
                )
            });
            data.push(value);
        }
        "i8" => {
            let value = i8::try_from(value).unwrap_or_else(|_| {
                panic!(
                    "{} record {record_index} {}[{value_index}] must be -128..127",
                    source_path.display(),
                    field.field
                )
            });
            data.extend_from_slice(&value.to_le_bytes());
        }
        "u16" => {
            let value = u16::try_from(value).unwrap_or_else(|_| {
                panic!(
                    "{} record {record_index} {}[{value_index}] must be 0..65535",
                    source_path.display(),
                    field.field
                )
            });
            data.extend_from_slice(&value.to_le_bytes());
        }
        "i16" => {
            let value = i16::try_from(value).unwrap_or_else(|_| {
                panic!(
                    "{} record {record_index} {}[{value_index}] must be -32768..32767",
                    source_path.display(),
                    field.field
                )
            });
            data.extend_from_slice(&value.to_le_bytes());
        }
        other => panic!("unsupported navigation value type {other}"),
    }
}

fn read_dialogue_source_json(source_path: &Path) -> Vec<u8> {
    let text = fs::read_to_string(source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let source: zelda3_dialogue::DialogueSourceDocument = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", source_path.display()));
    zelda3_dialogue::compile_dialogue_source_document(&source)
        .unwrap_or_else(|err| panic!("failed to compile {}: {err}", source_path.display()))
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
