use std::process::exit;

use parity::golden::{self, Manifest, SCHEMA};
use parity::merkle::MerkleIndex;
use parity::runner::{self, Paths};
use parity::{FrameFingerprint, RECORD_LEN, FINGERPRINT_MASK};

const FULL_ROUTE_FRAMES: u32 = 1_073_092;
const BLOCK_SIZE: u32 = 8192;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("capture") => cmd_capture(&args[1..]),
        Some("check") => parity_check::run(&args[1..]),   // Task 6
        Some("drill") => parity_drill::run(&args[1..]),   // Task 7
        _ => {
            eprintln!("usage: zparity <capture|check|drill> [options]");
            exit(2);
        }
    }
}

fn parse_frames(args: &[String]) -> u32 {
    if args.iter().any(|a| a == "--full") {
        return FULL_ROUTE_FRAMES;
    }
    if let Some(i) = args.iter().position(|a| a == "--frames") {
        return args[i + 1].parse().expect("--frames N");
    }
    3000
}

fn cmd_capture(args: &[String]) {
    let p = Paths::discover();
    let frames = parse_frames(args);
    let detail = args.iter().any(|a| a == "--detail");
    if !p.c_root.join("zelda3").exists() {
        eprintln!("C oracle binary missing: {:?} (build: make -C {:?} zelda3)", p.c_root.join("zelda3"), p.c_root);
        exit(1);
    }
    let tmp = p.cache_dir.join("capture.fp");
    std::fs::create_dir_all(&p.cache_dir).unwrap();
    eprintln!("zparity capture: running C oracle over {frames} frames");
    let status = runner::c_capture_cmd(&p, frames, &tmp).status().expect("spawn C oracle");
    if !status.success() {
        eprintln!("C oracle failed: {status}");
        exit(1);
    }
    let data = std::fs::read(&tmp).unwrap();
    let n = data.len() / RECORD_LEN;
    eprintln!("captured {n} frames");
    // Rollup column + merkle.
    let rollups: Vec<u32> = (0..n)
        .map(|i| FrameFingerprint::from_bytes(&data[i * RECORD_LEN..]).rollup)
        .collect();
    std::fs::create_dir_all(&p.golden_dir).unwrap();
    golden::write_rollup(&p.golden_dir.join("rollup.bin"), &rollups).unwrap();
    let merkle = MerkleIndex::build(&rollups, BLOCK_SIZE);
    std::fs::write(p.golden_dir.join("merkle.bin"), merkle.to_bytes()).unwrap();
    // Manifest.
    let oracle_rev = std::process::Command::new("git")
        .current_dir(&p.c_root).args(["rev-parse", "HEAD"]).output()
        .ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let man = Manifest {
        schema: SCHEMA, frames: n as u32,
        rom_sha256: runner::sha256_file(&p.rom).unwrap(),
        save_sha256: runner::sha256_file(&p.save).unwrap(),
        c_oracle_rev: oracle_rev,
        timing_hacks: runner::HACK_KEYS.iter().map(|s| s.to_string()).collect(),
        mask: FINGERPRINT_MASK.to_vec(), block_size: BLOCK_SIZE, page_kb: 1,
    };
    man.save(&p.golden_dir.join("manifest.json")).unwrap();
    // Tier B (optional).
    if detail {
        let blocks = n.div_ceil(BLOCK_SIZE as usize);
        for b in 0..blocks {
            let start = b * BLOCK_SIZE as usize * RECORD_LEN;
            let end = ((b + 1) * BLOCK_SIZE as usize * RECORD_LEN).min(data.len());
            golden::write_detail_block(&p.cache_dir, b, &data[start..end]).unwrap();
        }
        eprintln!("wrote {blocks} detail blocks to {:?}", p.cache_dir.join("detail"));
    }
    let _ = std::fs::remove_file(&tmp);
    eprintln!("golden written: {:?} (root=0x{:08x})", p.golden_dir, merkle.root);
}

// Task 6 / Task 7 provide these modules.
mod parity_check;
mod parity_drill;
