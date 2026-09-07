use crate::{
    ensure, json_bytes,
    room::{self, Pack},
    tiled, write_new, Result,
};
use serde_json::{json, Value};
use std::path::Path;
use zelda3::zelda_rtl::map_preview::{dungeon_snapshot, DungeonSnapshot};

fn tile_pixels(vram: &[u16], entry: u16) -> [u8; 64] {
    let base = 0x2000 + (entry as usize & 0x3ff) * 16;
    let mut pixels = [0; 64];
    for y in 0..8 {
        let sy = if entry & 0x8000 != 0 { 7 - y } else { y };
        let a = vram[base + sy];
        let b = vram[base + 8 + sy];
        for x in 0..8 {
            let shift = if entry & 0x4000 != 0 { x } else { 7 - x };
            pixels[y * 8 + x] = (((a >> shift) & 1)
                | (((a >> (shift + 8)) & 1) << 1)
                | (((b >> shift) & 1) << 2)
                | (((b >> (shift + 8)) & 1) << 3)) as u8;
        }
    }
    pixels
}

fn rgba(color: u16) -> [u8; 4] {
    [
        ((color & 31) << 3) as u8,
        (((color >> 5) & 31) << 3) as u8,
        (((color >> 10) & 31) << 3) as u8,
        255,
    ]
}

fn artwork(snapshot: &DungeonSnapshot) -> Vec<u8> {
    let mut out = rgba(snapshot.palette[0]).repeat(512 * 512);
    // Mode 1 background priority: BG2 low, BG1 low, BG2 high, BG1 high.
    // No sprites, windows, lighting, color math or animated frame progression.
    for high in [false, true] {
        for tiles in [&snapshot.bg2, &snapshot.bg1] {
            for (index, &entry) in tiles.iter().enumerate() {
                if (entry & 0x2000 != 0) != high {
                    continue;
                }
                let pixels = tile_pixels(&snapshot.vram, entry);
                for (p, &value) in pixels.iter().enumerate() {
                    if value == 0 {
                        continue;
                    }
                    let x = (index % 64) * 8 + p % 8;
                    let y = (index / 64) * 8 + p / 8;
                    let color =
                        snapshot.palette[((entry as usize >> 10) & 7) * 16 + value as usize];
                    out[(y * 512 + x) * 4..(y * 512 + x) * 4 + 4].copy_from_slice(&rgba(color));
                }
            }
        }
    }
    out
}

/// Categories derived from tile_detect.rs::tile_detect_execute_inner for the
/// indoor case. Unknown/conditional interactions are never called walkable.
fn collision_style(attr: u8) -> (&'static str, [u8; 4]) {
    match attr {
        0 => ("ordinary", [0, 0, 0, 0]),
        1..=4 | 0x0b | 0x26 | 0x27 | 0x43 | 0x6c..=0x7f => {
            ("solid / blocking object", [245, 70, 78, 160])
        }
        0x10..=0x13 | 0x18..=0x1b => ("slope (partial collision)", [255, 158, 55, 160]),
        8 | 9 => ("water", [56, 153, 255, 145]),
        0x20 | 0xb0..=0xbd => ("pit (state dependent)", [176, 101, 245, 160]),
        _ => ("special / conditional", [250, 215, 78, 125]),
    }
}

fn collision(attributes: &[u8]) -> Vec<u8> {
    let mut out = vec![0; 512 * 512 * 4];
    for (i, &attr) in attributes.iter().enumerate() {
        let (_, color) = collision_style(attr);
        for y in 0..8 {
            for x in 0..8 {
                let at = (((i / 64) * 8 + y) * 512 + (i % 64) * 8 + x) * 4;
                out[at..at + 4].copy_from_slice(&color);
            }
        }
    }
    out
}

fn png_bytes(pixels: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, 512, 512);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(pixels)?;
    }
    Ok(out)
}

pub fn export_preview(
    pack: &Pack,
    rom: &[u8],
    room: usize,
    source: Option<&Value>,
    out: &Path,
) -> Result<()> {
    let map = match source {
        Some(map) => {
            ensure(
                tiled::property_values(map)?["zelda.room_id"] == room,
                "source map room does not match --room",
            )?;
            map.clone()
        }
        None => tiled::export_map(pack, room)?,
    };
    let (compiled, _) = tiled::compile_map(pack, &map)?;
    let map = tiled::with_preview_layers(&map, &room::sha256(&compiled))?;
    let entrance = pack
        .entrances()?
        .into_iter()
        .find(|e| e["room"] == room)
        .ok_or("preview currently requires a room with a direct entrance")?;
    let index = entrance["index"].as_u64().ok_or("entrance index")?;
    ensure(index <= 255, "entrance index does not fit engine loader")?;
    let snapshot = dungeon_snapshot(rom, &compiled, index as u8)?;
    ensure(
        snapshot.room as usize == room,
        "preview loader resolved another room",
    )?;
    let art = png_bytes(&artwork(&snapshot))?;
    let bg1 = png_bytes(&collision(&snapshot.bg1_attributes))?;
    let bg2 = png_bytes(&collision(&snapshot.bg2_attributes))?;
    std::fs::create_dir_all(out.parent().unwrap_or(Path::new(".")))?;
    std::fs::create_dir(out)?;
    write_new(&out.join("room.png"), &art)?;
    write_new(&out.join("collision-bg1.png"), &bg1)?;
    write_new(&out.join("collision-bg2.png"), &bg2)?;
    write_new(&out.join("room.tmj"), &json_bytes(&map)?)?;
    let mut codes = std::collections::BTreeSet::new();
    codes.extend(snapshot.bg1_attributes.iter().copied());
    codes.extend(snapshot.bg2_attributes.iter().copied());
    let legend: Vec<_> = codes
        .into_iter()
        .map(|code| {
            let (label, color) = collision_style(code);
            json!({"attribute":code,"hex":format!("{code:02X}"),"category":label,"rgba":color})
        })
        .collect();
    write_new(
        &out.join("preview.json"),
        &json_bytes(&json!({"format":"zelda3_dungeon_preview_v1",
        "room_id":room,"entrance":index,"base_pack_sha256":pack.hash(),"compiled_pack_sha256":room::sha256(&compiled),
        "rom_sha256":room::sha256(rom),"construction_snapshot":true,"runtime_parity_verified":false,
        "bg1_tiles":snapshot.bg1,"bg2_tiles":snapshot.bg2,"palette":snapshot.palette,
        "bg1_attributes":snapshot.bg1_attributes,"bg2_attributes":snapshot.bg2_attributes,"collision_legend":legend}))?,
    )?;
    write_new(&out.join("README.txt"),b"Open room.tmj in Tiled. Select points in the six object layers to edit.\nToggle Collision BG2 or Collision BG1 to inspect each plane.\nRed: solid/blocking object. Orange: partial slope. Blue: water. Purple: pit. Yellow: special/conditional.\npreview.json contains exact attribute bytes (64 columns, row-major), tile words and the legend.\nThis is an initial construction preview, with no saved progress, sprites, lighting, color math, or animation.\nAfter moving objects, save the map and regenerate into a new directory using preview --map.\nBuild and validate receipts flag stale backgrounds with preview_stale: true.\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_bitplanes_flips_palette_and_transparency_follow_snes_format() {
        let mut vram = vec![0; 0x8000];
        // Top-left index 1, next pixel index 2, bottom-right index 12.
        vram[0x2000] = 0x4080;
        vram[0x200f] = 0x0101;
        let pixels = tile_pixels(&vram, 0);
        assert_eq!((pixels[0], pixels[1], pixels[63]), (1, 2, 12));
        assert_eq!(tile_pixels(&vram, 0x4000)[7], 1);
        assert_eq!(tile_pixels(&vram, 0x8000)[56], 1);
        assert_eq!(tile_pixels(&vram, 0xc000)[0], 12);
        assert_eq!(rgba(0x7fff), [248, 248, 248, 255]);
        assert_eq!(rgba(0x001f), [248, 0, 0, 255]);
    }

    #[test]
    fn collision_colors_preserve_planes_and_source_categories() {
        assert_eq!(collision_style(1).0, "solid / blocking object");
        assert_eq!(collision_style(0x70).0, "solid / blocking object");
        assert_eq!(collision_style(0x10).0, "slope (partial collision)");
        assert_eq!(collision_style(8).0, "water");
        assert_eq!(collision_style(0x20).0, "pit (state dependent)");
        assert_eq!(collision_style(0x8e).0, "special / conditional");
        let mut attrs = vec![0; 4096];
        attrs[65] = 1;
        let pixels = collision(&attrs);
        assert_eq!(
            &pixels[(8 * 512 + 8) * 4..(8 * 512 + 8) * 4 + 4],
            &collision_style(1).1
        );
        assert_eq!(&pixels[..4], &[0, 0, 0, 0]);
    }
}
