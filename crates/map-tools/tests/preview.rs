#![cfg(feature = "preview")]
use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};
use zelda3_map_tools::{json_bytes, preview::export_preview, read_json, room::Pack, tiled};

fn inputs() -> (Pack, Vec<u8>) {
    let pack = Pack::parse(
        fs::read(std::env::var_os("ZELDA3_ROOM_TEST_PACK").expect("set ZELDA3_ROOM_TEST_PACK"))
            .unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("ZELDA3_ROM").expect("set ZELDA3_ROM")).unwrap();
    (pack, rom)
}

fn pixels(path: &Path) -> Vec<u8> {
    let mut reader = png::Decoder::new(fs::File::open(path).unwrap())
        .read_info()
        .unwrap();
    let mut bytes = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut bytes).unwrap();
    assert_eq!((info.width, info.height), (512, 512));
    bytes.truncate(info.buffer_size());
    bytes
}

#[test]
#[ignore = "requires ZELDA3_ROOM_TEST_PACK and ZELDA3_ROM"]
fn house_castle_preview_and_edited_pot() {
    let (pack, rom) = inputs();
    let tmp = tempfile::tempdir().unwrap();
    for id in [0x104, 0x61] {
        let out = tmp.path().join(format!("room-{id:03x}"));
        export_preview(&pack, &rom, id, None, &out).unwrap();
        assert!(export_preview(&pack, &rom, id, None, &out).is_err());
        let map = read_json(&out.join("room.tmj")).unwrap();
        assert_eq!(map["layers"].as_array().unwrap().len(), 9);
        assert_eq!(tiled::compile_map(&pack, &map).unwrap().0, pack.data());
        let snapshot = read_json(&out.join("preview.json")).unwrap();
        assert_eq!(snapshot["room_id"], id);
        let art = pixels(&out.join("room.png"));
        assert!(
            art.chunks_exact(4)
                .collect::<std::collections::HashSet<_>>()
                .len()
                > 20,
            "room art must contain actual palette colors"
        );
        assert!(pixels(&out.join("collision-bg2.png"))
            .chunks_exact(4)
            .any(|p| p[3] != 0));
        assert!(
            snapshot["bg2_attributes"]
                .as_array()
                .unwrap()
                .contains(&json!(1)),
            "basic wall attributes must be initialized"
        );
        if id == 0x104 {
            let mut edited = map.clone();
            let layer = edited["layers"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|l| l["id"] == 1)
                .unwrap();
            let pot = layer["objects"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|o| tiled::property_values(o).unwrap()["zelda.order"] == 7)
                .unwrap();
            assert_eq!(pot["x"], 296);
            pot["x"] = json!(312);
            let (bytes, receipt) = tiled::compile_map(&pack, &edited).unwrap();
            assert_eq!(receipt["preview_stale"], true);
            assert_eq!(
                bytes
                    .iter()
                    .zip(pack.data())
                    .filter(|(a, b)| a != b)
                    .count(),
                1
            );
            let changed = tmp.path().join("edited");
            export_preview(&pack, &rom, id, Some(&edited), &changed).unwrap();
            assert_ne!(pixels(&changed.join("room.png")), art);
            assert_ne!(
                pixels(&changed.join("collision-bg2.png")),
                pixels(&out.join("collision-bg2.png"))
            );
            let refreshed = read_json(&changed.join("room.tmj")).unwrap();
            assert_eq!(
                tiled::compile_map(&pack, &refreshed).unwrap().1["preview_stale"],
                false
            );
            let source = read_json(&changed.join("preview.json")).unwrap();
            assert_ne!(snapshot["bg2_tiles"], source["bg2_tiles"]);
        }
    }
}

#[test]
#[ignore = "requires ZELDA3_ROOM_TEST_PACK, ZELDA3_ROM and ZELDA3_TILED_BIN"]
fn preview_maps_survive_actual_tiled_saves() {
    let (pack, rom) = inputs();
    let binary = std::env::var_os("ZELDA3_TILED_BIN").expect("set ZELDA3_TILED_BIN");
    let tmp = tempfile::tempdir().unwrap();
    for id in [0x104, 0x61] {
        let out = tmp.path().join(format!("room-{id:03x}"));
        export_preview(&pack, &rom, id, None, &out).unwrap();
        let saved = out.join("saved.tmj");
        let result = Command::new(&binary)
            .args(["--export-map", "json"])
            .arg(out.join("room.tmj"))
            .arg(&saved)
            .env(
                "QT_QPA_PLATFORM",
                if cfg!(target_os = "macos") {
                    "cocoa"
                } else {
                    "offscreen"
                },
            )
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let map = read_json(&saved).unwrap();
        let (bytes, receipt) = tiled::compile_map(&pack, &map).unwrap();
        assert_eq!(bytes, pack.data());
        assert_eq!(receipt["preview_stale"], false);
    }
}

#[test]
#[ignore = "requires ZELDA3_ROOM_TEST_PACK, ZELDA3_ROM and ZELDA3_HOUSE_WRAM (normal-boot room 0x104 snapshot)"]
fn house_tilemaps_match_normal_boot_snapshot() {
    let (pack, rom) = inputs();
    let ram =
        fs::read(std::env::var_os("ZELDA3_HOUSE_WRAM").expect("set ZELDA3_HOUSE_WRAM")).unwrap();
    assert_eq!(ram.len(), 0x20000);
    assert_eq!(u16::from_le_bytes([ram[0xa0], ram[0xa1]]), 0x104);
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("house");
    export_preview(&pack, &rom, 0x104, None, &out).unwrap();
    let snapshot = read_json(&out.join("preview.json")).unwrap();
    for (key, offset) in [("bg1_tiles", 0x4000), ("bg2_tiles", 0x2000)] {
        let expected: Vec<Value> = (0..4096)
            .map(|i| {
                json!(u16::from_le_bytes([
                    ram[offset + i * 2],
                    ram[offset + i * 2 + 1]
                ]))
            })
            .collect();
        assert_eq!(
            snapshot[key],
            json!(expected),
            "{key} differs from the normal-boot room draw"
        );
    }
    // This is state evidence for the preview, never a cold A/V parity receipt.
    fs::write(
        out.join("checked.json"),
        json_bytes(&json!({"normal_boot_tilemap_match":true,"runtime_parity_verified":false}))
            .unwrap(),
    )
    .unwrap();
}
