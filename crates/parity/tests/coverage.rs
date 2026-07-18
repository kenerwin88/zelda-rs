use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use parity::coverage::{CoverageFrame, CoverageUniverse, ModuleState, RouteCoverage};
use parity::runner::{self, Paths};

#[test]
fn report_lists_missed_expected_surfaces() {
    let mut coverage = RouteCoverage::default();
    coverage.record(CoverageFrame {
        frame: 1,
        main_module: 0x07,
        submodule: 0x02,
        subsubmodule: 0x03,
        indoor_room: Some(0x00a4),
        overworld_screen: None,
        sprite_types: vec![0xcb],
        ancilla_types: vec![0x05],
        active_item: Some(0x12),
    });
    coverage.record(CoverageFrame {
        frame: 2,
        main_module: 0x0e,
        submodule: 0x01,
        subsubmodule: 0x00,
        indoor_room: None,
        overworld_screen: Some(0x0040),
        sprite_types: vec![0xcb],
        ancilla_types: vec![],
        active_item: Some(0x12),
    });

    let report = coverage.report_with_universe(&CoverageUniverse {
        main_modules: vec![0x07, 0x09, 0x0e],
        module_states: vec![
            ModuleState {
                main: 0x07,
                sub: 0x02,
                subsub: 0x03,
            },
            ModuleState {
                main: 0x09,
                sub: 0x00,
                subsub: 0x00,
            },
            ModuleState {
                main: 0x0e,
                sub: 0x01,
                subsub: 0x00,
            },
        ],
        sprite_types: vec![0xcb, 0xcc],
        ancilla_types: vec![0x05, 0x06],
        indoor_rooms: vec![0x00a4, 0x00a5],
        overworld_screens: vec![0x0040, 0x0041],
        active_items: vec![0x12, 0x13],
    });

    assert_eq!(report.frames, 2);
    assert_eq!(
        report.category("main_modules").unwrap().covered,
        vec!["0x07", "0x0e"]
    );
    assert_eq!(
        report.category("module_states").unwrap().covered,
        vec!["0x07:0x02:0x03", "0x0e:0x01:0x00"]
    );
    assert_eq!(
        report.category("module_states").unwrap().missed,
        vec!["0x09:0x00:0x00"]
    );
    assert_eq!(
        report.category("main_modules").unwrap().missed,
        vec!["0x09"]
    );
    assert_eq!(
        report.category("sprite_types").unwrap().missed,
        vec!["0xcc"]
    );
    assert_eq!(
        report.category("ancilla_types").unwrap().missed,
        vec!["0x06"]
    );
    assert_eq!(
        report.category("indoor_rooms").unwrap().missed,
        vec!["0x00a5"]
    );
    assert_eq!(
        report.category("overworld_screens").unwrap().missed,
        vec!["0x0041"]
    );
    assert_eq!(
        report.category("active_items").unwrap().missed,
        vec!["0x13"]
    );

    let text = report.to_text();
    assert!(text.contains("frames: 2"));
    assert!(text.contains("sprite_types: 1/2"));
    assert!(text.contains("missed: 0xcc"));
}

#[test]
fn report_records_first_seen_frames_for_covered_surfaces() {
    let mut coverage = RouteCoverage::default();
    coverage.record(CoverageFrame {
        frame: 42,
        main_module: 0x07,
        submodule: 0x02,
        subsubmodule: 0x03,
        indoor_room: Some(0x0043),
        overworld_screen: None,
        sprite_types: vec![0x65],
        ancilla_types: vec![0x07],
        active_item: Some(0x01),
    });
    coverage.record(CoverageFrame {
        frame: 77,
        main_module: 0x07,
        submodule: 0x02,
        subsubmodule: 0x03,
        indoor_room: Some(0x0043),
        overworld_screen: None,
        sprite_types: vec![0x65],
        ancilla_types: vec![0x0e],
        active_item: Some(0x01),
    });

    let report = coverage.report_with_universe(&CoverageUniverse {
        main_modules: vec![0x07],
        module_states: vec![ModuleState {
            main: 0x07,
            sub: 0x02,
            subsub: 0x03,
        }],
        sprite_types: vec![0x65],
        ancilla_types: vec![0x07, 0x0e],
        indoor_rooms: vec![0x0043],
        overworld_screens: vec![],
        active_items: vec![0x01],
    });

    assert_eq!(
        report.category("indoor_rooms").unwrap().first_seen["0x0043"],
        42
    );
    assert_eq!(
        report.category("sprite_types").unwrap().first_seen["0x65"],
        42
    );
    assert_eq!(
        report.category("ancilla_types").unwrap().first_seen["0x07"],
        42
    );
    assert_eq!(
        report.category("ancilla_types").unwrap().first_seen["0x0e"],
        77
    );
}

#[test]
fn text_report_summarizes_long_miss_lists() {
    let coverage = RouteCoverage::default();
    let report = coverage.report_with_universe(&CoverageUniverse {
        main_modules: (0x01..=0x20).collect(),
        module_states: vec![],
        sprite_types: vec![],
        ancilla_types: vec![],
        indoor_rooms: vec![],
        overworld_screens: vec![],
        active_items: vec![],
    });

    let text = report.to_text();
    assert!(text.contains("main_modules: 0/32"));
    assert!(text.contains("0x18 ... +8 more (see report JSON)"));
}

#[test]
fn standard_universe_excludes_source_backed_invalid_ranges() {
    let report = RouteCoverage::default().report();

    let main_modules = report.category("main_modules").unwrap();
    assert_eq!(main_modules.expected, 25);
    assert!(!main_modules.missed.contains(&"0x0a".to_string()));
    assert!(!main_modules.missed.contains(&"0x0c".to_string()));
    assert!(!main_modules.missed.contains(&"0x0d".to_string()));

    let ancilla_types = report.category("ancilla_types").unwrap();
    assert_eq!(ancilla_types.expected, 60);
    assert!(!ancilla_types.missed.contains(&"0x03".to_string()));
    assert!(!ancilla_types.missed.contains(&"0x0e".to_string()));
    assert!(!ancilla_types.missed.contains(&"0x0f".to_string()));
    assert!(!ancilla_types.missed.contains(&"0x10".to_string()));
    assert!(!ancilla_types.missed.contains(&"0x12".to_string()));
    assert!(!ancilla_types.missed.contains(&"0x14".to_string()));
    assert!(!ancilla_types.missed.contains(&"0x25".to_string()));
    assert!(ancilla_types.missed.contains(&"0x33".to_string()));
    assert!(!ancilla_types.missed.contains(&"0x44".to_string()));

    let sprite_types = report.category("sprite_types").unwrap();
    assert_eq!(sprite_types.expected, 232);
    assert!(sprite_types.missed.contains(&"0x00".to_string()));
    assert!(!sprite_types.missed.contains(&"0x03".to_string()));
    assert!(!sprite_types.missed.contains(&"0x05".to_string()));
    assert!(!sprite_types.missed.contains(&"0x07".to_string()));
    assert!(!sprite_types.missed.contains(&"0x2d".to_string()));
    assert!(!sprite_types.missed.contains(&"0x5e".to_string()));
    assert!(sprite_types.missed.contains(&"0x65".to_string()));
    assert!(!sprite_types.missed.contains(&"0x77".to_string()));
    assert!(!sprite_types.missed.contains(&"0x98".to_string()));
    assert!(!sprite_types.missed.contains(&"0xb8".to_string()));
    assert!(sprite_types.missed.contains(&"0xe6".to_string()));
    assert!(!sprite_types.missed.contains(&"0xef".to_string()));
    assert!(!sprite_types.missed.contains(&"0xf0".to_string()));
    assert!(!sprite_types.missed.contains(&"0xf1".to_string()));
    assert!(!sprite_types.missed.contains(&"0xf3".to_string()));

    let active_items = report.category("active_items").unwrap();
    assert_eq!(active_items.expected, 0x14);
    assert!(!active_items.missed.contains(&"0x15".to_string()));
}

#[test]
fn source_assets_override_room_and_screen_universes_from_c_asset_filenames() {
    let root = temp_asset_root("source-assets");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-47.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-128.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-not-an-id.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/readme.txt"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-295.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-296.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-319.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml.bak"), "").unwrap();

    let standard = CoverageUniverse::standard();
    let universe = CoverageUniverse::from_c_assets_or_standard(&root);

    assert_eq!(universe.overworld_screens, vec![47, 128]);
    assert_eq!(universe.indoor_rooms, vec![3, 295]);
    assert_eq!(universe.main_modules, standard.main_modules);
    assert_eq!(universe.sprite_types, standard.sprite_types);
    assert_eq!(universe.ancilla_types, standard.ancilla_types);
    assert_eq!(universe.active_items, standard.active_items);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_assets_exclude_special_overworld_payloads_that_are_not_frame_screen_states() {
    let root = temp_asset_root("source-assets-special-overworld");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    for screen in [0x002f, 0x0080, 0x0081, 0x0082, 0x0088, 0x0093, 0x009f] {
        std::fs::write(
            root.join(format!("assets/overworld/overworld-{screen}.yaml")),
            "",
        )
        .unwrap();
    }
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();

    let universe = CoverageUniverse::from_c_assets_or_standard(&root);

    assert_eq!(
        universe.overworld_screens,
        vec![0x002f, 0x0080, 0x0081, 0x0088]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_seeded_coverage_records_frame_sampled_asset_surfaces_with_provenance() {
    let root = temp_asset_root("source-seeded-coverage");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-47.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-128.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-130.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-295.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-296.yaml"), "").unwrap();

    let coverage = RouteCoverage::source_seeded_from_c_assets(&root);

    assert_eq!(
        coverage.indoor_rooms.iter().copied().collect::<Vec<_>>(),
        vec![3, 295]
    );
    assert_eq!(
        coverage
            .overworld_screens
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![47, 128]
    );
    assert_eq!(
        coverage.provenance["indoor_rooms"]["0x0003"],
        "source-seeded:assets/dungeon/dungeon-3.yaml"
    );
    assert_eq!(
        coverage.provenance["overworld_screens"]["0x0080"],
        "source-seeded:assets/overworld/overworld-128.yaml"
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn route_evidence_report_does_not_count_source_seeded_only_surfaces() {
    let root = temp_asset_root("route-evidence-report");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-47.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-48.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-4.yaml"), "").unwrap();

    let mut coverage = RouteCoverage::default();
    coverage.record(CoverageFrame {
        frame: 10,
        main_module: 0x07,
        submodule: 0,
        subsubmodule: 0,
        indoor_room: Some(3),
        overworld_screen: Some(47),
        sprite_types: vec![],
        ancilla_types: vec![],
        active_item: None,
    });
    coverage.merge(&RouteCoverage::source_seeded_from_c_assets(&root));
    let universe = CoverageUniverse::from_c_assets_or_standard(&root);

    let source_surface_report = coverage.report_with_universe(&universe);
    let route_report = coverage.route_evidence_report_with_universe(&universe);

    assert_eq!(
        report_category_hit(&source_surface_report, "indoor_rooms"),
        Some((2, 2))
    );
    assert_eq!(
        report_category_hit(&source_surface_report, "overworld_screens"),
        Some((2, 2))
    );
    assert_eq!(
        report_category_hit(&route_report, "indoor_rooms"),
        Some((1, 2))
    );
    assert_eq!(
        route_report.category("indoor_rooms").unwrap().missed,
        vec!["0x0004"]
    );
    assert_eq!(
        report_category_hit(&route_report, "overworld_screens"),
        Some((1, 2))
    );
    assert_eq!(
        route_report.category("overworld_screens").unwrap().missed,
        vec!["0x0030"]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_source_assets_fall_back_to_static_room_and_screen_universes() {
    let root = temp_asset_root("missing-source-assets");
    let standard = CoverageUniverse::standard();

    let universe = CoverageUniverse::from_c_assets_or_standard(&root);

    assert_eq!(universe.indoor_rooms, standard.indoor_rooms);
    assert_eq!(universe.overworld_screens, standard.overworld_screens);
}

#[test]
fn coverage_cli_writes_source_seeded_coverage_log() {
    let root = temp_asset_root("coverage-source-seeded-cli");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-90.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-272.yaml"), "").unwrap();

    let seeded_log = root.join("coverage-source-seeded.json");
    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .arg("coverage")
        .arg("--seed-from-c-assets")
        .arg("--json")
        .arg(&seeded_log)
        .env("ZELDA3_C_REPO", &root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let seeded: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&seeded_log).unwrap()).unwrap();
    assert_eq!(seeded["indoor_rooms"], serde_json::json!([272]));
    assert_eq!(seeded["overworld_screens"], serde_json::json!([90]));
    assert_eq!(
        seeded["provenance"]["indoor_rooms"]["0x0110"],
        "source-seeded:assets/dungeon/dungeon-272.yaml"
    );
    assert_eq!(
        seeded["provenance"]["overworld_screens"]["0x005a"],
        "source-seeded:assets/overworld/overworld-90.yaml"
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn coverage_cli_route_full_gate_rejects_source_seeded_only_surfaces() {
    let root = temp_asset_root("coverage-route-full-cli");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-47.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-48.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-4.yaml"), "").unwrap();

    let route_log = root.join("coverage-route.json");
    let route_report = root.join("coverage-route-report.json");
    write_coverage_log(
        &route_log,
        CoverageFrame {
            frame: 10,
            main_module: 0x07,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: Some(3),
            overworld_screen: Some(47),
            sprite_types: vec![],
            ancilla_types: vec![],
            active_item: None,
        },
    );

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .arg("coverage")
        .arg("--from-json")
        .arg(&route_log)
        .arg("--seed-from-c-assets")
        .arg("--route-report-json")
        .arg(&route_report)
        .arg("--require-route-full")
        .env("ZELDA3_C_REPO", &root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("route coverage incomplete: indoor_rooms missed 1/2"),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&route_report).unwrap()).unwrap();
    assert_eq!(category_hit(&report, "indoor_rooms"), Some((1, 2)));
    assert_eq!(category_hit(&report, "overworld_screens"), Some((1, 2)));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn coverage_cli_writes_source_guided_route_worklist_for_route_misses() {
    let root = temp_asset_root("coverage-route-worklist-cli");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(
        root.join("assets/dungeon/dungeon-3.yaml"),
        r#"Header:
  hole0_dest: [0, 0]
  stair0_dest: [0, 0]
  stair1_dest: [0, 0]
  stair2_dest: [0, 0]
  stair3_dest: [0, 0]
Entrances:
- entrance_index: 130
  name: Chris Houlihan Room (Drop In)
Layer1: []
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("assets/dungeon/dungeon-4.yaml"),
        r#"Header:
  hole0_dest: [0, 0]
  stair0_dest: [0, 0]
  stair1_dest: [0, 0]
  stair2_dest: [8, 0]
  stair3_dest: [0, 0]
Entrances: []
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("assets/dungeon/dungeon-8.yaml"),
        r#"Header:
  hole0_dest: [0, 0]
  stair0_dest: [0, 0]
  stair1_dest: [0, 0]
  stair2_dest: [0, 0]
  stair3_dest: [0, 0]
Entrances:
- entrance_index: 56
  name: Fairy Cave 1
Layer1: []
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("assets/dungeon/dungeon-9.yaml"),
        r#"Header:
  hole0_dest: [0, 0]
  stair0_dest: [0, 0]
  stair1_dest: [0, 0]
  stair2_dest: [0, 0]
  stair3_dest: [0, 0]
Entrances: []
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("assets/overworld/overworld-85.yaml"),
        r#"Header:
  name: 'DW 085 : South Bend'
Travel: []
Entrances: []
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("assets/overworld/overworld-90.yaml"),
        r#"Header:
  name: 'DW 090 : West Woods'
Travel: []
Entrances:
- {index: 116, x: 20, y: 20, entrance_id: 87}
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("assets/overworld/overworld-127.yaml"),
        r#"Header:
  name: 'DW 127 : A Terrible Vacation Spot'
Travel:
- whirlpool_src_area: 85
  xy: [104, 184]
Entrances: []
"#,
    )
    .unwrap();

    let route_log = root.join("coverage-route.json");
    let worklist_path = root.join("coverage-route-worklist.json");
    write_coverage_log(
        &route_log,
        CoverageFrame {
            frame: 10,
            main_module: 0x07,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: Some(4),
            overworld_screen: Some(85),
            sprite_types: vec![],
            ancilla_types: vec![],
            active_item: None,
        },
    );

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .arg("coverage")
        .arg("--from-json")
        .arg(&route_log)
        .arg("--seed-from-c-assets")
        .arg("--route-worklist-json")
        .arg(&worklist_path)
        .env("ZELDA3_C_REPO", &root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let worklist: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&worklist_path).unwrap()).unwrap();
    let room_3 = worklist_entry(&worklist, "indoor_rooms", "0x0003").unwrap();
    assert_eq!(room_3["strategies"][0]["kind"], "direct_entrance");
    assert_eq!(room_3["strategies"][0]["entrance_index"], 130);

    let room_8 = worklist_entry(&worklist, "indoor_rooms", "0x0008").unwrap();
    assert!(worklist_strategy(room_8, "direct_entrance").is_some());
    let stair = worklist_strategy(room_8, "stair_or_hole_source").unwrap();
    assert_eq!(stair["source_id"], "0x0004");
    assert_eq!(stair["via"], "stair2_dest");
    assert_eq!(stair["route_source_covered"], true);

    let room_9 = worklist_entry(&worklist, "indoor_rooms", "0x0009").unwrap();
    assert_eq!(room_9["strategies"][0]["kind"], "unclassified");

    let screen_90 = worklist_entry(&worklist, "overworld_screens", "0x005a").unwrap();
    assert_eq!(screen_90["strategies"][0]["kind"], "overworld_entrance");
    assert_eq!(screen_90["strategies"][0]["entrance_index"], 116);
    assert_eq!(screen_90["strategies"][0]["entrance_id"], 87);

    let screen_127 = worklist_entry(&worklist, "overworld_screens", "0x007f").unwrap();
    assert_eq!(screen_127["strategies"][0]["kind"], "travel_source");
    assert_eq!(screen_127["strategies"][0]["source_id"], "0x0055");
    assert_eq!(screen_127["strategies"][0]["route_source_covered"], true);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn coverage_cli_merges_repeated_from_json_logs() {
    let root = temp_asset_root("merge-cli");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-47.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-48.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-4.yaml"), "").unwrap();

    let first_log = root.join("coverage-a.json");
    let second_log = root.join("coverage-b.json");
    let report_path = root.join("coverage-report.json");
    write_coverage_log(
        &first_log,
        CoverageFrame {
            frame: 10,
            main_module: 0x07,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: Some(3),
            overworld_screen: Some(47),
            sprite_types: vec![0x00],
            ancilla_types: vec![0x01],
            active_item: Some(0x01),
        },
    );
    write_coverage_log(
        &second_log,
        CoverageFrame {
            frame: 20,
            main_module: 0x09,
            submodule: 1,
            subsubmodule: 0,
            indoor_room: Some(4),
            overworld_screen: Some(48),
            sprite_types: vec![0x05],
            ancilla_types: vec![0x02],
            active_item: Some(0x02),
        },
    );

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .arg("coverage")
        .arg("--from-json")
        .arg(&first_log)
        .arg("--from-json")
        .arg(&second_log)
        .arg("--report-json")
        .arg(&report_path)
        .env("ZELDA3_C_REPO", &root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).unwrap()).unwrap();
    assert_eq!(category_hit(&report, "indoor_rooms"), Some((2, 2)));
    assert_eq!(category_hit(&report, "overworld_screens"), Some((2, 2)));
    assert_eq!(report["frames"], 2);
    assert_eq!(report["last_frame"], 20);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn coverage_cli_writes_merged_json_log_from_repeated_from_json_logs() {
    let root = temp_asset_root("merge-cli-json");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-47.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-48.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-4.yaml"), "").unwrap();

    let first_log = root.join("coverage-a.json");
    let second_log = root.join("coverage-b.json");
    let merged_log = root.join("coverage-merged.json");
    write_coverage_log(
        &first_log,
        CoverageFrame {
            frame: 10,
            main_module: 0x07,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: Some(3),
            overworld_screen: Some(47),
            sprite_types: vec![0x00],
            ancilla_types: vec![0x01],
            active_item: Some(0x01),
        },
    );
    write_coverage_log(
        &second_log,
        CoverageFrame {
            frame: 20,
            main_module: 0x09,
            submodule: 1,
            subsubmodule: 0,
            indoor_room: Some(4),
            overworld_screen: Some(48),
            sprite_types: vec![0x05],
            ancilla_types: vec![0x02],
            active_item: Some(0x02),
        },
    );

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .arg("coverage")
        .arg("--from-json")
        .arg(&first_log)
        .arg("--from-json")
        .arg(&second_log)
        .arg("--json")
        .arg(&merged_log)
        .env("ZELDA3_C_REPO", &root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let merged: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&merged_log).unwrap()).unwrap();
    assert_eq!(merged["frames"], 2);
    assert_eq!(merged["last_frame"], 20);
    assert_eq!(merged["indoor_rooms"], serde_json::json!([3, 4]));
    assert_eq!(merged["overworld_screens"], serde_json::json!([47, 48]));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn coverage_cli_require_full_succeeds_for_complete_source_backed_report() {
    let root = temp_asset_root("require-full-pass");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-47.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();

    let log_path = root.join("coverage.json");
    write_complete_coverage_log(&log_path, &[3], &[47]);

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .arg("coverage")
        .arg("--from-json")
        .arg(&log_path)
        .arg("--require-full")
        .env("ZELDA3_C_REPO", &root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn coverage_cli_require_full_fails_for_missed_source_backed_surface() {
    let root = temp_asset_root("require-full-fail");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-47.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-48.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();

    let log_path = root.join("coverage.json");
    write_complete_coverage_log(&log_path, &[3], &[47]);

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .arg("coverage")
        .arg("--from-json")
        .arg(&log_path)
        .arg("--require-full")
        .env("ZELDA3_C_REPO", &root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("coverage incomplete: overworld_screens missed 1/2"),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn coverage_cli_reports_new_surfaces_covered_against_a_base_log() {
    let root = temp_asset_root("coverage-delta-cli");
    std::fs::create_dir_all(root.join("assets/overworld")).unwrap();
    std::fs::create_dir_all(root.join("assets/dungeon")).unwrap();
    std::fs::write(root.join("assets/overworld/overworld-47.yaml"), "").unwrap();
    std::fs::write(root.join("assets/overworld/overworld-48.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-3.yaml"), "").unwrap();
    std::fs::write(root.join("assets/dungeon/dungeon-4.yaml"), "").unwrap();

    let base_log = root.join("coverage-base.json");
    let candidate_log = root.join("coverage-candidate.json");
    let delta_report_path = root.join("coverage-delta.json");
    write_complete_coverage_log(&base_log, &[3], &[47]);
    write_coverage_log(
        &candidate_log,
        CoverageFrame {
            frame: 77,
            main_module: 0x09,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: Some(4),
            overworld_screen: Some(48),
            sprite_types: vec![],
            ancilla_types: vec![],
            active_item: None,
        },
    );

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .arg("coverage")
        .arg("--diff-from-json")
        .arg(&base_log)
        .arg("--from-json")
        .arg(&candidate_log)
        .arg("--delta-report-json")
        .arg(&delta_report_path)
        .env("ZELDA3_C_REPO", &root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("coverage delta vs"), "{stdout}");
    assert!(stdout.contains("newly_covered_total: 2"), "{stdout}");
    assert!(stdout.contains("indoor_rooms: +1 0x0004"), "{stdout}");
    assert!(stdout.contains("overworld_screens: +1 0x0030"), "{stdout}");
    assert!(stdout.contains("sprite_types: +0"), "{stdout}");

    let delta_report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&delta_report_path).unwrap()).unwrap();
    assert_eq!(delta_report["total_newly_covered"], 2);
    assert_eq!(
        delta_category_newly_covered(&delta_report, "indoor_rooms"),
        Some(vec!["0x0004".to_string()])
    );
    assert_eq!(
        delta_category_newly_covered(&delta_report, "overworld_screens"),
        Some(vec!["0x0030".to_string()])
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn coverage_cli_passes_input_script_and_load_state_to_runner() {
    let root = temp_asset_root("coverage-window-cli");
    std::fs::create_dir_all(root.join("target/parity")).unwrap();
    std::fs::create_dir_all(root.join("saves")).unwrap();
    std::fs::write(root.join("saves/zelda3.sfc"), b"rom").unwrap();
    std::fs::write(root.join("saves/zelda3-combined-route.sav"), b"save").unwrap();
    let fake_runner = root.join("target/parity/zelda3");
    let args_log = root.join("runner-args.txt");
    let coverage_log = root.join("coverage-window.json");
    let input_script = root.join("window-input.txt");
    let load_state = root.join("window.sav");
    std::fs::write(&input_script, "0 0x0000\n").unwrap();
    std::fs::write(&load_state, b"state").unwrap();
    write_fake_coverage_runner(&fake_runner);

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(&root)
        .arg("coverage")
        .arg("--frames")
        .arg("123")
        .arg("--json")
        .arg(&coverage_log)
        .arg("--input-script")
        .arg(&input_script)
        .arg("--load-state")
        .arg(&load_state)
        .env("ZELDA3_REPO", &root)
        .env("ZELDA3_C_REPO", &root)
        .env("ZELDA3_NEW_BIN", &fake_runner)
        .env("FAKE_ZELDA3_ARGS_LOG", &args_log)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(args_log).unwrap();
    assert!(args.contains("--input-script\n"), "{args}");
    assert!(
        args.contains(&format!("{}\n", input_script.display())),
        "{args}"
    );
    assert!(args.contains("--load-state\n"), "{args}");
    assert!(
        args.contains(&format!("{}\n", load_state.display())),
        "{args}"
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn coverage_cli_passes_input_script_overlay_to_runner() {
    let root = temp_asset_root("coverage-overlay-cli");
    std::fs::create_dir_all(root.join("target/parity")).unwrap();
    std::fs::create_dir_all(root.join("saves")).unwrap();
    std::fs::write(root.join("saves/zelda3.sfc"), b"rom").unwrap();
    std::fs::write(root.join("saves/zelda3-combined-route.sav"), b"save").unwrap();
    let fake_runner = root.join("target/parity/zelda3");
    let args_log = root.join("runner-args.txt");
    let coverage_log = root.join("coverage-overlay.json");
    let input_script_overlay = root.join("overlay-input.txt");
    let load_state = root.join("window.sav");
    std::fs::write(&input_script_overlay, "10 NONE\n20 A+RIGHT\n").unwrap();
    std::fs::write(&load_state, b"state").unwrap();
    write_fake_coverage_runner(&fake_runner);

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(&root)
        .arg("coverage")
        .arg("--frames")
        .arg("123")
        .arg("--json")
        .arg(&coverage_log)
        .arg("--input-script-overlay")
        .arg(&input_script_overlay)
        .arg("--load-state")
        .arg(&load_state)
        .env("ZELDA3_REPO", &root)
        .env("ZELDA3_C_REPO", &root)
        .env("ZELDA3_NEW_BIN", &fake_runner)
        .env("FAKE_ZELDA3_ARGS_LOG", &args_log)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(args_log).unwrap();
    assert!(args.contains("--input-script-overlay\n"), "{args}");
    assert!(
        args.contains(&format!("{}\n", input_script_overlay.display())),
        "{args}"
    );
    assert!(!args.contains("--input-script\n"), "{args}");
    assert!(args.contains("--load-state\n"), "{args}");
    assert!(
        args.contains(&format!("{}\n", load_state.display())),
        "{args}"
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn coverage_cli_runs_direct_entrance_probes_from_route_worklist() {
    let root = temp_asset_root("coverage-direct-entrance-probes-cli");
    std::fs::create_dir_all(root.join("target/parity")).unwrap();
    std::fs::create_dir_all(root.join("saves")).unwrap();
    std::fs::write(root.join("saves/zelda3.sfc"), b"rom").unwrap();
    std::fs::write(root.join("saves/zelda3-combined-route.sav"), b"save").unwrap();
    let fake_runner = root.join("target/parity/zelda3");
    let args_log = root.join("runner-args.txt");
    let coverage_log = root.join("coverage-direct-entrance-probes.json");
    let worklist = root.join("coverage-route-worklist.json");
    std::fs::write(
        &worklist,
        r#"{
          "indoor_rooms": [
            {
              "id": "0x0003",
              "source": "assets/dungeon/dungeon-3.yaml",
              "strategies": [
                {"kind": "direct_entrance", "entrance_index": 130, "source": "assets/dungeon/dungeon-3.yaml"}
              ]
            },
            {
              "id": "0x0008",
              "source": "assets/dungeon/dungeon-8.yaml",
              "strategies": [
                {"kind": "direct_entrance", "entrance_index": 56, "source": "assets/dungeon/dungeon-8.yaml"},
                {"kind": "stair_or_hole_source", "source_id": "0x0004", "via": "stair2_dest", "route_source_covered": true}
              ]
            },
            {
              "id": "0x002d",
              "source": "assets/dungeon/dungeon-45.yaml",
              "strategies": [
                {"kind": "unclassified"}
              ]
            }
          ],
          "overworld_screens": [
            {
              "id": "0x005a",
              "source": "assets/overworld/overworld-90.yaml",
              "strategies": [
                {"kind": "overworld_entrance", "entrance_index": 116, "entrance_id": 87}
              ]
            },
            {
              "id": "0x007f",
              "source": "assets/overworld/overworld-127.yaml",
              "strategies": [
                {"kind": "travel_source", "source_id": "0x0055", "route_source_covered": true}
              ]
            }
          ]
        }"#,
    )
    .unwrap();
    write_fake_coverage_runner(&fake_runner);

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(&root)
        .arg("coverage")
        .arg("--route-probes-from-worklist")
        .arg(&worklist)
        .arg("--json")
        .arg(&coverage_log)
        .env("ZELDA3_REPO", &root)
        .env("ZELDA3_C_REPO", &root)
        .env("ZELDA3_NEW_BIN", &fake_runner)
        .env("FAKE_ZELDA3_ARGS_LOG", &args_log)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(args_log).unwrap();
    assert!(args.contains("--coverage-probe\n"), "{args}");
    assert!(args.contains("--coverage-log\n"), "{args}");
    assert!(
        args.contains(&format!("{}\n", coverage_log.display())),
        "{args}"
    );
    assert!(args.contains("--direct-entrance\n130\n"), "{args}");
    assert!(args.contains("--direct-entrance\n56\n"), "{args}");
    assert!(args.contains("--dungeon-room\n45\n"), "{args}");
    assert!(args.contains("--overworld-screen\n90\n"), "{args}");
    assert!(args.contains("--overworld-screen\n127\n"), "{args}");
    assert!(!args.contains("--replay-save\n"), "{args}");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn coverage_cli_passes_load_sram_to_runner() {
    let root = temp_asset_root("coverage-sram-cli");
    std::fs::create_dir_all(root.join("target/parity")).unwrap();
    std::fs::create_dir_all(root.join("saves")).unwrap();
    std::fs::write(root.join("saves/zelda3.sfc"), b"rom").unwrap();
    std::fs::write(root.join("saves/zelda3-combined-route.sav"), b"save").unwrap();
    let fake_runner = root.join("target/parity/zelda3");
    let args_log = root.join("runner-args.txt");
    let coverage_log = root.join("coverage-window.json");
    let input_script = root.join("window-input.txt");
    let load_sram = root.join("window.sram");
    std::fs::write(&input_script, "0 0x0000\n").unwrap();
    std::fs::write(&load_sram, b"sram").unwrap();
    write_fake_coverage_runner(&fake_runner);

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(&root)
        .arg("coverage")
        .arg("--frames")
        .arg("123")
        .arg("--json")
        .arg(&coverage_log)
        .arg("--input-script")
        .arg(&input_script)
        .arg("--load-sram")
        .arg(&load_sram)
        .env("ZELDA3_REPO", &root)
        .env("ZELDA3_C_REPO", &root)
        .env("ZELDA3_NEW_BIN", &fake_runner)
        .env("FAKE_ZELDA3_ARGS_LOG", &args_log)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(args_log).unwrap();
    assert!(args.contains("--input-script\n"), "{args}");
    assert!(
        args.contains(&format!("{}\n", input_script.display())),
        "{args}"
    );
    assert!(args.contains("--load-sram\n"), "{args}");
    assert!(
        args.contains(&format!("{}\n", load_sram.display())),
        "{args}"
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn coverage_cli_passes_stop_replay_after_load_to_runner() {
    let root = temp_asset_root("coverage-stop-replay-cli");
    std::fs::create_dir_all(root.join("target/parity")).unwrap();
    std::fs::create_dir_all(root.join("saves")).unwrap();
    std::fs::write(root.join("saves/zelda3.sfc"), b"rom").unwrap();
    std::fs::write(root.join("saves/zelda3-combined-route.sav"), b"save").unwrap();
    let fake_runner = root.join("target/parity/zelda3");
    let args_log = root.join("runner-args.txt");
    let coverage_log = root.join("coverage-window.json");
    let input_script = root.join("window-input.txt");
    let load_state = root.join("window.sav");
    std::fs::write(&input_script, "0 0x0000\n").unwrap();
    std::fs::write(&load_state, b"state").unwrap();
    write_fake_coverage_runner(&fake_runner);

    let output = Command::new(env!("CARGO_BIN_EXE_zparity"))
        .current_dir(&root)
        .arg("coverage")
        .arg("--frames")
        .arg("123")
        .arg("--json")
        .arg(&coverage_log)
        .arg("--input-script")
        .arg(&input_script)
        .arg("--load-state")
        .arg(&load_state)
        .arg("--stop-replay-after-load")
        .env("ZELDA3_REPO", &root)
        .env("ZELDA3_C_REPO", &root)
        .env("ZELDA3_NEW_BIN", &fake_runner)
        .env("FAKE_ZELDA3_ARGS_LOG", &args_log)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(args_log).unwrap();
    assert!(args.contains("--load-state\n"), "{args}");
    assert!(
        args.contains(&format!("{}\n", load_state.display())),
        "{args}"
    );
    assert!(args.contains("--stop-replay-after-load\n"), "{args}");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn sprite_type_zero_is_valid_when_a_sprite_slot_is_active() {
    let mut coverage = RouteCoverage::default();
    coverage.record(CoverageFrame {
        frame: 1,
        main_module: 0x09,
        submodule: 0,
        subsubmodule: 0,
        indoor_room: None,
        overworld_screen: Some(0),
        sprite_types: vec![0x00],
        ancilla_types: vec![0x00],
        active_item: Some(0x00),
    });

    let report = coverage.report_with_universe(&CoverageUniverse {
        main_modules: vec![0x09],
        module_states: vec![],
        sprite_types: vec![0x00],
        ancilla_types: vec![0x01],
        indoor_rooms: vec![],
        overworld_screens: vec![0],
        active_items: vec![0x01],
    });

    assert_eq!(
        report.category("sprite_types").unwrap().covered,
        vec!["0x00"]
    );
    assert_eq!(
        report.category("ancilla_types").unwrap().covered,
        Vec::<String>::new()
    );
    assert_eq!(
        report.category("active_items").unwrap().covered,
        Vec::<String>::new()
    );
}

fn temp_asset_root(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zelda3-rs-coverage-{name}-{}-{nanos}",
        std::process::id()
    ))
}

fn write_coverage_log(path: &Path, frame: CoverageFrame) {
    let mut coverage = RouteCoverage::default();
    coverage.record(frame);
    std::fs::write(path, serde_json::to_vec(&coverage).unwrap()).unwrap();
}

fn write_complete_coverage_log(path: &Path, indoor_rooms: &[u16], overworld_screens: &[u16]) {
    let mut coverage = RouteCoverage::default();

    for main_module in (0x00..=0x1b).filter(|module| !matches!(module, 0x0c | 0x0d)) {
        coverage.record(CoverageFrame {
            frame: u32::from(main_module) + 1,
            main_module,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: None,
            overworld_screen: None,
            sprite_types: vec![],
            ancilla_types: vec![],
            active_item: None,
        });
    }

    let mut frame = 100;
    for sprite_type in (0x00..=0xf2).filter(|ty| !matches!(ty, 0x03 | 0x2d | 0xb8)) {
        coverage.record(CoverageFrame {
            frame,
            main_module: 0x09,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: None,
            overworld_screen: None,
            sprite_types: vec![sprite_type],
            ancilla_types: vec![],
            active_item: None,
        });
        frame += 1;
    }

    for ancilla_type in (0x01..=0x43).filter(|ty| !matches!(ty, 0x03 | 0x14 | 0x25)) {
        coverage.record(CoverageFrame {
            frame,
            main_module: 0x09,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: None,
            overworld_screen: None,
            sprite_types: vec![],
            ancilla_types: vec![ancilla_type],
            active_item: None,
        });
        frame += 1;
    }

    for active_item in 0x01..=0x14 {
        coverage.record(CoverageFrame {
            frame,
            main_module: 0x09,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: None,
            overworld_screen: None,
            sprite_types: vec![],
            ancilla_types: vec![],
            active_item: Some(active_item),
        });
        frame += 1;
    }

    for &room in indoor_rooms {
        coverage.record(CoverageFrame {
            frame,
            main_module: 0x07,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: Some(room),
            overworld_screen: None,
            sprite_types: vec![],
            ancilla_types: vec![],
            active_item: None,
        });
        frame += 1;
    }

    for &screen in overworld_screens {
        coverage.record(CoverageFrame {
            frame,
            main_module: 0x09,
            submodule: 0,
            subsubmodule: 0,
            indoor_room: None,
            overworld_screen: Some(screen),
            sprite_types: vec![],
            ancilla_types: vec![],
            active_item: None,
        });
        frame += 1;
    }

    std::fs::write(path, serde_json::to_vec(&coverage).unwrap()).unwrap();
}

fn category_hit(report: &serde_json::Value, name: &str) -> Option<(u64, u64)> {
    let category = report["categories"]
        .as_array()?
        .iter()
        .find(|category| category["name"] == name)?;
    Some((category["hit"].as_u64()?, category["expected"].as_u64()?))
}

fn worklist_entry<'a>(
    worklist: &'a serde_json::Value,
    category: &str,
    id: &str,
) -> Option<&'a serde_json::Value> {
    worklist[category]
        .as_array()?
        .iter()
        .find(|entry| entry["id"] == id)
}

fn worklist_strategy<'a>(
    entry: &'a serde_json::Value,
    kind: &str,
) -> Option<&'a serde_json::Value> {
    entry["strategies"]
        .as_array()?
        .iter()
        .find(|strategy| strategy["kind"] == kind)
}

fn report_category_hit(
    report: &parity::coverage::CoverageReport,
    name: &str,
) -> Option<(u64, u64)> {
    let category = report.category(name)?;
    Some((category.hit as u64, category.expected as u64))
}

fn delta_category_newly_covered(report: &serde_json::Value, name: &str) -> Option<Vec<String>> {
    let category = report["categories"]
        .as_array()?
        .iter()
        .find(|category| category["name"] == name)?;
    Some(
        category["newly_covered"]
            .as_array()?
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect(),
    )
}

#[cfg(unix)]
fn write_fake_coverage_runner(path: &Path) {
    std::fs::write(
        path,
        r#"#!/bin/sh
python3 - "$@" <<'PY'
import json
import os
import sys

args = sys.argv[1:]
log_path = os.environ.get("FAKE_ZELDA3_ARGS_LOG")
if log_path:
    with open(log_path, "w") as f:
        for arg in args:
            f.write(arg + "\n")

coverage_path = None
for i, arg in enumerate(args):
    if arg == "--coverage-log" and i + 1 < len(args):
        coverage_path = args[i + 1]
        break

if coverage_path is None:
    sys.exit(2)

with open(coverage_path, "w") as f:
    json.dump({
        "frames": 1,
        "last_frame": 1,
        "main_modules": [9],
        "module_states": [{"main": 9, "sub": 0, "subsub": 0}],
        "indoor_rooms": [],
        "overworld_screens": [0],
        "sprite_types": [],
        "ancilla_types": [],
        "active_items": []
    }, f)
PY
"#,
    )
    .unwrap();

    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).unwrap();
}

#[test]
fn rust_coverage_command_requests_coverage_log_and_audio_advancement() {
    let paths = Paths {
        repo: PathBuf::from("/repo"),
        c_root: PathBuf::from("/c"),
        rom: PathBuf::from("/repo/saves/zelda3.sfc"),
        save: PathBuf::from("/repo/saves/route.sav"),
        rust_bin: PathBuf::from("/repo/target/parity/zelda3"),
        cache_dir: PathBuf::from("/repo/.cache/parity-golden"),
    };

    let cmd = runner::rust_coverage_cmd(&paths, 123, Path::new("/tmp/coverage.json"));
    let args: Vec<String> = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    assert!(args
        .windows(2)
        .any(|w| w == ["--coverage-log", "/tmp/coverage.json"]));
    assert!(args
        .windows(2)
        .any(|w| w == ["--audio-trace-log", "100000000"]));
    assert!(args.contains(&"123".to_string()));
}
