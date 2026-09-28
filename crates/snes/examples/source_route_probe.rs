//! Diagnose the first unsupported source-owned CPU/MMIO operation on a recorded take.
//!
//! Usage: cargo run -p snes --example source_route_probe -- ROM SRAM INPUT HOST_CALLS [ORACLE_JSONL_ZST]
//! `ZELDA3_SOURCE_CHECKPOINT_AT` and `ZELDA3_SOURCE_CHECKPOINT_PATH` save a
//! quiescent owner after one checked host; `ZELDA3_SOURCE_RESUME` reloads it.

use sha2::{Digest, Sha256};
use snes::{
    Snes9xColdCpuExecutor, Snes9xCpuQuiescentCheckpoint, SourceCpuAcceptedInterrupt,
    SourceCpuBusAccessKind,
};
use std::{
    env,
    error::Error,
    fs,
    io::{BufRead, BufReader},
};

#[derive(serde::Serialize, serde::Deserialize)]
struct RouteCheckpoint {
    completed_host: usize,
    rom_sha256: String,
    sram_sha256: String,
    input_sha256: String,
    cpu: Snes9xCpuQuiescentCheckpoint,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

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
    let checkpoint_at = env::var("ZELDA3_SOURCE_CHECKPOINT_AT")
        .ok()
        .map(|value| value.parse::<usize>())
        .transpose()?;
    let checkpoint_path = env::var("ZELDA3_SOURCE_CHECKPOINT_PATH").ok();
    if checkpoint_at.is_some() != checkpoint_path.is_some() {
        return Err("checkpoint host and path must be supplied together".into());
    }
    let (mut cpu, first_host) = if let Ok(path) = env::var("ZELDA3_SOURCE_RESUME") {
        let checkpoint: RouteCheckpoint = serde_json::from_slice(&fs::read(path)?)?;
        if checkpoint.rom_sha256 != sha256(&rom)
            || checkpoint.sram_sha256 != sha256(&sram)
            || checkpoint.input_sha256 != sha256(input.as_bytes())
        {
            return Err("source route checkpoint inputs differ".into());
        }
        (
            Snes9xColdCpuExecutor::from_quiescent_checkpoint(checkpoint.cpu)?,
            checkpoint.completed_host + 1,
        )
    } else {
        (
            Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&sram))?,
            0,
        )
    };
    // This probe checks CPU/OAM state, not audio. Consume the sidecar's
    // exactly-once sample output so a long route checkpoint stays bounded.
    drop(cpu.take_dsp_samples());
    if first_host > host_calls {
        return Err("source checkpoint is beyond requested host calls".into());
    }
    if checkpoint_at.is_some_and(|host| host < first_host || host >= host_calls) {
        return Err("checkpoint host must be within the replayed host range".into());
    }
    let trace_wram = env::var("ZELDA3_SOURCE_TRACE_WRAM")
        .ok()
        .map(|value| u16::from_str_radix(value.trim_start_matches("0x"), 16))
        .transpose()?;
    let trace_pc_range = env::var("ZELDA3_SOURCE_TRACE_PC_RANGE")
        .ok()
        .map(|value| {
            let (start, end) = value.split_once('-').ok_or("invalid trace PC range")?;
            Ok::<_, Box<dyn Error>>((
                u32::from_str_radix(start.trim_start_matches("0x"), 16)?,
                u32::from_str_radix(end.trim_start_matches("0x"), 16)?,
            ))
        })
        .transpose()?;
    let trace_returns = env::var_os("ZELDA3_SOURCE_TRACE_RETURNS").is_some();
    let trace_transactions = env::var_os("ZELDA3_SOURCE_TRACE_TRANSACTIONS").is_some();
    let trace_accesses = env::var_os("ZELDA3_SOURCE_TRACE_ACCESSES").is_some();
    let trace_interrupts = env::var_os("ZELDA3_SOURCE_TRACE_INTERRUPTS").is_some();
    let trace_ppu_reads = env::var_os("ZELDA3_SOURCE_TRACE_PPU_READS").is_some();
    let progress_every = env::var("ZELDA3_SOURCE_PROGRESS_EVERY")
        .ok()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .filter(|&value| value != 0);
    let trace_hosts = env::var("ZELDA3_SOURCE_TRACE_HOSTS")
        .ok()
        .map(|value| {
            let (start, end) = value.split_once('-').ok_or("invalid trace host range")?;
            Ok::<_, Box<dyn Error>>((start.parse::<usize>()?, end.parse::<usize>()?))
        })
        .transpose()?;
    for (host, buttons) in buttons.into_iter().enumerate() {
        if host < first_host {
            if let Some(reader) = oracle.as_mut() {
                let mut line = String::new();
                if reader.read_line(&mut line)? == 0 {
                    return Err(format!("oracle ended before resumed host call {host}").into());
                }
            }
            continue;
        }
        if progress_every.is_some_and(|interval| host % interval == 0) {
            eprintln!("source route starting host call {host}");
        }
        cpu.set_libretro_joypad_words(buttons, 0);
        let trace_this_host = trace_hosts.is_some_and(|(start, end)| (start..=end).contains(&host));
        let result = cpu.run_until_main_loop_return_with_state(|step, state| {
            if trace_this_host && trace_interrupts {
                let selected = match step.accepted_interrupt {
                    Some(SourceCpuAcceptedInterrupt::Nmi { started_at }) => {
                        Some(("nmi", started_at))
                    }
                    Some(SourceCpuAcceptedInterrupt::Irq { started_at }) => {
                        Some(("irq", started_at))
                    }
                    None => None,
                };
                if let Some((kind, started_at)) = selected {
                    eprintln!(
                        "source-interrupt host={host} kind={kind} pc={:06x} opcode={:02x} instruction_start={} acceptance={} step_end={}",
                        step.origin_pc,
                        step.opcode,
                        step.started_at.master_cycles(),
                        started_at.master_cycles(),
                        step.ended_at.master_cycles(),
                    );
                }
            }
            if trace_this_host && trace_ppu_reads {
                for access in &step.accesses {
                    let register = access.address & 0xffff;
                    if matches!(register, 0x2137 | 0x213c | 0x213d | 0x213f | 0x4201) {
                        eprintln!(
                            "source-ppu-bus host={host} pc={:06x} time={} address={:06x} kind={:?}",
                            step.origin_pc,
                            access.timestamp.master_cycles(),
                            access.address,
                            access.kind,
                        );
                    }
                }
            }
            if trace_this_host
                && trace_pc_range.is_some_and(|(start, end)| {
                    (start..=end).contains(&step.origin_pc)
                })
            {
                eprintln!(
                    "source-step host={host} pc={:06x} opcode={:02x} start={} end={} a={:04x} x={:04x} y={:04x} s={:04x} p={:02x}",
                    step.origin_pc,
                    step.opcode,
                    step.started_at.master_cycles(),
                    step.ended_at.master_cycles(),
                    state.a,
                    state.x,
                    state.y,
                    state.sp,
                    state.pack_flags(),
                );
                if trace_transactions {
                    for transaction in &step.transactions {
                        eprintln!(
                            "source-transaction host={host} pc={:06x} kind={:?} duration={} start={} end={} refresh={}..{}",
                            step.origin_pc,
                            transaction.kind,
                            transaction.duration_master_cycles,
                            transaction.started_at.master_cycles(),
                            transaction.ended_at.master_cycles(),
                            transaction.start_wram_refresh_position,
                            transaction.end_wram_refresh_position,
                        );
                    }
                }
                if trace_accesses {
                    for access in &step.accesses {
                        eprintln!(
                            "source-access host={host} pc={:06x} time={} address={:06x} kind={:?}",
                            step.origin_pc,
                            access.timestamp.master_cycles(),
                            access.address,
                            access.kind,
                        );
                    }
                }
            }
            if let Some(address) = trace_wram {
                if trace_this_host {
                    for access in &step.accesses {
                        if access.address & 0xffff == u32::from(address) {
                            if let SourceCpuBusAccessKind::Write { value, width } = access.kind {
                                eprintln!(
                                    "source host={host} pc={:06x} time={} address={:06x} value={value:04x} width={width}",
                                    step.origin_pc,
                                    access.timestamp.master_cycles(),
                                    access.address,
                                );
                            }
                        }
                    }
                }
            }
        });
        if trace_returns && trace_this_host {
            if result.is_ok() {
                let state = &cpu.machine().snes().cpu;
                let beam = cpu.raster_position();
                eprintln!(
                    "source-return host={host} pc={:06x} v={} cycles={} time={} a={} x={} y={} s={} p={}",
                    (u32::from(state.k) << 16) | u32::from(state.pc),
                    beam.scanline(),
                    beam.master_cycle(),
                    cpu.machine().timestamp().master_cycles(),
                    state.a,
                    state.x,
                    state.y,
                    state.sp,
                    state.pack_flags(),
                );
            }
        }
        if let Err(error) = result {
            let beam = cpu.raster_position();
            let state = &cpu.machine().snes().cpu;
            eprintln!(
                "source owner stopped at host call {host} pc={:06x} v={} cycles={}: {error}",
                (u32::from(state.k) << 16) | u32::from(state.pc),
                beam.scanline(),
                beam.master_cycle(),
            );
            return Err(error.into());
        }
        if trace_this_host && trace_ppu_reads {
            eprintln!(
                "source-ppu-owner host={host} time={} state={:?}",
                cpu.machine().timestamp().master_cycles(),
                cpu.machine().source_ppu_reads(),
            );
        }
        drop(cpu.take_dsp_samples());
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
            let differences: Vec<_> = expected
                .iter()
                .zip(actual)
                .enumerate()
                .filter_map(|(index, (expected, actual))| {
                    (expected.as_u64() != Some(u64::from(*actual)))
                        .then(|| format!("{index:#x}: source={actual:#04x} oracle={expected}"))
                })
                .collect();
            if !differences.is_empty() {
                return Err(format!(
                    "presented OAM differs at host call {host} in {} bytes; first: {}",
                    differences.len(),
                    differences[..differences.len().min(12)].join(", ")
                )
                .into());
            }
        }
        if checkpoint_at == Some(host) {
            let checkpoint = RouteCheckpoint {
                completed_host: host,
                rom_sha256: sha256(&rom),
                sram_sha256: sha256(&sram),
                input_sha256: sha256(input.as_bytes()),
                cpu: cpu.capture_quiescent_checkpoint()?,
            };
            fs::write(
                checkpoint_path.as_ref().unwrap(),
                serde_json::to_vec(&checkpoint)?,
            )?;
        }
    }
    if oracle.is_some() {
        println!("source owner matched presented OAM through {host_calls} recorded host calls");
    } else {
        println!("source owner completed {host_calls} recorded host calls");
    }
    Ok(())
}
