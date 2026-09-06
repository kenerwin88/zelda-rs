use std::process::exit;
use std::{collections::BTreeSet, path::PathBuf, str::FromStr};

use parity::av;
use parity::cold_evidence;
use parity::evidence::{self, TraceQuery};
use parity::receipts;

mod parity_coverage;

fn parse_frames(args: &[String]) -> u32 {
    if let Some(i) = args.iter().position(|a| a == "--frames") {
        let Some(v) = args.get(i + 1) else {
            eprintln!("--frames requires a value");
            exit(2);
        };
        return v.parse().expect("--frames N");
    }
    3000
}

fn option(args: &[String], name: &str) -> Option<String> {
    let index = args.iter().position(|argument| argument == name)?;
    args.get(index + 1).cloned().or_else(|| {
        eprintln!("{name} requires a value");
        exit(2);
    })
}

fn required_path(args: &[String], index: usize, usage: &str) -> PathBuf {
    args.get(index).map(PathBuf::from).unwrap_or_else(|| {
        eprintln!("usage: {usage}");
        exit(2);
    })
}

fn parse_option<T: FromStr>(args: &[String], name: &str) -> Option<T> {
    option(args, name).map(|value| {
        value.parse().unwrap_or_else(|_| {
            eprintln!("invalid {name} value: {value}");
            exit(2);
        })
    })
}

fn validate_options(
    args: &[String],
    positional: usize,
    value_options: &[&str],
    switch_options: &[&str],
    repeatable: &[&str],
) {
    if args.len() < positional {
        return;
    }
    let mut seen = BTreeSet::new();
    let mut index = positional;
    while index < args.len() {
        let option = args[index].as_str();
        if switch_options.contains(&option) {
            if !seen.insert(option.to_string()) {
                eprintln!("duplicate option: {option}");
                exit(2);
            }
            index += 1;
            continue;
        }
        if value_options.contains(&option) {
            if index + 1 >= args.len() {
                eprintln!("{option} requires a value");
                exit(2);
            }
            if !repeatable.contains(&option) && !seen.insert(option.to_string()) {
                eprintln!("duplicate option: {option}");
                exit(2);
            }
            index += 2;
            continue;
        }
        eprintln!(
            "unknown option or extra positional argument: {}",
            args[index]
        );
        exit(2);
    }
}

fn trace_index(args: &[String]) {
    validate_options(args, 1, &["--manifest", "--output"], &[], &[]);
    let trace = required_path(
        args,
        0,
        "zparity trace-index TRACE --manifest MANIFEST --output INDEX",
    );
    let manifest = option(args, "--manifest")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("trace-index requires --manifest MANIFEST");
            exit(2);
        });
    let output = option(args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("trace-index requires --output INDEX");
            exit(2);
        });
    let header = evidence::build_trace_index(&trace, &manifest, &output).unwrap_or_else(|error| {
        eprintln!("zparity trace-index: {error}");
        exit(1);
    });
    println!(
        "indexed {} record(s), {} byte source -> {}",
        header.records,
        header.source_bytes,
        output.display()
    );
    println!("source sha256 {}", header.source_sha256);
    println!(
        "coordinate: host_frame = {} + retro_run",
        header.comparison_start_frame
    );
}

fn trace_query(args: &[String]) {
    validate_options(
        args,
        1,
        &[
            "--host-frame",
            "--run",
            "--internal-frame",
            "--pc",
            "--wram",
            "--event",
            "--limit",
        ],
        &[],
        &["--event"],
    );
    let index = required_path(args, 0, "zparity trace-query INDEX [filters]");
    let pc = option(args, "--pc").map(|value| {
        evidence::parse_pc(&value).unwrap_or_else(|error| {
            eprintln!("zparity trace-query: {error}");
            exit(2);
        })
    });
    let wram = option(args, "--wram").map(|value| {
        evidence::parse_wram_range(&value).unwrap_or_else(|error| {
            eprintln!("zparity trace-query: {error}");
            exit(2);
        })
    });
    let events = args
        .windows(2)
        .filter(|window| window[0] == "--event")
        .map(|window| window[1].clone())
        .collect();
    let query = TraceQuery {
        host_frame: parse_option(args, "--host-frame"),
        run: parse_option(args, "--run"),
        internal_frame: parse_option(args, "--internal-frame"),
        pc,
        wram,
        events,
        limit: parse_option(args, "--limit"),
    };
    if query.limit == Some(0) {
        eprintln!("--limit must be greater than zero");
        exit(2);
    }
    let (_, matched) = evidence::query_trace_index(&index, &query, &mut std::io::stdout())
        .unwrap_or_else(|error| {
            eprintln!("zparity trace-query: {error}");
            exit(1);
        });
    eprintln!("zparity trace-query: {matched} matching record(s)");
}

fn cache_verify(args: &[String]) {
    validate_options(args, 1, &[], &["--json"], &[]);
    let root = required_path(args, 0, "zparity cache-verify CACHE_ROOT [--json]");
    let inventory = evidence::verify_oracle_cache_root(&root).unwrap_or_else(|error| {
        eprintln!("zparity cache-verify: {error}");
        exit(1);
    });
    if args.iter().any(|argument| argument == "--json") {
        println!("{}", serde_json::to_string(&inventory).unwrap());
    } else {
        println!(
            "verified {} cache entry/entries, {} artifact(s), {} byte(s)",
            inventory.entries, inventory.artifacts, inventory.bytes
        );
    }
}

fn receipt_compare(args: &[String]) {
    validate_options(
        args,
        2,
        &["--max-differing-frames", "--max-differences-per-frame"],
        &["--json", "--allow-incomplete"],
        &[],
    );
    let candidate = required_path(
        args,
        0,
        "zparity receipt-compare CANDIDATE_JSONL ORACLE_JSONL [options]",
    );
    let oracle = required_path(
        args,
        1,
        "zparity receipt-compare CANDIDATE_JSONL ORACLE_JSONL [options]",
    );
    let report = receipts::compare_receipts(
        &candidate,
        &oracle,
        parse_option(args, "--max-differing-frames").unwrap_or(16),
        parse_option(args, "--max-differences-per-frame").unwrap_or(64),
    )
    .unwrap_or_else(|error| {
        eprintln!("zparity receipt-compare: {error}");
        exit(2);
    });
    if args.iter().any(|argument| argument == "--json") {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!(
            "{}: {} candidate receipt(s), {} oracle receipt(s), {} paired",
            report.status,
            report.candidate_records,
            report.oracle_records,
            report.coverage.paired_frames
        );
        println!(
            "domains: engine={} input={} audio-boundary={} vram-words={} vram-hash={} unavailable-vram-hash={}",
            report.coverage.engine_frames,
            report.coverage.input_frames,
            report.coverage.audio_boundary_frames,
            report.coverage.vram_word_count_frames,
            report.coverage.vram_hash_frames,
            report.coverage.vram_hash_unavailable_frames,
        );
        println!(
            "recorded-frame coverage: {:?}..{:?} contiguous={}",
            report.coverage.first_frame, report.coverage.last_frame, report.coverage.contiguous,
        );
        for frame in &report.differing_frames {
            println!("frame {}:", frame.frame);
            for difference in &frame.differences {
                println!(
                    "  {} rust={} oracle={}",
                    difference.path, difference.rust, difference.oracle
                );
            }
        }
        if report.differing_frames_truncated {
            println!("additional differing frames omitted");
        }
    }
    if !report.matched {
        exit(1);
    }
    if !report.complete && !args.iter().any(|argument| argument == "--allow-incomplete") {
        eprintln!(
            "zparity receipt-compare: available semantic domains match, but evidence is incomplete; pass --allow-incomplete only for diagnostics"
        );
        exit(3);
    }
}

fn av_compare(args: &[String]) {
    validate_options(
        args,
        2,
        &["--max-differing-frames"],
        &[
            "--json",
            "--allow-incomplete",
            "--candidate-stopped-at-first-mismatch",
        ],
        &[],
    );
    let candidate = required_path(
        args,
        0,
        "zparity av-compare CANDIDATE_JSONL ORACLE_JSONL [options]",
    );
    let oracle = required_path(
        args,
        1,
        "zparity av-compare CANDIDATE_JSONL ORACLE_JSONL [options]",
    );
    let report = av::compare_av_ledgers(
        &candidate,
        &oracle,
        parse_option(args, "--max-differing-frames").unwrap_or(16),
        args.iter()
            .any(|argument| argument == "--candidate-stopped-at-first-mismatch"),
    )
    .unwrap_or_else(|error| {
        eprintln!("zparity av-compare: {error}");
        exit(2);
    });
    if args.iter().any(|argument| argument == "--json") {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!(
            "{}: {} candidate record(s), {} oracle record(s), {} paired",
            report.status,
            report.candidate_records,
            report.oracle_records,
            report.coverage.paired_frames
        );
        println!(
            "coverage: frames={:?}..{:?} contiguous={} video={} audio={} no-enabled-lane={}",
            report.coverage.first_frame,
            report.coverage.last_frame,
            report.coverage.contiguous,
            report.coverage.video_frames,
            report.coverage.audio_frames,
            report.coverage.frames_without_enabled_lanes,
        );
        for frame in &report.differing_frames {
            println!("frame {}:", frame.frame);
            for difference in &frame.differences {
                println!(
                    "  {} rust={} oracle={}",
                    difference.path, difference.rust, difference.oracle
                );
            }
        }
        if report.differing_frames_truncated {
            println!("additional differing frames omitted");
        }
    }
    if !report.matched {
        exit(1);
    }
    if !report.complete && !args.iter().any(|argument| argument == "--allow-incomplete") {
        eprintln!(
            "zparity av-compare: available A/V hashes match, but evidence is incomplete; pass --allow-incomplete only for diagnostics"
        );
        exit(3);
    }
}

fn cold_evidence(args: &[String]) {
    let Some(mode) = args.first().map(String::as_str) else {
        eprintln!("usage: zparity cold-evidence <find|list> PASS_ROOT [REQUEST_JSON]");
        exit(2);
    };
    match mode {
        "find" => {
            validate_options(&args[1..], 2, &[], &[], &[]);
            let pass_root = required_path(
                &args[1..],
                0,
                "zparity cold-evidence find PASS_ROOT REQUEST_JSON",
            );
            let request_path = required_path(
                &args[1..],
                1,
                "zparity cold-evidence find PASS_ROOT REQUEST_JSON",
            );
            let request = cold_evidence::load_request(&request_path).unwrap_or_else(|error| {
                eprintln!("zparity cold-evidence find: {error}");
                exit(1);
            });
            let output = cold_evidence::find_reusable_cold_evidence(&pass_root, &request)
                .unwrap_or_else(|error| {
                    eprintln!("zparity cold-evidence find: {error}");
                    exit(1);
                });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        "list" => {
            validate_options(&args[1..], 1, &[], &[], &[]);
            let pass_root = required_path(&args[1..], 0, "zparity cold-evidence list PASS_ROOT");
            let output =
                cold_evidence::list_verified_cold_evidence(&pass_root).unwrap_or_else(|error| {
                    eprintln!("zparity cold-evidence list: {error}");
                    exit(1);
                });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            eprintln!("usage: zparity cold-evidence <find|list> PASS_ROOT [REQUEST_JSON]");
            exit(2);
        }
    }
}

/// `zparity trace-decode TRACE.bin [--run N] [--event NAME] [--limit N]`:
/// render Z3TRACE1 records as canonical JSON Lines on stdout.
fn trace_decode(args: &[String]) {
    let Some(path) = args.first() else {
        eprintln!("usage: zparity trace-decode TRACE [--run N] [--run-range A-B] [--event NAME] [--limit N]");
        exit(2);
    };
    let mut run_filter: Option<(u64, u64)> = None;
    let mut event_filter: Option<String> = None;
    let mut limit: Option<usize> = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--run" => {
                let value: u64 = args
                    .get(index + 1)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--run needs an integer");
                        exit(2)
                    });
                run_filter = Some((value, value));
                index += 2;
            }
            "--run-range" => {
                let text = args.get(index + 1).cloned().unwrap_or_default();
                let (a, b) = text
                    .split_once('-')
                    .unwrap_or((text.as_str(), text.as_str()));
                run_filter = Some((a.parse().unwrap_or(0), b.parse().unwrap_or(u64::MAX)));
                index += 2;
            }
            "--event" => {
                event_filter = args.get(index + 1).cloned();
                index += 2;
            }
            "--limit" => {
                limit = args.get(index + 1).and_then(|v| v.parse().ok());
                index += 2;
            }
            other => {
                eprintln!("unknown trace-decode option {other}");
                exit(2);
            }
        }
    }
    let mut reader = parity::trace_format::open_trace(std::path::Path::new(path), 0)
        .unwrap_or_else(|error| {
            eprintln!("zparity trace-decode: {error}");
            exit(1);
        });
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let mut emitted = 0usize;
    loop {
        let record = match reader.next_record() {
            Ok(Some(record)) => record,
            Ok(None) => break,
            Err(error) => {
                eprintln!("zparity trace-decode: {error}");
                exit(1);
            }
        };
        if let Some((lo, hi)) = run_filter {
            if record.run < lo {
                continue;
            }
            if record.run > hi {
                break;
            }
        }
        if event_filter
            .as_deref()
            .is_some_and(|name| name != record.event())
        {
            continue;
        }
        use std::io::Write as _;
        if writeln!(out, "{}", record.to_json()).is_err() {
            break;
        }
        emitted += 1;
        if limit.is_some_and(|max| emitted >= max) {
            break;
        }
    }
}

/// `zparity trace-encode INPUT.jsonl OUTPUT.bin`: convert canonical JSON
/// Lines records (fixtures, older traces) into the Z3TRACE1 binary format.
fn trace_encode(args: &[String]) {
    let (Some(input), Some(output)) = (args.first(), args.get(1)) else {
        eprintln!("usage: zparity trace-encode INPUT.jsonl OUTPUT.bin");
        exit(2);
    };
    let text = std::fs::read_to_string(input).unwrap_or_else(|error| {
        eprintln!("zparity trace-encode: cannot read {input}: {error}");
        exit(1);
    });
    let mut bytes = parity::trace_format::MAGIC.to_vec();
    for (number, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|error| {
            eprintln!("zparity trace-encode: line {}: {error}", number + 1);
            exit(1);
        });
        let record = parity::trace_format::TraceRecord::from_json(&value).unwrap_or_else(|error| {
            eprintln!("zparity trace-encode: line {}: {error}", number + 1);
            exit(1);
        });
        bytes.extend(record.encode_framed());
    }
    std::fs::write(output, bytes).unwrap_or_else(|error| {
        eprintln!("zparity trace-encode: cannot write {output}: {error}");
        exit(1);
    });
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("coverage") => parity_coverage::run(&args[1..]),
        Some("trace-index") => trace_index(&args[1..]),
        Some("trace-query") => trace_query(&args[1..]),
        Some("cache-verify") => cache_verify(&args[1..]),
        Some("receipt-compare") => receipt_compare(&args[1..]),
        Some("av-compare") => av_compare(&args[1..]),
        Some("cold-evidence") => cold_evidence(&args[1..]),
        Some("trace-decode") => trace_decode(&args[1..]),
        Some("trace-encode") => trace_encode(&args[1..]),
        _ => {
            eprintln!(
                "usage: zparity <coverage|trace-index|trace-query|cache-verify|receipt-compare|av-compare|cold-evidence|trace-decode|trace-encode> [options]"
            );
            eprintln!("(capture/check/drill parity subcommands were retired with legacy parity)");
            exit(2);
        }
    }
}
