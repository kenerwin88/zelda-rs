use serde_json::json;
use std::{fs, process::Command};
use zelda3_map::{ActorRecord, RoomObject};
use zelda3_map_tools::{
    json_bytes,
    project::Project,
    read_json,
    room::{self, Pack},
    tiled,
};

fn words(values: impl IntoIterator<Item = u16>) -> Vec<u8> {
    values.into_iter().flat_map(u16::to_le_bytes).collect()
}
fn table(first: u16, rest: u16) -> Vec<u8> {
    words((0..320).map(|i| if i == 0 { first } else { rest }))
}

fn fixture() -> Pack {
    let mut names: Vec<_> = (0..165).map(|i| format!("asset_{i}")).collect();
    let mut assets = vec![Vec::new(); 165];
    for &(i, name) in room::ROOM_ASSETS {
        names[i] = name.into();
    }
    for (i, name) in [
        (8, "kDungeonRoomChests"),
        (9, "kDungeonRoomTeleMsg"),
        (10, "kDungeonPitsHurtPlayer"),
        (46, "kDungeonRoomDefault"),
        (47, "kDungeonRoomDefaultOffs"),
        (48, "kDungeonRoomOverlay"),
        (49, "kDungeonRoomOverlayOffs"),
        (50, "kDungeonSecrets"),
        (51, "kDungAttrsForTile_Offs"),
        (52, "kDungAttrsForTile"),
        (53, "kMovableBlockDataInit"),
        (54, "kTorchDataInit"),
        (55, "kTorchDataJunk"),
    ] {
        names[i] = name.into();
    }
    // Source-exact records independent of the new typed serializer.
    let layout = [
        0xe2, 0, 0x28, 0x50, 1, 0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    assets[3] = layout.repeat(2);
    assets[3].extend([0xaa, 0xbb]); // unused bytes
    assets[4] = table(0, 13);
    assets[5] = table(7, 20);
    assets[6] = vec![0; 28];
    // All header flag/plane bits, including the unassigned upper six bits.
    assets[6][0] = 0xff;
    assets[6][7] = 0xe4;
    assets[6][8] = 0xab;
    assets[7] = table(0, 14);
    assets[8] = vec![0, 0x80, 0x24];
    assets[9] = vec![0; 640];
    assets[10] = words([0]);
    for starting in [false, true] {
        for (n, &(suffix, field, size, count, _)) in room::ENTRANCE_FIELDS.iter().enumerate() {
            let index = if starting {
                if suffix == "musicTrack" {
                    45
                } else {
                    28 + n
                }
            } else {
                11 + n
            };
            names[index] = format!(
                "k{}_{suffix}",
                if starting {
                    "StartingPoint"
                } else {
                    "EntranceData"
                }
            );
            assets[index] = vec![0; size * count];
            if field == "player_x" || field == "player_y" {
                assets[index][0] = 32;
            }
            if field == "floor" {
                assets[index][0] = 255;
            }
        }
    }
    names[44] = "kStartingPoint_entrance".into();
    assets[44] = vec![0];
    assets[46] = vec![255, 255];
    assets[47] = words([0]);
    assets[48] = vec![0xff, 0x41, 0xa4, 255, 255];
    assets[49] = words([0]);
    assets[50] = table(640, 645);
    assets[50].extend([0x34, 0x12, 7, 255, 255, 255, 255]);
    assets[51] = words([0]);
    assets[52] = (0..128).collect();
    assets[53] = words([0, 0x1234]);
    assets[54] = words([0, 0x4567, 0xffff, 0xffff, 0xffff]);
    assets[55] = words([0xabcd]);
    assets[58] = vec![0, 14, 12, 0x4b, 0xff, 1, 0xfd, 0x12, 0xe4, 0xff];
    assets[59] = table(0, 5);
    assets[60] = b"unrelated graphics bytes".to_vec();
    let key_data = (names.join("\0") + "\0").into_bytes();
    let mut data = vec![0; 88];
    data[..16].copy_from_slice(room::SIGNATURE);
    data[80..84].copy_from_slice(&165u32.to_le_bytes());
    data[84..88].copy_from_slice(&(key_data.len() as u32).to_le_bytes());
    for asset in &assets {
        data.extend((asset.len() as u32).to_le_bytes());
    }
    data.extend(key_data);
    for asset in assets {
        while data.len() % 4 != 0 {
            data.push(0xa7);
        }
        data.extend(asset);
    }
    data.extend(b"preserved trailing bytes");
    Pack::parse(data).unwrap()
}

#[test]
fn all_modeled_bytes_are_regenerated_including_shared_resources_and_unknown_bits() {
    let pack = fixture();
    let project = Project::from_pack(&pack).unwrap();
    assert_eq!(project.build().unwrap().0, pack.data());
    assert_eq!(project.world.programs.len(), 2);
    assert_eq!(
        project.world.rooms[1].program,
        project.world.rooms[319].program
    );
    assert_ne!(
        project.world.rooms[0].program,
        project.world.rooms[1].program
    );
    let header = &project.world.headers[&project.world.rooms[0].header];
    assert_eq!(header.travel_planes, [0, 1, 2, 3, 3]);
    assert_eq!(header.travel_plane_reserved, 42);
    assert!(header.dark && header.reserved_flag);
    assert_eq!(project.world.entrances[0].floor, -1);
    assert_eq!(project.world.starting_points[0].spawn.floor, -1);
    assert!(project.world.chests[0].big);
    assert_eq!(
        project.world.secrets[&project.world.rooms[0].secrets][0].position_word,
        0x1234
    );
    assert_eq!(project.world.torches[1].room, 0xffff);
    let overlay = &project.world.overlay_programs[&project.world.overlays[0]][0];
    // The overlay decoder uses ordinary x/y bytes even when the low byte is
    // 0xfc..0xff (which selects another family in a room object stream).
    assert_eq!(
        (
            overlay.x,
            overlay.y,
            overlay.command,
            overlay.unused_x_bits,
            overlay.unused_y_bits
        ),
        (63, 16, 0xa4, 3, 1)
    );
    assert_eq!(project.compatibility.unused_spans.len(), 1);
    assert_eq!(project.compatibility.unused_spans[0].bytes, [0xaa, 0xbb]);
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("project");
    project.save_new(&root).unwrap();
    let shell = Pack::parse(fs::read(root.join("compatibility/pack-shell.bin")).unwrap()).unwrap();
    for i in (3..=55).chain([58, 59]) {
        assert!(shell.asset(i).unwrap().iter().all(|b| *b == 0), "asset {i}");
    }
    assert_eq!(shell.asset(60).unwrap(), pack.asset(60).unwrap());
    assert_eq!(
        Project::load(&root).unwrap().build().unwrap().0,
        pack.data()
    );
}

#[test]
fn canonical_position_edits_change_only_three_expected_bytes() {
    let pack = fixture();
    let mut project = Project::from_pack(&pack).unwrap();
    let r = project.world.rooms[0].clone();
    if let RoomObject::Type1 { x, .. } =
        &mut project.world.programs.get_mut(&r.program).unwrap().passes[0].objects[0]
    {
        *x = 11;
    } else {
        panic!("wrong object");
    }
    if let ActorRecord::Actor { x, .. } =
        &mut project.world.actors.get_mut(&r.actors).unwrap().records[0]
    {
        *x = 13;
    } else {
        panic!("wrong actor");
    }
    project.world.entrances[0].player_x = 40;
    let mut expected = pack.data().to_vec();
    expected[pack.range(3).unwrap().start + 2] = 0x2c;
    expected[pack.range(58).unwrap().start + 2] = 13;
    expected[pack.range(15).unwrap().start] = 40;
    let (out, receipt) = project.build().unwrap();
    assert_eq!(out, expected);
    assert_eq!(receipt["changed_sources"].as_array().unwrap().len(), 3);
    assert_eq!(receipt["byte_identical"], false);
}

#[test]
fn missing_sources_growth_and_shared_byte_conflicts_fail_closed() {
    let pack = fixture();
    let mut p = Project::from_pack(&pack).unwrap();
    p.world.rooms.pop();
    assert!(p.build().is_err());
    let mut p = Project::from_pack(&pack).unwrap();
    p.world.rooms[0].program = "missing".into();
    assert!(p.build().is_err());
    let mut p = Project::from_pack(&pack).unwrap();
    p.world.format = "future_format".into();
    assert!(p.build().is_err());
    let mut p = Project::from_pack(&pack).unwrap();
    p.world.object_lists.clear();
    assert!(p.build().is_err());
    let mut p = Project::from_pack(&pack).unwrap();
    let id = p.world.rooms[0].program.clone();
    let list = &mut p.world.programs.get_mut(&id).unwrap().passes[0].objects;
    list.push(list[0].clone());
    assert!(p.build().unwrap_err().to_string().contains("relocation"));
    let mut p = Project::from_pack(&pack).unwrap();
    let key = format!("programs/{}", p.world.rooms[1].program);
    p.compatibility.placements.get_mut(&key).unwrap().offset = 0;
    // A resource alias now overlaps room 0; an independent edit must fail.
    let id = p.world.rooms[1].program.clone();
    if let RoomObject::Type1 { x, .. } =
        &mut p.world.programs.get_mut(&id).unwrap().passes[0].objects[0]
    {
        *x = 11;
    }
    assert!(p
        .build()
        .unwrap_err()
        .to_string()
        .contains("shared byte conflict"));
    let mut p = Project::from_pack(&pack).unwrap();
    p.compatibility.unused_spans.clear();
    assert!(p.build().unwrap_err().to_string().contains("no source"));
}

#[test]
fn cli_build_works_after_the_original_pack_is_removed() {
    let pack = fixture();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let binary = env!("CARGO_BIN_EXE_dungeon-project");
    fs::write(root.join("original.dat"), pack.data()).unwrap();
    let run = |args: &[&str]| {
        Command::new(binary)
            .args(args)
            .current_dir(root)
            .output()
            .unwrap()
    };
    let result = run(&["export", "--base-pack", "original.dat", "--out", "portable"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    fs::remove_file(root.join("original.dat")).unwrap();
    let result = run(&["build", "--project", "portable", "--out", "rebuilt.dat"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(fs::read(root.join("rebuilt.dat")).unwrap(), pack.data());
    assert!(
        !run(&["build", "--project", "portable", "--out", "rebuilt.dat"])
            .status
            .success()
    );
    let mut source = read_json(&root.join("portable/world.json")).unwrap();
    source["unexpected"] = json!(true);
    fs::write(
        root.join("portable/world.json"),
        json_bytes(&source).unwrap(),
    )
    .unwrap();
    assert!(!run(&["validate", "--project", "portable"]).status.success());
}

#[test]
fn tiled_project_roundtrip_and_batch_import_keep_sources_authoritative() {
    let pack = fixture();
    let mut p = Project::from_pack(&pack).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let world = tmp.path().join("tiled");
    p.export_tiled(&world).unwrap();
    p.import_tiled_world(&world.join("dungeons.world")).unwrap();
    assert_eq!(p.build().unwrap().0, pack.data());
    let path = world.join("room-000.tmj");
    let mut map = read_json(&path).unwrap();
    map["layers"][0]["objects"][0]["x"] = json!(88);
    fs::write(&path, json_bytes(&map).unwrap()).unwrap();
    p.import_tiled_world(&world.join("dungeons.world")).unwrap();
    assert_eq!(
        p.build().unwrap().0,
        tiled::compile_map(&pack, &map).unwrap().0
    );
    let id = p.world.rooms[0].program.clone();
    assert!(matches!(
        p.world.programs[&id].passes[0].objects[0],
        RoomObject::Type1 { x: 11, .. }
    ));
    // A map based on the earlier project revision must not overwrite newer edits.
    assert!(p
        .import_tiled(&path)
        .unwrap_err()
        .to_string()
        .contains("SHA-256"));
}

#[test]
#[ignore = "requires ZELDA3_ROOM_TEST_PACK"]
fn real_portable_project() {
    let pack = Pack::parse(
        fs::read(std::env::var_os("ZELDA3_ROOM_TEST_PACK").expect("set ZELDA3_ROOM_TEST_PACK"))
            .unwrap(),
    )
    .unwrap();
    let p = Project::from_pack(&pack).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("portable");
    p.save_new(&root).unwrap();
    let mut p = Project::load(&root).unwrap();
    assert_eq!(p.build().unwrap().0, pack.data());
    let maps = tmp.path().join("maps");
    p.export_tiled(&maps).unwrap();
    p.import_tiled_world(&maps.join("dungeons.world")).unwrap();
    assert_eq!(p.build().unwrap().0, pack.data());
    let path = maps.join("room-104.tmj");
    let mut map = read_json(&path).unwrap();
    let objects = map["layers"][0]["objects"].as_array_mut().unwrap();
    let pot = objects
        .iter_mut()
        .find(|o| tiled::property_values(o).unwrap()["zelda.order"] == 7)
        .unwrap();
    assert_eq!(pot["x"], 296);
    pot["x"] = json!(312);
    fs::write(&path, json_bytes(&map).unwrap()).unwrap();
    p.import_tiled(&path).unwrap();
    let (out, receipt) = p.build().unwrap();
    assert_eq!(out, tiled::compile_map(&pack, &map).unwrap().0);
    assert_eq!(receipt["byte_identical"], false);
    assert_eq!(
        out.iter().zip(pack.data()).filter(|(a, b)| a != b).count(),
        1
    );
    eprintln!("320 portable rooms and Tiled world reproduce whole pack {}; pot edit changes exactly one byte",pack.hash());
}
