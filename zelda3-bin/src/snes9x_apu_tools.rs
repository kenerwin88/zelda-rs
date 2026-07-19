//! APU-bootstrap capture plus Snes9x-side audio/memory/frame debug tracers.

use crate::*;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::audio_trace::{
    first_peak_frame, max_peak_frame, print_audio_window, replay_checksum_samples, AudioFrameStats,
};
use crate::image_output::write_argb_frame_png;
use crate::input_script::InputScript;
use serde::{Deserialize, Serialize};
use snes::{cpu_run_opcode, load_rom, Snes};

pub(crate) const APU_BOOTSTRAP_CHECKPOINT_MAGIC: &[u8; 8] = b"Z3RSAPU1";

pub(crate) fn run_trace_rom_apu_upload(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-rom-apu-upload <path-to-rom.sfc> [opcode-budget] [apu-cycles-per-cpu-cycle]"
            );
            process::exit(2);
        }
    };
    let budget: u64 = args
        .get(1)
        .map(|s| s.parse().unwrap_or(1_000_000))
        .unwrap_or(1_000_000);
    let apu_cycles_per_cpu_cycle: f64 = args
        .get(2)
        .map(|s| s.parse().unwrap_or(0.286))
        .unwrap_or(0.286);

    let rom = match fs::read(rom_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to read {rom_path}: {e}");
            process::exit(1);
        }
    };

    let mut snes = Snes::new();
    if let Err(e) = load_rom(&mut snes, &rom) {
        eprintln!("loader: {e}");
        process::exit(1);
    }
    snes.reset(true);
    snes.cpu_seed_reset_vector();

    println!(
        "raw ROM APU trace: rom={} bytes budget={} apu_cycles_per_cpu_cycle={:.3} reset PC=${:04X} K={:02X}",
        rom.len(),
        budget,
        apu_cycles_per_cpu_cycle,
        snes.cpu.pc,
        snes.cpu.k
    );
    print_rom_apu_trace_line(0, "start", &snes);

    let mut apu_cycle_accum = 0.0f64;
    let mut next_payload_milestone = 256usize;
    let mut last_nonzero_0800 = snes.apu.ram[0x0800..0x0900].iter().any(|&b| b != 0);
    let mut last_nonzero_0878 = snes.apu.ram[0x0878..0x08c0].iter().any(|&b| b != 0);
    let mut last_reset = (snes.apu.ram[0xfffe], snes.apu.ram[0xffff]);
    let mut last_in_ports = [
        snes.apu.in_ports[0],
        snes.apu.in_ports[1],
        snes.apu.in_ports[2],
        snes.apu.in_ports[3],
    ];
    let mut next_dsp_milestone = 1usize;

    for op in 1..=budget {
        let cycles = cpu_run_opcode(&mut snes);
        while snes.dma.dma_busy {
            snes.dma_do();
        }

        apu_cycle_accum += f64::from(cycles) * apu_cycles_per_cpu_cycle;
        let apu_cycles = apu_cycle_accum as u32;
        for _ in 0..apu_cycles {
            snes.apu.cycle();
        }
        apu_cycle_accum -= f64::from(apu_cycles);

        let nonzero_0800 = snes.apu.ram[0x0800..0x0900].iter().any(|&b| b != 0);
        let nonzero_0878 = snes.apu.ram[0x0878..0x08c0].iter().any(|&b| b != 0);
        let reset = (snes.apu.ram[0xfffe], snes.apu.ram[0xffff]);
        let in_ports = [
            snes.apu.in_ports[0],
            snes.apu.in_ports[1],
            snes.apu.in_ports[2],
            snes.apu.in_ports[3],
        ];
        let dsp_writes = snes.apu.dsp_write_history.len();
        let hit_dsp_milestone = dsp_writes >= next_dsp_milestone;
        while dsp_writes >= next_dsp_milestone {
            if next_dsp_milestone >= 256 {
                next_dsp_milestone = usize::MAX;
                break;
            }
            next_dsp_milestone *= 2;
        }
        let payload_nonzero = apu_payload_nonzero(&snes);
        let hit_payload_milestone = payload_nonzero >= next_payload_milestone;
        while payload_nonzero >= next_payload_milestone {
            next_payload_milestone += 256;
        }
        let command_phase = !snes.apu.rom_readable || dsp_writes != 0;

        let changed = nonzero_0800 != last_nonzero_0800
            || nonzero_0878 != last_nonzero_0878
            || reset != last_reset
            || (command_phase && in_ports != last_in_ports)
            || hit_dsp_milestone
            || hit_payload_milestone
            || snes.cpu.stopped;
        let periodic = op % 100_000 == 0;
        if changed || periodic {
            print_rom_apu_trace_line(op, if changed { "change" } else { "tick" }, &snes);
        }

        last_nonzero_0800 = nonzero_0800;
        last_nonzero_0878 = nonzero_0878;
        last_reset = reset;
        last_in_ports = in_ports;
        if snes.cpu.stopped {
            break;
        }
    }

    print_rom_apu_trace_line(budget, "end", &snes);
}

pub(crate) fn run_trace_snes9x_spc_window(args: &[String]) {
    let Some(state_path) = args.first() else {
        eprintln!(
            "usage: zelda3 --trace-snes9x-spc-window <snes9x.state> [apu-cycles] [port-write-cycle] [port0 port1 port2 port3] [cycle-sequenced]"
        );
        process::exit(2);
    };
    let apu_cycles = args
        .get(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(17_056u32);
    let port_write_cycle = args
        .get(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(59u32);
    let mut ports = [0u8; 4];
    for (index, slot) in ports.iter_mut().enumerate() {
        if let Some(value) = args.get(3 + index) {
            *slot = value
                .parse::<u8>()
                .unwrap_or_else(|_| panic!("invalid port value `{value}`"));
        }
    }
    let cycle_sequenced = args.get(7).map(String::as_str) == Some("cycle-sequenced");

    let state = fs::read(state_path).unwrap_or_else(|error| {
        eprintln!("failed to read {state_path}: {error}");
        process::exit(1);
    });
    let snd = snes9x_snapshot_block(&state, b"SND").unwrap_or_else(|error| {
        eprintln!("failed to parse {state_path}: {error}");
        process::exit(1);
    });
    let mut apu = snes::apu::ApuState::new();
    apu.load_snes9x_1_63_smp_state(snd).unwrap_or_else(|error| {
        eprintln!("failed to import Snes9x SMP state: {error}");
        process::exit(1);
    });
    if cycle_sequenced {
        apu.schedule_input_port_write(port_write_cycle, ports);
    }
    while apu.cycles < apu_cycles {
        let cycle = apu.cycles;
        if !cycle_sequenced && apu.cpu_cycles_left == 0 && (0x0870..=0x0888).contains(&apu.spc.pc) {
            println!(
                "{}",
                serde_json::json!({
                    "kind": "instruction",
                    "apu_cycle": cycle,
                    "sample_floor": cycle / 32,
                    "phase": cycle % 32,
                    "pc": apu.spc.pc,
                    "a": apu.spc.a,
                    "x": apu.spc.x,
                    "y": apu.spc.y,
                    "timer0_counter": apu.timer[0].counter,
                })
            );
        }
        if !cycle_sequenced && cycle == port_write_cycle {
            for (port, value) in ports.into_iter().enumerate() {
                apu.write_snes_port(port as u8, value);
            }
        }
        if cycle_sequenced {
            apu.run_cycle_sequenced_instruction_without_dsp();
        } else {
            apu.cycle_without_dsp();
        }
    }
    let writes = apu.debug_dsp_write_trace.take().unwrap_or_default();
    for (cycle, address, value) in &writes {
        println!(
            "{}",
            serde_json::json!({
                "apu_cycle": cycle,
                "sample_floor": cycle / 32,
                "phase": cycle % 32,
                "address": address,
                "value": value,
            })
        );
    }
    eprintln!(
        "SPC window completed: cycles={} requested_cycles={apu_cycles} cycle_sequenced={cycle_sequenced} port_write_cycle={port_write_cycle} ports={ports:02x?} pc=${:04x} sp={:02x} a={:02x} x={:02x} y={:02x} z={} c={} timer0=({}, {}, {}, {}) ram43={} writes={}",
        apu.cycles,
        apu.spc.pc,
        apu.spc.sp,
        apu.spc.a,
        apu.spc.x,
        apu.spc.y,
        apu.spc.z,
        apu.spc.c,
        apu.timer[0].cycles,
        apu.timer[0].divider,
        apu.timer[0].counter,
        apu.timer[0].enabled,
        apu.ram[0x43],
        writes.len(),
    );
}

pub(crate) fn run_dump_snes9x_apu_ram(args: &[String]) {
    let (Some(state_path), Some(start), Some(length)) = (args.first(), args.get(1), args.get(2))
    else {
        eprintln!("usage: zelda3 --dump-snes9x-apu-ram <snes9x.state> <hex-start> <length>");
        process::exit(2);
    };
    let parse_hex = |value: &str| {
        usize::from_str_radix(value.trim_start_matches("0x"), 16)
            .unwrap_or_else(|_| panic!("invalid hexadecimal address `{value}`"))
    };
    let start = parse_hex(start);
    let length = length
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("invalid byte length `{length}`"));
    let state = fs::read(state_path).unwrap_or_else(|error| {
        eprintln!("failed to read {state_path}: {error}");
        process::exit(1);
    });
    let snd = snes9x_snapshot_block(&state, b"SND").unwrap_or_else(|error| {
        eprintln!("failed to parse {state_path}: {error}");
        process::exit(1);
    });
    let end = start
        .checked_add(length)
        .filter(|end| *end <= 0x10000)
        .unwrap_or_else(|| panic!("APU RAM range exceeds $ffff"));
    println!("{}", serde_json::to_string(&snd[start..end]).unwrap());
}

pub(crate) fn run_trace_spc_driver_startup(args: &[String]) {
    let (Some(driver_path), Some(bank_path)) = (args.first(), args.get(1)) else {
        eprintln!(
            "usage: zelda3 --trace-spc-driver-startup <spc-driver.bin> <song-bank.bin> [apu-cycles] [cycle-sequenced] [absolute-origin] [snes9x.state]"
        );
        process::exit(2);
    };
    let apu_cycles = args
        .get(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(32_000u32);
    let cycle_sequenced = args.get(3).map(String::as_str) == Some("cycle-sequenced");
    let absolute_origin = args.get(4).and_then(|value| value.parse::<u64>().ok());
    let driver = fs::read(driver_path).unwrap_or_else(|error| {
        eprintln!("failed to read {driver_path}: {error}");
        process::exit(1);
    });
    let bank = fs::read(bank_path).unwrap_or_else(|error| {
        eprintln!("failed to read {bank_path}: {error}");
        process::exit(1);
    });
    if driver.len() != 0x179e - 0x0800 {
        eprintln!("invalid SPC driver length {}", driver.len());
        process::exit(1);
    }

    let mut apu = snes::apu::ApuState::new();
    apu.reset();
    apu.ram[0x0800..0x179e].copy_from_slice(&driver);
    let mut cursor = 0usize;
    loop {
        let length = u16::from_le_bytes(bank[cursor..cursor + 2].try_into().unwrap()) as usize;
        if length == 0 {
            break;
        }
        let mut target =
            u16::from_le_bytes(bank[cursor + 2..cursor + 4].try_into().unwrap()) as usize;
        cursor += 4;
        for &byte in &bank[cursor..cursor + length] {
            apu.ram[target] = byte;
            target = (target + 1) & 0xffff;
        }
        cursor += length;
    }
    apu.rom_readable = false;
    apu.spc.pc = 0x0800;
    apu.debug_dsp_write_trace = Some(Vec::new());
    if let Some(origin) = absolute_origin {
        for (timer, frequency) in apu.timer.iter_mut().zip([128u64, 128, 16]) {
            let phase = (origin % frequency) as u8;
            timer.cycles = if phase == 0 {
                0
            } else {
                frequency as u8 - phase
            };
        }
    }
    while apu.cycles < apu_cycles {
        if cycle_sequenced {
            apu.run_cycle_sequenced_instruction_without_dsp();
        } else {
            apu.cycle_without_dsp();
        }
    }
    for (cycle, address, value) in apu.debug_dsp_write_trace.take().unwrap_or_default() {
        println!(
            "{}",
            serde_json::json!({
                "apu_cycle": cycle,
                "sample_floor": cycle / 32,
                "phase": cycle % 32,
                "address": address,
                "value": value,
            })
        );
    }
    eprintln!(
        "SPC startup completed: cycles={} requested_cycles={apu_cycles} cycle_sequenced={cycle_sequenced} pc=${:04x} timer0=({}, {}, {}, {})",
        apu.cycles,
        apu.spc.pc,
        apu.timer[0].cycles,
        apu.timer[0].divider,
        apu.timer[0].counter,
        apu.timer[0].enabled,
    );
    if let Some(state_path) = args.get(5) {
        let state = fs::read(state_path).unwrap_or_else(|error| {
            panic!("failed to read {state_path}: {error}");
        });
        let snd = snes9x_snapshot_block(&state, b"SND")
            .unwrap_or_else(|error| panic!("failed to parse {state_path}: {error}"));
        let mut oracle = snes::apu::ApuState::new();
        oracle
            .load_snes9x_1_63_smp_state(snd)
            .unwrap_or_else(|error| panic!("failed to import Snes9x SMP state: {error}"));
        let ram_mismatches = apu
            .ram
            .iter()
            .zip(&oracle.ram)
            .enumerate()
            .filter(|(_, (rust, oracle))| rust != oracle)
            .take(32)
            .map(|(address, (rust, oracle))| (address, *rust, *oracle))
            .collect::<Vec<_>>();
        eprintln!(
            "Snes9x state comparison: spc_rust={:?} spc_oracle={:?} timers_rust={:?} timers_oracle={:?} dsp_adr_rust={:02x} dsp_adr_oracle={:02x} ram_first_mismatches={ram_mismatches:?}",
            apu.spc,
            oracle.spc,
            apu.timer,
            oracle.timer,
            apu.dsp_adr,
            oracle.dsp_adr,
        );
    }
}

pub(crate) fn snes9x_snapshot_block<'a>(
    state: &'a [u8],
    wanted: &[u8; 3],
) -> Result<&'a [u8], String> {
    if !state.starts_with(b"#!s9xsnp:") {
        return Err("not an uncompressed Snes9x snapshot".to_string());
    }
    let mut cursor = state
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|position| position + 1)
        .ok_or_else(|| "snapshot header is missing its newline".to_string())?;
    while cursor < state.len() {
        let header = state
            .get(cursor..cursor + 11)
            .ok_or_else(|| "snapshot has a truncated block header".to_string())?;
        if header[3] != b':' || header[10] != b':' {
            return Err(format!("invalid snapshot block header at byte {cursor}"));
        }
        let length = std::str::from_utf8(&header[4..10])
            .map_err(|_| "snapshot block length is not UTF-8".to_string())?
            .parse::<usize>()
            .map_err(|_| "snapshot block length is not decimal".to_string())?;
        cursor += 11;
        let block = state
            .get(cursor..cursor + length)
            .ok_or_else(|| "snapshot has a truncated block payload".to_string())?;
        if &header[..3] == wanted {
            return Ok(block);
        }
        cursor += length;
    }
    Err(format!(
        "snapshot does not contain a {} block",
        String::from_utf8_lossy(wanted)
    ))
}

pub(crate) fn run_capture_rom_apu_bootstrap(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --capture-rom-apu-bootstrap <path-to-rom.sfc> <out.z3apu> [opcode-budget] [apu-cycles-per-cpu-cycle]"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(1) {
        Some(p) => Path::new(p),
        None => {
            eprintln!(
                "usage: zelda3 --capture-rom-apu-bootstrap <path-to-rom.sfc> <out.z3apu> [opcode-budget] [apu-cycles-per-cpu-cycle]"
            );
            process::exit(2);
        }
    };
    let budget: u64 = args
        .get(2)
        .map(|s| s.parse().unwrap_or(1_500_000))
        .unwrap_or(1_500_000);
    let apu_cycles_per_cpu_cycle: f64 = args
        .get(3)
        .map(|s| s.parse().unwrap_or(0.286))
        .unwrap_or(0.286);

    let (snes, opcodes) =
        match capture_raw_rom_apu_bootstrap(rom_path, budget, apu_cycles_per_cpu_cycle, true) {
            Ok(capture) => capture,
            Err(e) => {
                eprintln!("failed to capture raw ROM APU bootstrap: {e}");
                process::exit(1);
            }
        };
    let checkpoint = ApuBootstrapCheckpoint {
        magic: *APU_BOOTSTRAP_CHECKPOINT_MAGIC,
        opcodes,
        apu_cycles_per_cpu_cycle,
        cpu_k: snes.cpu.k,
        cpu_pc: snes.cpu.pc,
        spc_pc: snes.apu.spc.pc,
        rom_readable: snes.apu.rom_readable,
        payload_nonzero: apu_payload_nonzero(&snes),
        dsp_writes: snes.apu.dsp_write_history.len(),
        apu: snes.apu,
    };
    let bytes = match bincode::serialize(&checkpoint) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("failed to encode APU bootstrap checkpoint: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(out_path, bytes) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "saved APU bootstrap {}: opcodes={} cpu=${:02x}:{:04x} spc=${:04x} rom={} payload_nz={} dsp_writes={}",
        out_path.display(),
        checkpoint.opcodes,
        checkpoint.cpu_k,
        checkpoint.cpu_pc,
        checkpoint.spc_pc,
        checkpoint.rom_readable,
        checkpoint.payload_nonzero,
        checkpoint.dsp_writes,
    );
}

pub(crate) fn run_compare_bootstrap_apu_startup(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --compare-bootstrap-apu-startup <path-to-rom.sfc> <bootstrap.z3apu> [frames]"
            );
            process::exit(2);
        }
    };
    let bootstrap_path = match args.get(1) {
        Some(p) => Path::new(p),
        None => {
            eprintln!(
                "usage: zelda3 --compare-bootstrap-apu-startup <path-to-rom.sfc> <bootstrap.z3apu> [frames]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(120);
    let checkpoint = match load_apu_bootstrap_checkpoint(bootstrap_path) {
        Ok(checkpoint) => checkpoint,
        Err(e) => {
            eprintln!(
                "failed to load APU bootstrap checkpoint {}: {e}",
                bootstrap_path.display()
            );
            process::exit(1);
        }
    };
    let mut game = load_play_state(rom_path);
    let mut full_apu = checkpoint.apu;
    full_apu.dsp.sample_offset = 0;
    full_apu.dsp_write_history.clear();

    let mut high_audio = vec![0i16; 735 * 2];
    let mut full_audio = vec![0i16; 735 * 2];
    let mut high_stats = Vec::with_capacity(frames as usize);
    let mut full_stats = Vec::with_capacity(frames as usize);
    let mut debug = Vec::with_capacity(frames as usize);

    println!(
        "bootstrap source: opcodes={} cpu=${:02x}:{:04x} spc=${:04x} rom={} payload_nz={} dsp_writes={}",
        checkpoint.opcodes,
        checkpoint.cpu_k,
        checkpoint.cpu_pc,
        checkpoint.spc_pc,
        checkpoint.rom_readable,
        checkpoint.payload_nonzero,
        checkpoint.dsp_writes,
    );

    for _ in 0..frames {
        game.zelda_run_frame(0);
        let ports = game.zelda_debug_apu_write_ports();
        for (port, &value) in ports.iter().enumerate() {
            full_apu.write_snes_port(port as u8, value);
        }
        game.zelda_render_audio(&mut high_audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        render_full_apu_audio(&mut full_apu, &mut full_audio, 735, 2);
        high_stats.push(AudioFrameStats::from_interleaved_stereo(&high_audio));
        full_stats.push(AudioFrameStats::from_interleaved_stereo(&full_audio));
        debug.push(format!(
            "ports={ports:02x?} main={:02x} sub={:02x} subsub={:02x} full_pc={:04x} full_in={:02x?} full_out={:02x?} full_dsp_writes={} full_last_dsp={:02x?} {}",
            game.ram[0x10],
            game.ram[0x11],
            game.ram[0xb0],
            full_apu.spc.pc,
            &full_apu.in_ports[..4],
            full_apu.out_ports,
            full_apu.dsp_write_history.len(),
            full_apu.dsp_write_history.last().copied(),
            game.zelda_audio_debug_summary(),
        ));
    }

    let threshold = 512i16;
    let high_onset = first_peak_frame(&high_stats, threshold);
    let full_onset = first_peak_frame(&full_stats, threshold);
    let high_max = max_peak_frame(&high_stats);
    let full_max = max_peak_frame(&full_stats);
    println!(
        "bootstrap APU startup threshold={threshold}: high_onset={high_onset:?} full_onset={full_onset:?} high_max={high_max:?} full_max={full_max:?}",
    );
    if let (Some(high_onset), Some(full_onset)) = (high_onset, full_onset) {
        let delta = full_onset as i32 - high_onset as i32;
        println!("bootstrap APU onset_delta_full_minus_high={delta} frames");
    }
    print_audio_window(
        "high",
        &high_stats,
        &debug,
        high_onset.or(high_max.map(|(i, _)| i)),
    );
    print_audio_window(
        "bootstrap-full-apu",
        &full_stats,
        &[],
        full_onset.or(full_max.map(|(i, _)| i)),
    );
}

pub(crate) fn capture_raw_rom_apu_bootstrap(
    rom_path: &str,
    budget: u64,
    apu_cycles_per_cpu_cycle: f64,
    stop_when_ready: bool,
) -> Result<(Snes, u64), Box<dyn Error>> {
    let rom = fs::read(rom_path)?;
    let mut snes = Snes::new();
    load_rom(&mut snes, &rom)?;
    snes.reset(true);
    snes.cpu_seed_reset_vector();

    let mut apu_cycle_accum = 0.0f64;
    for op in 1..=budget {
        step_raw_snes_with_apu(&mut snes, apu_cycles_per_cpu_cycle, &mut apu_cycle_accum);
        if stop_when_ready && raw_rom_apu_bootstrap_ready(&snes) {
            return Ok((snes, op));
        }
        if snes.cpu.stopped {
            return Ok((snes, op));
        }
    }
    Ok((snes, budget))
}

pub(crate) fn step_raw_snes_with_apu(
    snes: &mut Snes,
    apu_cycles_per_cpu_cycle: f64,
    apu_cycle_accum: &mut f64,
) {
    let cycles = cpu_run_opcode(snes);
    while snes.dma.dma_busy {
        snes.dma_do();
    }

    *apu_cycle_accum += f64::from(cycles) * apu_cycles_per_cpu_cycle;
    let apu_cycles = *apu_cycle_accum as u32;
    for _ in 0..apu_cycles {
        snes.apu.cycle();
    }
    *apu_cycle_accum -= f64::from(apu_cycles);
}

pub(crate) fn raw_rom_apu_bootstrap_ready(snes: &Snes) -> bool {
    !snes.apu.rom_readable
        && snes.apu.spc.pc >= 0x0800
        && snes.apu.spc.pc < 0x1000
        // The SPC has entered the driver after 16 writes, but its upload
        // handshake and DSP initialization are still draining at that point.
        // Capture only after both sides have acknowledged zero and the full
        // reset register pass has completed, so playback starts at the same
        // silent epoch as the first libretro frame.
        && snes.apu.in_ports[..4] == [0; 4]
        && snes.apu.out_ports == [0; 4]
        && snes.apu.dsp_write_history.len() >= 128
}

pub(crate) fn print_rom_apu_trace_line(op: u64, label: &str, snes: &Snes) {
    let nz_0800 = count_nonzero(&snes.apu.ram[0x0800..0x0900]);
    let nz_0878 = count_nonzero(&snes.apu.ram[0x0878..0x08c0]);
    let nz_prog = count_nonzero(&snes.apu.ram[0x0800..0x1000]);
    let payload_nonzero = apu_payload_nonzero(snes);
    let payload_range = apu_payload_range(snes)
        .map(|(first, last)| format!("${first:04x}-${last:04x}"))
        .unwrap_or_else(|| "none".to_string());
    let last_dsp = snes
        .apu
        .dsp_write_history
        .last()
        .map(|(addr, value)| format!("${addr:02x}=${value:02x}"))
        .unwrap_or_else(|| "none".to_string());
    let upload_addr = (u16::from(snes.apu.in_ports[3]) << 8) | u16::from(snes.apu.in_ports[2]);
    println!(
        "op={op:>8} {label:<6} cpu=${:02x}:{:04x} spc=${:04x} apu_cycles={} out={:02x?} upload=${:04x} in={:02x?} rom={} reset=${:02x}{:02x} payload_nz={} payload_range={} nz0800={} nz0878={} nz0800_1000={} dsp_writes={} last_dsp={} dsp_samples={}",
        snes.cpu.k,
        snes.cpu.pc,
        snes.apu.spc.pc,
        snes.apu.cycles,
        snes.apu.out_ports,
        upload_addr,
        [
            snes.apu.in_ports[0],
            snes.apu.in_ports[1],
            snes.apu.in_ports[2],
            snes.apu.in_ports[3]
        ],
        snes.apu.rom_readable,
        snes.apu.ram[0xffff],
        snes.apu.ram[0xfffe],
        payload_nonzero,
        payload_range,
        nz_0800,
        nz_0878,
        nz_prog,
        snes.apu.dsp_write_history.len(),
        last_dsp,
        snes.apu.dsp.sample_offset
    );
}

pub(crate) fn apu_payload_nonzero(snes: &Snes) -> usize {
    count_nonzero(&snes.apu.ram[..0xffc0])
}

pub(crate) fn apu_payload_range(snes: &Snes) -> Option<(usize, usize)> {
    let first = snes.apu.ram[..0xffc0].iter().position(|&b| b != 0)?;
    let last = snes.apu.ram[..0xffc0].iter().rposition(|&b| b != 0)?;
    Some((first, last))
}

pub(crate) fn count_nonzero(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b != 0).count()
}

pub(crate) fn run_trace_startup_audio(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("usage: zelda3 --trace-startup-audio <path-to-rom.sfc> [frames] [--jsonl]");
            process::exit(2);
        }
    };
    let mut frames = 360u32;
    let mut jsonl = false;
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--jsonl" => jsonl = true,
            value if !value.starts_with("--") => {
                frames = match value.parse() {
                    Ok(frames) => frames,
                    Err(e) => {
                        eprintln!("invalid frame count `{value}`: {e}");
                        process::exit(2);
                    }
                };
            }
            flag => {
                eprintln!("unknown --trace-startup-audio option: {flag}");
                process::exit(2);
            }
        }
    }
    let mut game = load_game_state(rom_path, true);
    let mut audio = vec![0i16; 735 * 2];
    let mut last_ports = [0u8; 4];
    let mut last_nonzero = false;
    for frame_index in 0..frames {
        game.zelda_run_frame(0);
        let ports = game.zelda_debug_apu_write_ports();
        game.zelda_render_audio(&mut audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        let peak = audio
            .iter()
            .map(|sample| sample.saturating_abs())
            .max()
            .unwrap_or(0);
        let first_nonzero = audio.iter().position(|&sample| sample != 0);
        let mean_abs = if audio.is_empty() {
            0.0
        } else {
            audio
                .iter()
                .map(|sample| i64::from(sample.saturating_abs()))
                .sum::<i64>() as f64
                / audio.len() as f64
        };
        let hash = replay_checksum_samples(&audio);
        let nonzero = first_nonzero.is_some();
        if jsonl {
            let first_nonzero = first_nonzero
                .map(|value| value.to_string())
                .unwrap_or_else(|| "null".to_string());
            println!(
                "{{\"frame\":{frame_index},\"samples\":735,\"channels\":2,\"peak\":{peak},\"first_nonzero\":{first_nonzero},\"mean_abs\":{mean_abs:.6},\"hash\":\"0x{hash:08x}\",\"ports\":[{},{},{},{}],\"apui\":[{},{},{},{}],\"music\":[{},{},{}],\"main\":{},\"sub\":{},\"subsub\":{},\"inidisp\":{}}}",
                ports[0],
                ports[1],
                ports[2],
                ports[3],
                game.ram[0x0648],
                game.ram[0x012c],
                game.ram[0x012d],
                game.ram[0x012e],
                game.ram[0x012f],
                game.ram[0x0132],
                game.ram[0x0133],
                game.ram[0x10],
                game.ram[0x11],
                game.ram[0xb0],
                game.ram[0x13],
            );
        } else if ports != last_ports || nonzero != last_nonzero || peak >= 12_000 {
            println!(
                "{frame_index:>5}: peak={peak:>5} first_nonzero={first_nonzero:?} ports={ports:02x?} apui={:02x}/{:02x}/{:02x}/{:02x} music={:02x}/{:02x}/{:02x} main={:02x} sub={:02x} subsub={:02x} inidisp={:02x} {}",
                game.ram[0x0648],
                game.ram[0x012c],
                game.ram[0x012d],
                game.ram[0x012e],
                game.ram[0x012f],
                game.ram[0x0132],
                game.ram[0x0133],
                game.ram[0x10],
                game.ram[0x11],
                game.ram[0xb0],
                game.ram[0x13],
                game.zelda_audio_debug_summary(),
            );
        }
        last_ports = ports;
        last_nonzero = nonzero;
    }
}

pub(crate) fn run_trace_snes9x_audio(args: &[String]) {
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-snes9x-audio <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-snes9x-audio <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(360);
    let mut game = load_play_state(rom_path);
    let mut audio = vec![0i16; 735 * 2];
    let mut snes9x = match LibretroCore::load(core_path, rom_path) {
        Ok(core) => core,
        Err(e) => {
            eprintln!("failed to initialize libretro core: {e}");
            process::exit(1);
        }
    };
    println!(
        "snes9x geometry={}x{} fps={:.9} sample_rate={:.3}",
        snes9x.geometry.base_width,
        snes9x.geometry.base_height,
        snes9x.av_info.timing.fps,
        snes9x.av_info.timing.sample_rate,
    );
    for frame_index in 0..frames {
        game.zelda_run_frame(0);
        let ports = game.zelda_debug_apu_write_ports();
        game.zelda_render_audio(&mut audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        let rust_peak = audio
            .iter()
            .map(|sample| sample.saturating_abs())
            .max()
            .unwrap_or(0);
        let rust_first = audio.iter().position(|&sample| sample != 0);

        let capture = snes9x.run_frame();
        let ref_peak = capture
            .audio
            .iter()
            .map(|sample| sample.saturating_abs())
            .max()
            .unwrap_or(0);
        let ref_first = capture.audio.iter().position(|&sample| sample != 0);
        if rust_peak >= 12_000
            || ref_peak >= 12_000
            || rust_first.is_some() != ref_first.is_some()
            || ports != [0, 0, 0, 0]
        {
            println!(
                "{frame_index:>5}: rust_peak={rust_peak:>5} rust_first={rust_first:?} ref_peak={ref_peak:>5} ref_first={ref_first:?} ref_samples={} ports={ports:02x?} {}",
                capture.audio.len() / 2,
                game.zelda_audio_debug_summary(),
            );
        }
    }
}

pub(crate) fn run_compare_snes9x_startup_audio(args: &[String]) {
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --compare-snes9x-startup-audio <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --compare-snes9x-startup-audio <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(180);
    let mut game = load_play_state(rom_path);
    let mut audio = vec![0i16; 735 * 2];
    let mut snes9x = match LibretroCore::load(core_path, rom_path) {
        Ok(core) => core,
        Err(e) => {
            eprintln!("failed to initialize libretro core: {e}");
            process::exit(1);
        }
    };

    let mut rust_stats = Vec::with_capacity(frames as usize);
    let mut ref_stats = Vec::with_capacity(frames as usize);
    let mut rust_debug = Vec::with_capacity(frames as usize);
    let mut ref_video_frames = 0usize;
    let mut ref_video_meta = None;
    for _ in 0..frames {
        game.zelda_run_frame(0);
        let ports = game.zelda_debug_apu_write_ports();
        game.zelda_render_audio(&mut audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        rust_stats.push(AudioFrameStats::from_interleaved_stereo(&audio));
        rust_debug.push(format!(
            "ports={ports:02x?} main={:02x} sub={:02x} subsub={:02x} {}",
            game.ram[0x10],
            game.ram[0x11],
            game.ram[0xb0],
            game.zelda_audio_debug_summary(),
        ));

        let capture = snes9x.run_frame();
        if !capture.video.is_empty() {
            ref_video_frames += 1;
            ref_video_meta.get_or_insert((
                capture.video_width,
                capture.video_height,
                capture.video_pitch,
                capture.pixel_format,
            ));
        }
        ref_stats.push(AudioFrameStats::from_interleaved_stereo(&capture.audio));
    }

    let threshold = 512i16;
    let rust_onset = first_peak_frame(&rust_stats, threshold);
    let ref_onset = first_peak_frame(&ref_stats, threshold);
    let rust_max = max_peak_frame(&rust_stats);
    let ref_max = max_peak_frame(&ref_stats);
    println!(
        "snes9x geometry={}x{} fps={:.9} sample_rate={:.3} video_frames={ref_video_frames}/{frames} first_video={ref_video_meta:?}",
        snes9x.geometry.base_width,
        snes9x.geometry.base_height,
        snes9x.av_info.timing.fps,
        snes9x.av_info.timing.sample_rate,
    );
    println!(
        "startup audio threshold={threshold}: rust_onset={rust_onset:?} ref_onset={ref_onset:?} rust_max={rust_max:?} ref_max={ref_max:?}",
    );
    if let (Some(rust_onset), Some(ref_onset)) = (rust_onset, ref_onset) {
        let delta = ref_onset as i32 - rust_onset as i32;
        println!("startup audio onset_delta_ref_minus_rust={delta} frames");
    }
    print_audio_window(
        "rust",
        &rust_stats,
        &rust_debug,
        rust_onset.or(rust_max.map(|(i, _)| i)),
    );
    print_audio_window(
        "snes9x",
        &ref_stats,
        &[],
        ref_onset.or(ref_max.map(|(i, _)| i)),
    );
}

pub(crate) fn run_dump_snes9x_frame(args: &[String]) {
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --dump-snes9x-frame <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--skip-snes9x-frames <n>]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --dump-snes9x-frame <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--skip-snes9x-frames <n>]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = match args.get(2).and_then(|s| s.parse().ok()) {
        Some(frames) => frames,
        None => {
            eprintln!(
                "usage: zelda3 --dump-snes9x-frame <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--skip-snes9x-frames <n>]"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(3) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-snes9x-frame <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--skip-snes9x-frames <n>]"
            );
            process::exit(2);
        }
    };
    let mut input_script = InputScript::default();
    let mut load_sram = None::<PathBuf>;
    let mut skip_snes9x_frames = 0u32;
    let mut i = 4usize;
    while i < args.len() {
        match args[i].as_str() {
            "--input-script" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--input-script requires a path");
                    process::exit(2);
                };
                input_script = match InputScript::from_path(Path::new(path)) {
                    Ok(script) => script,
                    Err(e) => {
                        eprintln!("failed to parse input script {}: {e}", path);
                        process::exit(2);
                    }
                };
                i += 2;
            }
            "--load-sram" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                };
                load_sram = Some(PathBuf::from(path));
                i += 2;
            }
            "--skip-snes9x-frames" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--skip-snes9x-frames requires a count");
                    process::exit(2);
                };
                skip_snes9x_frames = match value.parse() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("invalid --skip-snes9x-frames `{value}`: {e}");
                        process::exit(2);
                    }
                };
                i += 2;
            }
            flag => {
                eprintln!("unknown --dump-snes9x-frame option: {flag}");
                process::exit(2);
            }
        }
    }

    let load_sram_bytes = load_sram
        .as_deref()
        .map(|path| read_file_or_exit(path, "SRAM"));
    let mut snes9x =
        match LibretroCore::load_with_sram(core_path, rom_path, load_sram_bytes.as_deref()) {
            Ok(core) => core,
            Err(e) => {
                eprintln!("failed to initialize libretro core: {e}");
                process::exit(1);
            }
        };
    for _ in 0..skip_snes9x_frames {
        let _ = snes9x.run_frame_with_input(0);
    }
    let mut capture = None;
    for frame_index in 0..frames {
        let input = input_script.input_for_frame(frame_index);
        capture = Some(snes9x.run_frame_with_input(input));
    }
    let Some(capture) = capture else {
        eprintln!("frame count must be greater than zero");
        process::exit(2);
    };
    let Some(stride) = snes9x_pixel_stride(capture.pixel_format) else {
        eprintln!("unsupported snes9x pixel format {}", capture.pixel_format);
        process::exit(1);
    };
    let mut frame = vec![0u8; capture.video_width as usize * capture.video_height as usize * 4];
    for y in 0..capture.video_height as usize {
        for x in 0..capture.video_width as usize {
            let src = y * capture.video_pitch + x * stride;
            let Some([r, g, b, _]) = snes9x_rgba_pixel_at(&capture, src) else {
                eprintln!("failed to decode snes9x pixel at {x},{y}");
                process::exit(1);
            };
            let dst = (y * capture.video_width as usize + x) * 4;
            frame[dst] = b;
            frame[dst + 1] = g;
            frame[dst + 2] = r;
        }
    }
    if let Err(e) =
        write_argb_frame_png(&out_path, &frame, capture.video_width, capture.video_height)
    {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "dumped snes9x frame {frames} to {}; skip={skip_snes9x_frames}; geometry={}x{}; pixel_format={}; pitch={}",
        out_path.display(),
        capture.video_width,
        capture.video_height,
        capture.pixel_format,
        capture.video_pitch,
    );
}

pub(crate) fn run_trace_snes9x_memory(args: &[String]) {
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-snes9x-memory <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-snes9x-memory <path-to-snes9x-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(120);
    let mut snes9x = match LibretroCore::load(core_path, rom_path) {
        Ok(core) => core,
        Err(e) => {
            eprintln!("failed to load snes9x core: {e}");
            process::exit(1);
        }
    };
    println!(
        "snes9x memory probe geometry={}x{} fps={:.9} sample_rate={:.3}",
        snes9x.geometry.base_width,
        snes9x.geometry.base_height,
        snes9x.av_info.timing.fps,
        snes9x.av_info.timing.sample_rate,
    );
    for id in 0..=7 {
        let size = snes9x.memory_size(id);
        let present = snes9x.memory_bytes(id).is_some();
        println!(
            "memory id={id} name={} size={} present={present}",
            libretro_memory_name(id),
            size
        );
    }

    let mut last_apui00 = None;
    let mut last_system_digest = None;
    let mut last_save_digest = None;
    for frame in 0..frames {
        let capture = snes9x.run_frame();
        let system_ram = snes9x.memory_bytes(RETRO_MEMORY_SYSTEM_RAM);
        let save_ram = snes9x.memory_bytes(RETRO_MEMORY_SAVE_RAM);
        let apui00 = system_ram.and_then(|ram| ram.get(0x0648).copied());
        let system_digest = system_ram.map(fnv1a64);
        let save_digest = save_ram.map(fnv1a64);
        let changed = apui00 != last_apui00
            || system_digest != last_system_digest
            || save_digest != last_save_digest
            || frame < 4;
        if changed {
            println!(
                "frame={frame:>4} audio_samples={} video={}x{} apui00={} system_ram={} system_digest={} save_ram={} save_digest={}",
                capture.audio.len() / 2,
                capture.video_width,
                capture.video_height,
                format_optional_u8(apui00),
                system_ram.map(|ram| ram.len()).unwrap_or(0),
                format_optional_u64(system_digest),
                save_ram.map(|ram| ram.len()).unwrap_or(0),
                format_optional_u64(save_digest),
            );
        }
        last_apui00 = apui00;
        last_system_digest = system_digest;
        last_save_digest = save_digest;
    }
}

pub(crate) fn run_trace_song_bank(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("usage: zelda3 --trace-song-bank <path-to-rom.sfc> [asset-index]");
            process::exit(2);
        }
    };
    let asset = args
        .get(1)
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(0);
    let game = load_play_state(rom_path);
    println!("{}", game.zelda_debug_song_bank_summary(asset));
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ApuBootstrapCheckpoint {
    pub(crate) magic: [u8; 8],
    pub(crate) opcodes: u64,
    pub(crate) apu_cycles_per_cpu_cycle: f64,
    pub(crate) cpu_k: u8,
    pub(crate) cpu_pc: u16,
    pub(crate) spc_pc: u16,
    pub(crate) rom_readable: bool,
    pub(crate) payload_nonzero: usize,
    pub(crate) dsp_writes: usize,
    pub(crate) apu: snes::apu::ApuState,
}

pub(crate) fn load_play_crash_checkpoint(
    path: &Path,
) -> Result<PlayCrashCheckpoint, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let checkpoint: PlayCrashCheckpoint = bincode::deserialize(&bytes)?;
    if &checkpoint.magic != PLAY_CRASH_CHECKPOINT_MAGIC {
        return Err("not a zelda3-rs playable crash checkpoint".into());
    }
    Ok(checkpoint)
}

pub(crate) fn load_apu_bootstrap_checkpoint(
    path: &Path,
) -> Result<ApuBootstrapCheckpoint, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let checkpoint: ApuBootstrapCheckpoint = bincode::deserialize(&bytes)?;
    if &checkpoint.magic != APU_BOOTSTRAP_CHECKPOINT_MAGIC {
        return Err("not a zelda3-rs APU bootstrap checkpoint".into());
    }
    Ok(checkpoint)
}
