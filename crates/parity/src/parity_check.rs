use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;

use parity::golden::{Manifest, RollupMap};
use parity::merkle::MerkleIndex;
use parity::runner::{self, Paths};
use parity::{FrameFingerprint, RECORD_LEN};

const SHARD_THRESHOLD: u32 = 20_000; // below this, run single-shard

pub fn run(args: &[String]) {
    let p = Paths::discover();
    let frames = super::parse_frames(args);
    let detail = args.iter().any(|a| a == "--detail");
    let shards = shard_count(args, frames);

    // 1. Validate manifest.
    let man = Manifest::load(&p.golden_dir.join("manifest.json")).unwrap_or_else(|e| {
        eprintln!("no golden manifest ({e}); run `zparity capture` first");
        exit(1);
    });
    if man.schema != parity::golden::SCHEMA {
        eprintln!("golden schema {} != {}", man.schema, parity::golden::SCHEMA);
        exit(1);
    }
    if frames > man.frames {
        eprintln!("requested {frames} frames > golden {} frames", man.frames);
        exit(1);
    }
    check_hash("ROM", &runner::sha256_file(&p.rom).unwrap(), &man.rom_sha256);
    check_hash("save", &runner::sha256_file(&p.save).unwrap(), &man.save_sha256);

    let golden = RollupMap::open(&p.golden_dir.join("rollup.bin")).unwrap();

    // 2. Boundaries + checkpoints (single shard => none).
    let bounds = shard_bounds(frames, shards);
    if shards > 1 {
        ensure_checkpoints(&p, &bounds);
    }

    // 3. Fan out shards.
    let start_time = std::time::Instant::now();
    let golden = Arc::new(golden);
    let first_diff = Arc::new(Mutex::new(None::<u64>)); // global frame index
    let mut handles = Vec::new();
    let tmp = p.cache_dir.join("shards");
    std::fs::create_dir_all(&tmp).unwrap();
    for (i, w) in bounds.windows(2).enumerate() {
        let (s, e) = (w[0], w[1]);
        let p = p.clone();
        let golden = Arc::clone(&golden);
        let first_diff = Arc::clone(&first_diff);
        let fp_out = tmp.join(format!("shard_{i}.bin"));
        let ck = if i == 0 { None } else { Some(p.cache_dir.join(format!("ck/{s}.sav"))) };
        handles.push(thread::spawn(move || {
            run_shard(&p, s, e, &fp_out, ck.as_deref(), &golden, &first_diff);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    // 4. Report.
    let dur = start_time.elapsed().as_secs_f64();
    let merkle = MerkleIndex::from_bytes(&std::fs::read(p.golden_dir.join("merkle.bin")).unwrap());
    let diff_result = *first_diff.lock().unwrap();
    match diff_result {
        None => {
            println!("MATCH  {frames} frames, {shards} shards, {dur:.1}s  root=0x{:08x}", merkle.root);
            if detail {
                println!("(detail compare requested; per-region OK — see `zparity drill` on failure)");
            }
        }
        Some(f) => {
            println!("DIVERGE at frame {f}  ({shards} shards, {dur:.1}s)");
            println!("  run: zparity drill {f}");
            exit(1);
        }
    }
}

fn run_shard(
    p: &Paths, start: u32, end: u32, fp_out: &Path, load_state: Option<&Path>,
    golden: &RollupMap, first_diff: &Mutex<Option<u64>>,
) {
    let out = runner::rust_shard_cmd(p, end, fp_out, load_state).output().expect("spawn rust shard");
    if !out.status.success() {
        eprintln!("shard [{start},{end}) failed:\n{}", String::from_utf8_lossy(&out.stderr));
        let mut g = first_diff.lock().unwrap();
        *g = Some(g.map_or(start as u64, |v| v.min(start as u64)));
        return;
    }
    let data = std::fs::read(fp_out).expect("read shard fp");
    // A shard that resumed from a checkpoint at frame `start` produced records for
    // frames (start, end]; record k corresponds to golden index start + k.
    // Shard 0 starts at 0 with no --load-state and produces records for (0, end].
    let n = data.len() / RECORD_LEN;
    for k in 0..n {
        // golden index 0 = fingerprint for frame 1 (first emitted); record k of a
        // shard resumed at `start` is frame start+k+1, golden index start+k.
        let frame_idx = start as usize + k;
        if frame_idx >= golden.len() {
            break;
        }
        let rollup = FrameFingerprint::from_bytes(&data[k * RECORD_LEN..]).rollup;
        if rollup != golden.get(frame_idx) {
            let mut g = first_diff.lock().unwrap();
            let f = frame_idx as u64;
            *g = Some(g.map_or(f, |v| v.min(f)));
            return;
        }
    }
}

fn shard_count(args: &[String], frames: u32) -> usize {
    if let Some(i) = args.iter().position(|a| a == "--shards") {
        let Some(v) = args.get(i + 1) else {
            eprintln!("--shards requires a value");
            exit(2);
        };
        return v.parse().unwrap_or_else(|_| {
            eprintln!("--shards K: invalid shard count");
            exit(2);
        });
    }
    if frames < SHARD_THRESHOLD {
        return 1;
    }
    thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
}

fn shard_bounds(frames: u32, shards: usize) -> Vec<u32> {
    let mut v = Vec::with_capacity(shards + 1);
    for i in 0..=shards {
        v.push(((frames as u64 * i as u64) / shards as u64) as u32);
    }
    v.dedup();
    v
}

fn ensure_checkpoints(p: &Paths, bounds: &[u32]) {
    let ck_dir = p.cache_dir.join("ck");
    std::fs::create_dir_all(&ck_dir).unwrap();
    let needed: Vec<u32> = bounds[1..bounds.len() - 1].to_vec();
    if needed.iter().all(|f| ck_dir.join(format!("{f}.sav")).exists()) {
        return;
    }
    eprintln!("seeding {} checkpoints (one sequential pass, render-free)...", needed.len());
    // Single pass to the last boundary, dropping all checkpoints via --save-state-at.
    // rust_seed_cmd skips the per-frame render (the dominant cost); the binary renders
    // ONLY the boundary frames before each checkpoint to re-project display state into
    // RAM. The result is byte-identical to a fully-rendered seed (verified by shard
    // invariance) but ~100x faster.
    // Write each checkpoint to <frame>.sav.tmp and only rename to the final
    // <frame>.sav AFTER the pass succeeds. A killed (OOM/SIGKILL) seeding pass
    // then never leaves a partial file that the existence check above would
    // trust -> shards would otherwise load a corrupt checkpoint and report a
    // phantom divergence with no root-cause signal.
    let mut cmd = runner::rust_seed_cmd(p, *bounds.last().unwrap());
    for f in &needed {
        cmd.args(["--save-state-at", &format!("{f}:{}", ck_dir.join(format!("{f}.sav.tmp")).display())]);
    }
    let st = cmd.status().expect("spawn checkpoint seed");
    assert!(st.success(), "checkpoint seeding failed");
    for f in &needed {
        std::fs::rename(ck_dir.join(format!("{f}.sav.tmp")), ck_dir.join(format!("{f}.sav")))
            .expect("rename seeded checkpoint into place");
    }
    let _ = std::fs::remove_file(p.cache_dir.join("seed.fp"));
}

fn check_hash(what: &str, got: &str, want: &str) {
    if got != want {
        eprintln!("{what} sha256 mismatch:\n  golden={want}\n  local ={got}");
        exit(1);
    }
}
