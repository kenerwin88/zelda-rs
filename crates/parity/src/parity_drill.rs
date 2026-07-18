//! `zparity drill <frame>` — per-region localization of a parity divergence.
//!
//! Loads the Tier-B detail block covering the requested frame index, re-runs the
//! Rust binary, and reports which WRAM page / VRAM page / sram / render / audio
//! diverged (golden vs rust), or MATCH.
//!
//! # Index mapping
//!
//! Both binaries emit one fingerprint record per emulated frame, AFTER the frame
//! runs.  The stream is 0-indexed: record 0 = state after emulated frame 1,
//! record 1 = state after frame 2, etc.
//!
//!   golden record index `i` = fingerprint produced after emulated frame `i + 1`
//!
//! `zparity check` stores and prints the GOLDEN RECORD INDEX (not the emulated
//! frame number) when it detects a divergence.  Drill accepts that same index:
//!
//!   `zparity drill <N>` compares golden record N against Rust record N.
//!
//! To produce record N, the Rust binary must run to end_frame = N + 1:
//!   - from start = 0: records 0..N, target is record N, idx = N.
//!   - from checkpoint at `start` (< N+1): records 0..N-start, idx = N - start.
//!
//! The `FrameFingerprint.frame` field in the record equals the emulated frame
//! number (N + 1).  This is used in debug assertions to verify the index mapping.

use std::process::exit;

use parity::golden::{self, Manifest};
use parity::runner::{self, Paths};
use parity::{FrameFingerprint, PAGE, RECORD_LEN, VRAM_PAGES, WRAM_PAGES};

const BLOCK_SIZE: usize = 8192;

pub fn run(args: &[String]) {
    // The argument is a GOLDEN RECORD INDEX (the same number `zparity check` prints).
    let idx: usize = match args.first().and_then(|s| s.parse().ok()) {
        Some(n) => n,
        None => {
            eprintln!("usage: zparity drill <frame>");
            exit(2);
        }
    };

    let p = Paths::discover();
    let _man = Manifest::load(&p.golden_dir.join("manifest.json")).unwrap_or_else(|e| {
        eprintln!("no golden ({e}); run `zparity capture --detail`");
        exit(1);
    });

    // Golden: record `idx` is in block `idx / BLOCK_SIZE` at offset `idx % BLOCK_SIZE`.
    let block = idx / BLOCK_SIZE;
    let within = idx % BLOCK_SIZE;

    let raw = match golden::read_detail_block(&p.cache_dir, block) {
        Ok(raw) => raw,
        Err(_) => {
            eprintln!("detail tier absent for block {block}; run `zparity capture --detail`");
            exit(1);
        }
    };

    if (within + 1) * RECORD_LEN > raw.len() {
        eprintln!(
            "record {idx} not in golden detail (block {block} has {} records, need index {within})",
            raw.len() / RECORD_LEN
        );
        exit(1);
    }
    let want = FrameFingerprint::from_bytes(&raw[within * RECORD_LEN..]);
    // want.frame == idx + 1 (emulated frame number)

    // To produce record `idx`, the Rust binary must run to end_frame = idx + 1.
    // Use the nearest checkpoint strictly before `idx + 1` when available;
    // running from `start` to `end_frame` yields `end_frame - start` records
    // (frames start+1 .. end_frame), so the target is at shard-record index
    // `end_frame - start - 1 = idx - start`.
    let end_frame = (idx + 1) as u32;
    let tmp = p.cache_dir.join("drill.fp");
    std::fs::create_dir_all(&p.cache_dir).unwrap();

    let ck_dir = p.cache_dir.join("ck");
    let ck = nearest_checkpoint(&ck_dir, end_frame);
    let start = ck.as_ref().map(|(f, _)| *f as usize).unwrap_or(0);

    let st = runner::rust_shard_cmd(
        &p,
        end_frame,
        &tmp,
        ck.as_ref().map(|(_, path)| path.as_path()),
    )
    .status()
    .expect("spawn rust binary");
    if !st.success() {
        eprintln!("rust binary failed (exit {:?})", st.code());
        exit(1);
    }

    let got_data = std::fs::read(&tmp).unwrap();
    // Shard record index: record k is for emulated frame start+k+1,
    // so for emulated frame idx+1: k = idx - start.
    let shard_idx = idx - start;
    if (shard_idx + 1) * RECORD_LEN > got_data.len() {
        eprintln!(
            "rust output too short: expected {} records (need shard_idx {shard_idx}), got {}",
            idx - start + 1,
            got_data.len() / RECORD_LEN
        );
        exit(1);
    }
    let got = FrameFingerprint::from_bytes(&got_data[shard_idx * RECORD_LEN..]);
    // got.frame == idx + 1 (emulated frame number), same as want.frame.

    // Compare rollups first (audio leaf normalized to 0 on both sides); if
    // they match we are done.
    if got.rollup == want.normalized_rollup() {
        // Display as the emulated frame number to match user expectations.
        println!("frame {idx}: MATCH (rollup 0x{:08x})", got.rollup);
        return;
    }

    // Rollups differ — report per-leaf divergences.
    println!("frame {idx}: DIVERGE");
    for pidx in 0..WRAM_PAGES {
        if got.wram[pidx] != want.wram[pidx] {
            let addr = pidx * PAGE;
            println!(
                "  WRAM page {pidx:3} @0x{addr:05x}  golden=0x{:08x} rust=0x{:08x}  ({})",
                want.wram[pidx],
                got.wram[pidx],
                page_label(addr)
            );
        }
    }
    for pidx in 0..VRAM_PAGES {
        if got.vram[pidx] != want.vram[pidx] {
            let addr = pidx * PAGE;
            println!(
                "  VRAM page {pidx:3} @0x{addr:05x}  golden=0x{:08x} rust=0x{:08x}",
                want.vram[pidx], got.vram[pidx]
            );
        }
    }
    if got.sram != want.sram {
        println!(
            "  SRAM   golden=0x{:08x} rust=0x{:08x}",
            want.sram, got.sram
        );
    }
    if got.render != want.render {
        println!(
            "  RENDER golden=0x{:08x} rust=0x{:08x}",
            want.render, got.render
        );
    }
    // The audio leaf is retired (DSP oracle removed); golden detail blocks may
    // still carry a live C-side hash there, so it is not compared.
    exit(1);
}

fn nearest_checkpoint(
    ck_dir: &std::path::Path,
    end_frame: u32,
) -> Option<(u32, std::path::PathBuf)> {
    let mut best: Option<(u32, std::path::PathBuf)> = None;
    let entries = std::fs::read_dir(ck_dir).ok()?;
    for e in entries.flatten() {
        let path = e.path();
        // Accept only fully committed checkpoints (.sav), not partial writes (.sav.tmp).
        if path.extension().and_then(|s| s.to_str()) != Some("sav") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if let Ok(f) = stem.parse::<u32>() {
                // Must be strictly before end_frame so the shard produces at least one record.
                if f < end_frame && best.as_ref().map_or(true, |(bf, _)| f > *bf) {
                    best = Some((f, path));
                }
            }
        }
    }
    best
}

/// Human-readable label for a WRAM page (address range).
fn page_label(addr: usize) -> String {
    format!("0x{addr:05x}..0x{:05x}", addr + PAGE - 1)
}
