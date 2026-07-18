use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::exit;

use parity::coverage::{CoverageReport, CoverageUniverse, RouteCoverage, RouteCoverageWorklist};
use parity::runner::{self, Paths};

pub fn run(args: &[String]) {
    let p = Paths::discover();
    let from_json_paths = path_args(args, "--from-json");
    let coverage_path_arg = path_arg(args, "--json");
    let coverage_path = coverage_path_arg
        .clone()
        .unwrap_or_else(|| p.cache_dir.join("coverage.json"));
    let input_script = path_arg(args, "--input-script");
    let input_script_overlay = path_arg(args, "--input-script-overlay");
    let load_state = path_arg(args, "--load-state");
    let load_sram = path_arg(args, "--load-sram");
    let stop_replay_after_load = args.iter().any(|arg| arg == "--stop-replay-after-load");
    let diff_from_json = path_arg(args, "--diff-from-json");
    let delta_report_path = path_arg(args, "--delta-report-json");
    let route_probes_from_worklist = path_arg(args, "--route-probes-from-worklist");
    if delta_report_path.is_some() && diff_from_json.is_none() {
        eprintln!("--delta-report-json requires --diff-from-json");
        exit(2);
    }
    if route_probes_from_worklist.is_some() && !from_json_paths.is_empty() {
        eprintln!("--route-probes-from-worklist cannot be combined with --from-json");
        exit(2);
    }
    if load_state.is_some() && load_sram.is_some() {
        eprintln!(
            "--load-sram cannot be combined with --load-state; checkpoints already include SRAM"
        );
        exit(2);
    }

    let coverage = if let Some(worklist_path) = route_probes_from_worklist.as_ref() {
        let probe_targets = route_probe_targets_from_worklist(worklist_path);
        if let Some(parent) = coverage_path.parent() {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!(
                    "failed to create coverage directory {}: {e}",
                    parent.display()
                );
                exit(1);
            });
        }
        if probe_targets.is_empty() {
            eprintln!(
                "zparity coverage: no route probes found in {}",
                worklist_path.display()
            );
            write_json_report(
                &coverage_path,
                &RouteCoverage::default(),
                "coverage probe log",
            );
        } else {
            eprintln!(
                "zparity coverage: running {} direct entrance probes, {} dungeon room probes, and {} overworld screen probes from {}",
                probe_targets.direct_entrance_indices.len(),
                probe_targets.dungeon_rooms.len(),
                probe_targets.overworld_screens.len(),
                worklist_path.display()
            );
            let status = runner::rust_direct_entrance_probe_cmd(
                &p,
                &coverage_path,
                &probe_targets.direct_entrance_indices,
                &probe_targets.dungeon_rooms,
                &probe_targets.overworld_screens,
            )
            .status()
            .expect("spawn Rust coverage probes");
            if !status.success() {
                eprintln!("Rust coverage probes failed: {status}");
                exit(1);
            }
        }
        load_coverage(&coverage_path)
    } else if from_json_paths.is_empty() {
        let frames = super::parse_frames(args);
        if let Some(parent) = coverage_path.parent() {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!(
                    "failed to create coverage directory {}: {e}",
                    parent.display()
                );
                exit(1);
            });
        }
        eprintln!("zparity coverage: running Rust route over {frames} frames");
        let options = runner::CoverageRunOptions {
            input_script: input_script.as_deref(),
            input_script_overlay: input_script_overlay.as_deref(),
            load_state: load_state.as_deref(),
            load_sram: load_sram.as_deref(),
            stop_replay_after_load,
        };
        let status = runner::rust_coverage_cmd_with_options(&p, frames, &coverage_path, &options)
            .status()
            .expect("spawn Rust coverage route");
        if !status.success() {
            eprintln!("Rust coverage route failed: {status}");
            exit(1);
        }
        load_coverage(&coverage_path)
    } else {
        load_merged_coverage(&from_json_paths)
    };
    let universe = CoverageUniverse::standard();
    let report = coverage.report_with_universe(&universe);
    if !from_json_paths.is_empty() {
        if let Some(coverage_path) = coverage_path_arg.as_ref() {
            write_json_report(coverage_path, &coverage, "merged coverage log");
        }
    }
    if let Some(report_path) = path_arg(args, "--report-json") {
        write_json_report(&report_path, &report, "coverage report");
    }
    let route_report = if path_arg(args, "--route-report-json").is_some()
        || args.iter().any(|arg| arg == "--require-route-full")
        || path_arg(args, "--route-worklist-json").is_some()
    {
        Some(coverage.route_evidence_report_with_universe(&universe))
    } else {
        None
    };
    if let (Some(route_report_path), Some(route_report)) =
        (path_arg(args, "--route-report-json"), route_report.as_ref())
    {
        write_json_report(&route_report_path, route_report, "route coverage report");
    }
    if let Some(route_worklist_path) = path_arg(args, "--route-worklist-json") {
        let route_worklist = coverage.route_worklist_with_universe(&universe);
        write_json_report(
            &route_worklist_path,
            &route_worklist,
            "route coverage worklist",
        );
    }
    print!("{}", report.to_text());
    if let Some(base_path) = diff_from_json.as_ref() {
        let base_report = load_coverage(base_path).report_with_universe(&universe);
        let delta = report.delta_from(&base_report);
        println!("{}", delta.to_text(&base_path.display().to_string()));
        if let Some(delta_report_path) = delta_report_path.as_ref() {
            write_json_report(delta_report_path, &delta, "coverage delta report");
        }
    }
    if from_json_paths.is_empty() {
        eprintln!("coverage log: {}", coverage_path.display());
    } else {
        eprintln!(
            "coverage logs: {}",
            from_json_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        if let Some(coverage_path) = coverage_path_arg.as_ref() {
            eprintln!("merged coverage log: {}", coverage_path.display());
        }
    }
    if args.iter().any(|arg| arg == "--require-full") {
        require_full_coverage(&report, "coverage");
    }
    if args.iter().any(|arg| arg == "--require-route-full") {
        require_full_coverage(route_report.as_ref().unwrap(), "route coverage");
    }
}

fn write_json_report<T: serde::Serialize>(path: &PathBuf, report: &T, label: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!(
                "failed to create {label} directory {}: {e}",
                parent.display()
            );
            exit(1);
        });
    }
    let json = serde_json::to_vec_pretty(report).unwrap_or_else(|e| {
        eprintln!("failed to encode {label}: {e}");
        exit(1);
    });
    std::fs::write(path, json).unwrap_or_else(|e| {
        eprintln!("failed to write {label} {}: {e}", path.display());
        exit(1);
    });
}

fn load_coverage(path: &PathBuf) -> RouteCoverage {
    let data = std::fs::read(path).unwrap_or_else(|e| {
        eprintln!("failed to read coverage log {}: {e}", path.display());
        exit(1);
    });
    serde_json::from_slice(&data).unwrap_or_else(|e| {
        eprintln!("failed to parse coverage log {}: {e}", path.display());
        exit(1);
    })
}

fn load_merged_coverage(paths: &[PathBuf]) -> RouteCoverage {
    let mut merged = RouteCoverage::default();
    for path in paths {
        merged.merge(&load_coverage(path));
    }
    merged
}

struct RouteProbeTargets {
    direct_entrance_indices: Vec<u16>,
    dungeon_rooms: Vec<u16>,
    overworld_screens: Vec<u16>,
}

impl RouteProbeTargets {
    fn is_empty(&self) -> bool {
        self.direct_entrance_indices.is_empty()
            && self.dungeon_rooms.is_empty()
            && self.overworld_screens.is_empty()
    }
}

fn route_probe_targets_from_worklist(path: &PathBuf) -> RouteProbeTargets {
    let data = std::fs::read(path).unwrap_or_else(|e| {
        eprintln!("failed to read route worklist {}: {e}", path.display());
        exit(1);
    });
    let worklist: RouteCoverageWorklist = serde_json::from_slice(&data).unwrap_or_else(|e| {
        eprintln!("failed to parse route worklist {}: {e}", path.display());
        exit(1);
    });
    let mut indices = BTreeSet::new();
    let mut rooms = BTreeSet::new();
    let mut screens = BTreeSet::new();
    for entry in &worklist.indoor_rooms {
        let mut has_direct_entrance = false;
        for strategy in &entry.strategies {
            if strategy.kind == "direct_entrance" {
                if let Some(entrance_index) = strategy.entrance_index {
                    indices.insert(entrance_index);
                    has_direct_entrance = true;
                }
            }
        }
        if !has_direct_entrance {
            if let Some(room) = parse_worklist_id_u16(&entry.id) {
                rooms.insert(room);
            }
        }
    }
    for entry in &worklist.overworld_screens {
        if let Some(screen) = parse_worklist_id_u16(&entry.id) {
            screens.insert(screen);
        }
    }
    RouteProbeTargets {
        direct_entrance_indices: indices.into_iter().collect(),
        dungeon_rooms: rooms.into_iter().collect(),
        overworld_screens: screens.into_iter().collect(),
    }
}

fn parse_worklist_id_u16(id: &str) -> Option<u16> {
    u16::from_str_radix(id.strip_prefix("0x")?, 16).ok()
}

fn require_full_coverage(report: &CoverageReport, label: &str) {
    let mut missing = false;
    for category in &report.categories {
        if category.missed.is_empty() {
            continue;
        }
        missing = true;
        eprintln!(
            "{label} incomplete: {} missed {}/{}",
            category.name,
            category.missed.len(),
            category.expected
        );
    }
    if missing {
        exit(1);
    }
}

fn path_arg(args: &[String], flag: &str) -> Option<PathBuf> {
    let i = args.iter().position(|arg| arg == flag)?;
    let value = args.get(i + 1).unwrap_or_else(|| {
        eprintln!("{flag} requires a path");
        exit(2);
    });
    Some(PathBuf::from(value))
}

fn path_args(args: &[String], flag: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == flag {
            let value = args.next().unwrap_or_else(|| {
                eprintln!("{flag} requires a path");
                exit(2);
            });
            paths.push(PathBuf::from(value));
        }
    }
    paths
}
