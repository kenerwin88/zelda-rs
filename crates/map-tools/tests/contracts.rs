use serde_json::{json, Value};
use std::{fs, process::Command};
use zelda3_map_tools::{cli::write_build, json_bytes, read_json, room::*, tiled, write_new};

fn table(first: u16, rest: u16) -> Vec<u8> {
    (0..ROOM_COUNT)
        .flat_map(|i| if i == 0 { first } else { rest }.to_le_bytes())
        .collect()
}

fn fixture() -> Pack {
    let mut names: Vec<_> = (0..165).map(|i| format!("asset_{i}")).collect();
    let mut payloads = vec![Vec::<u8>::new(); 165];
    for &(i, name) in ROOM_ASSETS {
        names[i] = name.into();
    }
    // Independently encoded from the engine's layout stream contract. Room 0
    // has its own storage; all other rooms share a second allocation.
    let room = [
        0xe2, 0x1c, 0x28, 0x50, 0x01, 0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    payloads[3] = room.repeat(2);
    payloads[4] = table(0, room.len() as u16);
    payloads[5] = table(7, room.len() as u16 + 7);
    payloads[6] = vec![0; 28];
    payloads[7] = table(0, 14);
    payloads[58] = vec![0, 0x0e, 0x0c, 0x4b, 0xff, 0, 0xff];
    payloads[59] = table(0, 5);
    for (i, &(suffix, field, size, values, _)) in ENTRANCE_FIELDS.iter().enumerate() {
        names[11 + i] = format!("kEntranceData_{suffix}");
        payloads[11 + i] = vec![0; size * values];
        if field == "player_x" || field == "player_y" {
            payloads[11 + i][0] = 32;
        }
        if field == "floor" {
            payloads[11 + i][0] = 255;
        }
    }
    let name_bytes = (names.join("\0") + "\0").into_bytes();
    let mut data = vec![0; 88];
    data[..16].copy_from_slice(SIGNATURE);
    data[80..84].copy_from_slice(&165u32.to_le_bytes());
    data[84..88].copy_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    for payload in &payloads {
        data.extend((payload.len() as u32).to_le_bytes());
    }
    data.extend(name_bytes);
    for payload in payloads {
        while !data.len().is_multiple_of(4) {
            data.push(0xa7);
        }
        data.extend(payload);
    }
    data.extend(b"opaque trailing bytes");
    Pack::parse(data).unwrap()
}

#[test]
fn source_exact_object_encodings() {
    let cases = [
        (
            [0x29, 0x52, 0x01],
            json!({"kind":"type1","id":1,"x":10,"y":20,"width_bits":1,"height_bits":2}),
        ),
        (
            [0xff, 0x41, 0x43],
            json!({"kind":"type2","id":3,"x":52,"y":5}),
        ),
        (
            [0x29, 0x52, 0xfa],
            json!({"kind":"type3","id":41,"x":10,"y":20}),
        ),
    ];
    for (raw, expected) in cases {
        assert_eq!(decode_object(&raw).unwrap(), expected);
        assert_eq!(encode_object(&expected).unwrap(), raw);
    }
}

#[test]
fn full_pack_roundtrip_preserves_padding_trailer_and_signed_fields() {
    let pack = fixture();
    let source = export_room(&pack, 0).unwrap();
    assert_eq!(source["entrances"][0]["floor"], -1);
    assert_eq!(source["layout"]["layers"][0]["doors_raw"], json!([]));
    assert_eq!(source["layout"]["layers"][1]["doors_raw"], Value::Null);
    let (out, receipt) = compile_room(&pack, &source).unwrap();
    assert_eq!(out, pack.data());
    assert_eq!(receipt["byte_identical"], true);
    assert_eq!(receipt["changes"], json!([]));
}

#[test]
fn positions_change_exactly_three_known_bytes_and_no_other_rooms() {
    let pack = fixture();
    let mut source = export_room(&pack, 0).unwrap();
    source["layout"]["layers"][0]["objects"][0]["x"] = json!(11);
    source["sprites"]["records"][0]["x"] = json!(13);
    source["entrances"][0]["player_x"] = json!(40);
    let (out, receipt) = compile_room(&pack, &source).unwrap();
    let mut expected = pack.data().to_vec();
    expected[pack.range(3).unwrap().start + 2] = 0x2c;
    expected[pack.range(58).unwrap().start + 2] = 0x0d;
    expected[pack.range(15).unwrap().start] = 40;
    assert_eq!(out, expected);
    assert_eq!(
        receipt["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["changed_bytes"].as_u64().unwrap())
            .sum::<u64>(),
        3
    );
    let rebuilt = Pack::parse(out).unwrap();
    for room in 1..ROOM_COUNT {
        let mut a = export_room(&pack, room).unwrap();
        let b = export_room(&rebuilt, room).unwrap();
        a["base_pack_sha256"] = b["base_pack_sha256"].clone();
        assert_eq!(a, b);
    }
}

#[test]
fn rejects_unsupported_native_edits_and_malformed_shapes() {
    let pack = fixture();
    let source = export_room(&pack, 0).unwrap();
    for (pointer, value) in [
        ("/base_pack_sha256", json!("wrong")),
        ("/layout/layers/0/objects/0/id", json!(2)),
        ("/layout/layers/0/objects/0/x", json!(true)),
        ("/layout/layers/0/objects/0/x", json!(63)),
        ("/layout/layers/0/objects/0/x", json!(1.0)),
        ("/sprites/records/0/id", json!(0xe4)),
        ("/entrances/0/room", json!(1)),
        ("/entrances/0/player_x", json!(512)),
        ("/layout/layers", json!([])),
        ("/layout", json!(null)),
        ("/sprites/records", json!(4)),
        ("/entrances/0", json!(null)),
        ("/header_raw", json!([])),
    ] {
        let mut edited = source.clone();
        *edited.pointer_mut(pointer).unwrap() = value;
        assert!(compile_room(&pack, &edited).is_err(), "{pointer}");
    }
    let mut edited = source.clone();
    edited["typo"] = json!("ignored?");
    assert!(compile_room(&pack, &edited).is_err());
    let mut edited = source.clone();
    let obj = edited["layout"]["layers"][0]["objects"][0].clone();
    edited["layout"]["layers"][0]["objects"]
        .as_array_mut()
        .unwrap()
        .push(obj);
    assert!(compile_room(&pack, &edited).is_err());
}

#[test]
fn shared_layout_suffix_sprite_and_door_ownership_are_protected() {
    let base = fixture();
    // Identical pointers and a shared suffix (beginning at the object byte)
    // must both reject the edit. Door pointers can independently claim bytes.
    for (table_id, pointer, field) in [
        (4, 0, "object"),
        (4, 2, "object"),
        (5, 2, "object"),
        (59, 0, "sprite"),
    ] {
        let mut data = base.data().to_vec();
        let at = base.range(table_id).unwrap().start + 2;
        data[at..at + 2].copy_from_slice(&(pointer as u16).to_le_bytes());
        let pack = Pack::parse(data).unwrap();
        let mut source = export_room(&pack, 0).unwrap();
        if field == "object" {
            source["layout"]["layers"][0]["objects"][0]["x"] = json!(11);
        } else {
            source["sprites"]["records"][0]["x"] = json!(13);
        }
        let error = compile_room(&pack, &source).unwrap_err().to_string();
        assert!(
            error.contains(if table_id == 5 {
                "overlaps room"
            } else {
                "shares edited bytes"
            }),
            "{error}"
        );
    }
}

#[test]
fn sprite_controls_flags_and_truncation() {
    let raw = [1, 0x8e, 0xec, 0x42, 0xfd, 0, 0xe4, 0xfe, 0, 0xe4, 0xff];
    let (source, end) = decode_sprites(&raw, 0).unwrap();
    assert_eq!(end, raw.len());
    assert_eq!(
        source["records"][0],
        json!({"id":0x42,"x":12,"y":14,"x_flags":0xe0,"y_flags":0x80})
    );
    assert_eq!(encode_sprites(&source).unwrap(), raw);
    assert!(decode_layout(&[0, 0, 0xff], 0).is_err());
    assert!(decode_sprites(&[0, 1, 2], 0).is_err());
    let pack = fixture();
    for len in [0, 15, 87, 100] {
        assert!(Pack::parse(pack.data()[..len].to_vec()).is_err());
    }
    let mut corrupt = pack.data().to_vec();
    corrupt[80..84].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Pack::parse(corrupt).is_err());
    assert!(take(&[], usize::MAX, 2).is_err());
}

#[test]
fn outputs_and_receipts_are_create_only() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("pack.dat");
    write_new(&path, b"original").unwrap();
    assert!(write_new(&path, b"modified").is_err());
    assert_eq!(fs::read(&path).unwrap(), b"original");
    let out = tmp.path().join("new.dat");
    write_new(&tmp.path().join("new.dat.json"), b"receipt").unwrap();
    assert!(write_build(&out, b"modified", &json!({})).is_err());
    assert!(!out.exists());
}

#[test]
fn tiled_coordinates_noop_and_exact_edited_bytes() {
    let pack = fixture();
    let mut map = tiled::export_map(&pack, 0).unwrap();
    for (layer, x, y) in [(0, 80, 160), (3, 192, 224), (4, 32, 32)] {
        assert_eq!(map["layers"][layer]["objects"][0]["x"], x);
        assert_eq!(map["layers"][layer]["objects"][0]["y"], y);
    }
    assert_eq!(tiled::compile_map(&pack, &map).unwrap().0, pack.data());
    map["layers"][0]["objects"][0]["x"] = json!(88);
    map["layers"][3]["objects"][0]["x"] = json!(208);
    map["layers"][4]["objects"][0]["x"] = json!(40);
    let mut expected = pack.data().to_vec();
    expected[pack.range(3).unwrap().start + 2] = 0x2c;
    expected[pack.range(58).unwrap().start + 2] = 0x0d;
    expected[pack.range(15).unwrap().start] = 40;
    assert_eq!(tiled::compile_map(&pack, &map).unwrap().0, expected);
}

#[test]
fn tiled_cosmetics_reordering_and_float_coordinates() {
    let pack = fixture();
    let mut map = tiled::export_map(&pack, 0).unwrap();
    map["layers"].as_array_mut().unwrap().reverse();
    map["properties"].as_array_mut().unwrap().reverse();
    map["tiledversion"] = json!("1.12.2");
    map["backgroundcolor"] = json!("#000000");
    for layer in map["layers"].as_array_mut().unwrap() {
        layer["name"] = json!("My name");
        layer["visible"] = json!(false);
        layer["objects"].as_array_mut().unwrap().reverse();
        for obj in layer["objects"].as_array_mut().unwrap() {
            obj["name"] = json!("My label");
            obj["x"] = json!(obj["x"].as_f64().unwrap());
            obj["y"] = json!(obj["y"].as_f64().unwrap());
            let ty = obj.as_object_mut().unwrap().remove("type").unwrap();
            obj["class"] = ty;
            obj["properties"].as_array_mut().unwrap().reverse();
            obj["properties"]
                .as_array_mut()
                .unwrap()
                .push(json!({"name":"notes","type":"string","value":"note"}));
        }
    }
    assert_eq!(tiled::compile_map(&pack, &map).unwrap().0, pack.data());
}

#[test]
fn tiled_rejects_unsupported_edits() {
    let pack = fixture();
    let map = tiled::export_map(&pack, 0).unwrap();
    for (pointer, value) in [
        ("/layers/0/objects/0/x", json!(80.5)),
        ("/layers/0/objects/0/x", json!(-8)),
        ("/layers/0/objects/0/x", json!(1e100)),
        ("/layers/0/objects/0/x", json!(true)),
        ("/layers/3/objects/0/x", json!(200)),
        ("/layers/0/objects/0/rotation", json!(90)),
        ("/layers/0/objects/0/width", json!(32)),
        ("/layers/0/objects/0/type", json!("Other")),
        ("/layers/0/objects/0/id", json!(9999)),
        ("/layers/0/objects", json!([])),
        ("/layers/0/objects/0/properties", json!([])),
        ("/tilewidth", json!(16)),
        ("/tilesets", json!([{"firstgid":1,"source":"tiles.tsj"}])),
        ("/properties", json!([])),
    ] {
        let mut edited = map.clone();
        *edited.pointer_mut(pointer).unwrap() = value;
        assert!(tiled::compile_map(&pack, &edited).is_err(), "{pointer}");
    }
    for field in [
        "gid", "template", "polygon", "polyline", "text", "ellipse", "capsule", "class",
    ] {
        let mut edited = map.clone();
        edited["layers"][0]["objects"][0][field] = json!(1);
        assert!(tiled::compile_map(&pack, &edited).is_err(), "{field}");
    }
    for field in ["offsetx", "offsety", "parallaxx", "parallaxy"] {
        let mut edited = map.clone();
        edited["layers"][0][field] = json!(8);
        assert!(tiled::compile_map(&pack, &edited).is_err());
    }
    let mut edited = map.clone();
    let duplicate = edited["layers"][0]["objects"][0].clone();
    edited["layers"][0]["objects"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(tiled::compile_map(&pack, &edited).is_err());
}

#[test]
fn doors_are_reference_only() {
    let base = fixture();
    let mut data = base.data().to_vec();
    // Replace the empty door marker with a door record; append an extra layer
    // terminator in existing second-room storage. Only room 0 is decoded here.
    // No original game bytes are included in this fixture.
    let start = base.range(3).unwrap().start;
    data[start + 7..start + 9].copy_from_slice(&0x0120u16.to_le_bytes());
    data[start + 13..start + 15].copy_from_slice(&[255, 255]);
    let pack = Pack::parse(data).unwrap();
    let mut map = tiled::export_map(&pack, 0).unwrap();
    assert_eq!(map["layers"][5]["objects"][0]["x"], 368);
    assert_eq!(map["layers"][5]["objects"][0]["y"], 32);
    assert_eq!(tiled::compile_map(&pack, &map).unwrap().0, pack.data());
    map["layers"][5]["objects"][0]["x"] = json!(376);
    assert!(tiled::compile_map(&pack, &map).is_err());
}

#[test]
fn world_roundtrip_edit_duplicate_and_path_validation() {
    let pack = fixture();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("world");
    tiled::export_world(&pack, &root).unwrap();
    assert!(tiled::export_world(&pack, &root).is_err());
    let world_path = root.join("dungeons.world");
    let mut world = read_json(&world_path).unwrap();
    assert_eq!(
        world["maps"][0x61],
        json!({"fileName":"room-061.tmj","width":512,"height":512,"x":512,"y":3072})
    );
    let (output, receipt) = tiled::compile_world(&pack, &world_path).unwrap();
    assert_eq!(output, pack.data());
    assert_eq!(receipt["map_count"], 320);
    let path = root.join("room-000.tmj");
    let mut edited = read_json(&path).unwrap();
    edited["layers"][0]["objects"][0]["x"] = json!(88);
    fs::write(&path, json_bytes(&edited).unwrap()).unwrap();
    let (output, receipt) = tiled::compile_world(&pack, &world_path).unwrap();
    assert_eq!(output, tiled::compile_map(&pack, &edited).unwrap().0);
    assert_eq!(receipt["changed_bytes"], 1);
    world["maps"][1]["fileName"] = json!("room-000.tmj");
    fs::write(&world_path, json_bytes(&world).unwrap()).unwrap();
    assert!(tiled::compile_world(&pack, &world_path)
        .unwrap_err()
        .to_string()
        .contains("duplicate room"));
    fs::write(tmp.path().join("escape.tmj"), json_bytes(&edited).unwrap()).unwrap();
    world["maps"][0]["fileName"] = json!("../escape.tmj");
    fs::write(&world_path, json_bytes(&world).unwrap()).unwrap();
    assert!(tiled::compile_world(&pack, &world_path)
        .unwrap_err()
        .to_string()
        .contains("inside the world directory"));
}

#[test]
fn rust_cli_export_build_validate_play_and_errors() {
    let pack = fixture();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("base.dat"), pack.data()).unwrap();
    let run = |bin: &str, args: &[&str]| {
        Command::new(bin)
            .args(args)
            .current_dir(root)
            .output()
            .unwrap()
    };
    let room_bin = env!("CARGO_BIN_EXE_dungeon-room");
    let tiled_bin = env!("CARGO_BIN_EXE_dungeon-tiled");
    for (bin, args) in [
        (
            room_bin,
            vec![
                "export",
                "--base-pack",
                "base.dat",
                "--room",
                "0x0",
                "--out",
                "room.json",
            ],
        ),
        (
            room_bin,
            vec![
                "validate",
                "--base-pack",
                "base.dat",
                "--source",
                "room.json",
            ],
        ),
        (
            room_bin,
            vec![
                "build",
                "--base-pack",
                "base.dat",
                "--source",
                "room.json",
                "--out",
                "native.dat",
            ],
        ),
        (
            tiled_bin,
            vec![
                "export",
                "--base-pack",
                "base.dat",
                "--room",
                "0",
                "--out",
                "room.tmj",
            ],
        ),
        (
            tiled_bin,
            vec![
                "build",
                "--base-pack",
                "base.dat",
                "--map",
                "room.tmj",
                "--out",
                "tiled.dat",
            ],
        ),
    ] {
        let out = run(bin, &args);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert_eq!(fs::read(root.join("native.dat")).unwrap(), pack.data());
    assert_eq!(fs::read(root.join("tiled.dat")).unwrap(), pack.data());
    for args in [
        vec![
            "export",
            "--base-pack",
            "base.dat",
            "--room",
            "0",
            "--out",
            "room.json",
        ],
        vec![
            "export",
            "--base-pack",
            "base.dat",
            "--room",
            "0",
            "--room",
            "1",
            "--out",
            "bad.json",
        ],
        vec![
            "export",
            "--base-pack",
            "base.dat",
            "--room",
            "320",
            "--out",
            "bad.json",
        ],
        vec!["validate", "--base-pack", "base.dat", "--source"],
    ] {
        assert_eq!(run(room_bin, &args).status.code(), Some(2));
    }
    let out = run(
        room_bin,
        &[
            "play",
            "--pack",
            "base.dat",
            "--binary",
            room_bin,
            "--rom",
            "base.dat",
            "--save-dir",
            "saves",
            "--dry-run",
        ],
    );
    assert!(out.status.success());
    let printed: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(printed["env"]["ZELDA3_SAVE_DIR"]
        .as_str()
        .unwrap()
        .ends_with("/saves"));
    assert!(!root.join("saves").exists());
}

#[test]
fn generated_preview_layers_preserve_bytes_and_detect_stale_artwork() {
    let pack = fixture();
    let plain = tiled::export_map(&pack, 0).unwrap();
    let mut map = tiled::with_preview_layers(&plain, pack.hash()).unwrap();
    let (bytes, receipt) = tiled::compile_map(&pack, &map).unwrap();
    assert_eq!(bytes, pack.data());
    assert_eq!(receipt["preview_stale"], false);
    // The three reference images precede the six editable/reference groups.
    map["layers"][3]["objects"][0]["x"] = json!(88);
    let (_, receipt) = tiled::compile_map(&pack, &map).unwrap();
    assert_eq!(receipt["preview_stale"], true);
    let refreshed =
        tiled::with_preview_layers(&map, receipt["output_pack_sha256"].as_str().unwrap()).unwrap();
    assert_eq!(refreshed["layers"].as_array().unwrap().len(), 9);
    assert_eq!(
        tiled::compile_map(&pack, &refreshed).unwrap().1["preview_stale"],
        false
    );
    for (field, value) in [
        ("offsetx", json!(8)),
        ("image", json!("other.png")),
        ("objects", json!([])),
        ("repeatx", json!(true)),
    ] {
        let mut edited = map.clone();
        edited["layers"][0][field] = value;
        assert!(tiled::compile_map(&pack, &edited).is_err(), "{field}");
    }
}
