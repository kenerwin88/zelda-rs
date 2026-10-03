//! Cold native reset/upload clock. The main CPU executes a semantic upload
//! protocol over native asset bytes; no ROM CPU, trace, or measured timestamp
//! supplies its work or the receiver's acknowledgements.

use snes::apu::{ApuHostPortProbe, ApuHostPortTiming, ApuState};
use snes::cpu_timeline::CpuSynchronousTimelineEvent;
use snes::{CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PortAccess {
    pub master_clock: u64,
    pub port: u8,
    pub value: u8,
    pub read: bool,
}

/// Constructed only by completing the native cold upload. This is not a
/// caller-supplied clock seed or a source CPU snapshot.
#[derive(Clone, Debug)]
pub(crate) struct NativeBootstrapOrigin {
    pub(super) apu: ApuState,
    pub(super) apu_clock: snes::snes9x_apu_clock::Snes9xApuClockCheckpoint,
    cpu: NativeBootstrapCpu,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeBootstrapCpu {
    pub(crate) timeline: CpuMasterTimeline,
    pub(crate) upload_pointer: [u8; 3],
    pub(crate) startup: crate::cycle_models::startup::Startup,
    pub(crate) poly_worker: Option<crate::cycle_models::poly_worker::PolyWorker>,
    pub(crate) ppu: crate::cycle_models::native_ppu::NativePpuState,
}

impl NativeBootstrapOrigin {
    pub(crate) fn into_cpu(self) -> NativeBootstrapCpu {
        self.cpu
    }
}

pub(crate) fn native_bootstrap_origin(stream: &[u8]) -> Result<NativeBootstrapOrigin, String> {
    // Asset loading can repeat on reset and restore. Cache only this pure
    // result, keyed by every input byte; never cache an observed source clock.
    static CACHE: std::sync::Mutex<Option<(Vec<u8>, NativeBootstrapOrigin)>> =
        std::sync::Mutex::new(None);
    if let Some((bytes, origin)) = &*CACHE.lock().unwrap() {
        if bytes == stream {
            return Ok(origin.clone());
        }
    }
    let origin = ColdUpload::run(stream, |_| {})?.into_origin()?;
    *CACHE.lock().unwrap() = Some((stream.to_vec(), origin.clone()));
    Ok(origin)
}

pub(super) struct ColdUpload<F> {
    timeline: CpuMasterTimeline,
    apu: ApuHostPortTiming,
    observe: F,
    upload_bank: u8,
    ppu: crate::cycle_models::native_ppu::NativePpuState,
}

impl<F: FnMut(PortAccess)> ColdUpload<F> {
    fn new(observe: F) -> Result<Self, String> {
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        let probe = ApuHostPortProbe::from_snes9x_coroutine(apu).map_err(|e| e.to_string())?;
        let apu = ApuHostPortTiming::new(probe, Default::default()).map_err(|e| e.to_string())?;
        // cpu.cpp:S9xSoftResetCPU initializes Cycles=182, then reads the
        // reset vector with one16-clock ROM word transaction. This is the
        // hardware reset contract, not a sampled entry into a game routine.
        let mut timeline = CpuMasterTimeline::new(
            182,
            CpuBusWorkload::with_dynamic_hdma(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline
            .begin_synchronous_timeline()
            .map_err(|e| e.to_string())?;
        let mut this = Self {
            timeline,
            apu,
            observe,
            upload_bank: 0x19,
            ppu: crate::cycle_models::native_ppu::NativePpuState::reset(),
        };
        this.work(16)?;
        Ok(this)
    }

    fn work(&mut self, cycles: u32) -> Result<(), String> {
        // Fail closed on an invalid asset/receiver that never acknowledges.
        // This is a work limit, not a selected execution or handoff timestamp.
        if self.timeline.clock_master_cycles() > 1 << 28 {
            return Err("native bootstrap receiver did not finish".to_owned());
        }
        let apu = &mut self.apu;
        let ppu = &mut self.ppu;
        self.timeline
            .advance_synchronous_after_semantics_with(cycles, |event, at| {
                ppu.timeline_event(event);
                if matches!(event, CpuSynchronousTimelineEvent::HMax { .. }) {
                    apu.end_scanline_at(at.master_cycles())
                        .map_err(|e| e.to_string())?;
                }
                // Reset disables HDMA/DMA; NMI remains masked throughout this
                // upload. The timeline itself owns refresh and field rollover.
                Ok(0)
            })
    }

    fn instructions(&mut self, costs: &[u32]) -> Result<(), String> {
        for &cost in costs {
            self.work(cost)?;
        }
        Ok(())
    }

    fn read(&mut self, port: u8) -> Result<u8, String> {
        let master_clock = self.timeline.clock_master_cycles();
        let value = self
            .apu
            .read_cpu_port_at(master_clock, port)
            .map_err(|e| e.to_string())?;
        (self.observe)(PortAccess {
            master_clock,
            port,
            value,
            read: true,
        });
        self.work(6)?;
        Ok(value)
    }

    fn write(&mut self, port: u8, value: u8) -> Result<(), String> {
        let master_clock = self.timeline.clock_master_cycles();
        self.apu
            .write_cpu_port_at(master_clock, port, value)
            .map_err(|e| e.to_string())?;
        (self.observe)(PortAccess {
            master_clock,
            port,
            value,
            read: false,
        });
        self.work(6)
    }

    fn store(&mut self, port: u8, value: u8) -> Result<(), String> {
        self.work(24)?; // slow-ROM opcode/address fetches
        self.write(port, value)
    }

    fn await_ack(&mut self, token: u8) -> Result<(), String> {
        loop {
            self.work(24)?; // CMP abs, before APUI read
            let equal = self.read(0)? == token;
            self.work(if equal { 16 } else { 22 })?; // BNE
            if equal {
                return Ok(());
            }
        }
    }

    /// Execute through Sound_LoadIntroSongBank's return, retaining the native
    /// physical clock and the one APU owner. `stream` is the cold upload asset,
    /// with the original optional terminal destination accepted for tests.
    pub(super) fn run(stream: &[u8], observe: F) -> Result<Self, String> {
        let mut this = Self::new(observe)?;
        this.instructions(&[14, 30, 30, 30])?; // SEI; mask NMI, HDMA, DMA
        for port in 0..4 {
            this.store(port, 0)?;
        }
        this.instructions(&[16, 30, 14, 14, 22, 24, 14, 24, 14, 22, 46])?;
        // Intro pointer stores, SEI/JSR, then PHP/REP/LDY/LDA in LoadSongBank.
        this.instructions(&[16, 24, 16, 24, 16, 24, 14, 46, 22, 22, 24, 24])?;
        loop {
            this.work(24)?;
            let low = this.read(0)?;
            let high = this.read(1)?;
            let ready = low == 0xaa && high == 0xbb;
            this.work(if ready { 16 } else { 22 })?;
            if ready {
                break;
            }
        }
        this.instructions(&[22, 16, 22])?; // SEP; LDA #CC; BRA header
        let mut cursor = 0usize;
        let mut token = 0xccu8;
        loop {
            let length = read_word(stream, &mut cursor)?;
            let target = if length == 0 && cursor == stream.len() {
                super::SPC_DRIVER_START as u16
            } else {
                read_word(stream, &mut cursor)?
            };
            this.instructions(&[22, 22, 56, 14, 14, 14, 56, 14, 14])?;
            this.work(24)?; // STA16 APUI2/3
            this.write(2, target as u8)?;
            this.write(3, (target >> 8) as u8)?;
            this.instructions(&[22, 24, 16, 14])?;
            this.store(1, u8::from(length != 0))?;
            this.instructions(&[16, 28])?;
            this.store(0, token)?;
            this.await_ack(token)?;
            this.work(if length == 0 { 16 } else { 22 })?; // BVS
            if length == 0 {
                break;
            }

            let mut counter = 0u8;
            this.instructions(&[48, 14, 20, 16, 22])?; // first byte, XBA, LDA0, BRA
            for remaining in (0..length).rev() {
                let value = *stream
                    .get(cursor)
                    .ok_or("truncated native bootstrap block")?;
                cursor += 1;
                this.work(22)?; // REP
                this.work(24)?; // STA16 APUI0/1
                this.write(0, counter)?;
                this.write(1, value)?;
                this.instructions(&[22, 14])?; // SEP/DEX
                this.work(if remaining == 0 { 16 } else { 22 })?;
                if remaining != 0 {
                    // Prefetch the next byte before awaiting the preceding
                    // acknowledgement. Y wraps and advances the ROM bank.
                    this.instructions(&[20, 48, 14, 24])?;
                    let next_y = (cursor + 1) & 0x7fff;
                    this.work(if next_y == 0 { 16 } else { 22 })?;
                    if next_y == 0 {
                        this.instructions(&[24, 38])?;
                        this.upload_bank = this.upload_bank.wrapping_add(1);
                    }
                    this.work(20)?;
                }
                this.await_ack(counter)?;
                if remaining != 0 {
                    this.work(14)?; // INC A
                    counter = counter.wrapping_add(1);
                }
            }
            // Equal CMP leaves carry set. ADC #3 therefore advances by4;
            // if the result is zero, the taken BEQ repeats ADC with new carry.
            let mut carry = true;
            token = counter;
            loop {
                this.work(16)?;
                let sum = u16::from(token) + 3 + u16::from(carry);
                token = sum as u8;
                carry = sum > 255;
                this.work(if token == 0 { 22 } else { 16 })?;
                if token != 0 {
                    break;
                }
            }
        }
        for port in 0..4 {
            this.store(port, 0)?;
        }
        this.instructions(&[28, 42, 14, 42])?; // PLP/RTS; intro CLI/RTS
        Ok(this)
    }

    fn into_origin(self) -> Result<NativeBootstrapOrigin, String> {
        let apu_clock = self.apu.clock().checkpoint();
        let probe = self
            .apu
            .into_probe()
            .map_err(|_| "native bootstrap returned with a suspended receiver instruction")?;
        // The owned IPL receiver has finished its jump into the uploaded
        // native program. Accept only the complete instruction boundary;
        // never round a partly executed receiver to a desired entry clock.
        if probe.machine().spc.pc != super::SPC_DRIVER_START as u16 {
            return Err("native bootstrap did not return at the uploaded driver entry".to_owned());
        }
        Ok(NativeBootstrapOrigin {
            apu_clock,
            apu: probe.into_machine_without_dsp()
                .map_err(|_| "native bootstrap lost its completed receiver boundary")?,
            cpu: NativeBootstrapCpu {
                startup: crate::cycle_models::startup::Startup::after_upload(&self.timeline),
                poly_worker: None,
                timeline: self.timeline,
                upload_pointer: [0x00, 0x80, self.upload_bank],
                ppu: self.ppu,
            },
        })
    }
}

fn read_word(stream: &[u8], cursor: &mut usize) -> Result<u16, String> {
    let bytes = stream
        .get(*cursor..*cursor + 2)
        .ok_or("truncated native bootstrap header")?;
    *cursor += 2;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn native_reset_ppu_and_startup_reach_first_nmi_on_the_cold_source_clock() {
        use crate::cycle_models::startup::StartupStep;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
        let mut cpu = native_bootstrap_origin(&rom[0xc8000..0xd5337]).unwrap().into_cpu();
        let mut ram = vec![0x55; 0x20000];
        ram[..3].copy_from_slice(&cpu.upload_pointer);
        let mut sram = vec![0xff; 0x2000];
        // The source starts at reset too. No source snapshot seeds the native
        // registers, memory, clock or PPU bus state.
        let reset = snes::Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&sram)).unwrap();
        let mut source = reset;
        let mut source_upload_instructions = 0;
        while (source.machine().snes().cpu.k, source.machine().snes().cpu.pc) != (0, 0x802c) {
            assert!(source_upload_instructions < 2_000_000);
            source.step().unwrap();
            source_upload_instructions += 1;
        }
        assert_eq!(cpu.timeline, *source.machine().timeline());
        let mut instructions = 0;
        let context = loop {
            assert!(instructions < 100_000);
            let step = cpu.startup.step(&mut cpu.timeline, &mut ram, &mut sram, &mut cpu.ppu);
            match step {
                StartupStep::Instruction | StartupStep::ClearWindow => {
                    let receipt = source.step().unwrap();
                    instructions += 1;
                    if receipt.accepted_interrupt.is_some() {
                        assert_eq!(cpu.startup.step(&mut cpu.timeline, &mut ram, &mut sram,
                            &mut cpu.ppu), StartupStep::AcceptNmi);
                        let mut context = cpu.startup.accepted_context();
                        context.accept_interrupt_clocked(&mut ram, &mut cpu.timeline);
                        assert_eq!(cpu.timeline, *source.machine().timeline());
                        break context;
                    }
                    assert_eq!(cpu.timeline, *source.machine().timeline(), "startup instruction {instructions}");
                }
                StartupStep::MemoryComplete | StartupStep::HostReturn => {}
                other => panic!("unexpected first-reset boundary {other:?}"),
            }
        };
        assert_eq!(ram.iter().zip(&source.machine().snes().ram).position(|(a, b)| a != b), None,
            "first differing startup WRAM byte");
        assert_eq!(sram.iter().zip(&source.machine().snes().cart.ram).position(|(a, b)| a != b), None,
            "first differing startup SRAM byte");
        // Reset/upload/initialization have not read any PPU registers.
        assert_eq!(cpu.ppu.read_state(), snes::SourcePpuReadState::snes9x_reset());
        let expected = &source.machine().snes().cpu;
        assert_eq!((context.a, context.x, context.y, context.d, context.db, context.bank,
            context.pc, context.sp, context.p),
            (expected.a, expected.x, expected.y, expected.dp, expected.db, expected.k,
                expected.pc, expected.sp, expected.pack_flags()));
        eprintln!("cold source upload {source_upload_instructions} instructions; native startup {instructions} instructions; first NMI acceptance at {}", cpu.timeline.clock_master_cycles());
    }

    #[test]
    fn native_upload_handoff_preserves_receiver_state_and_continues_driver_clock() {
        use super::super::{AbsoluteDspEventClock, SPC_DRIVER_START, SPC_DRIVER_END};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let Ok(rom) = std::fs::read(root.join("saves/zelda3.sfc")) else { return };
        let upload = ColdUpload::run(&rom[0xc8000..0xd5337], |_| {}).unwrap();
        let receiver = bincode::serialize(upload.apu.machine()).unwrap();
        let origin = upload.into_origin().unwrap();
        assert_eq!(bincode::serialize(&origin.apu).unwrap(), receiver);
        // An empty standalone song bank deliberately cannot supply the actual
        // uploaded RAM, register or hardware state required at this handoff.
        let mut clock = AbsoluteDspEventClock::new(
            &origin.apu.ram[SPC_DRIVER_START..SPC_DRIVER_END], &[0, 0],
        ).unwrap();
        assert_ne!(bincode::serialize(&clock.apu).unwrap(), receiver);
        clock.configure_rom_bootstrap(&origin);
        assert_eq!(bincode::serialize(&clock.apu).unwrap(), receiver);
        assert_eq!(clock.apu_cycle_origin + u64::from(clock.apu.cycles),
            u64::from(origin.apu.cycles));
        assert_eq!(clock.host_acknowledgements(), origin.apu.out_ports);
        // A pre-upload output window must neither run the driver early nor
        // reconstruct its clock/ports. It is still the same native receiver.
        clock.advance(Default::default(), 32, 0);
        assert_eq!(bincode::serialize(&clock.apu).unwrap(), receiver);
        // Continue the actual retained driver, comparing its instruction
        // boundary state and timer phase to uninterrupted execution.
        let mut expected = origin.apu.clone();
        let samples = (origin.apu.cycles / 32) + 200;
        let end = u64::from(samples + 32) * 32 + super::super::SMP_CPU_LOOKAHEAD_CYCLES;
        while u64::from(expected.cycles) < end {
            expected.run_cycle_sequenced_instruction_without_dsp();
        }
        clock.advance(Default::default(), samples, 0);
        assert_eq!(clock.apu.save_c_saveload_prefix(), expected.save_c_saveload_prefix());
        assert_eq!(bincode::serialize(&clock.apu.spc).unwrap(),
            bincode::serialize(&expected.spc).unwrap());
    }

    #[test]
    fn native_startup_memory_clock_matches_pinned_source() {
        use crate::cycle_models::startup_memory::StartupMemoryWork;
        use sha2::{Digest, Sha256};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let Ok(rom) = std::fs::read(root.join("saves/zelda3.sfc")) else {
            return;
        };
        let mut clock = native_bootstrap_origin(&rom[0xc8000..0xd5337])
            .unwrap()
            .into_cpu()
            .timeline;
        let mut ram = vec![0x55; 0x20000];
        let mut sram = vec![0xff; 0x2000];
        ram[0x1fe..0x200].copy_from_slice(&0x802eu16.to_le_bytes());
        let mut writes = vec![(0x1feu32, 0x2eu8), (0x1ff, 0x80)];
        let mut rows = vec![(clock.clock_master_cycles(), 0x802cu32)];
        clock
            .advance_synchronous_after_semantics_with(46, |_, _| Ok::<u32, String>(0))
            .unwrap();
        let mut work = StartupMemoryWork::default();
        loop {
            rows.push((clock.clock_master_cycles(), work.next_pc()));
            let before = ram[..0x2000].to_vec();
            let Some(cycles) = work.step(&mut ram, &mut sram) else {
                break;
            };
            for (address, (&old, &new)) in before.iter().zip(&ram).enumerate() {
                if old != new {
                    writes.push((address as u32, new));
                }
            }
            clock
                .advance_synchronous_after_semantics_with(cycles, |_, _| Ok::<u32, String>(0))
                .unwrap();
        }
        if let Some(path) = std::env::var_os("ZELDA3_STARTUP_MEMORY_CLOCK_LOG") {
            let mut out = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
            for (at, pc) in &rows {
                writeln!(out, "{at} {pc}").unwrap();
            }
        }
        let mut digest = Sha256::new();
        for (at, pc) in &rows {
            digest.update(at.to_le_bytes());
            digest.update(pc.to_le_bytes());
        }
        // Independent pinned Snes9x trace, including the caller JSR and its
        // return: LE u64 physical clock + LE u32 instruction address. Track
        // physical V rollover; the source's frame label updates after V0.
        assert_eq!(rows.len(), 5651);
        assert_eq!(
            format!("{:x}", digest.finalize()),
            "70c54c0670c998e33c52dea0de5c84d0e0d9e2c2b63c6b3791d84ac14bcf8630"
        );
        assert_eq!(clock.clock_master_cycles(), 28_617_174);
        // This input changes every stored low-WRAM byte, so the diff above
        // records all stores, including the saved stack word's restoration.
        let mut digest = Sha256::new();
        for (address, value) in &writes {
            digest.update(address.to_le_bytes());
            digest.update([*value]);
        }
        assert_eq!(writes.len(), 8182);
        assert_eq!(
            format!("{:x}", digest.finalize()),
            "7b565272f045a14c3a1440742178538d5ce4b0d8aa9e8764cc3ec88dfce7c5df"
        );
    }

    #[test]
    fn cold_native_upload_clock_and_ports() {
        use sha2::{Digest, Sha256};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let Ok(rom) = std::fs::read(root.join("saves/zelda3.sfc")) else {
            return;
        };
        let stream = &rom[0xc8000..0xd5337];
        let mut actual = Vec::new();
        let upload = ColdUpload::run(stream, |access| actual.push(access)).unwrap();
        let origin = upload.into_origin().unwrap();
        if let Some(path) = std::env::var_os("ZELDA3_BOOTSTRAP_PORT_LOG") {
            let mut file = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
            for access in &actual {
                writeln!(
                    file,
                    "{} {} {} {}",
                    access.master_clock,
                    access.port,
                    access.value,
                    u8::from(access.read)
                )
                .unwrap();
            }
        }
        assert_eq!(
            actual[0],
            PortAccess {
                master_clock: 326,
                port: 0,
                value: 0,
                read: false
            }
        );
        assert_eq!(actual[4].master_clock, 998);
        let mut digest = Sha256::new();
        for access in &actual {
            digest.update(access.master_clock.to_le_bytes());
            digest.update([access.port, access.value, u8::from(access.read)]);
        }
        // Independent frozen Snes9x1.63 bootstrap fixture, normalized as
        // little-endian u64 physical master clock + port/value/read bytes.
        // Its frame labels are host calls; physical fields advance on V wrap.
        // The longest access gap is less than one scanline, so no wrap is lost.
        assert_eq!(actual.len(), 356_174);
        assert_eq!(
            format!("{:x}", digest.finalize()),
            "12fa1ec25c55ebca38c995695faa71ebd1404324d54689a8e7422cc998c09de5"
        );
        assert_eq!(origin.apu.cycles, 1_355_567);
        assert_eq!(origin.cpu.timeline.clock_master_cycles(), 28_396_112);
        assert_eq!(origin.cpu.upload_pointer, [0x00, 0x80, 0x1a]);
        // Accept an omitted final destination word as well. Both encodings
        // must derive the same clock, rather than selecting a measured seed.
        let compact = native_bootstrap_origin(&stream[..stream.len() - 2]).unwrap();
        assert_eq!(compact.cpu, origin.cpu);
        assert_eq!(compact.apu_clock, origin.apu_clock);
        assert_eq!(bincode::serialize(&compact.apu).unwrap(), bincode::serialize(&origin.apu).unwrap());
        eprintln!(
            "native cold upload: {} exact frozen accesses; cpu={:?}; apu_cycle={} ports={:02x?}",
            actual.len(), origin.cpu, origin.apu.cycles, origin.apu.out_ports
        );
    }
}
