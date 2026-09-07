//! Explicit local acceptance tests. Selecting an ignored test without its
//! required inputs fails, rather than reporting a silently skipped success.
use std::{fs, path::PathBuf, process::Command};
use zelda3_map_tools::{
    json_bytes, read_json,
    room::{self, Pack, ROOM_COUNT},
    tiled,
};

fn pack() -> Pack {
    Pack::parse(
        fs::read(std::env::var_os("ZELDA3_ROOM_TEST_PACK").expect("set ZELDA3_ROOM_TEST_PACK"))
            .unwrap(),
    )
    .unwrap()
}

#[test]
#[ignore = "requires ZELDA3_ROOM_TEST_PACK"]
fn real_corpus() {
    let pack = pack();
    let tmp = tempfile::tempdir().unwrap();
    let world = tmp.path().join("world");
    tiled::export_world(&pack, &world).unwrap();
    for id in 0..ROOM_COUNT {
        assert_eq!(
            room::compile_room(&pack, &room::export_room(&pack, id).unwrap())
                .unwrap()
                .0,
            pack.data(),
            "native room {id:03x}"
        );
        assert_eq!(
            tiled::compile_map(
                &pack,
                &read_json(&world.join(format!("room-{id:03x}.tmj"))).unwrap()
            )
            .unwrap()
            .0,
            pack.data(),
            "Tiled room {id:03x}"
        );
    }
    let (out, receipt) = tiled::compile_world(&pack, &world.join("dungeons.world")).unwrap();
    assert_eq!(out, pack.data());
    assert_eq!(receipt["changed_bytes"], 0);
    eprintln!(
        "320 native maps, 320 Tiled maps, combined world: identical whole pack {}",
        pack.hash()
    );
}

#[test]
#[ignore = "requires ZELDA3_ROOM_TEST_PACK and ZELDA3_TILED_BIN"]
fn real_tiled_editor() {
    let pack = pack();
    let binary = PathBuf::from(std::env::var_os("ZELDA3_TILED_BIN").expect("set ZELDA3_TILED_BIN"))
        .canonicalize()
        .unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let world = tmp.path().join("world");
    tiled::export_world(&pack, &world).unwrap();
    for id in 0..ROOM_COUNT {
        let source = tmp.path().join("source.tmj");
        let saved = world.join(format!("room-{id:03x}.tmj"));
        fs::write(
            &source,
            json_bytes(&tiled::export_map(&pack, id).unwrap()).unwrap(),
        )
        .unwrap();
        fs::remove_file(&saved).unwrap();
        let output = Command::new(&binary)
            .args(["--export-map", "json"])
            .arg(&source)
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
            output.status.success(),
            "Tiled room {id:03x}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            tiled::compile_map(&pack, &read_json(&saved).unwrap())
                .unwrap()
                .0,
            pack.data(),
            "editor room {id:03x}"
        );
        if id % 64 == 63 {
            eprintln!("Tiled re-saved {} / 320 rooms", id + 1);
        }
    }
    assert_eq!(
        tiled::compile_world(&pack, &world.join("dungeons.world"))
            .unwrap()
            .0,
        pack.data()
    );
    eprintln!(
        "320 actual Tiled saves and combined world: identical whole pack {}",
        pack.hash()
    );
}
