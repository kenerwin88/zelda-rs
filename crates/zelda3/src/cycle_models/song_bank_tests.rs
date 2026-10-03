use super::super::intro_clear::Access;
use super::*;
use snes::apu::{ApuHostPortProbe, ApuHostPortTiming, ApuState};
use snes::{Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};
use std::collections::{BTreeMap, BTreeSet};

fn rom() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"))
        .unwrap()
}
fn data() -> BTreeMap<u32, u8> {
    let mut state = crate::ZeldaState::new();
    state
        .set_assets(
            &std::fs::read(
                std::env::var_os("ZELDA3_ASSETS").expect("song tests require native assets"),
            )
            .unwrap(),
        )
        .unwrap();
    state.native_song_upload_data().unwrap()
}
fn normalize(a: u32) -> u32 {
    if a & 0x400000 == 0 && a & 65535 < 0x6000 {
        a & 65535
    } else {
        a
    }
}
struct Bus<'a> {
    ram: &'a mut [u8],
    data: &'a BTreeMap<u32, u8>,
    apu: &'a mut ApuHostPortTiming,
    accesses: &'a mut Vec<(u64, Access, u16)>,
    vblanks: &'a mut Vec<u64>,
    lines: &'a mut usize,
    nmi: &'a mut Nmi,
    in_vblank: bool,
}
#[derive(Default)]
struct Nmi {
    enabled: bool,
    deferred_enable: bool,
    deadline: Option<u64>,
}
impl WorkBus for Bus<'_> {
    fn observe_clock(&mut self, clock: &CpuMasterTimeline) {
        self.in_vblank = clock.synchronous_beam_position().unwrap().scanline >= 225;
    }
    fn access(&mut self, at: u64, access: Access) -> u16 {
        let (access, value) = match access {
            Access::Read(a, width) => {
                let a = normalize(a);
                let value = if (0x2140..=0x2143).contains(&a) {
                    assert_eq!(width, 1);
                    u16::from(self.apu.read_cpu_port_at(at, (a - 0x2140) as u8).unwrap())
                } else {
                    let byte = |a| {
                        if a == 0xffea {
                            0xc9
                        } else if a == 0xffeb {
                            0x80
                        } else if a < 0x2000 {
                            self.ram[a as usize]
                        } else {
                            *self
                                .data
                                .get(&a)
                                .unwrap_or_else(|| panic!("unbound native song byte {a:06x}"))
                        }
                    };
                    if width == 1 {
                        u16::from(byte(a))
                    } else {
                        u16::from_le_bytes([byte(a), byte(a + 1)])
                    }
                };
                (Access::Read(a, width), value)
            }
            Access::Write(a, value, width) => {
                let a = normalize(a);
                if (0x2140..=0x2143).contains(&a) {
                    assert_eq!(width, 1);
                    self.apu
                        .write_cpu_port_at(at, (a - 0x2140) as u8, value as u8)
                        .unwrap();
                } else if matches!(a, 0x4200 | 0x420c) {
                    assert_eq!(width, 1);
                    assert!(value == 0 || (a == 0x4200 && value == 0x81));
                    if a == 0x4200 {
                        let enabled = value & 0x80 != 0;
                        self.nmi.deferred_enable |= enabled && !self.nmi.enabled && self.in_vblank;
                        self.nmi.enabled = enabled;
                    }
                } else {
                    assert!(a < 0x2000);
                    self.ram[a as usize..a as usize + width as usize]
                        .copy_from_slice(&value.to_le_bytes()[..width as usize]);
                }
                (Access::Write(a, value, width), 0)
            }
            Access::Vblank { at, .. } => {
                self.vblanks.push(at);
                if self.nmi.enabled {
                    self.nmi.deadline =
                        Some(at + u64::from(snes::SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES));
                }
                return 0;
            }
        };
        self.accesses.push((at, access, value));
        value
    }
    fn end_scanline(&mut self, at: u64) {
        self.apu.end_scanline_at(at).unwrap();
        *self.lines += 1;
    }
}
struct Fixture {
    source: Snes9xColdCpuExecutor,
    cpu: NativeNmiContext,
    clock: CpuMasterTimeline,
    ram: Vec<u8>,
    apu: ApuHostPortTiming,
    visited: BTreeSet<u32>,
    instructions: usize,
    ports: usize,
    lines: usize,
    bank_wraps: usize,
    initialization_instructions: usize,
    nmi: Nmi,
    interrupt_return: Option<NativeNmiContext>,
    interrupts: usize,
}
impl Fixture {
    fn new(rom: &[u8]) -> Self {
        // Test-only CPU work lets the independently uploaded driver finish
        // initialization. Song routines and upload data remain original.
        let mut rom = rom.to_vec();
        rom[..9].copy_from_slice(&[0x78, 0x9c, 0, 0x42, 0xad, 0x40, 0x21, 0x80, 0xfb]);
        let source = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
        let mut apu = ApuState::new();
        apu.reset_snes9x_coroutine();
        // Independent hardware reset for each APU. No source machine state or
        // upload acknowledgement is copied into the native receiver.
        assert_eq!(
            bincode::serialize(&apu).unwrap(),
            bincode::serialize(&source.machine().snes().apu).unwrap()
        );
        let apu = ApuHostPortTiming::new(
            ApuHostPortProbe::from_snes9x_coroutine(apu).unwrap(),
            Default::default(),
        )
        .unwrap();
        Self {
            clock: source.machine().timeline().clone(),
            ram: source.machine().snes().ram.clone(),
            source,
            apu,
            cpu: NativeNmiContext {
                a: 0,
                x: 0,
                y: 0,
                d: 0,
                db: 0,
                bank: 0,
                pc: 0,
                sp: 0x1fb,
                p: 0x34,
            },
            visited: BTreeSet::new(),
            instructions: 0,
            ports: 0,
            lines: 0,
            bank_wraps: 0,
            initialization_instructions: 0,
            nmi: Nmi::default(),
            interrupt_return: None,
            interrupts: 0,
        }
    }
    fn enter(&mut self, entry: u32, variant: usize, init: impl FnOnce(&mut [u8])) -> SongBank {
        self.interrupt_return = None;
        let far = matches!(entry, 0x008913 | 0x008925);
        self.cpu = NativeNmiContext {
            a: 0xab00 | (variant & 255) as u16,
            x: 0x51,
            y: 0x83,
            d: 0,
            db: [0, 2, 9, 0x1b][variant & 3],
            bank: (entry >> 16) as u8,
            pc: entry as u16,
            sp: if far { 0x1fa } else { 0x1fb },
            p: [0x34, 0xf5, 0x74, 0x35][variant & 3],
        };
        let mut c = self.source.machine().snes().cpu.clone();
        c.a = self.cpu.a;
        c.x = self.cpu.x;
        c.y = self.cpu.y;
        c.dp = 0;
        c.db = self.cpu.db;
        c.k = self.cpu.bank;
        c.pc = self.cpu.pc;
        c.sp = self.cpu.sp;
        c.e = false;
        c.unpack_flags(self.cpu.p);
        init(&mut self.ram);
        if far {
            self.ram[0x1fb..0x1fe].copy_from_slice(&[0, 0xc2, 2]);
        } else {
            self.ram[0x1fc..0x1fe].copy_from_slice(&[0, 0xc2]);
        }
        let sram = self.source.machine().snes().cart.ram.clone();
        self.source
            .rebase_private_timing_state_at(self.source.machine().timestamp(), &c, &self.ram, &sram)
            .unwrap();
        SongBank::from_entry(entry)
    }
    fn step_cpu(
        &mut self,
        data: &BTreeMap<u32, u8>,
        execute: impl FnOnce(&mut NativeNmiContext, &mut CpuMasterTimeline, Bus<'_>),
    ) {
        let origin = (u32::from(self.cpu.bank) << 16) | u32::from(self.cpu.pc);
        if origin >= 0x008888 {
            self.visited.insert(origin);
        }
        self.bank_wraps += usize::from(origin == 0x0088b0);
        let source = self.source.step().unwrap();
        let mut first = true;
        let expected: Vec<_> = source
            .accesses
            .into_iter()
            .filter_map(|a| match a.kind {
                Kind::Read { value, width } => {
                    let operand = first && a.address == origin + 1;
                    first = false;
                    (!operand).then_some((
                        a.timestamp.master_cycles(),
                        Access::Read(normalize(a.address), width),
                        value,
                    ))
                }
                Kind::Write { value, width } => Some((
                    a.timestamp.master_cycles(),
                    Access::Write(normalize(a.address), value, width),
                    0,
                )),
                _ => None,
            })
            .collect();
        let mut actual = Vec::new();
        let mut vblanks = Vec::new();
        let old_lines = self.lines;
        execute(
            &mut self.cpu,
            &mut self.clock,
            Bus {
                ram: &mut self.ram,
                data,
                apu: &mut self.apu,
                accesses: &mut actual,
                vblanks: &mut vblanks,
                lines: &mut self.lines,
                nmi: &mut self.nmi,
                in_vblank: false,
            },
        );
        // Select the interrupt from the native port edge and beam, independently
        // of the source receipt. The fixture stops at entry; the game handler
        // belongs to the eventual composed Startup test.
        let accepted = if self
            .nmi
            .deadline
            .is_some_and(|at| self.clock.clock_master_cycles() >= at)
        {
            self.nmi.deadline = None;
            let at = self.clock.clock_master_cycles();
            let interrupted = (u32::from(self.cpu.bank) << 16) | u32::from(self.cpu.pc);
            self.interrupt_return = Some(self.cpu);
            self.interrupts += 1;
            let mut w = Work {
                cpu: &mut self.cpu,
                clock: &mut self.clock,
                bus: Bus {
                    ram: &mut self.ram,
                    data,
                    apu: &mut self.apu,
                    accesses: &mut actual,
                    vblanks: &mut vblanks,
                    lines: &mut self.lines,
                    nmi: &mut self.nmi,
                    in_vblank: false,
                },
                cycles: 0,
            };
            w.drain(14);
            w.push(u16::from(w.cpu.bank), 1);
            w.push(w.cpu.pc, 2);
            w.push(u16::from(w.cpu.p), 1);
            w.cpu.p = (w.cpu.p | 4) & !8;
            w.cpu.pc = w.read(0xffea, 2);
            w.cpu.bank = 0;
            Some((at, interrupted))
        } else {
            None
        };
        if self.nmi.deferred_enable {
            self.nmi.deferred_enable = false;
            self.nmi.deadline = Some(self.clock.clock_master_cycles() + 6);
        }
        assert_eq!(
            accepted,
            source.accepted_interrupt.map(|interrupt| match interrupt {
                snes::SourceCpuAcceptedInterrupt::Nmi {
                    started_at,
                    interrupted_pc,
                } => (started_at.master_cycles(), interrupted_pc),
                _ => panic!("unexpected fixture IRQ"),
            }),
            "interrupt pc{origin:06x}"
        );
        assert_eq!(actual, expected, "pc{origin:06x}");
        self.ports += actual
            .iter()
            .filter(|(_, a, _)| {
                matches!(
                    a,
                    Access::Read(0x2140..=0x2143, _) | Access::Write(0x2140..=0x2143, _, _)
                )
            })
            .count();
        let c = &self.source.machine().snes().cpu;
        assert_eq!(
            self.cpu,
            NativeNmiContext {
                a: c.a,
                x: c.x,
                y: c.y,
                d: c.dp,
                db: c.db,
                bank: c.k,
                pc: c.pc,
                sp: c.sp,
                p: c.pack_flags()
            },
            "pc{origin:06x}"
        );
        assert_eq!(
            self.clock,
            *self.source.machine().timeline(),
            "pc{origin:06x}"
        );
        assert_eq!(self.ram, self.source.machine().snes().ram, "pc{origin:06x}");
        let native = self.apu.machine();
        let source_apu = &self.source.machine().snes().apu;
        assert_eq!(
            (native.cycles, native.in_ports, native.out_ports),
            (source_apu.cycles, source_apu.in_ports, source_apu.out_ports),
            "APU pc{origin:06x}"
        );
        if self.lines != old_lines || self.cpu.pc == 0xc201 || accepted.is_some() {
            // The probe reports a whole instruction's cost in cycles_used;
            // the source micro-stepper reports the last pseudo-step's cost.
            // This diagnostic return value is not SPC hardware state. Keep
            // all registers, clocks, RAM, timers and the SMP/DSP continuation
            // in the comparison; normalize only that statistic on clones.
            let mut native = native.clone();
            let mut source_apu = source_apu.clone();
            native.spc.cycles_used = 0;
            source_apu.spc.cycles_used = 0;
            assert_eq!(
                bincode::serialize(&native).unwrap(),
                bincode::serialize(&source_apu).unwrap(),
                "APU hardware pc{origin:06x}"
            );
            assert_eq!(
                native.capture_snes9x_apu_coroutine_checkpoint(),
                source_apu.capture_snes9x_apu_coroutine_checkpoint(),
                "APU continuation pc{origin:06x}"
            );
            self.apu = self.apu.clone();
        }
        for at in vblanks {
            let host = self.source.run_until_main_loop_return().unwrap();
            assert_eq!(host.instruction_count, 0);
            assert_eq!(host.vblank_event_at.master_cycles(), at);
        }
        self.instructions += 1;
    }
    fn run(&mut self, owner: &mut SongBank, data: &BTreeMap<u32, u8>) {
        let original_db = self.cpu.db;
        let start = self.instructions;
        while !owner.complete() {
            self.step_cpu(data, |cpu, clock, bus| {
                owner.step(cpu, clock, bus).unwrap();
            });
            *owner = owner.clone();
            assert!(
                self.interrupt_return.is_none() || owner.complete(),
                "interrupt before song return"
            );
            assert!(
                self.instructions - start < 2_000_000,
                "upload did not return entry{:06x} pc{:04x}, SPC{:04x}",
                owner.entry,
                self.cpu.pc,
                self.apu.machine().spc.pc
            );
        }
        let returned = self.interrupt_return.unwrap_or(self.cpu);
        assert_eq!(
            (returned.pc, returned.sp, returned.db),
            (0xc201, 0x1fd, original_db)
        );
    }
    fn initialize_driver(&mut self, data: &BTreeMap<u32, u8>) {
        self.cpu.bank = 0;
        self.cpu.pc = 0x8000;
        let mut c = self.source.machine().snes().cpu.clone();
        c.k = 0;
        c.pc = 0x8000;
        let sram = self.source.machine().snes().cart.ram.clone();
        self.source
            .rebase_private_timing_state_at(self.source.machine().timestamp(), &c, &self.ram, &sram)
            .unwrap();
        let start = self.instructions;
        loop {
            self.step_cpu(data, |cpu, clock, bus| {
                let mut w = Work {
                    cpu,
                    clock,
                    bus,
                    cycles: 0,
                };
                w.fetch();
                w.cpu.pc = match w.cpu.pc {
                    0x8000 => {
                        w.drain(6);
                        w.cpu.p |= 4;
                        0x8001
                    }
                    0x8001 => {
                        w.drain(16);
                        w.write_io(0x4200, 0, 1);
                        0x8004
                    }
                    0x8004 => {
                        w.drain(16);
                        let value = w.read_io(0x2140, 1);
                        w.load(value, 1);
                        0x8007
                    }
                    0x8007 => {
                        w.branch(true);
                        0x8004
                    }
                    _ => unreachable!(),
                };
            });
            // Stop on the observed native driver lifecycle boundary, not a
            // guessed duration. Both APUs were compared throughout this work.
            if self.cpu.pc == 0x8007 && self.apu.machine().spc.pc == 0x0878 {
                break;
            }
            assert!(
                self.instructions - start < 100_000,
                "driver did not reach its poll loop: SPC{:04x}, ports{:02x?}",
                self.apu.machine().spc.pc,
                self.apu.machine().out_ports
            );
        }
        self.initialization_instructions += self.instructions - start;
    }
}

#[test]
fn song_bank_native_upload_streams_match_source_bytes() {
    let data = data();
    let rom = rom();
    for (&a, &v) in &data {
        assert_eq!(
            v,
            rom[(a as usize >> 16) * 0x8000 + (a as usize & 0x7fff)],
            "{a:06x}"
        );
    }
    eprintln!(
        "{} native bootstrap and runtime song-stream bytes match source",
        data.len()
    );
}

#[test]
fn song_bank_selection_skip_paths_match_cold_source_all_commands() {
    let rom = rom();
    let data = BTreeMap::new();
    let mut f = Fixture::new(&rom);
    let mut cases = 0;
    for command in 0..=255u8 {
        let old_bank = if matches!(command, 3 | 7 | 14) { 0 } else { 1 };
        let mut owner = f.enter(0x029bd7, command as usize, |r| {
            r[0x132] = command;
            r[0x136] = old_bank;
        });
        f.run(&mut owner, &data);
        cases += 1;
    }
    for command in [0xff, 0xf2] {
        let mut owner = f.enter(0x029bd7, command as usize, |r| {
            r[0x132] = command;
            r[0x136] = 0;
        });
        f.run(&mut owner, &data);
        cases += 1;
    }
    assert_eq!(f.ports, 0);
    eprintln!(
        "{cases} song selection skip fixtures matched {} source instructions across {} positions",
        f.instructions,
        f.visited.len()
    );
}

#[test]
fn song_bank_upload_matches_cold_source_live_ipl_driver_and_nested_returns() {
    let rom = rom();
    let data = data();
    let mut f = Fixture::new(&rom);
    let mut owner = f.enter(0x008888, 0, |r| {
        r[..3].copy_from_slice(&[0, 0x80, 0x19]);
    });
    f.run(&mut owner, &data);
    assert_eq!(f.bank_wraps, 1);
    let bootstrap_instructions = f.instructions;
    eprintln!(
        "song bootstrap completed: {bootstrap_instructions} instructions, master{}, SPC{:04x}",
        f.clock.clock_master_cycles(),
        f.apu.machine().spc.pc
    );
    // Keep both independently executed APUs and their clocks across actual
    // runtime bank changes. Only routine-entry CPU/WRAM fixture inputs change.
    for (variant, command, bank) in [
        (0, 0x10, 0),
        (1, 3, 1),
        (2, 0x11, 0),
        (3, 7, 1),
        (0, 0x12, 0),
        (1, 14, 1),
    ] {
        f.initialize_driver(&data);
        let mut owner = f.enter(0x029bd7, variant, |r| {
            r[0x132] = command;
            r[0x136] = bank;
        });
        f.run(&mut owner, &data);
    }
    assert_eq!(f.visited, OWNED_POSITIONS.iter().copied().collect());
    assert!(
        f.interrupts > 0,
        "fixture must exercise the live NMI enable edge"
    );
    eprintln!("song fixture lifecycle matched {} setup instructions and {} independently selected NMI entries", f.initialization_instructions, f.interrupts);
    eprintln!("native cold song upload matched {} bootstrap and {} runtime source instructions, {} port transactions, {} scanlines and {} positions",bootstrap_instructions,f.instructions-bootstrap_instructions-f.initialization_instructions,f.ports,f.lines,f.visited.len());
}
