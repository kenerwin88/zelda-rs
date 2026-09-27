//! Diagnose the first unsupported source-owned CPU/MMIO operation on a recorded take.
//!
//! Usage: cargo run -p snes --example source_route_probe -- ROM SRAM INPUT HOST_CALLS

use snes::Snes9xColdCpuExecutor;
use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let rom = fs::read(args.next().ok_or("missing ROM path")?)?;
    let sram = fs::read(args.next().ok_or("missing SRAM path")?)?;
    let input = fs::read_to_string(args.next().ok_or("missing input path")?)?;
    let host_calls: usize = args.next().ok_or("missing host call count")?.parse()?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let mut buttons = vec![0u16; host_calls];
    for line in input
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let (range, value) = line.split_once(' ').ok_or("invalid input line")?;
        let (start, end) = range.split_once("..").unwrap_or((range, range));
        let start: usize = start.parse()?;
        let end: usize = end.parse()?;
        if start > end || end >= host_calls {
            return Err(format!("input range {range} exceeds {host_calls} host calls").into());
        }
        buttons[start..=end].fill(u16::from_str_radix(value.trim_start_matches("0x"), 16)?);
    }
    let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&sram))?;
    for (host, buttons) in buttons.into_iter().enumerate() {
        cpu.set_joypad_serial_state(buttons, 0);
        if let Err(error) = cpu.run_until_main_loop_return() {
            eprintln!("source owner stopped at host call {host}: {error}");
            return Err(error.into());
        }
    }
    println!("source owner completed {host_calls} recorded host calls");
    Ok(())
}
