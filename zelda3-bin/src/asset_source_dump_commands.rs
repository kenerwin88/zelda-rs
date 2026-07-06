use std::collections::HashMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::process;

use crate::developer_room_commands::load_developer_destination;
use crate::image_output::write_assets_index_png;
use crate::input_script::InputScript;
use crate::{load_play_or_checkpoint, load_play_state, load_translated_replay_state};
use renderer::modern_extract::{decode_snes_2bpp_tile_indices, decode_snes_4bpp_tile_indices};
use renderer::modern_source_atlas::modern_source_key;
use serde::{Deserialize, Serialize};
use zelda3::{
    chr_content_hash32, ZeldaState, CHR_KIND_BG, CHR_KIND_BG3, CHR_KIND_BG_STREAM, CHR_KIND_LINK,
    CHR_KIND_LINK_CONTENT, CHR_KIND_NONE, CHR_KIND_SPRITE,
};

const PLAYER_IS_INDOORS: usize = 0x001b;
const CHR_KIND_BG3_CONTENT: u8 = 7;

#[derive(Debug)]
struct ScriptedDumpRoute {
    name: String,
    frames: u32,
    input_script: String,
    checkpoint_path: Option<String>,
}

#[derive(Clone, Debug)]
struct ScriptedDumpCheckpoint {
    name: String,
    frame: u32,
    checkpoint_path: String,
    input_script: String,
}

#[derive(Debug)]
struct DumpAssetsBySourceOptions {
    max_frames: u32,
    merge_existing: bool,
    write_palette_usage: bool,
    only_window: Option<String>,
    window_frames: Option<u32>,
    developer_destination: Option<String>,
    skip_startup: bool,
    skip_replay: bool,
}

#[derive(Debug, Serialize)]
struct AssetsBySourceManifest {
    format: &'static str,
    cell_count: u32,
    cells: Vec<AssetsBySourceCell>,
}

#[derive(Debug, Serialize)]
struct AssetsBySourceCell {
    id: u32,
    key: u64,
    kind: u8,
    pack: u16,
    tile_off: u16,
}

#[derive(Debug, Deserialize)]
struct ExistingAssetsBySourceManifest {
    cells: Vec<ExistingAssetsBySourceCell>,
}

#[derive(Debug, Deserialize)]
struct ExistingAssetsBySourceCell {
    id: u32,
    key: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PaletteUsageKey {
    source_kind: &'static str,
    asset: &'static str,
    pack: u16,
    tile: u16,
    bpp: u8,
    preview_palette: &'static str,
    preview_palette_row: u8,
}

#[derive(Debug, Serialize)]
struct PaletteUsageManifest {
    format: &'static str,
    entries: Vec<PaletteUsageEntry>,
}

#[derive(Debug, Serialize)]
struct PaletteUsageEntry {
    source_kind: &'static str,
    asset: &'static str,
    pack: u16,
    tile: u16,
    bpp: u8,
    preview_palette: &'static str,
    preview_palette_row: u8,
    evidence_count: u32,
}

fn palette_usage_key_from_chr_source(
    src: zelda3::LogicalChrSrc,
    preview_palette: &'static str,
    preview_palette_row: u8,
) -> Option<PaletteUsageKey> {
    let (source_kind, asset) = match src.kind {
        CHR_KIND_BG | CHR_KIND_BG_STREAM => ("bg", "kBgGfx"),
        CHR_KIND_SPRITE => ("sprite", "kSprGfx"),
        _ => return None,
    };
    Some(PaletteUsageKey {
        source_kind,
        asset,
        pack: src.pack,
        tile: src.tile_off,
        bpp: 3,
        preview_palette,
        preview_palette_row,
    })
}

fn record_palette_usage_count(
    counts: &mut HashMap<PaletteUsageKey, u32>,
    src: zelda3::LogicalChrSrc,
    preview_palette: &'static str,
    preview_palette_row: u8,
) {
    if let Some(key) = palette_usage_key_from_chr_source(src, preview_palette, preview_palette_row)
    {
        *counts.entry(key).or_insert(0) += 1;
    }
}

fn content_hash_source_key(vram: &[u16], slot: usize) -> Option<u64> {
    content_hash_source_key_for_kind(vram, slot, CHR_KIND_BG_STREAM)
}

fn content_hash_source_key_for_kind(vram: &[u16], slot: usize, kind: u8) -> Option<u64> {
    let base = slot.checked_mul(16)?;
    let end = base.checked_add(16)?;
    let words = vram.get(base..end)?;
    let h = chr_content_hash32(words);
    Some(modern_source_key(
        kind,
        (h >> 16) as u16,
        (h & 0xffff) as u16,
    ))
}

fn parse_dump_assets_by_source_options(args: &[String]) -> DumpAssetsBySourceOptions {
    let mut max_frames = 60_000;
    let mut max_frames_set = false;
    let mut merge_existing = false;
    let mut write_palette_usage = false;
    let mut only_window = None;
    let mut window_frames = None;
    let mut developer_destination = None;
    let mut skip_startup = false;
    let mut skip_replay = false;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--merge-existing" => {
                merge_existing = true;
                i += 1;
            }
            "--write-palette-usage" => {
                write_palette_usage = true;
                i += 1;
            }
            "--only-window" => {
                let Some(name) = args.get(i + 1) else {
                    eprintln!("--only-window requires an oracle window name");
                    process::exit(2);
                };
                only_window = Some(name.clone());
                i += 2;
            }
            "--window-frames" => {
                let Some(frames) = args.get(i + 1) else {
                    eprintln!("--window-frames requires a frame count");
                    process::exit(2);
                };
                window_frames = Some(frames.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --window-frames value: {frames}");
                    process::exit(2);
                }));
                i += 2;
            }
            "--developer-destination" => {
                let Some(id) = args.get(i + 1) else {
                    eprintln!("--developer-destination requires a destination id");
                    process::exit(2);
                };
                developer_destination = Some(id.clone());
                i += 2;
            }
            "--skip-startup" => {
                skip_startup = true;
                i += 1;
            }
            "--skip-replay" => {
                skip_replay = true;
                i += 1;
            }
            value if !value.starts_with("--") && !max_frames_set => {
                max_frames = value.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid frame count: {value}");
                    process::exit(2);
                });
                max_frames_set = true;
                i += 1;
            }
            other => {
                eprintln!(
                    "usage: zelda3 --dump-assets-by-source [frames] [--merge-existing] [--write-palette-usage] [--only-window <name>] [--window-frames <n>] [--developer-destination <id>] [--skip-startup] [--skip-replay]"
                );
                eprintln!("unknown --dump-assets-by-source argument: {other}");
                process::exit(2);
            }
        }
    }
    if developer_destination.is_some() && !max_frames_set {
        max_frames = 1;
    }
    DumpAssetsBySourceOptions {
        max_frames,
        merge_existing,
        write_palette_usage,
        only_window,
        window_frames,
        developer_destination,
        skip_startup,
        skip_replay,
    }
}

fn load_existing_assets_by_source(
    json_path: &Path,
    png_path: &Path,
) -> Result<Vec<(u64, [u8; 64])>, String> {
    let manifest_bytes = fs::read(json_path)
        .map_err(|e| format!("failed to read existing {}: {e}", json_path.display()))?;
    let manifest: ExistingAssetsBySourceManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse existing {}: {e}", json_path.display()))?;

    let file = fs::File::open(png_path)
        .map_err(|e| format!("failed to open existing {}: {e}", png_path.display()))?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("failed to read PNG header {}: {e}", png_path.display()))?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("failed to decode {}: {e}", png_path.display()))?;
    if info.bit_depth != png::BitDepth::Eight || info.color_type != png::ColorType::Indexed {
        return Err(format!(
            "{}: expected 8-bit indexed PNG, got {:?}/{:?}",
            png_path.display(),
            info.color_type,
            info.bit_depth
        ));
    }
    let width = info.width as usize;
    let height = info.height as usize;
    if width % 8 != 0 || height % 8 != 0 {
        return Err(format!(
            "{}: PNG size {}x{} is not aligned to 8x8 cells",
            png_path.display(),
            info.width,
            info.height
        ));
    }
    let cols = width / 8;
    let data = &buf[..info.buffer_size()];
    let mut cells = Vec::with_capacity(manifest.cells.len());
    for cell in manifest.cells {
        let id = cell.id as usize;
        let cx = (id % cols) * 8;
        let cy = (id / cols) * 8;
        if cy + 8 > height || cx + 8 > width {
            return Err(format!(
                "{}: manifest cell {} is outside PNG grid {}x{}",
                json_path.display(),
                id,
                info.width,
                info.height
            ));
        }
        let mut pattern = [0u8; 64];
        for row in 0..8usize {
            let src = (cy + row) * width + cx;
            pattern[row * 8..row * 8 + 8].copy_from_slice(&data[src..src + 8]);
        }
        cells.push((cell.key, pattern));
    }
    Ok(cells)
}

fn index_pattern_hash32(indices: &[u8; 64]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &b in indices {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

fn bg3_content_source_key(indices: &[u8; 64]) -> u64 {
    let h = index_pattern_hash32(indices);
    modern_source_key(CHR_KIND_BG3_CONTENT, (h >> 16) as u16, (h & 0xffff) as u16)
}

fn palette_usage_entries_from_counts(
    counts: &HashMap<PaletteUsageKey, u32>,
) -> Vec<PaletteUsageEntry> {
    let mut best_by_tile: HashMap<
        (&'static str, &'static str, u16, u16, u8),
        (PaletteUsageKey, u32),
    > = HashMap::new();
    for (&key, &count) in counts {
        let tile_key = (key.source_kind, key.asset, key.pack, key.tile, key.bpp);
        match best_by_tile.get(&tile_key) {
            Some((best_key, best_count))
                if count < *best_count
                    || (count == *best_count
                        && (key.preview_palette, key.preview_palette_row)
                            >= (best_key.preview_palette, best_key.preview_palette_row)) => {}
            _ => {
                best_by_tile.insert(tile_key, (key, count));
            }
        }
    }

    let mut entries: Vec<_> = best_by_tile
        .into_values()
        .map(|(key, evidence_count)| PaletteUsageEntry {
            source_kind: key.source_kind,
            asset: key.asset,
            pack: key.pack,
            tile: key.tile,
            bpp: key.bpp,
            preview_palette: key.preview_palette,
            preview_palette_row: key.preview_palette_row,
            evidence_count,
        })
        .collect();
    entries.sort_by_key(|entry| {
        (
            entry.source_kind,
            entry.asset,
            entry.pack,
            entry.tile,
            entry.bpp,
            entry.preview_palette,
            entry.preview_palette_row,
        )
    });
    entries
}

fn scripted_dump_checkpoints(repo_root: &Path) -> Vec<ScriptedDumpCheckpoint> {
    let path = repo_root.join("docs/porting/oracle_checkpoints.tsv");
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut checkpoints = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        if line_no == 0 || line.trim().is_empty() {
            continue;
        }
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 4 {
            continue;
        }
        let frame = cols[1].parse::<u32>().unwrap_or_else(|_| {
            eprintln!(
                "{}:{}: invalid checkpoint frame: {}",
                path.display(),
                line_no + 1,
                cols[1]
            );
            process::exit(2);
        });
        checkpoints.push(ScriptedDumpCheckpoint {
            name: cols[0].to_owned(),
            frame,
            checkpoint_path: cols[2].to_owned(),
            input_script: cols[3].to_owned(),
        });
    }
    checkpoints
}

fn scripted_dump_routes(repo_root: &Path, frame_cap: u32) -> Vec<ScriptedDumpRoute> {
    let path = repo_root.join("docs/porting/oracle_windows.tsv");
    let Ok(text) = fs::read_to_string(&path) else {
        eprintln!(
            "[warn] scripted source dump routes missing: {}",
            path.display()
        );
        return Vec::new();
    };
    let checkpoints = scripted_dump_checkpoints(repo_root);
    let mut routes = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        if line_no == 0 || line.trim().is_empty() {
            continue;
        }
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 4 || cols[1] != "pass" || cols[3].is_empty() {
            continue;
        }
        let input_script = cols[3].to_owned();
        let script_path = repo_root.join(&input_script);
        if script_path.with_extension("sram").is_file() {
            continue;
        }
        let frames = cols[2].parse::<u32>().unwrap_or_else(|_| {
            eprintln!(
                "{}:{}: invalid frame count for scripted dump route: {}",
                path.display(),
                line_no + 1,
                cols[2]
            );
            process::exit(2);
        });
        let checkpoint = checkpoints
            .iter()
            .filter(|checkpoint| {
                checkpoint.name == cols[0]
                    && checkpoint.input_script == input_script
                    && checkpoint.frame < frames
                    && repo_root.join(&checkpoint.checkpoint_path).is_file()
            })
            .max_by_key(|checkpoint| checkpoint.frame);
        let route_frames = checkpoint
            .map(|checkpoint| frames.saturating_sub(checkpoint.frame))
            .unwrap_or(frames)
            .min(frame_cap);
        routes.push(ScriptedDumpRoute {
            name: cols[0].to_owned(),
            frames: route_frames,
            input_script,
            checkpoint_path: checkpoint.map(|checkpoint| checkpoint.checkpoint_path.clone()),
        });
    }
    routes
}

/// Walk the combined-route replay and dump an asset library keyed by the LOGICAL
/// CHR SOURCE (Milestone 2 of the animation-modeled asset renderer), NOT by VRAM
/// appearance.
///
/// At each frame the CHR tile slots actually USED that frame are enumerated by
/// walking the three BG tilemaps + OAM and mapping every referenced tile back to
/// its VRAM CHR slot (`tile_word_base / 16`). For each used slot the M1
/// bookkeeping table (`game.vram_chr_source()`) names the logical source that
/// filled it (`kind/pack/tile_off`); BG1/BG2 slots with no recorded source are
/// keyed by their frame-end content hash. Each unique logical source key
/// (`(kind<<24)|(pack<<8)|(tile_off&0xff)`) becomes one cell, whose 8x8 4bpp
/// palette-index pattern is decoded offline from live VRAM at that slot.
///
/// Emits `developer_tilesets/assets_by_source.{bin,json}`.
pub(crate) fn run_dump_assets_by_source(args: &[String]) {
    let options = parse_dump_assets_by_source_options(args);
    let rekey_content_hash = |vram: &[u16], slot: usize, src: zelda3::LogicalChrSrc| -> u64 {
        if src.kind == CHR_KIND_BG_STREAM {
            if let Some(key) = content_hash_source_key(vram, slot) {
                return key;
            }
        }
        modern_source_key(src.kind, src.pack, src.tile_off)
    };

    const SPRITE_SIZES: [[i32; 2]; 8] = [
        [8, 16],
        [8, 32],
        [8, 64],
        [16, 32],
        [16, 64],
        [32, 64],
        [16, 32],
        [16, 32],
    ];

    const OUT_PNG: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/assets_by_source.png"
    );
    const OUT_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/assets_by_source.json"
    );
    const PALETTE_USAGE_OUT_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../generated/zelda3_assets/atlas/palette_usage.json"
    );
    const REPO_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let replay = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );

    let max_frames = options.max_frames;
    let startup_frames = std::env::var("ZELDA3_DUMP_STARTUP_FRAMES")
        .ok()
        .map(|s| {
            s.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("invalid ZELDA3_DUMP_STARTUP_FRAMES: {s}");
                process::exit(2);
            })
        })
        .unwrap_or(30_000);

    let watch_key: Option<u64> = std::env::var("ZELDA3_DUMP_WATCH_KEY")
        .ok()
        .and_then(|s| u64::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok());
    #[allow(clippy::type_complexity)]
    let mut watch_patterns: std::collections::BTreeMap<
        u32,
        (u32, u32, u32, usize, u8, u8, u8, u16),
    > = std::collections::BTreeMap::new();

    let mut cell_by_key: HashMap<u64, usize> = HashMap::new();
    let mut cells: Vec<[u8; 64]> = Vec::new();
    let mut collisions: usize = 0;
    let mut collision_patterns: std::collections::HashSet<(u64, u32)> =
        std::collections::HashSet::new();
    let mut palette_usage_counts: HashMap<PaletteUsageKey, u32> = HashMap::new();
    let mut ambiguous_keys: std::collections::HashSet<u64> = std::collections::HashSet::new();

    if options.merge_existing {
        match load_existing_assets_by_source(Path::new(OUT_JSON), Path::new(OUT_PNG)) {
            Ok(existing_cells) => {
                for (key, pattern) in existing_cells {
                    if cell_by_key.contains_key(&key) {
                        continue;
                    }
                    let id = cells.len();
                    cell_by_key.insert(key, id);
                    cells.push(pattern);
                }
                eprintln!(
                    "[dump] seeded existing assets-by-source cells={}",
                    cells.len()
                );
            }
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    }

    let mut record_keyed =
        |key: u64, pattern: [u8; 64], dbg_slot: usize| match cell_by_key.get(&key) {
            Some(&id) => {
                if cells[id] != pattern {
                    let pattern_hash = index_pattern_hash32(&pattern);
                    if collision_patterns.insert((key, pattern_hash)) {
                        collisions += 1;
                        ambiguous_keys.insert(key);
                        if collisions <= 10 {
                            eprintln!(
                                "[warn] key 0x{key:016x} decoded to a different pattern at \
                                 slot {dbg_slot:#x}; keeping first"
                            );
                        }
                    }
                }
            }
            None => {
                let id = cells.len();
                cell_by_key.insert(key, id);
                cells.push(pattern);
            }
        };

    let mut collect_used_slots = |game: &ZeldaState, cur_frame: u32| {
        let ppu = &game.ppu;

        for layer_index in 0..3usize {
            let bg = &ppu.bg_layer[layer_index];
            let base = bg.tilemap_adr as usize;
            let chr_base = bg.tile_adr as usize;
            if base == 0 && chr_base == 0 {
                continue;
            }
            let is_bg3 = layer_index == 2;
            let wide = bg.tilemap_wider;
            let tall = bg.tilemap_higher;
            let cols = if wide { 64usize } else { 32 };
            let rows = if tall { 64usize } else { 32 };
            for ty in 0..rows {
                for tx in 0..cols {
                    let q = (if wide && tx >= 32 { 1 } else { 0 })
                        + (if tall && ty >= 32 {
                            if wide {
                                2
                            } else {
                                1
                            }
                        } else {
                            0
                        });
                    let within = (ty % 32) * 32 + (tx % 32);
                    let addr = base + q * 0x400 + within;
                    let entry_word = ppu.vram.get(addr).copied().unwrap_or(0);
                    let tile_number = usize::from(entry_word & 0x03ff);
                    if is_bg3 {
                        if entry_word == 0 {
                            continue;
                        }
                        let palette = ((entry_word >> 10) & 7) as u16;
                        let pack = (tile_number as u16) | (palette << 10);
                        let key = modern_source_key(CHR_KIND_BG3, pack, 0);
                        let raw =
                            decode_snes_2bpp_tile_indices(&ppu.vram, chr_base, entry_word & 0x03ff);
                        let mut baked = [0u8; 64];
                        for (b, &p) in baked.iter_mut().zip(raw.iter()) {
                            *b = if p == 0 { 0 } else { (palette as u8) * 4 + p };
                        }
                        record_keyed(key, baked, chr_base + tile_number * 8);
                        record_keyed(
                            bg3_content_source_key(&baked),
                            baked,
                            chr_base + tile_number * 8,
                        );
                        continue;
                    }
                    let slot = (chr_base + tile_number * 16) / 16;
                    let mut src = game.vram_chr_source().get(slot);
                    if src.kind == CHR_KIND_NONE {
                        let Some(key) = content_hash_source_key(&ppu.vram, slot) else {
                            continue;
                        };
                        src = zelda3::LogicalChrSrc {
                            kind: CHR_KIND_BG_STREAM,
                            pack: ((key >> 16) & 0xffff) as u16,
                            tile_off: (key & 0xffff) as u16,
                        };
                    }
                    let palette_row = ((entry_word >> 10) & 7) as u8;
                    let scene = renderer::ModernAssetFrameScene::from_player_indoors_flag(
                        game.ram.get(PLAYER_IS_INDOORS).copied().unwrap_or(0),
                    );
                    let preview_src = game.vram_chr_preview_source().get(slot);
                    let usage_src =
                        if src.kind == CHR_KIND_BG_STREAM && preview_src.kind == CHR_KIND_BG {
                            preview_src
                        } else {
                            src
                        };
                    record_palette_usage_count(
                        &mut palette_usage_counts,
                        usage_src,
                        scene.bg_palette_name(),
                        palette_row,
                    );
                    let key = rekey_content_hash(&ppu.vram, slot, src);
                    let pattern = decode_snes_4bpp_tile_indices(&ppu.vram, slot * 16, 0);
                    if watch_key == Some(key) {
                        let mut h: u32 = 0x811c_9dc5;
                        for &b in pattern.iter() {
                            h ^= b as u32;
                            h = h.wrapping_mul(0x0100_0193);
                        }
                        let module = game.ram.get(0x10).copied().unwrap_or(0);
                        let submodule = game.ram.get(0x11).copied().unwrap_or(0);
                        let indoor = game.ram.get(0x1b).copied().unwrap_or(0);
                        let anim_pack = game.animated_tile_pack;
                        let e = watch_patterns.entry(h).or_insert((
                            cur_frame, cur_frame, 0, slot, module, submodule, indoor, anim_pack,
                        ));
                        e.1 = cur_frame;
                        e.2 += 1;
                    }
                    record_keyed(key, pattern, slot);
                    if src.kind != CHR_KIND_BG_STREAM {
                        // The renderer rekeys non-injective BG1/BG2 source tags to the
                        // frame-end 4bpp content hash and resolves that exact key from the
                        // source atlas when present. Emit the same key here so generic BG
                        // draws can use PNG/source art instead of staying live-indexed.
                        if let Some(hash_key) = content_hash_source_key(&ppu.vram, slot) {
                            record_keyed(hash_key, pattern, slot);
                        }
                    }
                }
            }
        }

        for sprite_num in 0..128usize {
            let idx = sprite_num * 2;
            let oam0 = ppu.oam.get(idx).copied().unwrap_or(0);
            let y_byte = ((oam0 >> 8) & 0xff) as i32;
            if y_byte == 0xf0 {
                continue;
            }
            let hi_word = ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
            let hi_bits = (hi_word >> (idx % 16)) as i32;
            let size = SPRITE_SIZES[(ppu.obj_size & 7) as usize][((hi_bits >> 1) & 1) as usize];
            let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
            if object_x > 256 && object_x + size - 1 < 512 {
                continue;
            }
            let mut x = object_x;
            if x >= 256 {
                x -= 512;
            }
            if x <= -size {
                continue;
            }
            let oam1 = ppu.oam.get(idx + 1).copied().unwrap_or(0);
            let obj_addr = if oam1 & 0x0100 != 0 {
                ppu.obj_tile_adr2
            } else {
                ppu.obj_tile_adr1
            };
            let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
            let tile_col_base = (oam1 & 0x0f) as i32;
            let tiles_per_side = size / 8;
            for sty in 0..tiles_per_side {
                for stx in 0..tiles_per_side {
                    let used_tile =
                        (((tile_row_base + sty) << 4) | ((tile_col_base + stx) & 0x0f)) as u16;
                    let tile_word_base =
                        obj_addr.wrapping_add(used_tile.wrapping_mul(16)) as usize & 0x7fff;
                    let slot = tile_word_base / 16;
                    let src = game.vram_chr_source().get(slot);
                    if src.kind == CHR_KIND_NONE {
                        continue;
                    }
                    let preview_src = game.vram_chr_preview_source().get(slot);
                    let usage_src =
                        if src.kind == CHR_KIND_BG_STREAM && preview_src.kind == CHR_KIND_SPRITE {
                            preview_src
                        } else {
                            src
                        };
                    let palette_row = ((oam1 >> 9) & 7) as u8;
                    record_palette_usage_count(
                        &mut palette_usage_counts,
                        usage_src,
                        renderer::ModernAssetFrameScene::SPRITE_PALETTE_NAME,
                        palette_row,
                    );
                    let key = if src.kind == CHR_KIND_LINK {
                        content_hash_source_key_for_kind(&ppu.vram, slot, CHR_KIND_LINK_CONTENT)
                            .unwrap_or_else(|| modern_source_key(src.kind, src.pack, src.tile_off))
                    } else {
                        rekey_content_hash(&ppu.vram, slot, src)
                    };
                    let pattern = decode_snes_4bpp_tile_indices(&ppu.vram, slot * 16, 0);
                    record_keyed(key, pattern, slot);
                }
            }
        }
    };

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let walk = panic::catch_unwind(AssertUnwindSafe(|| {
        let repo_root = Path::new(REPO_ROOT);
        if let Some(destination_id) = options.developer_destination.as_deref() {
            let (mut game, start_frame) = load_developer_destination(destination_id)
                .unwrap_or_else(|e| {
                    eprintln!("failed to load developer destination {destination_id}: {e}");
                    process::exit(1);
                });
            let mut frames = 0u32;
            while frames < max_frames {
                let step = panic::catch_unwind(AssertUnwindSafe(|| {
                    game.zelda_run_frame(0);
                }));
                if step.is_err() {
                    eprintln!(
                        "[warn] developer destination {destination_id} frame {frames} panicked; stopping walk early"
                    );
                    break;
                }
                frames = frames.wrapping_add(1);
                collect_used_slots(&game, start_frame.wrapping_add(frames));
            }
            return (frames, frames);
        }

        let mut scripted_routes = scripted_dump_routes(repo_root, max_frames);
        if options.window_frames.is_some() && options.only_window.is_none() {
            eprintln!("--window-frames requires --only-window");
            process::exit(2);
        }
        if let Some(only_window) = options.only_window.as_deref() {
            scripted_routes.retain(|route| route.name == only_window);
            if scripted_routes.is_empty() {
                eprintln!("--only-window did not match a passing oracle window: {only_window}");
                process::exit(2);
            }
            if let Some(window_frames) = options.window_frames {
                for route in &mut scripted_routes {
                    route.frames = window_frames;
                }
            }
        }
        let mut startup_game = load_play_state(rom);
        let mut startup_walked = 0u32;
        if !options.skip_startup {
            while startup_walked < startup_frames {
                let step = panic::catch_unwind(AssertUnwindSafe(|| {
                    startup_game.zelda_run_frame(0);
                }));
                if step.is_err() {
                    eprintln!(
                        "[warn] startup frame {startup_walked} panicked; stopping startup walk early"
                    );
                    break;
                }
                startup_walked = startup_walked.wrapping_add(1);
                collect_used_slots(&startup_game, startup_walked);
            }
        }

        let mut scripted_walked = 0u32;
        for route in scripted_routes {
            let script_path = repo_root.join(&route.input_script);
            let script = match InputScript::from_path(&script_path) {
                Ok(script) => script,
                Err(e) => {
                    eprintln!(
                        "[warn] failed to parse scripted source dump route {} ({}): {e}",
                        route.name,
                        script_path.display()
                    );
                    continue;
                }
            };
            let (mut game, start_frame) = match route.checkpoint_path.as_deref() {
                Some(checkpoint_path) => {
                    load_play_or_checkpoint(rom, Some(&repo_root.join(checkpoint_path)))
                }
                None => (load_play_state(rom), 0),
            };
            let mut frames = 0u32;
            while frames < route.frames {
                let absolute_frame = start_frame.wrapping_add(frames);
                let input = script.input_for_frame(absolute_frame);
                let step = panic::catch_unwind(AssertUnwindSafe(|| {
                    game.zelda_run_frame(input as i32);
                }));
                if step.is_err() {
                    eprintln!(
                        "[warn] scripted route {} frame {frames} panicked; stopping route walk early",
                        route.name
                    );
                    break;
                }
                frames = frames.wrapping_add(1);
                scripted_walked = scripted_walked.wrapping_add(1);
                collect_used_slots(&game, absolute_frame.wrapping_add(1));
            }
        }

        let mut frames = 0u32;
        if !options.skip_replay {
            let mut game = load_translated_replay_state(rom);
            if let Err(e) = game.replay_save_file(Path::new(replay)) {
                eprintln!("failed to load replay save {replay}: {e}");
                process::exit(1);
            }
            frames = game.state_recorder.replay_frame_counter;
            while frames < max_frames && game.state_recorder.replay_mode {
                let step = panic::catch_unwind(AssertUnwindSafe(|| {
                    game.zelda_run_frame_with_replay_input_override(0, None);
                }));
                if step.is_err() {
                    eprintln!("[warn] replay frame {frames} panicked; stopping walk early");
                    break;
                }
                frames = frames.wrapping_add(1);
                collect_used_slots(&game, frames);
            }
        }
        (
            startup_walked
                .wrapping_add(scripted_walked)
                .wrapping_add(frames),
            scripted_walked,
        )
    }));

    panic::set_hook(original_hook);

    let (frames_walked, scripted_frames_walked) = match walk {
        Ok(f) => f,
        Err(_) => {
            eprintln!("assets-by-source walk aborted by panic");
            process::exit(1);
        }
    };

    if let Some(wk) = watch_key {
        eprintln!(
            "[WATCH] key 0x{wk:016x}: {} distinct pattern(s) over the route{}",
            watch_patterns.len(),
            if watch_patterns.len() > 1 {
                "  => AMBIGUOUS KEY (non-injective / collision)"
            } else {
                "  => injective (mismatch is elsewhere: stale tag or gap)"
            }
        );
        for (h, (first, last, count, slot, module, submodule, indoor, anim_pack)) in &watch_patterns
        {
            eprintln!(
                "[WATCH]   pattern 0x{h:08x}: frames {first}..{last} (x{count}), first slot 0x{slot:03x} \
                 | @first: module=0x{module:02x} submodule=0x{submodule:02x} indoor={indoor} anim_pack=0x{anim_pack:04x}"
            );
        }
    }

    let mut bin = Vec::with_capacity(cells.len() * 64);
    let mut manifest_cells = Vec::with_capacity(cells.len());
    let mut count_bg = 0usize;
    let mut count_sprite = 0usize;
    let mut count_link = 0usize;
    let mut count_bg3 = 0usize;
    let mut dropped_bg3 = 0usize;
    let mut key_by_id = vec![0u64; cells.len()];
    for (&key, &id) in &cell_by_key {
        key_by_id[id] = key;
    }
    for (id, pattern) in cells.iter().enumerate() {
        let key = key_by_id[id];
        let (kind, pack, tile_off) = if key < (1u64 << 32) {
            (
                CHR_KIND_LINK,
                ((key >> 14) & 0x3ff) as u16,
                (key & 0x3fff) as u16,
            )
        } else {
            (
                (key >> 32) as u8,
                ((key >> 16) & 0xffff) as u16,
                (key & 0xffff) as u16,
            )
        };
        if kind == CHR_KIND_BG3 && ambiguous_keys.contains(&key) {
            dropped_bg3 += 1;
            continue;
        }
        let new_id = manifest_cells.len() as u32;
        bin.extend_from_slice(&pattern[..]);
        match kind {
            CHR_KIND_BG => count_bg += 1,
            CHR_KIND_SPRITE => count_sprite += 1,
            CHR_KIND_LINK => count_link += 1,
            CHR_KIND_LINK_CONTENT => count_link += 1,
            CHR_KIND_BG3 => count_bg3 += 1,
            _ => {}
        }
        manifest_cells.push(AssetsBySourceCell {
            id: new_id,
            key,
            kind,
            pack,
            tile_off,
        });
    }
    let cell_count = manifest_cells.len();

    let no_write = std::env::var("ZELDA3_DUMP_NO_WRITE").is_ok();

    if !no_write {
        if let Err(e) = write_assets_index_png(OUT_PNG, &bin, cell_count) {
            eprintln!("failed to write assets index PNG {OUT_PNG}: {e}");
            process::exit(1);
        }
    }

    let manifest = AssetsBySourceManifest {
        format: "zelda3_assets_by_source_v2_png",
        cell_count: cell_count as u32,
        cells: manifest_cells,
    };
    let json = match serde_json::to_vec_pretty(&manifest) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("failed to serialize assets manifest: {e}");
            process::exit(1);
        }
    };
    if !no_write {
        if let Err(e) = fs::write(OUT_JSON, &json) {
            eprintln!("failed to write assets manifest {OUT_JSON}: {e}");
            process::exit(1);
        }
        if !options.merge_existing || options.write_palette_usage {
            let usage_manifest = PaletteUsageManifest {
                format: "zelda3_palette_usage_v1",
                entries: palette_usage_entries_from_counts(&palette_usage_counts),
            };
            let usage_json = match serde_json::to_vec_pretty(&usage_manifest) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("failed to serialize palette usage manifest: {e}");
                    process::exit(1);
                }
            };
            if let Some(parent) = Path::new(PALETTE_USAGE_OUT_JSON).parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!(
                        "failed to create palette usage dir {}: {e}",
                        parent.display()
                    );
                    process::exit(1);
                }
            }
            if let Err(e) = fs::write(PALETTE_USAGE_OUT_JSON, &usage_json) {
                eprintln!("failed to write palette usage manifest {PALETTE_USAGE_OUT_JSON}: {e}");
                process::exit(1);
            }
        }
    }
    if no_write {
        eprintln!("[dump] ZELDA3_DUMP_NO_WRITE set - atlas files NOT written (diagnostic run)");
    }

    if collisions > 0 {
        eprintln!("[warn] {collisions} source->pattern collisions (kept first per key)");
    }
    println!(
        "dumped assets-by-source cells={cell_count} kind_counts(bg/sprite/link/bg3)={count_bg}/{count_sprite}/{count_link}/{count_bg3} dropped_bg3_ambiguous={dropped_bg3} palette_usage_entries={} frames={frames_walked} startup_frames={startup_frames} scripted_frames={scripted_frames_walked} replay_max_frames={max_frames}",
        palette_usage_entries_from_counts(&palette_usage_counts).len()
    );
}

#[cfg(test)]
mod palette_usage_tests {
    use super::*;

    #[test]
    fn developer_destination_dump_defaults_to_one_frame() {
        let args = vec![
            "--developer-destination".to_string(),
            "route-late-checkpoint".to_string(),
            "--merge-existing".to_string(),
        ];

        let options = parse_dump_assets_by_source_options(&args);

        assert_eq!(options.max_frames, 1);
        assert_eq!(
            options.developer_destination.as_deref(),
            Some("route-late-checkpoint")
        );
        assert!(options.merge_existing);
    }

    #[test]
    fn developer_destination_dump_accepts_explicit_frame_count() {
        let args = vec![
            "4".to_string(),
            "--developer-destination".to_string(),
            "preset-dev-sandbox".to_string(),
        ];

        let options = parse_dump_assets_by_source_options(&args);

        assert_eq!(options.max_frames, 4);
        assert_eq!(
            options.developer_destination.as_deref(),
            Some("preset-dev-sandbox")
        );
    }

    #[test]
    fn only_window_dump_accepts_explicit_local_frame_count() {
        let args = vec![
            "--only-window".to_string(),
            "coverage-branch-496311-pikit-shield-drop".to_string(),
            "--window-frames".to_string(),
            "10000".to_string(),
            "--merge-existing".to_string(),
        ];

        let options = parse_dump_assets_by_source_options(&args);

        assert_eq!(
            options.only_window.as_deref(),
            Some("coverage-branch-496311-pikit-shield-drop")
        );
        assert_eq!(options.window_frames, Some(10000));
        assert!(options.merge_existing);
    }

    #[test]
    fn raw_chr_sources_map_to_base_tile_usage_keys() {
        let bg = zelda3::LogicalChrSrc {
            kind: CHR_KIND_BG,
            pack: 5,
            tile_off: 17,
        };
        let sprite = zelda3::LogicalChrSrc {
            kind: CHR_KIND_SPRITE,
            pack: 12,
            tile_off: 3,
        };
        let streamed = zelda3::LogicalChrSrc {
            kind: CHR_KIND_BG_STREAM,
            pack: 0x1234,
            tile_off: 0x5678,
        };

        self::assert_palette_usage_key(
            palette_usage_key_from_chr_source(bg, "palette_dung_bg_main", 2),
            "bg",
            "kBgGfx",
            5,
            17,
            "palette_dung_bg_main",
            2,
        );
        self::assert_palette_usage_key(
            palette_usage_key_from_chr_source(sprite, "palette_main_spr", 6),
            "sprite",
            "kSprGfx",
            12,
            3,
            "palette_main_spr",
            6,
        );
        self::assert_palette_usage_key(
            palette_usage_key_from_chr_source(streamed, "palette_dung_bg_main", 1),
            "bg",
            "kBgGfx",
            0x1234,
            0x5678,
            "palette_dung_bg_main",
            1,
        );
    }

    #[test]
    fn content_hash_source_key_encodes_frame_end_tile_bytes() {
        let mut vram = vec![0u16; 0x40];
        for (i, word) in vram[0x20..0x30].iter_mut().enumerate() {
            *word = 0x1000u16.wrapping_add(i as u16);
        }
        let h = chr_content_hash32(&vram[0x20..0x30]);

        assert_eq!(
            content_hash_source_key(&vram, 2),
            Some(modern_source_key(
                CHR_KIND_BG_STREAM,
                (h >> 16) as u16,
                (h & 0xffff) as u16
            ))
        );
        assert_eq!(content_hash_source_key(&vram, 4), None);
    }

    #[test]
    fn content_hash_source_key_can_encode_link_content_kind() {
        let mut vram = vec![0u16; 0x40];
        for (i, word) in vram[0x20..0x30].iter_mut().enumerate() {
            *word = 0x2000u16.wrapping_add(i as u16);
        }
        let h = chr_content_hash32(&vram[0x20..0x30]);

        assert_eq!(
            content_hash_source_key_for_kind(&vram, 2, CHR_KIND_LINK_CONTENT),
            Some(modern_source_key(
                CHR_KIND_LINK_CONTENT,
                (h >> 16) as u16,
                (h & 0xffff) as u16
            ))
        );
    }

    #[test]
    fn bg3_content_source_key_uses_distinct_content_kind() {
        let mut indices = [0u8; 64];
        indices[0] = 14;
        indices[63] = 3;
        let h = index_pattern_hash32(&indices);

        assert_eq!(
            bg3_content_source_key(&indices),
            modern_source_key(CHR_KIND_BG3_CONTENT, (h >> 16) as u16, (h & 0xffff) as u16)
        );
    }

    fn assert_palette_usage_key(
        got: Option<PaletteUsageKey>,
        source_kind: &str,
        asset: &str,
        pack: u16,
        tile: u16,
        palette: &str,
        palette_row: u8,
    ) {
        let got = got.expect("expected usage key");
        assert_eq!(got.source_kind, source_kind);
        assert_eq!(got.asset, asset);
        assert_eq!(got.pack, pack);
        assert_eq!(got.tile, tile);
        assert_eq!(got.bpp, 3);
        assert_eq!(got.preview_palette, palette);
        assert_eq!(got.preview_palette_row, palette_row);
    }
}
