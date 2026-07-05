use std::collections::HashMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::process;

use crate::image_output::write_rgba_frame_png;
use crate::load_translated_replay_state;
use renderer::modern_extract::{decode_snes_2bpp_tile_indices, decode_snes_4bpp_tile_indices};
use renderer::modern_palette::snes_cgram_to_rgba;
use serde::Serialize;
use zelda3::ZeldaState;

const TRACE_MAIN_MODULE_INDEX: usize = 0x10;

#[derive(Debug, Serialize)]
struct SpriteSheetPngManifest {
    format: &'static str,
    tile_width_px: u8,
    tile_height_px: u8,
    cell_count: u32,
    columns: u32,
    cells: Vec<SpriteSheetPngCell>,
}

#[derive(Debug, Serialize)]
struct SpriteSheetPngCell {
    id: u32,
    atlas_x_px: u32,
    atlas_y_px: u32,
}

/// Walk the combined-route replay and extract a REAL colored sprite sheet: every
/// visible OAM 8x8 tile is decoded from live VRAM, colored with the live sprite
/// palette (CGRAM), and deduped by its 8x8 RGBA appearance so each unique colored
/// pose = one cell (captures all of Link's animation poses + every sprite seen).
///
/// Emits `developer_tilesets/sprite_sheet.{png,json}`.
pub(crate) fn run_dump_sprite_sheet_png(args: &[String]) {
    /// Hardware OBJ sizes by `obj_size` (small, large) - mirrors SPRITE_SIZES.
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
        "/developer_tilesets/sprite_sheet.png"
    );
    const OUT_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/sprite_sheet.json"
    );
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let replay = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );

    let max_frames = args
        .first()
        .map(|s| {
            s.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("invalid frame count: {s}");
                process::exit(2);
            })
        })
        .unwrap_or(60_000);

    let mut cells: Vec<Vec<u8>> = Vec::new();
    let mut index_by_rgba: HashMap<Vec<u8>, usize> = HashMap::new();

    let mut collect_visible_sprite_tiles = |game: &ZeldaState| {
        let ppu = &game.ppu;
        for sprite_num in 0..128usize {
            let idx = sprite_num * 2;
            let oam0 = ppu.oam.get(idx).copied().unwrap_or(0);

            // Off-screen sentinel: hidden sprites are parked at y == 0xf0.
            let y_byte = ((oam0 >> 8) & 0xff) as i32;
            if y_byte == 0xf0 {
                continue;
            }
            let top_y = ((y_byte + 1) & 0xff) - 1;

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
            let hflip = oam1 & 0x4000 != 0;
            let vflip = oam1 & 0x8000 != 0;
            let palette = ((oam1 & 0x0e00) >> 9) as u32;
            let obj_addr = if oam1 & 0x0100 != 0 {
                ppu.obj_tile_adr2
            } else {
                ppu.obj_tile_adr1
            };
            let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
            let tile_col_base = (oam1 & 0x0f) as i32;
            let _ = top_y; // screen position is irrelevant for an appearance-deduped sheet

            let tiles_per_side = size / 8;
            for sty in 0..tiles_per_side {
                for stx in 0..tiles_per_side {
                    let src_col_tile = if hflip { tiles_per_side - 1 - stx } else { stx };
                    let src_row_tile = if vflip { tiles_per_side - 1 - sty } else { sty };
                    let used_tile = (((tile_row_base + src_row_tile) << 4)
                        | ((tile_col_base + src_col_tile) & 0x0f))
                        as u16;
                    let tile_word_base =
                        obj_addr.wrapping_add(used_tile.wrapping_mul(16)) as usize & 0x7fff;
                    // Apply 8x8-level flips while decoding the 4bpp pattern.
                    let entry = (u16::from(hflip) * 0x4000) | (u16::from(vflip) * 0x8000);
                    let indices = decode_snes_4bpp_tile_indices(&ppu.vram, tile_word_base, entry);
                    if indices == [0u8; 64] {
                        continue;
                    }
                    let mut rgba = vec![0u8; 64 * 4];
                    for (px, &index) in indices.iter().enumerate() {
                        if index == 0 {
                            continue; // transparent
                        }
                        let cgram_idx = 0x80 + palette as usize * 16 + index as usize;
                        let color =
                            snes_cgram_to_rgba(ppu.cgram.get(cgram_idx).copied().unwrap_or(0));
                        rgba[px * 4..px * 4 + 4].copy_from_slice(&color);
                    }
                    if !index_by_rgba.contains_key(&rgba) {
                        let id = cells.len();
                        index_by_rgba.insert(rgba.clone(), id);
                        cells.push(rgba);
                    }
                }
            }
        }
    };

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let walk = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut game = load_translated_replay_state(rom);
        if let Err(e) = game.replay_save_file(Path::new(replay)) {
            eprintln!("failed to load replay save {replay}: {e}");
            process::exit(1);
        }
        let mut frames = game.state_recorder.replay_frame_counter;
        while frames < max_frames && game.state_recorder.replay_mode {
            let step = panic::catch_unwind(AssertUnwindSafe(|| {
                game.zelda_run_frame_with_replay_input_override(0, None);
            }));
            if step.is_err() {
                eprintln!("[warn] replay frame {frames} panicked; stopping walk early");
                break;
            }
            frames = frames.wrapping_add(1);
            collect_visible_sprite_tiles(&game);
        }
        frames
    }));

    panic::set_hook(original_hook);

    let frames_walked = match walk {
        Ok(f) => f,
        Err(_) => {
            eprintln!("sprite-sheet walk aborted by panic");
            process::exit(1);
        }
    };

    let cell_count = cells.len();
    let columns = 64usize;
    let rows = if cell_count == 0 {
        0
    } else {
        (cell_count + columns - 1) / columns
    };
    let tile_px = 8usize;
    let width = columns * tile_px;
    let height = rows * tile_px;
    let mut atlas = vec![0u8; width * height * 4]; // transparent background

    let mut manifest_cells = Vec::with_capacity(cell_count);
    for (id, rgba) in cells.iter().enumerate() {
        let col = id % columns;
        let row = id / columns;
        let dst_x = col * tile_px;
        let dst_y = row * tile_px;
        for y in 0..tile_px {
            for x in 0..tile_px {
                let src = (y * tile_px + x) * 4;
                let dst = ((dst_y + y) * width + dst_x + x) * 4;
                atlas[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
            }
        }
        manifest_cells.push(SpriteSheetPngCell {
            id: id as u32,
            atlas_x_px: dst_x as u32,
            atlas_y_px: dst_y as u32,
        });
    }

    if let Err(e) = write_rgba_frame_png(Path::new(OUT_PNG), &atlas, width as u32, height as u32) {
        eprintln!("failed to write sprite sheet png {OUT_PNG}: {e}");
        process::exit(1);
    }

    let manifest = SpriteSheetPngManifest {
        format: "zelda3_sprite_sheet_png_v1",
        tile_width_px: 8,
        tile_height_px: 8,
        cell_count: cell_count as u32,
        columns: columns as u32,
        cells: manifest_cells,
    };
    let json = match serde_json::to_vec_pretty(&manifest) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("failed to serialize sprite sheet manifest: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(OUT_JSON, &json) {
        eprintln!("failed to write sprite sheet manifest {OUT_JSON}: {e}");
        process::exit(1);
    }

    println!(
        "dumped sprite sheet cells={cell_count} frames={frames_walked} png={width}x{height} columns={columns}"
    );
}

#[derive(Debug, Serialize)]
struct DungeonSheetPngManifest {
    format: &'static str,
    tile_width_px: u8,
    tile_height_px: u8,
    cell_count: u32,
    columns: u32,
    cells: Vec<DungeonSheetPngCell>,
}

#[derive(Debug, Serialize)]
struct DungeonSheetPngCell {
    id: u32,
    atlas_x_px: u32,
    atlas_y_px: u32,
}

/// Walk the combined-route replay and extract a colored dungeon BG tile sheet.
///
/// For every DUNGEON frame (main_module == 7 or 16) each BG tilemap entry is
/// decoded from live VRAM and colored with the live CGRAM:
///  - BG1 / BG2 (layer 0/1): 4bpp, palette base = ((word>>10)&7)*16.
///  - BG3 HUD (layer 2): 2bpp (8 words/tile), palette base = ((word>>10)&7)*4.
///
/// Tiles are deduped by their 8x8 RGBA appearance (not tile number); each unique
/// colored 8x8 appearance becomes one cell in the atlas.
///
/// Emits `developer_tilesets/dungeon_sheet.{png,json}`.
pub(crate) fn run_dump_dungeon_sheet_png(args: &[String]) {
    const OUT_PNG: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/dungeon_sheet.png"
    );
    const OUT_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/dungeon_sheet.json"
    );
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let replay = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );

    let max_frames = args
        .first()
        .map(|s| {
            s.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("invalid frame count: {s}");
                process::exit(2);
            })
        })
        .unwrap_or(60_000);

    let mut cells: Vec<Vec<u8>> = Vec::new();
    let mut index_by_rgba: HashMap<Vec<u8>, usize> = HashMap::new();

    let mut collect_dungeon_bg_tiles = |game: &ZeldaState| {
        let main_module = game.ram[TRACE_MAIN_MODULE_INDEX];
        // Only process dungeon frames.
        if main_module != 7 && main_module != 16 {
            return;
        }
        let ppu = &game.ppu;

        for layer_index in 0..3usize {
            // BG3 in PPU mode 1 is 2bpp; BG1/BG2 are 4bpp.
            let is_2bpp = layer_index == 2;
            let bg = &ppu.bg_layer[layer_index];
            let base = bg.tilemap_adr as usize;
            let chr_base = bg.tile_adr as usize;
            // Skip layers that haven't been set up yet (base==0 means the PPU
            // register was never written; VRAM[0] is usually zero/garbage).
            if base == 0 && chr_base == 0 {
                continue;
            }
            let wide = bg.tilemap_wider;
            let tall = bg.tilemap_higher;
            let cols = if wide { 64usize } else { 32 };
            let rows = if tall { 64usize } else { 32 };

            for ty in 0..rows {
                for tx in 0..cols {
                    // Quadrant layout (mirrors extract_modern_dungeon_frame_from_vram):
                    //   q0 = top-left, q1 = top-right (wide), q2 = bottom-left (tall),
                    //   q3 = bottom-right (wide+tall).
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
                    if entry_word == 0 {
                        continue;
                    }

                    // Decode 8x8 palette indices from live VRAM.
                    let indices: [u8; 64] = if is_2bpp {
                        decode_snes_2bpp_tile_indices(&ppu.vram, chr_base, entry_word)
                    } else {
                        decode_snes_4bpp_tile_indices(&ppu.vram, chr_base, entry_word)
                    };
                    if indices == [0u8; 64] {
                        continue; // fully transparent tile - skip
                    }

                    // Color with live CGRAM.
                    let palette = ((entry_word >> 10) & 7) as usize;
                    // 4bpp: 16 colors/palette (BG1/BG2 start at palette*16 in CGRAM).
                    // 2bpp: 4 colors/palette (BG3 starts at palette*4 in low CGRAM).
                    let colors_per_pal = if is_2bpp { 4usize } else { 16usize };
                    let palette_base = palette * colors_per_pal;
                    let mut rgba = vec![0u8; 64 * 4];
                    let mut all_transparent = true;
                    for (px, &index) in indices.iter().enumerate() {
                        if index == 0 {
                            continue; // palette index 0 is transparent
                        }
                        let cgram_idx = palette_base + index as usize;
                        let color =
                            snes_cgram_to_rgba(ppu.cgram.get(cgram_idx).copied().unwrap_or(0));
                        rgba[px * 4..px * 4 + 4].copy_from_slice(&color);
                        all_transparent = false;
                    }
                    if all_transparent {
                        continue;
                    }
                    if !index_by_rgba.contains_key(&rgba) {
                        let id = cells.len();
                        index_by_rgba.insert(rgba.clone(), id);
                        cells.push(rgba);
                    }
                }
            }
        }
    };

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let walk = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut game = load_translated_replay_state(rom);
        if let Err(e) = game.replay_save_file(Path::new(replay)) {
            eprintln!("failed to load replay save {replay}: {e}");
            process::exit(1);
        }
        let mut frames = game.state_recorder.replay_frame_counter;
        while frames < max_frames && game.state_recorder.replay_mode {
            let step = panic::catch_unwind(AssertUnwindSafe(|| {
                game.zelda_run_frame_with_replay_input_override(0, None);
            }));
            if step.is_err() {
                eprintln!("[warn] replay frame {frames} panicked; stopping walk early");
                break;
            }
            frames = frames.wrapping_add(1);
            collect_dungeon_bg_tiles(&game);
        }
        frames
    }));

    panic::set_hook(original_hook);

    let frames_walked = match walk {
        Ok(f) => f,
        Err(_) => {
            eprintln!("dungeon-sheet walk aborted by panic");
            process::exit(1);
        }
    };

    let cell_count = cells.len();
    let columns = 64usize;
    let rows = if cell_count == 0 {
        0
    } else {
        (cell_count + columns - 1) / columns
    };
    let tile_px = 8usize;
    let width = columns * tile_px;
    let height = rows * tile_px;
    let mut atlas = vec![0u8; width * height * 4]; // transparent background

    let mut manifest_cells = Vec::with_capacity(cell_count);
    for (id, rgba) in cells.iter().enumerate() {
        let col = id % columns;
        let row = id / columns;
        let dst_x = col * tile_px;
        let dst_y = row * tile_px;
        for y in 0..tile_px {
            for x in 0..tile_px {
                let src = (y * tile_px + x) * 4;
                let dst = ((dst_y + y) * width + dst_x + x) * 4;
                atlas[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
            }
        }
        manifest_cells.push(DungeonSheetPngCell {
            id: id as u32,
            atlas_x_px: dst_x as u32,
            atlas_y_px: dst_y as u32,
        });
    }

    if let Err(e) = write_rgba_frame_png(Path::new(OUT_PNG), &atlas, width as u32, height as u32) {
        eprintln!("failed to write dungeon sheet png {OUT_PNG}: {e}");
        process::exit(1);
    }

    let manifest = DungeonSheetPngManifest {
        format: "zelda3_dungeon_sheet_v1",
        tile_width_px: 8,
        tile_height_px: 8,
        cell_count: cell_count as u32,
        columns: columns as u32,
        cells: manifest_cells,
    };
    let json = match serde_json::to_vec_pretty(&manifest) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("failed to serialize dungeon sheet manifest: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(OUT_JSON, &json) {
        eprintln!("failed to write dungeon sheet manifest {OUT_JSON}: {e}");
        process::exit(1);
    }

    println!(
        "dumped dungeon sheet cells={cell_count} frames={frames_walked} png={width}x{height} columns={columns}"
    );
}
