//! Diagnose the first unsupported source-owned CPU/MMIO operation on a recorded take.
//!
//! Usage: cargo run -p snes --example source_route_probe -- ROM SRAM INPUT HOST_CALLS [ORACLE_JSONL_ZST]

use snes::Snes9xColdCpuExecutor;
use std::{
    env,
    error::Error,
    fs,
    io::{BufRead, BufReader},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let rom = fs::read(args.next().ok_or("missing ROM path")?)?;
    let sram = fs::read(args.next().ok_or("missing SRAM path")?)?;
    let input = fs::read_to_string(args.next().ok_or("missing input path")?)?;
    let host_calls: usize = args.next().ok_or("missing host call count")?.parse()?;
    let oracle_path = args.next();
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let mut oracle: Option<Box<dyn BufRead>> = if let Some(path) = oracle_path {
        let file = fs::File::open(&path)?;
        if path.ends_with(".zst") {
            Some(Box::new(BufReader::new(zstd::Decoder::new(file)?)))
        } else {
            Some(Box::new(BufReader::new(file)))
        }
    } else {
        None
    };
    let mut buttons = vec![0u16; host_calls];
    for line in input
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let (range, value) = line.split_once(' ').ok_or("invalid input line")?;
        let (start, end) = range.split_once("..").unwrap_or((range, range));
        let start: usize = start.parse()?;
        let end: usize = end.parse()?;
        if start > end {
            return Err(format!("invalid input range {range}").into());
        }
        if start < host_calls {
            buttons[start..=end.min(host_calls - 1)]
                .fill(u16::from_str_radix(value.trim_start_matches("0x"), 16)?);
        }
    }
    let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&sram))?;
    for (host, buttons) in buttons.into_iter().enumerate() {
        cpu.set_joypad_serial_state(buttons, 0);
        if let Err(error) = cpu.run_until_main_loop_return() {
            eprintln!("source owner stopped at host call {host}: {error}");
            return Err(error.into());
        }
        if let Some(reader) = oracle.as_mut() {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                return Err(format!("oracle ended before host call {host}").into());
            }
            let receipt: serde_json::Value = serde_json::from_str(&line)?;
            if receipt["host_call"].as_u64() != Some(host as u64) {
                return Err(format!("oracle host index mismatch at {host}").into());
            }
            let expected = receipt["presented_oam"]["bytes"]
                .as_array()
                .ok_or("oracle receipt has no presented OAM bytes")?;
            let actual = cpu.presented_oam();
            if expected.len() != actual.len() {
                return Err(format!("oracle OAM length mismatch at host call {host}").into());
            }
            for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
                if expected.as_u64() != Some(u64::from(*actual)) {
                    return Err(format!(
                        "presented OAM differs at host call {host}, byte {index:#x}: source={actual:#04x} oracle={expected}"
                    ).into());
                }
            }
        }
    }
    if oracle.is_some() {
        println!("source owner matched presented OAM through {host_calls} recorded host calls");
    } else {
        println!("source owner completed {host_calls} recorded host calls");
    }
    Ok(())
}
