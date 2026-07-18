use std::process::exit;

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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("coverage") => parity_coverage::run(&args[1..]),
        _ => {
            eprintln!("usage: zparity <coverage> [options]");
            eprintln!("(the C-oracle capture/check/drill subcommands were retired with C parity)");
            exit(2);
        }
    }
}
