use crate::{
    ensure, json_bytes, read_json,
    room::{self, Pack},
    tiled, write_new, Result,
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

const ROOM_HELP: &str = "dungeon-room export --base-pack PACK --room ID --out ROOM.json
dungeon-room validate --base-pack PACK --source ROOM.json
dungeon-room build --base-pack PACK --source ROOM.json --out MOD.dat
dungeon-room play --pack MOD.dat --binary BIN --rom ROM --save-dir DIR [--dry-run]
Room IDs accept decimal or 0x-prefixed hex. Outputs are create-only.";
const TILED_HELP: &str = "dungeon-tiled export --base-pack PACK (--room ID | --all) --out PATH
dungeon-tiled validate --base-pack PACK (--map ROOM.tmj | --world dungeons.world)
dungeon-tiled build --base-pack PACK (--map ROOM.tmj | --world dungeons.world) --out MOD.dat
dungeon-tiled preview --base-pack PACK --rom ROM --room ID [--map ROOM.tmj] --out NEW_DIR
Preview requires cargo build -p zelda3-map-tools --features preview.
Room IDs accept decimal or 0x-prefixed hex. Outputs are create-only.";

struct Args {
    command: String,
    options: BTreeMap<String, OsString>,
}
impl Args {
    fn parse(args: impl Iterator<Item = OsString>, tiled: bool) -> Result<Self> {
        let mut args = args.peekable();
        let command = args
            .next()
            .ok_or("missing command; use --help")?
            .into_string()
            .map_err(|_| "invalid command")?;
        let allowed: &[&str] = match (tiled, command.as_str()) {
            (false, "export") => &["--base-pack", "--room", "--out"],
            (false, "validate") => &["--base-pack", "--source"],
            (false, "build") => &["--base-pack", "--source", "--out"],
            (false, "play") => &["--pack", "--binary", "--rom", "--save-dir", "--dry-run"],
            (true, "export") => &["--base-pack", "--room", "--all", "--out"],
            (true, "validate") => &["--base-pack", "--map", "--world"],
            (true, "build") => &["--base-pack", "--map", "--world", "--out"],
            (true, "preview") => &["--base-pack", "--rom", "--room", "--map", "--out"],
            _ => return Err("unknown command; use --help".into()),
        };
        let mut options = BTreeMap::new();
        while let Some(key) = args.next() {
            let key = key.into_string().map_err(|_| "invalid option")?;
            ensure(
                allowed.contains(&key.as_str()),
                format!("unknown option {key}"),
            )?;
            let value = if key == "--all" || key == "--dry-run" {
                OsString::new()
            } else {
                let v = args
                    .next()
                    .ok_or_else(|| format!("missing value for {key}"))?;
                ensure(
                    !v.to_string_lossy().starts_with("--"),
                    format!("missing value for {key}"),
                )?;
                v
            };
            ensure(
                options.insert(key.clone(), value).is_none(),
                format!("duplicate option {key}"),
            )?;
        }
        Ok(Self { command, options })
    }
    fn path(&self, name: &str) -> Result<PathBuf> {
        self.options
            .get(name)
            .map(PathBuf::from)
            .ok_or_else(|| format!("required option {name}").into())
    }
    fn has(&self, name: &str) -> bool {
        self.options.contains_key(name)
    }
    fn room(&self) -> Result<usize> {
        let value = self
            .options
            .get("--room")
            .and_then(|v| v.to_str())
            .ok_or("required option --room")?;
        let room = if let Some(hex) = value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
        {
            usize::from_str_radix(hex, 16)?
        } else {
            value.parse()?
        };
        ensure(room < room::ROOM_COUNT, "room_id out of range")?;
        Ok(room)
    }
}

pub fn write_build(out: &Path, data: &[u8], receipt: &Value) -> Result<()> {
    let mut report = out.as_os_str().to_os_string();
    report.push(".json");
    let report = PathBuf::from(report);
    ensure(
        !out.exists() && !report.exists(),
        "output or receipt already exists; choose a new path",
    )?;
    let bytes = json_bytes(receipt)?;
    write_new(out, data)?;
    write_new(&report, &bytes)
}

fn run(args: Args, use_tiled: bool) -> Result<i32> {
    if args.command == "play" {
        let pack = args.path("--pack")?.canonicalize()?;
        Pack::parse(std::fs::read(&pack)?)?;
        let binary = args.path("--binary")?.canonicalize()?;
        let rom = args.path("--rom")?.canonicalize()?;
        let save_dir = args.path("--save-dir")?;
        let save_dir = if save_dir.is_absolute() {
            save_dir
        } else {
            std::env::current_dir()?.join(save_dir)
        };
        if args.has("--dry-run") {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"argv":[binary,rom], "env":{
                "ZELDA3_ASSET_PACK":pack, "ZELDA3_SAVE_DIR":save_dir}}))?
            );
            return Ok(0);
        }
        return Ok(Command::new(binary)
            .arg(rom)
            .env("ZELDA3_ASSET_PACK", pack)
            .env("ZELDA3_SAVE_DIR", save_dir)
            .status()?
            .code()
            .unwrap_or(1));
    }
    let pack = Pack::parse(std::fs::read(args.path("--base-pack")?)?)?;
    if args.command == "preview" {
        #[cfg(feature = "preview")]
        {
            let source = if args.has("--map") {
                Some(read_json(&args.path("--map")?)?)
            } else {
                None
            };
            crate::preview::export_preview(
                &pack,
                &std::fs::read(args.path("--rom")?)?,
                args.room()?,
                source.as_ref(),
                &args.path("--out")?,
            )?;
            println!("exported preview to {}", args.path("--out")?.display());
            return Ok(0);
        }
        #[cfg(not(feature = "preview"))]
        return Err("rebuild with cargo build -p zelda3-map-tools --features preview".into());
    }
    if args.command == "export" {
        let out = args.path("--out")?;
        if use_tiled {
            ensure(
                args.has("--all") != args.has("--room"),
                "choose exactly one of --all or --room",
            )?;
            if args.has("--all") {
                tiled::export_world(&pack, &out)?;
            } else {
                write_new(&out, &json_bytes(&tiled::export_map(&pack, args.room()?)?)?)?;
            }
        } else {
            write_new(&out, &json_bytes(&room::export_room(&pack, args.room()?)?)?)?;
        }
        println!("exported to {}", out.display());
    } else {
        // Validate required output before doing compilation work.
        let out = if args.command == "build" {
            Some(args.path("--out")?)
        } else {
            None
        };
        let (data, receipt) = if use_tiled {
            ensure(
                args.has("--map") != args.has("--world"),
                "choose exactly one of --map or --world",
            )?;
            if args.has("--world") {
                tiled::compile_world(&pack, &args.path("--world")?)?
            } else {
                tiled::compile_map(&pack, &read_json(&args.path("--map")?)?)?
            }
        } else {
            room::compile_room(&pack, &read_json(&args.path("--source")?)?)?
        };
        if let Some(out) = out {
            write_build(&out, &data, &receipt)?;
        }
        println!("{}", serde_json::to_string_pretty(&receipt)?);
    }
    Ok(0)
}

pub fn main(use_tiled: bool) {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", if use_tiled { TILED_HELP } else { ROOM_HELP });
        return;
    }
    match Args::parse(args.into_iter(), use_tiled).and_then(|args| run(args, use_tiled)) {
        Ok(0) => (),
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    }
}
