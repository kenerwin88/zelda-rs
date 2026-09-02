//! Absolute DSP event clock driven by the original Zelda 3 SPC program.
//!
//! The SPC700 executes only to preserve instruction workload, timer polling,
//! and DSP-register write order. [`snes::apu::ApuState::cycle_without_dsp`]
//! deliberately skips sample synthesis; [`crate::modern_audio::ModernAudioEngine`]
//! remains the sole renderer in production.

use crate::game_output::{DspWriteEvent, EngineAudioCommandBatch};
use snes::{apu::ApuState, snes9x_wram_refresh_cycle, CpuFieldTiming};

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
const SNES_SHORT_SCANLINE: u64 = 240;
const SNES_WRAM_REFRESH_STALL_MASTER_CLOCKS: u64 = 40;
const NMI_AUDIO_VCOUNTER: u64 = 225;
// Normal NTSC NMI entry jitters with the interrupted CPU instruction. H=84 is
// the route-stable nominal entry used by the absolute clock; the SPC scheduler
// below still rounds each write to the real SPC opcode boundary.
const NMI_AUDIO_NOMINAL_ENTRY_HCLOCK: u64 = 84;
// Snes9x 1.63's NTSC main-CPU-to-APU conversion ratio.
const SNES9X_APU_RATIO_NUMERATOR: u64 = 15_664;
const SNES9X_APU_RATIO_DENOMINATOR: u64 = 328_125;
const SMP_CPU_LOOKAHEAD_CYCLES: u64 = 19;
// Main-CPU instruction paths through the ROM's $80:8888 LoadSongBank
// routine, expressed in SNES master clocks. These are protocol timings, not
// route coordinates: every runtime song-bank upload executes these paths.
const SONG_BANK_WRITE_TO_FIRST_READY_POLL_MASTER_CLOCKS: u64 = 386;
const SONG_BANK_READY_LOW_TO_HIGH_MASTER_CLOCKS: u64 = 6;
const SONG_BANK_READY_HIGH_TO_NEXT_LOW_MASTER_CLOCKS: u64 = 52;
const SONG_BANK_ACK_POLL_MASTER_CLOCKS: u64 = 52;
const SONG_BANK_READY_TO_HEADER_MASTER_CLOCKS: u64 = 332;
const SONG_BANK_LAST_BYTE_ACK_TO_HEADER_MASTER_CLOCKS: u64 = 304;
const SONG_BANK_HEADER_TARGET_LOW_TO_HIGH_MASTER_CLOCKS: u64 = 6;
const SONG_BANK_HEADER_TARGET_HIGH_TO_FLAG_MASTER_CLOCKS: u64 = 106;
const SONG_BANK_HEADER_FLAG_TO_TOKEN_MASTER_CLOCKS: u64 = 74;
const SONG_BANK_HEADER_TOKEN_TO_ACK_POLL_MASTER_CLOCKS: u64 = 30;
const SONG_BANK_HEADER_ACK_TO_FIRST_BYTE_MASTER_CLOCKS: u64 = 210;
const SONG_BANK_BYTE_COUNTER_TO_VALUE_MASTER_CLOCKS: u64 = 6;
const SONG_BANK_BYTE_VALUE_TO_ACK_POLL_MASTER_CLOCKS: u64 = 236;
const SONG_BANK_LAST_BYTE_VALUE_TO_ACK_POLL_MASTER_CLOCKS: u64 = 82;
const SONG_BANK_BYTE_ACK_TO_NEXT_COUNTER_MASTER_CLOCKS: u64 = 82;
const SONG_BANK_TERMINAL_ACK_TO_CLEAR_MASTER_CLOCKS: u64 = 62;
const SONG_BANK_CLEAR_PORT_MASTER_CLOCKS: u64 = 30;
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
        effect2_input: u8,
        vwf_glyph_tone_boundary_policy: u8,
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

        // The original NMI always latches the two one-shot effect ports. A VWF
        // glyph which was already complete at the queued NMI boundary retains
        // its captured APUI03 value once. A merely owned marker is not enough:
        // frame 20620's still-drawing glyph has a real clear at this boundary,
        // while frames 20764 and 20857 retain a completed click even if an
        // intervening NMI has already cleared the physical port latch.
        writes[2] = Some(ports[2]);
        writes[3] = Some(match vwf_glyph_tone_boundary_policy {
            // Version-8 snapshots created before value-carrying markers only
            // recorded "complete". Prefer the physical latch, with the ROM's
            // dialogue click as the only reconstructable fallback.
            2 | 3 => effect2_input.max(12),
            marker if marker & 0xc0 == 0x80 => marker & 0x3f,
            _ => ports[3],
        });
        HostPortWrites { writes }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HostPortWrites {
    writes: [Option<u8>; 4],
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SongBankHostTransfer {
    bank_id: u8,
    stream: Vec<u8>,
    cursor: usize,
    phase: SongBankHostTransferPhase,
    command_pending: bool,
    next_host_access_master_clock: Option<u64>,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
enum SongBankHostTransferPhase {
    AwaitReceiverReadyLow,
    AwaitReceiverReadyHigh {
        low_matched: bool,
    },
    WriteBlockTargetLow {
        token: u8,
        length: u16,
        target: u16,
    },
    WriteBlockTargetHigh {
        token: u8,
        length: u16,
        target: u16,
    },
    WriteBlockFlag {
        token: u8,
        length: u16,
    },
    WriteBlockToken {
        token: u8,
        length: u16,
    },
    AwaitBlockAck {
        token: u8,
        length: u16,
    },
    WriteByteCounter {
        counter: u8,
        value: u8,
        bytes_remaining: u16,
    },
    WriteByteValue {
        counter: u8,
        value: u8,
        bytes_remaining: u16,
    },
    AwaitByteAck {
        counter: u8,
        bytes_remaining: u16,
    },
    ClearPort {
        port: u8,
    },
}

impl SongBankHostTransfer {
    fn new(bank_id: u8, stream: &[u8]) -> Self {
        Self {
            bank_id,
            stream: stream.to_vec(),
            cursor: 0,
            phase: SongBankHostTransferPhase::AwaitReceiverReadyLow,
            command_pending: true,
            next_host_access_master_clock: None,
        }
    }

    fn suppresses_nmi_transport(&self) -> bool {
        !self.command_pending
    }

    fn mark_command_scheduled(&mut self, command_apu_cycle: u64) {
        if !self.command_pending {
            return;
        }
        self.command_pending = false;
        let command_master_clock = apu_cycle_to_snes_master_clock(command_apu_cycle);
        self.next_host_access_master_clock =
            Some(command_master_clock + SONG_BANK_WRITE_TO_FIRST_READY_POLL_MASTER_CLOCKS);
    }

    fn advance_host_cpu_until(
        &mut self,
        execution_apu_cycle: u64,
        in_ports: &mut [u8; 6],
        out_ports: [u8; 4],
    ) -> bool {
        while self
            .next_host_access_apu_cycle()
            .is_some_and(|cycle| cycle <= execution_apu_cycle)
        {
            if self.perform_host_access(in_ports, out_ports) {
                return true;
            }
        }
        false
    }

    fn perform_host_access(&mut self, in_ports: &mut [u8; 6], out_ports: [u8; 4]) -> bool {
        match self.phase {
            SongBankHostTransferPhase::AwaitReceiverReadyLow => {
                self.phase = SongBankHostTransferPhase::AwaitReceiverReadyHigh {
                    low_matched: out_ports[0] == 0xaa,
                };
                self.schedule_after(SONG_BANK_READY_LOW_TO_HIGH_MASTER_CLOCKS);
            }
            SongBankHostTransferPhase::AwaitReceiverReadyHigh { low_matched } => {
                if low_matched && out_ports[1] == 0xbb {
                    self.schedule_block_header(0xcc, SONG_BANK_READY_TO_HEADER_MASTER_CLOCKS);
                } else {
                    self.phase = SongBankHostTransferPhase::AwaitReceiverReadyLow;
                    self.schedule_after(SONG_BANK_READY_HIGH_TO_NEXT_LOW_MASTER_CLOCKS);
                }
            }
            SongBankHostTransferPhase::WriteBlockTargetLow {
                token,
                length,
                target,
            } => {
                Self::write_input_port(in_ports, 2, target as u8);
                self.phase = SongBankHostTransferPhase::WriteBlockTargetHigh {
                    token,
                    length,
                    target,
                };
                self.schedule_after(SONG_BANK_HEADER_TARGET_LOW_TO_HIGH_MASTER_CLOCKS);
            }
            SongBankHostTransferPhase::WriteBlockTargetHigh {
                token,
                length,
                target,
            } => {
                Self::write_input_port(in_ports, 3, (target >> 8) as u8);
                self.phase = SongBankHostTransferPhase::WriteBlockFlag { token, length };
                self.schedule_after(SONG_BANK_HEADER_TARGET_HIGH_TO_FLAG_MASTER_CLOCKS);
            }
            SongBankHostTransferPhase::WriteBlockFlag { token, length } => {
                Self::write_input_port(in_ports, 1, u8::from(length != 0));
                self.phase = SongBankHostTransferPhase::WriteBlockToken { token, length };
                self.schedule_after(SONG_BANK_HEADER_FLAG_TO_TOKEN_MASTER_CLOCKS);
            }
            SongBankHostTransferPhase::WriteBlockToken { token, length } => {
                Self::write_input_port(in_ports, 0, token);
                self.phase = SongBankHostTransferPhase::AwaitBlockAck { token, length };
                self.schedule_after(SONG_BANK_HEADER_TOKEN_TO_ACK_POLL_MASTER_CLOCKS);
            }
            SongBankHostTransferPhase::AwaitBlockAck { token, length } => {
                if out_ports[0] != token {
                    self.schedule_after(SONG_BANK_ACK_POLL_MASTER_CLOCKS);
                } else if length == 0 {
                    self.phase = SongBankHostTransferPhase::ClearPort { port: 0 };
                    self.schedule_after(SONG_BANK_TERMINAL_ACK_TO_CLEAR_MASTER_CLOCKS);
                } else {
                    self.schedule_byte(0, length, SONG_BANK_HEADER_ACK_TO_FIRST_BYTE_MASTER_CLOCKS);
                }
            }
            SongBankHostTransferPhase::WriteByteCounter {
                counter,
                value,
                bytes_remaining,
            } => {
                Self::write_input_port(in_ports, 0, counter);
                self.phase = SongBankHostTransferPhase::WriteByteValue {
                    counter,
                    value,
                    bytes_remaining,
                };
                self.schedule_after(SONG_BANK_BYTE_COUNTER_TO_VALUE_MASTER_CLOCKS);
            }
            SongBankHostTransferPhase::WriteByteValue {
                counter,
                value,
                bytes_remaining,
            } => {
                Self::write_input_port(in_ports, 1, value);
                self.phase = SongBankHostTransferPhase::AwaitByteAck {
                    counter,
                    bytes_remaining,
                };
                self.schedule_after(if bytes_remaining == 0 {
                    SONG_BANK_LAST_BYTE_VALUE_TO_ACK_POLL_MASTER_CLOCKS
                } else {
                    SONG_BANK_BYTE_VALUE_TO_ACK_POLL_MASTER_CLOCKS
                });
            }
            SongBankHostTransferPhase::AwaitByteAck {
                counter,
                bytes_remaining,
            } => {
                if out_ports[0] != counter {
                    self.schedule_after(SONG_BANK_ACK_POLL_MASTER_CLOCKS);
                } else if bytes_remaining == 0 {
                    self.schedule_block_header(
                        // The preceding CMP $2140 equality leaves carry set,
                        // so the ROM's ADC #$03 advances the next block token
                        // by four, not three.
                        counter.wrapping_add(4),
                        SONG_BANK_LAST_BYTE_ACK_TO_HEADER_MASTER_CLOCKS,
                    );
                } else {
                    self.schedule_byte(
                        counter.wrapping_add(1),
                        bytes_remaining,
                        SONG_BANK_BYTE_ACK_TO_NEXT_COUNTER_MASTER_CLOCKS,
                    );
                }
            }
            SongBankHostTransferPhase::ClearPort { port } => {
                Self::write_input_port(in_ports, port, 0);
                if port == 3 {
                    self.next_host_access_master_clock = None;
                    return true;
                }
                self.phase = SongBankHostTransferPhase::ClearPort { port: port + 1 };
                self.schedule_after(SONG_BANK_CLEAR_PORT_MASTER_CLOCKS);
            }
        }
        false
    }

    fn schedule_block_header(&mut self, token: u8, delay_master_clocks: u64) {
        let length = self.read_stream_word();
        let target = if length == 0 {
            // Zelda's runtime receiver returns to the resident driver after a
            // zero-length block. The extracted asset ends at the length word;
            // the ROM's adjacent stream supplies this $0800 return address.
            SPC_DRIVER_START as u16
        } else {
            self.read_stream_word()
        };
        self.phase = SongBankHostTransferPhase::WriteBlockTargetLow {
            token,
            length,
            target,
        };
        self.schedule_after(delay_master_clocks);
    }

    fn schedule_byte(&mut self, counter: u8, bytes_remaining: u16, delay_master_clocks: u64) {
        let value = *self
            .stream
            .get(self.cursor)
            .expect("validated song-bank stream ended inside a block");
        self.cursor += 1;
        self.phase = SongBankHostTransferPhase::WriteByteCounter {
            counter,
            value,
            bytes_remaining: bytes_remaining - 1,
        };
        self.schedule_after(delay_master_clocks);
    }

    fn schedule_after(&mut self, master_clocks: u64) {
        let next = self
            .next_host_access_master_clock
            .expect("scheduled song-bank host transfer has no CPU clock");
        self.next_host_access_master_clock =
            Some(advance_snes_cpu_master_clock(next, master_clocks));
    }

    fn next_host_access_apu_cycle(&self) -> Option<u64> {
        self.next_host_access_master_clock
            .map(snes_master_clock_to_apu_cycle)
    }

    fn write_input_port(in_ports: &mut [u8; 6], port: u8, value: u8) {
        in_ports[usize::from(port)] = value;
    }

    fn read_stream_word(&mut self) -> u16 {
        let bytes = self
            .stream
            .get(self.cursor..self.cursor + 2)
            .expect("validated song-bank stream ended before its terminator");
        self.cursor += 2;
        u16::from_le_bytes(bytes.try_into().unwrap())
    }
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
    pending_main_cpu_port_writes: Vec<(u8, u8)>,
    #[serde(default)]
    song_bank_transfer: Option<SongBankHostTransfer>,
    #[serde(default)]
    completed_song_bank_id: Option<u8>,
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
            pending_main_cpu_port_writes: Vec::new(),
            song_bank_transfer: None,
            completed_song_bank_id: None,
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
        self.pending_main_cpu_port_writes.clear();
        self.song_bank_transfer = None;
        self.completed_song_bank_id = None;
        self.host_frame_index = 0;
        self.host_transport = HostPortTransport::default();
        self.apu.dsp_write_history.clear();
        self.apu.debug_dsp_write_trace = Some(Vec::new());
    }

    pub(crate) fn advance(
        &mut self,
        commands: EngineAudioCommandBatch,
        samples_per_channel: u32,
        vwf_glyph_tone_boundary_policy: u8,
    ) -> DspClockWindow {
        let frame_start_cycle = self.absolute_apu_cycle;
        let apu_cycles = u64::from(samples_per_channel) * APU_CYCLES_PER_DSP_SAMPLE;
        let frame_end_cycle = frame_start_cycle.wrapping_add(apu_cycles);
        // The SPC's local 32-bit cycle counter would wrap after ~250k host
        // frames (route frame 251820), freezing the execution loop below.
        // Move whole DSP-sample multiples into the 64-bit origin instead.
        if self.apu.cycles >= 1 << 30 {
            let shift = self.apu.cycles & !0x3f;
            self.apu.rebase_cycles(shift);
            self.apu_cycle_origin += u64::from(shift);
        }
        if self.apu.debug_dsp_write_trace.is_none() {
            self.apu.debug_dsp_write_trace = Some(Vec::new());
        }
        self.apu.debug_dsp_write_trace.as_mut().unwrap().clear();
        let mut polls = Vec::new();
        let host_writes = if self
            .song_bank_transfer
            .as_ref()
            .is_some_and(SongBankHostTransfer::suppresses_nmi_transport)
        {
            HostPortWrites { writes: [None; 4] }
        } else {
            self.host_transport.frame_writes(
                commands,
                self.apu.out_ports,
                self.apu.in_ports[3],
                vwf_glyph_tone_boundary_policy,
            )
        };
        let host_port_targets = host_port_target_cycles(self.host_frame_index, host_writes);
        let debug_transport = std::env::var("ZELDA3_DEBUG_SPC_TRANSPORT_FRAME")
            .ok()
            .and_then(|frame| frame.parse::<u64>().ok())
            == Some(self.host_frame_index);
        if debug_transport {
            eprintln!(
                "spc_transport host={} window=[{}, {}) targets={:?} relative={:?} writes={:?} vwf_boundary_policy={} execution={} origin={} local={} port_latches={:02x?}",
                self.host_frame_index,
                frame_start_cycle,
                frame_end_cycle,
                host_port_targets,
                host_port_targets.map(|target| target.saturating_sub(frame_start_cycle)),
                host_writes.writes,
                vwf_glyph_tone_boundary_policy,
                self.apu_cycle_origin + u64::from(self.apu.cycles),
                self.apu_cycle_origin,
                self.apu.cycles,
                &self.apu.ram[0x08..0x0c],
            );
        }
        let mut host_port_events: Vec<(u64, u8, u8)> = host_writes
            .writes
            .into_iter()
            .enumerate()
            .filter_map(|(port, value)| {
                value.map(|value| (host_port_targets[port], port as u8, value))
            })
            .collect();
        // Main-CPU port writes occur after the frame's NMI audio handler. The
        // translated engine executes main work as one semantic slice, so the
        // end of the current host window is its hardware-ordering boundary.
        // Keeping these writes separate from the NMI latches preserves their
        // one-shot nature and lets the real SPC program decide when it sees
        // them at its next instruction boundary.
        host_port_events.extend(
            std::mem::take(&mut self.pending_main_cpu_port_writes)
                .into_iter()
                .map(|(port, value)| (frame_end_cycle, port, value)),
        );
        if let Some(transfer) = self.song_bank_transfer.as_mut() {
            transfer.mark_command_scheduled(frame_end_cycle);
        }
        host_port_events.sort_by_key(|&(target, port, _)| (target, port));
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
                while next_host_port < host_port_events.len() {
                    let (event_target, port, value) = host_port_events[next_host_port];
                    if event_target > instruction_end {
                        break;
                    }
                    let target = event_target.max(last_host_boundary);
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
                        port,
                        value,
                    );
                    if debug_transport {
                        eprintln!(
                            "spc_transport_event host={} port={} value={:02x} target={} instruction=[{}, {}] boundary={}",
                            self.host_frame_index,
                            port,
                            value,
                            event_target,
                            instruction_start,
                            instruction_end,
                            boundary,
                        );
                    }
                    last_host_boundary = boundary;
                    next_host_port += 1;
                }
            }
            if execution_cycle >= self.apu_cycle_origin {
                self.apu.run_cycle_sequenced_instruction_without_dsp();
            }
            execution_cycle = self.apu_cycle_origin + u64::from(self.apu.cycles);
            let transfer_completed = self.song_bank_transfer.as_mut().is_some_and(|transfer| {
                transfer.advance_host_cpu_until(
                    execution_cycle,
                    &mut self.apu.in_ports,
                    self.apu.out_ports,
                )
            });
            if transfer_completed {
                let transfer = self.song_bank_transfer.take().unwrap();
                self.completed_song_bank_id = Some(transfer.bank_id);
            }
            if let Some(step_durations) = step_durations {
                let predicted_end =
                    instruction_start + step_durations.iter().copied().map(u64::from).sum::<u64>();
                debug_assert_eq!(execution_cycle, predicted_end, "opcode ${opcode:02x}");
            } else {
                // Single-step opcodes do not yield back to the CPU until the
                // whole instruction completes. Writes whose nominal clock
                // falls inside one become visible immediately afterward.
                while next_host_port < host_port_events.len() {
                    let (event_target, port, value) = host_port_events[next_host_port];
                    if event_target > execution_cycle {
                        break;
                    }
                    self.apu.in_ports[usize::from(port)] = value;
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

    pub(crate) fn queue_main_cpu_port_write(&mut self, port: u8, value: u8) {
        debug_assert!(port < 4);
        self.pending_main_cpu_port_writes.push((port, value));
    }

    pub(crate) fn begin_song_bank_transfer(&mut self, bank_id: u8, stream: &[u8]) {
        debug_assert!(self.song_bank_transfer.is_none());
        self.song_bank_transfer = Some(SongBankHostTransfer::new(bank_id, stream));
        self.queue_main_cpu_port_write(0, 0xff);
    }

    pub(crate) fn take_completed_song_bank_id(&mut self) -> Option<u8> {
        self.completed_song_bank_id.take()
    }

    pub(crate) fn debug_state_summary(&self) -> String {
        format!(
            "abs={} origin={} local={} pc={:04x} sp={:02x} a={:02x} x={:02x} y={:02x} z={} c={} t0={},{},{},{},{} ram43={} in={:02x?} out={:02x?} transfer={:?}",
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
            self.apu.out_ports,
            self.song_bank_transfer
                .as_ref()
                .map(|transfer| transfer.phase),
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

fn snes_master_clock_to_apu_cycle(master_clock: u64) -> u64 {
    master_clock.saturating_mul(SNES9X_APU_RATIO_NUMERATOR) / SNES9X_APU_RATIO_DENOMINATOR
}

fn apu_cycle_to_snes_master_clock(apu_cycle: u64) -> u64 {
    let scaled = u128::from(apu_cycle) * u128::from(SNES9X_APU_RATIO_DENOMINATOR);
    let numerator = u128::from(SNES9X_APU_RATIO_NUMERATOR);
    ((scaled + numerator - 1) / numerator) as u64
}

fn snes_frame_start_master_clock(frame: u64) -> u64 {
    frame
        .saturating_mul(SNES_MASTER_CLOCKS_PER_SCANLINE * SNES_NTSC_SCANLINES_PER_FRAME)
        .saturating_sub((frame / 2) * SNES_SHORT_SCANLINE_MASTER_CLOCKS)
}

fn snes_frame_for_master_clock(master_clock: u64) -> u64 {
    let nominal_frame_clocks = SNES_MASTER_CLOCKS_PER_SCANLINE * SNES_NTSC_SCANLINES_PER_FRAME;
    let mut frame = master_clock / (nominal_frame_clocks - 2);
    while snes_frame_start_master_clock(frame) > master_clock {
        frame -= 1;
    }
    while snes_frame_start_master_clock(frame + 1) <= master_clock {
        frame += 1;
    }
    frame
}

fn snes_scanline_clock(master_clock: u64) -> (u64, u64, u64, u64) {
    let frame = snes_frame_for_master_clock(master_clock);
    let frame_start = snes_frame_start_master_clock(frame);
    let frame_clock = master_clock - frame_start;
    let short_field = frame & 1 != 0;
    let short_line_start = SNES_SHORT_SCANLINE * SNES_MASTER_CLOCKS_PER_SCANLINE;

    if short_field && frame_clock >= short_line_start {
        let short_line_length = SNES_MASTER_CLOCKS_PER_SCANLINE - SNES_SHORT_SCANLINE_MASTER_CLOCKS;
        if frame_clock < short_line_start + short_line_length {
            return (
                frame,
                SNES_SHORT_SCANLINE,
                frame_start + short_line_start,
                frame_clock - short_line_start,
            );
        }
        let after_short_line = frame_clock - short_line_start - short_line_length;
        let scanline = SNES_SHORT_SCANLINE + 1 + after_short_line / SNES_MASTER_CLOCKS_PER_SCANLINE;
        let scanline_start = frame_start
            + short_line_start
            + short_line_length
            + (scanline - SNES_SHORT_SCANLINE - 1) * SNES_MASTER_CLOCKS_PER_SCANLINE;
        return (
            frame,
            scanline,
            scanline_start,
            master_clock - scanline_start,
        );
    }

    let scanline = frame_clock / SNES_MASTER_CLOCKS_PER_SCANLINE;
    let scanline_start = frame_start + scanline * SNES_MASTER_CLOCKS_PER_SCANLINE;
    (
        frame,
        scanline,
        scanline_start,
        master_clock - scanline_start,
    )
}

/// Return the next pinned-core WRAM-refresh event on the physical timeline.
///
/// Source authority in Snes9x 1.63: `ppu.h` defines `SModel::_5A22`,
/// `globals.cpp` gives the selected M1SNES model `_5A22 == 2`, `cpu.cpp`
/// initializes that model at v2/H=538, and `cpuexec.cpp` carries the phase by
/// toggling H=534/H=538 at HMax. The one exception is the transition into
/// odd-field, non-interlace V=240, where `cpuexec.cpp` deliberately does not
/// toggle. Consequently the phase must carry across field boundaries rather
/// than being reconstructed from local scanline parity.
fn next_wram_refresh_master_clock(master_clock: u64) -> u64 {
    let (frame, scanline, scanline_start, scanline_clock) = snes_scanline_clock(master_clock);
    let field_timing = CpuFieldTiming::NON_INTERLACE_EVEN;
    let refresh_cycle = u64::from(snes9x_wram_refresh_cycle(
        frame,
        scanline as u16,
        field_timing,
    ));
    if scanline_clock <= refresh_cycle {
        return scanline_start + refresh_cycle;
    }

    let scanline_length = if frame & 1 != 0 && scanline == SNES_SHORT_SCANLINE {
        SNES_MASTER_CLOCKS_PER_SCANLINE - SNES_SHORT_SCANLINE_MASTER_CLOCKS
    } else {
        SNES_MASTER_CLOCKS_PER_SCANLINE
    };
    let next_scanline_start = scanline_start + scanline_length;
    let (next_frame, next_scanline, _, next_scanline_clock) =
        snes_scanline_clock(next_scanline_start);
    debug_assert_eq!(next_scanline_clock, 0);
    next_scanline_start
        + u64::from(snes9x_wram_refresh_cycle(
            next_frame,
            next_scanline as u16,
            field_timing,
        ))
}

fn advance_snes_cpu_master_clock(mut master_clock: u64, mut cpu_master_clocks: u64) -> u64 {
    while cpu_master_clocks != 0 {
        let refresh = next_wram_refresh_master_clock(master_clock);
        let clocks_until_refresh = refresh - master_clock;
        if cpu_master_clocks <= clocks_until_refresh {
            return master_clock + cpu_master_clocks;
        }
        master_clock = refresh + SNES_WRAM_REFRESH_STALL_MASTER_CLOCKS;
        cpu_master_clocks -= clocks_until_refresh;
    }
    master_clock
}

fn host_port_target_cycles(frame_index: u64, writes: HostPortWrites) -> [u64; 4] {
    // Libretro frame zero begins while the reset/bootstrap frame is already in
    // progress, so normal NMI frame N is one host callback behind the callback
    // index. Non-interlace NTSC loses one dot on scanline 240 every other field.
    let completed_video_frames = frame_index.saturating_sub(1);
    let frame_start_master_clock = snes_frame_start_master_clock(completed_video_frames);
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
    master_clock_offsets
        .map(|offset| snes_master_clock_to_apu_cycle(nmi_entry_master_clock + offset))
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
    fn runtime_song_bank_transfer_follows_the_rom_block_protocol() {
        let stream = [
            2, 0, 0x00, 0x20, 0x11, 0x22, // first block
            1, 0, 0x00, 0x30, 0x33, // second block
            0, 0, // return to the resident driver
        ];
        let mut transfer = SongBankHostTransfer::new(1, &stream);
        transfer.mark_command_scheduled(0);
        let mut in_ports = [0u8; 6];
        let mut targets = Vec::new();
        let mut tokens = Vec::new();
        let mut bytes = Vec::new();
        let mut completed = false;

        for _ in 0..64 {
            let phase = transfer.phase;
            match phase {
                SongBankHostTransferPhase::WriteBlockTargetLow { target, .. } => {
                    targets.push(target);
                }
                SongBankHostTransferPhase::WriteBlockToken { token, .. } => tokens.push(token),
                SongBankHostTransferPhase::WriteByteValue { counter, value, .. } => {
                    bytes.push((counter, value));
                }
                _ => {}
            }
            let mut out_ports = [0u8; 4];
            match phase {
                SongBankHostTransferPhase::AwaitReceiverReadyLow => out_ports[0] = 0xaa,
                SongBankHostTransferPhase::AwaitReceiverReadyHigh { .. } => out_ports[1] = 0xbb,
                SongBankHostTransferPhase::AwaitBlockAck { token, .. } => out_ports[0] = token,
                SongBankHostTransferPhase::AwaitByteAck { counter, .. } => {
                    out_ports[0] = counter;
                }
                _ => {}
            }
            if transfer.perform_host_access(&mut in_ports, out_ports) {
                completed = true;
                break;
            }
        }

        assert!(completed);
        assert_eq!(targets, [0x2000, 0x3000, SPC_DRIVER_START as u16]);
        // CMP $2140 leaves carry set, so the ROM's ADC #3 advances the
        // terminal counter by four between blocks.
        assert_eq!(tokens, [0xcc, 5, 4]);
        assert_eq!(bytes, [(0, 0x11), (1, 0x22), (0, 0x33)]);
        assert_eq!(&in_ports[..4], &[0; 4]);
    }

    #[test]
    fn m1_v2_wram_refresh_phase_carries_through_the_odd_short_field() {
        use snes::CpuRasterPosition;

        // The carried phase starts at H=538, alternates on normal HMax, skips
        // the transition into odd non-interlace V=240, and therefore reaches
        // the next even field shifted. These are state-history assertions, not
        // a local scanline-parity approximation.
        let cases = [
            (0, 0, 538),
            (0, 1, 534),
            (1, 239, 534),
            (1, 240, 534),
            (1, 241, 538),
            (2, 0, 534),
        ];
        for (field, scanline, refresh_cycle) in cases {
            let scanline_start = CpuFieldTiming::NON_INTERLACE_EVEN
                .master_cycles_at(field, CpuRasterPosition::new(scanline, 0));
            assert_eq!(
                next_wram_refresh_master_clock(scanline_start),
                scanline_start + refresh_cycle,
            );
            assert_eq!(
                advance_snes_cpu_master_clock(scanline_start + refresh_cycle - 6, 6),
                scanline_start + refresh_cycle,
            );
            assert_eq!(
                advance_snes_cpu_master_clock(scanline_start + refresh_cycle, 6),
                scanline_start + refresh_cycle + 46,
            );
            assert_eq!(
                advance_snes_cpu_master_clock(scanline_start + refresh_cycle - 6, 12),
                scanline_start + refresh_cycle + 46,
            );
        }
    }

    #[test]
    fn m1_v2_wram_refresh_phase_remains_exact_after_24000_fields() {
        use snes::CpuRasterPosition;

        // By this point 12,001 odd V=240 transitions have skipped their
        // toggle. A field-local parity formula would incorrectly return H=538.
        let field = 24_002;
        let scanline_start = CpuFieldTiming::NON_INTERLACE_EVEN
            .master_cycles_at(field, CpuRasterPosition::new(0, 0));
        assert_eq!(
            next_wram_refresh_master_clock(scanline_start),
            scanline_start + 534,
        );
        assert_eq!(
            advance_snes_cpu_master_clock(scanline_start + 528, 12),
            scanline_start + 580,
        );
    }

    #[test]
    fn host_transport_holds_commands_until_the_spc_acknowledges_them() {
        let mut transport = HostPortTransport::default();
        let track_one = EngineAudioCommandBatch::from_legacy_ports([1, 2, 3, 4]);
        let track_two = EngineAudioCommandBatch::from_legacy_ports([2, 2, 3, 4]);
        let clear = EngineAudioCommandBatch::from_legacy_ports([0, 2, 3, 4]);

        assert_eq!(
            transport.frame_writes(track_one, [0; 4], 0, 0).writes,
            [Some(1), Some(2), Some(3), Some(4)]
        );
        assert_eq!(
            transport.frame_writes(track_one, [0; 4], 0, 0).writes,
            [None, None, Some(3), Some(4)]
        );
        assert_eq!(
            transport.frame_writes(track_one, [1, 0, 3, 0], 0, 0).writes,
            [Some(0), None, Some(3), Some(4)]
        );
        assert_eq!(
            transport.frame_writes(clear, [1, 2, 3, 4], 0, 0).writes,
            [Some(0), None, Some(3), Some(4)]
        );
        assert_eq!(
            transport.frame_writes(track_two, [1, 2, 3, 4], 0, 0).writes,
            [Some(2), None, Some(3), Some(4)]
        );
    }

    #[test]
    fn ambient_clear_and_same_effect_retrigger_remain_distinct_writes() {
        let mut transport = HostPortTransport::default();
        let effect = EngineAudioCommandBatch::from_legacy_ports([0, 3, 0, 0]);
        let clear = EngineAudioCommandBatch::from_legacy_ports([0, 0, 0, 0]);

        assert_eq!(
            transport.frame_writes(effect, [1, 0, 0, 0], 0, 0).writes[1],
            Some(3)
        );
        assert_eq!(
            transport.frame_writes(clear, [1, 3, 0, 0], 0, 0).writes[1],
            Some(0)
        );
        assert_eq!(
            transport.frame_writes(effect, [1, 3, 0, 0], 0, 0).writes[1],
            Some(3)
        );
    }

    #[test]
    fn host_transport_distinguishes_owned_and_completed_vwf_boundaries() {
        let mut transport = HostPortTransport::default();
        let effect = EngineAudioCommandBatch::from_legacy_ports([0, 0, 0, 12]);
        let clear = EngineAudioCommandBatch::from_legacy_ports([0, 0, 0, 0]);

        assert_eq!(
            transport.frame_writes(effect, [0; 4], 0, 1).writes[3],
            Some(12)
        );
        assert_eq!(
            transport.frame_writes(clear, [0; 4], 12, 1).writes[3],
            Some(0)
        );
        assert_eq!(
            transport.frame_writes(clear, [0; 4], 12, 2).writes[3],
            Some(12)
        );
        assert_eq!(
            transport.frame_writes(clear, [0; 4], 0, 0x80 | 12).writes[3],
            Some(12)
        );
        assert_eq!(
            transport.frame_writes(clear, [0; 4], 12, 0x80).writes[3],
            Some(0)
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
