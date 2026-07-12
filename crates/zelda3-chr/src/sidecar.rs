//! Editable CHR sheet sidecar (v2, with v1 fallback) and PNG decode.
//!
//! A sheet is an indexed PNG plus a JSON sidecar. In v2 each tile's PNG pixel is
//! `palette_row.base + chr_index`, so decoding to raw CHR indices subtracts the
//! per-tile row base (assigned by `blocks[].tile_palette_rows`). In v1 the pixels
//! are already raw indices. Port of `chr_editable_sheets.read_editable_chr_sheet`.

use serde::Deserialize;
use std::path::Path;

/// The 18 tracked sheet base names, in source-pack order.
pub const SHEET_NAMES: [&str; 18] = [
    "2m-2q", "2r-2w", "a-h", "i-p", "q-x", "y-1f", "1g-1n", "1o-1v", "1w-2d", "2e-2l", "2x-3b",
    "3c-3j", "3k-3r", "3s-3z", "4a-4h", "4i-4p", "4q-4s", "4t-4x",
];

const FORMAT_V1: &str = "zelda3_editable_chr_sheet_v1";
const FORMAT_V2: &str = "zelda3_editable_chr_sheet_v2";

#[derive(Debug, Deserialize)]
pub struct SidecarManifest {
    pub format: String,
    pub sheet: String,
    pub layout: SidecarLayout,
    #[serde(default)]
    pub palette_rows: Vec<SidecarPaletteRow>,
    pub blocks: Vec<SidecarBlock>,
}

#[derive(Debug, Deserialize)]
pub struct SidecarLayout {
    pub columns: usize,
    pub rows: usize,
    pub tile_width: usize,
    pub tile_height: usize,
}

#[derive(Debug, Deserialize)]
pub struct SidecarPaletteRow {
    pub id: u32,
    pub base: u32,
    pub colors_per_row: u32,
    #[serde(default)]
    pub index_to_rgb: Vec<[u8; 3]>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SidecarBlock {
    pub block: String,
    pub source_kind: String,
    pub source_pack: u32,
    pub source_bpp: u8,
    pub tile_start: usize,
    pub tile_count: usize,
    #[serde(default)]
    pub tile_palette_rows: Vec<u32>,
}

/// A decoded sheet: raw CHR index tiles plus block provenance.
#[derive(Debug, Clone)]
pub struct DecodedSheet {
    pub name: String,
    pub tiles: Vec<[u8; 64]>,
    pub blocks: Vec<SidecarBlock>,
}

impl DecodedSheet {
    /// The concatenated 64-byte index arrays of one block (for hashing/compile).
    pub fn block_tiles(&self, block: &SidecarBlock) -> &[[u8; 64]] {
        &self.tiles[block.tile_start..block.tile_start + block.tile_count]
    }
}

/// Parse a sidecar JSON document.
pub fn parse_manifest(json: &[u8]) -> Result<SidecarManifest, String> {
    serde_json::from_slice(json).map_err(|err| format!("sidecar parse failed: {err}"))
}

/// Decode an indexed sheet PNG into row-major 8-bit palette indices.
fn decode_indexed_png(png_bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    let mut decoder = png::Decoder::new(png_bytes);
    decoder.set_transformations(png::Transformations::IDENTITY);
    let mut reader = decoder
        .read_info()
        .map_err(|err| format!("PNG header decode failed: {err}"))?;
    let (color_type, bit_depth) = reader.output_color_type();
    if color_type != png::ColorType::Indexed {
        return Err(format!(
            "editable CHR sheets must be indexed PNGs, found {color_type:?}"
        ));
    }
    if bit_depth != png::BitDepth::Eight {
        return Err(format!(
            "editable CHR sheets must be 8-bit indexed PNGs, found {bit_depth:?}"
        ));
    }
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|err| format!("PNG pixel decode failed: {err}"))?;
    buf.truncate(info.buffer_size());
    Ok((buf, info.width, info.height))
}

/// Slice a row-major indexed bitmap into 8x8 tiles in reading order.
fn tiles_from_pixels(
    pixels: &[u8],
    width: u32,
    columns: usize,
    tile_total: usize,
) -> Result<Vec<[u8; 64]>, String> {
    let width = width as usize;
    if width != columns * 8 {
        return Err(format!(
            "expected sheet width {}, found {width}",
            columns * 8
        ));
    }
    let mut tiles = Vec::with_capacity(tile_total);
    for tile_index in 0..tile_total {
        let tile_x = tile_index % columns;
        let tile_y = tile_index / columns;
        let mut tile = [0u8; 64];
        for y in 0..8 {
            let src = (tile_y * 8 + y) * width + tile_x * 8;
            let row = pixels
                .get(src..src + 8)
                .ok_or_else(|| format!("tile {tile_index} row {y} runs past PNG data"))?;
            tile[y * 8..y * 8 + 8].copy_from_slice(row);
        }
        tiles.push(tile);
    }
    Ok(tiles)
}

/// Decode a sheet PNG + sidecar into raw CHR index tiles.
pub fn decode_sheet(png_bytes: &[u8], manifest: &SidecarManifest) -> Result<DecodedSheet, String> {
    if manifest.format != FORMAT_V1 && manifest.format != FORMAT_V2 {
        return Err(format!(
            "sheet {}: unsupported editable CHR sheet format {}",
            manifest.sheet, manifest.format
        ));
    }
    let tile_total = manifest
        .blocks
        .iter()
        .map(|b| b.tile_start + b.tile_count)
        .max()
        .unwrap_or(0);

    let (pixels, width, _height) = decode_indexed_png(png_bytes)?;
    let raw_tiles = tiles_from_pixels(&pixels, width, manifest.layout.columns, tile_total)?;

    if manifest.format == FORMAT_V1 {
        return Ok(DecodedSheet {
            name: manifest.sheet.clone(),
            tiles: raw_tiles,
            blocks: manifest.blocks.clone(),
        });
    }

    let mut rows_by_id = std::collections::HashMap::new();
    for row in &manifest.palette_rows {
        rows_by_id.insert(row.id, row);
    }
    let mut tiles = raw_tiles.clone();
    for block in &manifest.blocks {
        if block.tile_palette_rows.len() != block.tile_count {
            return Err(format!(
                "sheet {}: block {} tile_palette_rows length {} != tile_count {}",
                manifest.sheet,
                block.block,
                block.tile_palette_rows.len(),
                block.tile_count
            ));
        }
        for (offset, &row_id) in block.tile_palette_rows.iter().enumerate() {
            let row = rows_by_id.get(&row_id).ok_or_else(|| {
                format!(
                    "sheet {}: block {} references undefined palette row {row_id}",
                    manifest.sheet, block.block
                )
            })?;
            let tile_index = block.tile_start + offset;
            let mut decoded = [0u8; 64];
            for (dst, &value) in decoded.iter_mut().zip(raw_tiles[tile_index].iter()) {
                let index = value as i32 - row.base as i32;
                if index < 0 || index >= row.colors_per_row as i32 {
                    return Err(format!(
                        "sheet {}: tile {tile_index} pixel value {value} is outside its palette \
                         row (base {}, {} colors); edits must stay within the tile's assigned row",
                        manifest.sheet, row.base, row.colors_per_row
                    ));
                }
                *dst = index as u8;
            }
            tiles[tile_index] = decoded;
        }
    }

    Ok(DecodedSheet {
        name: manifest.sheet.clone(),
        tiles,
        blocks: manifest.blocks.clone(),
    })
}

/// Load and decode all 18 tracked sheets from a directory.
pub fn read_sheets_dir(dir: &Path) -> Result<Vec<DecodedSheet>, String> {
    let mut sheets = Vec::with_capacity(SHEET_NAMES.len());
    for name in SHEET_NAMES {
        let json_path = dir.join(format!("{name}.json"));
        let png_path = dir.join(format!("{name}.png"));
        let json =
            std::fs::read(&json_path).map_err(|err| format!("{}: {err}", json_path.display()))?;
        let png =
            std::fs::read(&png_path).map_err(|err| format!("{}: {err}", png_path.display()))?;
        let manifest =
            parse_manifest(&json).map_err(|err| format!("{}: {err}", json_path.display()))?;
        let sheet = decode_sheet(&png, &manifest)
            .map_err(|err| format!("{}: {err}", png_path.display()))?;
        sheets.push(sheet);
    }
    Ok(sheets)
}
