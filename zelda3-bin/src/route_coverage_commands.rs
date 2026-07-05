use std::path::{Path, PathBuf};
use std::process;

use zelda3::ZeldaState;

use crate::{
    load_translated_replay_state, parse_u16_auto, read_le_u16, PLAYER_IS_INDOORS,
    TRACE_MAIN_MODULE_INDEX, TRACE_SUBMODULE_INDEX, TRACE_SUBSUBMODULE_INDEX,
};

pub(crate) fn route_coverage_frame_from_game(
    frame: u32,
    game: &ZeldaState,
) -> parity::coverage::CoverageFrame {
    let indoors = game.ram[PLAYER_IS_INDOORS] != 0;
    let sprite_types = (0..16)
        .filter(|&k| game.ram[0x0dd0 + k] != 0)
        .map(|k| game.ram[0x0e20 + k])
        .collect();
    let ancilla_types = (0..10)
        .map(|k| game.ram[0x0c4a + k])
        .filter(|&ty| ty != 0)
        .collect();
    parity::coverage::CoverageFrame {
        frame,
        main_module: game.ram[TRACE_MAIN_MODULE_INDEX],
        submodule: game.ram[TRACE_SUBMODULE_INDEX],
        subsubmodule: game.ram[TRACE_SUBSUBMODULE_INDEX],
        indoor_room: indoors.then(|| read_le_u16(&game.ram, 0x48e)),
        overworld_screen: (!indoors).then(|| read_le_u16(&game.ram, 0x8a)),
        sprite_types,
        ancilla_types,
        active_item: (game.ram[0x0202] != 0).then_some(game.ram[0x0202]),
    }
}

pub(crate) fn write_route_coverage_log_or_exit(
    path: &Path,
    coverage: &parity::coverage::RouteCoverage,
    label: &str,
) {
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!(
                "failed to create {label} directory {}: {e}",
                parent.display()
            );
            process::exit(1);
        }
    }
    let json = serde_json::to_vec_pretty(coverage).unwrap_or_else(|e| {
        eprintln!("failed to encode {label}: {e}");
        process::exit(1);
    });
    if let Err(e) = std::fs::write(path, json) {
        eprintln!("failed to write {label} {}: {e}", path.display());
        process::exit(1);
    }
}

pub(crate) fn run_coverage_probe(args: &[String]) {
    let Some(rom_path) = args.first() else {
        eprintln!(
            "usage: zelda3 --coverage-probe <path-to-rom.sfc> --coverage-log <path> [--direct-entrance <index>]..."
        );
        process::exit(2);
    };
    let mut coverage_log = None::<PathBuf>;
    let mut direct_entrances = Vec::<u16>::new();
    let mut dungeon_rooms = Vec::<u16>::new();
    let mut overworld_screens = Vec::<u16>::new();
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--coverage-log" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--coverage-log requires a path");
                    process::exit(2);
                };
                coverage_log = Some(PathBuf::from(path));
                i += 2;
            }
            "--direct-entrance" => {
                let Some(index) = args.get(i + 1) else {
                    eprintln!("--direct-entrance requires an index");
                    process::exit(2);
                };
                let entrance = parse_u16_auto(index).unwrap_or_else(|| {
                    eprintln!("invalid --direct-entrance index: {index}");
                    process::exit(2);
                });
                direct_entrances.push(entrance);
                i += 2;
            }
            "--dungeon-room" => {
                let Some(index) = args.get(i + 1) else {
                    eprintln!("--dungeon-room requires an index");
                    process::exit(2);
                };
                let room = parse_u16_auto(index).unwrap_or_else(|| {
                    eprintln!("invalid --dungeon-room index: {index}");
                    process::exit(2);
                });
                dungeon_rooms.push(room);
                i += 2;
            }
            "--overworld-screen" => {
                let Some(index) = args.get(i + 1) else {
                    eprintln!("--overworld-screen requires an index");
                    process::exit(2);
                };
                let screen = parse_u16_auto(index).unwrap_or_else(|| {
                    eprintln!("invalid --overworld-screen index: {index}");
                    process::exit(2);
                });
                overworld_screens.push(screen);
                i += 2;
            }
            flag => {
                eprintln!("unknown --coverage-probe option: {flag}");
                process::exit(2);
            }
        }
    }
    let Some(coverage_log) = coverage_log else {
        eprintln!("--coverage-log is required");
        process::exit(2);
    };

    let base = load_translated_replay_state(rom_path);
    let mut coverage = parity::coverage::RouteCoverage::default();
    for (index, entrance) in direct_entrances.iter().copied().enumerate() {
        let mut game = base.clone();
        let room = game.parity_probe_direct_entrance(entrance);
        coverage.record(route_coverage_frame_from_game(index as u32 + 1, &game));
        println!("coverage-probe direct-entrance entrance=0x{entrance:04x} room=0x{room:04x}");
    }
    let dungeon_frame_base = direct_entrances.len() as u32 + 1;
    for (index, room) in dungeon_rooms.iter().copied().enumerate() {
        let mut game = base.clone();
        let loaded_room = game.parity_probe_dungeon_room(room);
        coverage.record(route_coverage_frame_from_game(
            dungeon_frame_base + index as u32,
            &game,
        ));
        println!("coverage-probe dungeon-room requested=0x{room:04x} room=0x{loaded_room:04x}");
    }
    let frame_base = direct_entrances.len() as u32 + dungeon_rooms.len() as u32 + 1;
    for (index, screen) in overworld_screens.iter().copied().enumerate() {
        let mut game = base.clone();
        let loaded_screen = game.parity_probe_overworld_screen(screen);
        coverage.record(route_coverage_frame_from_game(
            frame_base + index as u32,
            &game,
        ));
        println!(
            "coverage-probe overworld-screen requested=0x{screen:04x} screen=0x{loaded_screen:04x}"
        );
    }
    write_route_coverage_log_or_exit(&coverage_log, &coverage, "coverage probe log");
}
