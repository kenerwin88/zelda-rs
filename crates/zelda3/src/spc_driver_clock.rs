//! Absolute DSP event clock driven by the original Zelda 3 SPC program.
//!
//! The SPC700 executes only to preserve instruction workload, timer polling,
//! and DSP-register write order. [`snes::apu::ApuState::cycle_without_dsp`]
//! deliberately skips sample synthesis; [`crate::modern_audio::ModernAudioEngine`]
//! remains the sole renderer in production.

use crate::game_output::{DspWriteEvent, EngineAudioCommandBatch};
use snes::apu::ApuState;

const SPC_DRIVER_START: usize = 0x0800;
const SPC_DRIVER_END: usize = 0x179e;
const SPC_DRIVER_LENGTH: usize = SPC_DRIVER_END - SPC_DRIVER_START;
const DRIVER_POLL_PUSH_Y_PC: u16 = 0x0878;
const DRIVER_POLL_AFTER_PUSH_PC: u16 = 0x0879;
const MAX_INITIALIZATION_APU_CYCLES: usize = 1_000_000;
const APU_CYCLES_PER_DSP_SAMPLE: u64 = 32;
const SNES_MASTER_CLOCKS_PER_SCANLINE: u64 = 1_364;
const SNES_NTSC_SCANLINES_PER_FRAME: u64 = 262;
const SNES_SHORT_SCANLINE_MASTER_CLOCKS: u64 = 4;
const NMI_AUDIO_VCOUNTER: u64 = 225;
// Normal NTSC NMI entry jitters with the interrupted CPU instruction. H=84 is
// the route-stable nominal entry used by the absolute clock; the SPC scheduler
// below still rounds each write to the real SPC opcode boundary.
const NMI_AUDIO_NOMINAL_ENTRY_HCLOCK: u64 = 84;
// Snes9x 1.63's NTSC main-CPU-to-APU conversion ratio.
const SNES9X_APU_RATIO_NUMERATOR: u64 = 15_664;
const SNES9X_APU_RATIO_DENOMINATOR: u64 = 328_125;
const SMP_CPU_LOOKAHEAD_CYCLES: u64 = 19;
// Clean-ROM Snes9x 1.63 reaches the uploaded Zelda SPC entry point at this
// absolute APU cycle. The translated CPU skips the IPL transfer itself, so the
// timing clock holds the extracted program dormant until the equivalent
// lifecycle boundary, then executes the real program from $0800.
const ROM_BOOTSTRAP_DRIVER_EXECUTION_ORIGIN: u64 = 1_355_567;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DriverPollEvent {
    pub(crate) absolute_apu_cycle: u64,
    pub(crate) absolute_dsp_cycle: u64,
    pub(crate) ticks: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DspClockWindow {
    pub(crate) writes: Vec<DspWriteEvent>,
    pub(crate) polls: Vec<DriverPollEvent>,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct HostPortTransport {
    requested_music: u8,
    requested_ambient: u8,
    ambient_input: u8,
}

impl HostPortTransport {
    fn frame_writes(
        &mut self,
        commands: EngineAudioCommandBatch,
        acknowledgements: [u8; 4],
    ) -> HostPortWrites {
        let ports = commands.legacy_ports();
        let mut writes = [None; 4];

        // The ROM only writes APUI00 for a new music command, or writes zero
        // after the SPC has acknowledged the last one. Merely retaining the
        // typed command latch does not perform another host-port write.
        if ports[0] != self.requested_music {
            self.requested_music = ports[0];
            writes[0] = Some(ports[0]);
        } else if acknowledgements[0] == self.requested_music {
            writes[0] = Some(0);
        }

        // APUI01 is likewise conditional. Track the engine latch separately
        // from the last acknowledged effect so 0 -> same-effect retriggers are
        // still visible after an intervening clear.
        if ports[1] != self.ambient_input {
            self.ambient_input = ports[1];
            writes[1] = Some(ports[1]);
            if ports[1] != 0 {
                self.requested_ambient = ports[1];
            }
        } else if ports[1] == 0 && acknowledgements[1] == self.requested_ambient {
            writes[1] = Some(0);
        }

        // The original NMI always latches the two one-shot effect ports.
        writes[2] = Some(ports[2]);
        writes[3] = Some(ports[3]);
        HostPortWrites { writes }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HostPortWrites {
    writes: [Option<u8>; 4],
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct AbsoluteDspEventClock {
    apu: ApuState,
    startup_template: ApuState,
    absolute_apu_cycle: u64,
    apu_cycle_origin: u64,
    next_poll_boundary: u16,
    pending_writes: Vec<(u64, u8, u8)>,
    #[serde(default)]
    host_frame_index: u64,
    #[serde(default)]
    host_transport: HostPortTransport,
}

impl AbsoluteDspEventClock {
    pub(crate) fn new(driver: &[u8], intro_bank: &[u8]) -> Result<Self, String> {
        if driver.len() != SPC_DRIVER_LENGTH {
            return Err(format!(
                "SPC timing program has {} bytes, expected {SPC_DRIVER_LENGTH}",
                driver.len()
            ));
        }
        if driver.get(..4) != Some(&[0x20, 0xcd, 0xcf, 0xbd][..]) {
            return Err("SPC timing program has an invalid $0800 entry point".to_string());
        }

        let mut apu = ApuState::new();
        apu.reset();
        apu.ram[SPC_DRIVER_START..SPC_DRIVER_END].copy_from_slice(driver);
        upload_song_bank_to_ram(&mut apu.ram, intro_bank)?;
        apu.rom_readable = false;
        apu.spc.pc = SPC_DRIVER_START as u16;
        apu.debug_dsp_write_trace = Some(Vec::new());

        let startup_template = apu.clone();
        let mut clock = Self {
            apu,
            startup_template,
            absolute_apu_cycle: 0,
            apu_cycle_origin: 0,
            next_poll_boundary: DRIVER_POLL_PUSH_Y_PC,
            pending_writes: Vec::new(),
            host_frame_index: 0,
            host_transport: HostPortTransport::default(),
        };
        clock.synchronize_after_driver_initialization()?;
        Ok(clock)
    }

    pub(crate) fn upload_song_bank(&mut self, bank: &[u8]) -> Result<(), String> {
        upload_song_bank_to_ram(&mut self.apu.ram, bank)?;
        upload_song_bank_to_ram(&mut self.startup_template.ram, bank)
    }

    pub(crate) fn configure_rom_bootstrap(&mut self) {
        self.apu = self.startup_template.clone();
        self.absolute_apu_cycle = 0;
        self.apu_cycle_origin = ROM_BOOTSTRAP_DRIVER_EXECUTION_ORIGIN;
        for (timer, frequency) in self.apu.timer.iter_mut().zip([128u64, 128, 16]) {
            let elapsed_phase = (self.apu_cycle_origin % frequency) as u8;
            timer.cycles = if elapsed_phase == 0 {
                0
            } else {
                frequency as u8 - elapsed_phase
            };
        }
        self.next_poll_boundary = DRIVER_POLL_PUSH_Y_PC;
        self.pending_writes.clear();
        self.host_frame_index = 0;
        self.host_transport = HostPortTransport::default();
        self.apu.dsp_write_history.clear();
        self.apu.debug_dsp_write_trace = Some(Vec::new());
    }

    pub(crate) fn advance(
        &mut self,
        commands: EngineAudioCommandBatch,
        samples_per_channel: u32,
    ) -> DspClockWindow {
        let frame_start_cycle = self.absolute_apu_cycle;
        let apu_cycles = u64::from(samples_per_channel) * APU_CYCLES_PER_DSP_SAMPLE;
        let frame_end_cycle = frame_start_cycle.wrapping_add(apu_cycles);
        if self.apu.debug_dsp_write_trace.is_none() {
            self.apu.debug_dsp_write_trace = Some(Vec::new());
        }
        self.apu.debug_dsp_write_trace.as_mut().unwrap().clear();
        let mut polls = Vec::new();
        let host_writes = self
            .host_transport
            .frame_writes(commands, self.apu.out_ports);
        let host_port_targets = host_port_target_cycles(self.host_frame_index, host_writes);
        let mut next_host_port = 0usize;
        let mut last_host_boundary = 0u64;
        let execution_target = frame_end_cycle + SMP_CPU_LOOKAHEAD_CYCLES;
        let mut execution_cycle = self.apu_cycle_origin + u64::from(self.apu.cycles);
        while execution_cycle < execution_target {
            let instruction_start = execution_cycle;
            let opcode = self.apu.ram[self.apu.spc.pc as usize];
            let step_durations = snes9x_opcode_step_durations(opcode);
            if let Some(step_durations) = step_durations {
                let instruction_cycles = step_durations.iter().copied().map(u64::from).sum::<u64>();
                let instruction_end = instruction_start + instruction_cycles;
                while next_host_port < 4 {
                    let Some(value) = host_writes.writes[next_host_port] else {
                        next_host_port += 1;
                        continue;
                    };
                    if host_port_targets[next_host_port] > instruction_end {
                        break;
                    }
                    let target = host_port_targets[next_host_port].max(last_host_boundary);
                    let boundary = if target <= instruction_start {
                        instruction_start
                    } else {
                        let relative_target = target - instruction_start;
                        let mut relative_boundary = 0u64;
                        for &duration in step_durations {
                            relative_boundary += u64::from(duration);
                            if relative_boundary >= relative_target {
                                break;
                            }
                        }
                        instruction_start + relative_boundary
                    };
                    self.apu.schedule_input_port_event(
                        boundary.saturating_sub(self.apu_cycle_origin) as u32,
                        next_host_port as u8,
                        value,
                    );
                    last_host_boundary = boundary;
                    next_host_port += 1;
                }
            }
            if execution_cycle >= self.apu_cycle_origin {
                self.apu.run_cycle_sequenced_instruction_without_dsp();
            }
            execution_cycle = self.apu_cycle_origin + u64::from(self.apu.cycles);
            if let Some(step_durations) = step_durations {
                let predicted_end =
                    instruction_start + step_durations.iter().copied().map(u64::from).sum::<u64>();
                debug_assert_eq!(execution_cycle, predicted_end, "opcode ${opcode:02x}");
            } else {
                // Single-step opcodes do not yield back to the CPU until the
                // whole instruction completes. Writes whose nominal clock
                // falls inside one become visible immediately afterward.
                while next_host_port < 4 {
                    let Some(value) = host_writes.writes[next_host_port] else {
                        next_host_port += 1;
                        continue;
                    };
                    if host_port_targets[next_host_port] > execution_cycle {
                        break;
                    }
                    self.apu.in_ports[next_host_port] = value;
                    last_host_boundary = execution_cycle;
                    next_host_port += 1;
                }
            }
            if execution_cycle >= self.apu_cycle_origin {
                if let Some(ticks) = self.observe_poll_boundary() {
                    if execution_cycle < frame_end_cycle {
                        polls.push(DriverPollEvent {
                            absolute_apu_cycle: execution_cycle,
                            absolute_dsp_cycle: execution_cycle / APU_CYCLES_PER_DSP_SAMPLE,
                            ticks,
                        });
                    }
                }
            }
        }
        self.absolute_apu_cycle = frame_end_cycle;
        self.host_frame_index = self.host_frame_index.wrapping_add(1);

        let mut absolute_writes = std::mem::take(&mut self.pending_writes);
        absolute_writes.extend(
            self.apu
                .debug_dsp_write_trace
                .as_mut()
                .unwrap()
                .drain(..)
                .map(|(apu_cycle, addr, value)| {
                    (self.apu_cycle_origin + u64::from(apu_cycle), addr, value)
                }),
        );
        let mut writes = Vec::new();
        for (absolute_apu_cycle, addr, value) in absolute_writes {
            if absolute_apu_cycle >= frame_end_cycle {
                self.pending_writes.push((absolute_apu_cycle, addr, value));
            } else if absolute_apu_cycle >= frame_start_cycle {
                let relative_apu_cycle = absolute_apu_cycle.saturating_sub(frame_start_cycle);
                writes.push(DspWriteEvent::new(
                    addr,
                    value,
                    (relative_apu_cycle / APU_CYCLES_PER_DSP_SAMPLE) as i32,
                    (absolute_apu_cycle % APU_CYCLES_PER_DSP_SAMPLE) as u8,
                ));
            }
        }
        DspClockWindow { writes, polls }
    }

    pub(crate) fn absolute_dsp_cycle(&self) -> u64 {
        self.absolute_apu_cycle / APU_CYCLES_PER_DSP_SAMPLE
    }

    pub(crate) fn host_acknowledgements(&self) -> [u8; 4] {
        self.apu.out_ports
    }

    pub(crate) fn debug_state_summary(&self) -> String {
        format!(
            "abs={} origin={} local={} pc={:04x} sp={:02x} a={:02x} x={:02x} y={:02x} z={} c={} t0={},{},{},{},{} ram43={} ports={:02x?}",
            self.absolute_apu_cycle,
            self.apu_cycle_origin,
            self.apu.cycles,
            self.apu.spc.pc,
            self.apu.spc.sp,
            self.apu.spc.a,
            self.apu.spc.x,
            self.apu.spc.y,
            self.apu.spc.z,
            self.apu.spc.c,
            self.apu.timer[0].cycles,
            self.apu.timer[0].divider,
            self.apu.timer[0].target,
            self.apu.timer[0].counter,
            self.apu.timer[0].enabled,
            self.apu.ram[0x43],
            self.apu.in_ports,
        )
    }

    pub(crate) fn begin_debug_instruction_trace(&mut self) {
        self.apu.debug_spc_instruction_trace = Some(Vec::new());
    }

    pub(crate) fn take_debug_instruction_trace(
        &mut self,
    ) -> (u64, Vec<snes::apu::SpcInstructionTrace>) {
        (
            self.apu_cycle_origin,
            self.apu
                .debug_spc_instruction_trace
                .take()
                .unwrap_or_default(),
        )
    }

    fn synchronize_after_driver_initialization(&mut self) -> Result<(), String> {
        for _ in 0..MAX_INITIALIZATION_APU_CYCLES {
            self.apu.run_cycle_sequenced_instruction_without_dsp();
            if self.observe_poll_boundary().is_some() {
                self.absolute_apu_cycle = 0;
                self.apu.cycles = 0;
                self.apu.dsp_write_history.clear();
                self.apu.debug_dsp_write_trace.as_mut().unwrap().clear();
                return Ok(());
            }
        }
        Err(format!(
            "SPC timing program did not reach driver poll loop within {MAX_INITIALIZATION_APU_CYCLES} cycles"
        ))
    }

    fn observe_poll_boundary(&mut self) -> Option<u8> {
        if self.apu.spc.pc != self.next_poll_boundary {
            return None;
        }
        self.next_poll_boundary = if self.next_poll_boundary == DRIVER_POLL_PUSH_Y_PC {
            DRIVER_POLL_AFTER_PUSH_PC
        } else {
            DRIVER_POLL_PUSH_Y_PC
        };
        (self.next_poll_boundary == DRIVER_POLL_PUSH_Y_PC).then_some(self.apu.spc.y)
    }
}

/// Snes9x 1.63's SMP coroutine yields only between opcode micro-steps. Most
/// opcodes are a single step; these instructions have resumable internal
/// steps. Durations are SPC hardware cycles and are independent of frame or
/// game route.
fn snes9x_opcode_step_durations(opcode: u8) -> Option<&'static [u8]> {
    Some(match opcode {
        0x7e => &[2, 1],
        0x8f => &[3, 1, 1],
        0xaa => &[3, 1],
        0xaf => &[3, 1],
        0xba => &[2, 2, 1],
        0xbf => &[2, 2],
        0xc4 => &[2, 1, 1],
        0xc5 => &[2, 1, 1, 1],
        0xc6 => &[2, 1, 1],
        0xc7 => &[3, 1, 1, 1, 1],
        0xc9 => &[2, 1, 1, 1],
        0xca => &[3, 2, 1],
        0xcb => &[2, 1, 1],
        0xcc => &[2, 1, 1, 1],
        0xd4 => &[3, 1, 1],
        0xd5 => &[4, 1, 1],
        0xd6 => &[4, 1, 1],
        0xd7 => &[2, 1, 2, 1, 1],
        0xd8 => &[2, 1, 1],
        0xd9 => &[3, 1, 1],
        0xda => &[2, 1, 1, 1],
        0xdb => &[3, 1, 1],
        0xe4 => &[2, 1],
        0xe5 => &[2, 1, 1],
        0xe6 => &[2, 1],
        0xe7 => &[3, 1, 1, 1],
        0xe9 => &[3, 1],
        0xeb => &[2, 1],
        0xec => &[3, 1],
        0xf4 => &[3, 1],
        0xf5 => &[4, 1],
        0xf6 => &[4, 1],
        0xf7 => &[3, 1, 1, 1],
        0xf8 => &[2, 1],
        0xf9 => &[3, 1],
        0xfa => &[2, 1, 1, 1],
        0xfb => &[3, 1],
        _ => return None,
    })
}

fn host_port_target_cycles(frame_index: u64, writes: HostPortWrites) -> [u64; 4] {
    // Libretro frame zero begins while the reset/bootstrap frame is already in
    // progress, so normal NMI frame N is one host callback behind the callback
    // index. Non-interlace NTSC loses one dot on scanline 240 every other field.
    let completed_video_frames = frame_index.saturating_sub(1);
    let short_scanlines = completed_video_frames / 2;
    let frame_start_master_clock = completed_video_frames
        .saturating_mul(SNES_MASTER_CLOCKS_PER_SCANLINE * SNES_NTSC_SCANLINES_PER_FRAME)
        .saturating_sub(short_scanlines * SNES_SHORT_SCANLINE_MASTER_CLOCKS);
    let nmi_entry_master_clock = frame_start_master_clock
        + NMI_AUDIO_VCOUNTER * SNES_MASTER_CLOCKS_PER_SCANLINE
        + NMI_AUDIO_NOMINAL_ENTRY_HCLOCK;

    // These are instruction-path durations from the clean ROM's NMI entry to
    // its APUI stores. A written music port lengthens the path to the ambient
    // branch; the ambient branch then has separate no-write, clear, and new-
    // effect paths. This models the transport protocol rather than assigning a
    // special phase to any route frame or DSP command.
    let (port_zero_offset, music_suffix): (u64, u64) = match writes.writes[0] {
        Some(0) => (508, 46),
        Some(_) => (444, 128),
        None => (0, 0),
    };
    let (port_one_offset, port_two_offset, port_three_offset): (u64, u64, u64) =
        match writes.writes[1] {
            Some(0) => (640, 724, 786),
            Some(_) => (600, 694, 756),
            None => (0, 678, 740),
        };
    let master_clock_offsets = [
        port_zero_offset,
        port_one_offset + music_suffix,
        port_two_offset + music_suffix,
        port_three_offset + music_suffix,
    ];
    master_clock_offsets.map(|offset| {
        (nmi_entry_master_clock + offset).saturating_mul(SNES9X_APU_RATIO_NUMERATOR)
            / SNES9X_APU_RATIO_DENOMINATOR
    })
}

fn upload_song_bank_to_ram(ram: &mut [u8], data: &[u8]) -> Result<(), String> {
    let mut cursor = 0usize;
    loop {
        let header = data
            .get(cursor..cursor + 2)
            .ok_or_else(|| "song bank upload is missing its terminator".to_string())?;
        let length = usize::from(u16::from_le_bytes(header.try_into().unwrap()));
        if length == 0 {
            return Ok(());
        }
        let target_bytes = data
            .get(cursor + 2..cursor + 4)
            .ok_or_else(|| "song bank upload has a truncated block header".to_string())?;
        let mut target = usize::from(u16::from_le_bytes(target_bytes.try_into().unwrap()));
        cursor += 4;
        let payload = data
            .get(cursor..cursor + length)
            .ok_or_else(|| "song bank upload has a truncated block payload".to_string())?;
        for &byte in payload {
            ram[target] = byte;
            target = (target + 1) & 0xffff;
        }
        cursor += length;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn song_bank_upload_materializes_blocks_and_wraps_address_space() {
        let mut ram = vec![0; 0x10000];
        let bank = [
            3, 0, 0x00, 0x20, 1, 2, 3, // regular block
            3, 0, 0xff, 0xff, 4, 5, 6, // wrapping block
            0, 0,
        ];

        upload_song_bank_to_ram(&mut ram, &bank).unwrap();

        assert_eq!(&ram[0x2000..0x2003], &[1, 2, 3]);
        assert_eq!(ram[0xffff], 4);
        assert_eq!(&ram[..2], &[5, 6]);
    }

    #[test]
    fn song_bank_upload_rejects_truncated_payload() {
        let mut ram = vec![0; 0x10000];
        let error = upload_song_bank_to_ram(&mut ram, &[3, 0, 0, 0x20, 1]).unwrap_err();

        assert!(error.contains("truncated block payload"));
    }

    #[test]
    fn host_transport_holds_commands_until_the_spc_acknowledges_them() {
        let mut transport = HostPortTransport::default();
        let track_one = EngineAudioCommandBatch::from_legacy_ports([1, 2, 3, 4]);
        let track_two = EngineAudioCommandBatch::from_legacy_ports([2, 2, 3, 4]);
        let clear = EngineAudioCommandBatch::from_legacy_ports([0, 2, 3, 4]);

        assert_eq!(
            transport.frame_writes(track_one, [0; 4]).writes,
            [Some(1), Some(2), Some(3), Some(4)]
        );
        assert_eq!(
            transport.frame_writes(track_one, [0; 4]).writes,
            [None, None, Some(3), Some(4)]
        );
        assert_eq!(
            transport.frame_writes(track_one, [1, 0, 3, 0]).writes,
            [Some(0), None, Some(3), Some(4)]
        );
        assert_eq!(
            transport.frame_writes(clear, [1, 2, 3, 4]).writes,
            [Some(0), None, Some(3), Some(4)]
        );
        assert_eq!(
            transport.frame_writes(track_two, [1, 2, 3, 4]).writes,
            [Some(2), None, Some(3), Some(4)]
        );
    }

    #[test]
    fn ambient_clear_and_same_effect_retrigger_remain_distinct_writes() {
        let mut transport = HostPortTransport::default();
        let effect = EngineAudioCommandBatch::from_legacy_ports([0, 3, 0, 0]);
        let clear = EngineAudioCommandBatch::from_legacy_ports([0, 0, 0, 0]);

        assert_eq!(
            transport.frame_writes(effect, [1, 0, 0, 0]).writes[1],
            Some(3)
        );
        assert_eq!(
            transport.frame_writes(clear, [1, 3, 0, 0]).writes[1],
            Some(0)
        );
        assert_eq!(
            transport.frame_writes(effect, [1, 3, 0, 0]).writes[1],
            Some(3)
        );
    }

    #[test]
    fn nmi_host_port_targets_follow_the_ntsc_frame_phase() {
        // The nominal CPU clocks land just before Snes9x's observed coroutine
        // boundaries at +46 and +58; the opcode-step scheduler performs that
        // final rounding rather than baking the observed cycles into a table.
        let writes = HostPortWrites {
            writes: [Some(0), Some(0), Some(0), Some(0)],
        };
        assert_eq!(host_port_target_cycles(2339, writes)[3] - 39_900_704, 46);
        assert_eq!(host_port_target_cycles(3423, writes)[3] - 58_393_632, 54);
    }

    #[test]
    fn nmi_host_ports_are_separate_ordered_events() {
        let targets = host_port_target_cycles(
            3423,
            HostPortWrites {
                writes: [Some(0), Some(0), Some(0), Some(0)],
            },
        );

        assert!(targets.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(targets[3] - targets[0], 16);
    }

    #[test]
    fn skipped_music_write_advances_a_new_ambient_effect_to_its_real_nmi_path() {
        let common = host_port_target_cycles(
            1136,
            HostPortWrites {
                writes: [Some(0), Some(0), Some(0), Some(0)],
            },
        );
        let ambient = host_port_target_cycles(
            1136,
            HostPortWrites {
                writes: [None, Some(3), Some(0), Some(0)],
            },
        );

        assert_eq!(common[1] - ambient[1], 4);
        assert!(ambient.windows(2).skip(1).all(|pair| pair[0] < pair[1]));
    }
}
