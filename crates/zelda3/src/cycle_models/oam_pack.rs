//! Native extended-OAM packing before NMI_PrepareSprites' source-word suffix.
//! An interrupted shift/OR chain retains its accumulator, including after a
//! completed output byte and before the group's loop decision.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PackingGroup {
    start: u8,
    instruction: u8,
    accumulator: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub store: Option<(usize, u8)>,
}

impl PackingGroup {
    pub(crate) fn new(start: u8) -> Self {
        assert!(start <= 28 && start & 3 == 0);
        Self {
            start,
            instruction: 0,
            accumulator: 0,
        }
    }

    pub(crate) fn is_complete(self) -> bool {
        self.instruction == 53
    }

    pub(crate) fn step(&mut self, mut read: impl FnMut(usize) -> u8) -> Option<Instruction> {
        let mut store = None;
        let cost = match self.instruction {
            0 => {
                self.accumulator = self.start;
                14
            }
            1 | 2 => {
                self.accumulator <<= 1;
                14
            }
            3 => 14, // TAX: subsequent reads use this group's retained start
            4..=47 => {
                let column = (self.instruction - 4) / 11;
                let op = (self.instruction - 4) % 11;
                let source = usize::from(self.start + column) * 4;
                match op {
                    0 => {
                        self.accumulator = read(source + 3);
                        32
                    }
                    3 | 6 | 9 => {
                        self.accumulator |= read(source + usize::from(3 - op / 3));
                        32
                    }
                    10 => {
                        store = Some((usize::from(self.start + column), self.accumulator));
                        38
                    }
                    _ => {
                        self.accumulator <<= 1;
                        14
                    }
                }
            }
            48..=51 => 14, // four DEYs
            52 => {
                if self.start == 0 {
                    16
                } else {
                    22
                }
            }
            53 => return None,
            _ => unreachable!(),
        };
        self.instruction += 1;
        Some(Instruction {
            master_cycles: cost,
            store,
        })
    }

    pub(crate) fn next_pc(self) -> u32 {
        match self.instruction {
            0..=3 => 0x00_85fe + u32::from(self.instruction),
            4..=47 => {
                let column = (self.instruction - 4) / 11;
                let op = (self.instruction - 4) % 11;
                0x00_8602
                    + u32::from(column) * 0x15
                    + [0, 3, 4, 5, 8, 9, 10, 13, 14, 15, 18][usize::from(op)]
            }
            48..=52 => 0x00_8656 + u32::from(self.instruction - 48),
            53 => {
                if self.start == 0 {
                    0x00_865c
                } else {
                    0x00_85fe
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ExtendedOamPacking {
    entered: bool,
    group: PackingGroup,
    complete: bool,
}

impl Default for ExtendedOamPacking {
    fn default() -> Self {
        Self {
            entered: false,
            group: PackingGroup::new(28),
            complete: false,
        }
    }
}

impl ExtendedOamPacking {
    pub(crate) fn is_complete(self) -> bool {
        self.complete
    }

    pub(crate) fn step(&mut self, read: impl FnMut(usize) -> u8) -> Option<Instruction> {
        if self.complete {
            return None;
        }
        if !self.entered {
            self.entered = true;
            return Some(Instruction {
                master_cycles: 16,
                store: None,
            });
        }
        let instruction = self.group.step(read).expect("active packing group");
        if self.group.is_complete() {
            if self.group.start == 0 {
                self.complete = true;
            } else {
                self.group = PackingGroup::new(self.group.start - 4);
            }
        }
        Some(instruction)
    }

    pub(crate) fn next_pc(self) -> u32 {
        if !self.entered {
            0x00_85fc
        } else if self.complete {
            0x00_865c
        } else {
            self.group.next_pc()
        }
    }
}

/// Live JSR and extended-OAM packing. The CPU context owns flags, stack and
/// registers; the existing packing continuation owns its read/shift progress.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CallerPacking {
    called: bool,
    packing: ExtendedOamPacking,
    body_master_cycles: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BusAccess {
    Read(u16),
    Write(u16, u16, u8),
    Vblank { field: u64, at: u64 },
}

impl CallerPacking {
    pub(crate) fn complete(&self) -> bool {
        self.packing.is_complete()
    }
    pub(crate) fn body_master_cycles(&self) -> u64 {
        self.body_master_cycles
    }
    pub(crate) fn pending_call(&self) -> bool {
        !self.called
    }

    pub(crate) fn step(
        &mut self,
        cpu: &mut super::nmi_stack::NativeNmiContext,
        clock: &mut snes::CpuMasterTimeline,
        mut bus: impl FnMut(u64, BusAccess) -> u16,
    ) -> Option<u64> {
        self.step_with_bus(cpu, clock, |at, access| {
            use super::intro_clear::Access;
            bus(at, match access {
                Access::Read(address, 1) => BusAccess::Read(address as u16),
                Access::Write(address, value, width) => BusAccess::Write(address as u16, value, width),
                Access::Vblank { field, at } => BusAccess::Vblank { field, at },
                _ => unreachable!("OAM packing reads bytes"),
            })
        })
    }

    pub(crate) fn step_with_bus(
        &mut self,
        cpu: &mut super::nmi_stack::NativeNmiContext,
        clock: &mut snes::CpuMasterTimeline,
        bus: impl super::native_work::WorkBus,
    ) -> Option<u64> {
        use super::nmi_stack::{CARRY, NEGATIVE};
        if self.complete() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.d, cpu.db, cpu.p & 0x30), (0, 0, 0, 0x30));
        assert_eq!(
            u32::from(cpu.pc),
            if self.called {
                self.packing.next_pc()
            } else {
                0x805a
            }
        );
        let mut w = ClockedWork {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        if !self.called {
            // JSR abs pushes the actual caller's return word after its operands.
            w.drain(16);
            w.drain(6);
            let returned = w.cpu.pc.wrapping_add(2);
            w.access(
                w.clock.clock_master_cycles(),
                BusAccess::Write(w.cpu.sp - 1, returned, 2),
            );
            w.drain(16);
            w.cpu.sp -= 2;
            w.cpu.pc = 0x85fc;
            self.called = true;
            return Some(w.cycles);
        }
        if !self.packing.entered {
            w.drain(8);
            self.packing.step(|_| unreachable!()).unwrap();
            w.cpu.y = 28;
            w.flags(28);
        } else {
            let instruction = self.packing.group.instruction;
            let old_a = w.cpu.a as u8;
            let start = self.packing.group.start;
            // Preserve the relationship between retained packing progress and
            // the live registers restored by RTI. Never reconstruct a load.
            if instruction == 0 {
                assert_eq!(w.cpu.y, u16::from(start));
            } else {
                assert_eq!(old_a, self.packing.group.accumulator);
            }
            let result = self
                .packing
                .step(|index| {
                    assert_eq!(u16::from(start) * 4, w.cpu.x);
                    w.drain(16);
                    let value = w.access(
                        w.clock.clock_master_cycles(),
                        BusAccess::Read(0xa20 + index as u16),
                    );
                    w.drain(8);
                    value as u8
                })
                .unwrap();
            if let Some((index, value)) = result.store {
                w.drain(16);
                w.drain(6);
                w.access(
                    w.clock.clock_master_cycles(),
                    BusAccess::Write(0xa00 + index as u16, u16::from(value), 1),
                );
                w.drain(8);
            } else if instruction == 52 {
                assert_eq!(w.cpu.y, u16::from(start.wrapping_sub(4)));
                assert_eq!(w.cpu.p & NEGATIVE != 0, start == 0);
                // BPL fetches its operand before the taken-branch cycle.
                // A scanline event between them must see the first endpoint.
                w.drain(8);
                if result.master_cycles == 22 { w.drain(6); }
            } else {
                if w.cycles == 8 {
                    w.drain(result.master_cycles - 8);
                }
                match instruction {
                    3 => {
                        w.cpu.x = u16::from(old_a);
                        w.flags(old_a);
                    }
                    48..=51 => {
                        w.cpu.y = (w.cpu.y as u8).wrapping_sub(1).into();
                        w.flags(w.cpu.y as u8);
                    }
                    _ => {
                        // After the final loop decision ExtendedOamPacking may
                        // select another group; no accumulator changes there.
                        let value = self.packing.group.accumulator;
                        if matches!(instruction, 1 | 2)
                            || (4..=47).contains(&instruction)
                                && matches!((instruction - 4) % 11, 1 | 2 | 4 | 5 | 7 | 8)
                        {
                            w.cpu.p = (w.cpu.p & !CARRY) | (old_a >> 7);
                        }
                        w.cpu.a = (w.cpu.a & 0xff00) | u16::from(value);
                        w.flags(value);
                    }
                }
            }
            assert_eq!(w.cycles, u64::from(result.master_cycles));
        }
        w.cpu.pc = self.packing.next_pc() as u16;
        // Width bits, overflow and interrupt mask survive the packer.
        self.body_master_cycles += w.cycles;
        Some(w.cycles)
    }
}

struct ClockedWork<'a, B> {
    cpu: &'a mut super::nmi_stack::NativeNmiContext,
    clock: &'a mut snes::CpuMasterTimeline,
    bus: B,
    cycles: u64,
}
impl<B: super::native_work::WorkBus> ClockedWork<'_, B> {
    fn access(&mut self, at: u64, access: BusAccess) -> u16 {
        use super::intro_clear::Access;
        self.bus.access(at, match access {
            BusAccess::Read(address) => Access::Read(u32::from(address), 1),
            BusAccess::Write(address, value, width) => Access::Write(u32::from(address), value, width),
            BusAccess::Vblank { field, at } => Access::Vblank { field, at },
        })
    }
    fn fetch(&mut self) {
        self.cycles += 8;
        self.clock.advance_synchronous_pcbase_opcode_fetch(8);
    }
    fn drain(&mut self, n: u32) {
        self.cycles += u64::from(n);
        let bus = &mut self.bus;
        self.clock
            .advance_synchronous_after_semantics_with_clock(n, |event, clock| {
                Ok::<u32, String>(bus.retire_event(event, clock))
            })
            .unwrap();
    }
    fn flags(&mut self, value: u8) {
        use super::nmi_stack::{NEGATIVE, ZERO};
        self.cpu.p = (self.cpu.p & !(NEGATIVE | ZERO))
            | (value & NEGATIVE)
            | if value == 0 { ZERO } else { 0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};

    #[test]
    fn live_caller_packing_matches_rom_registers_bus_and_clock() {
        use crate::cycle_models::nmi_stack::NativeNmiContext;
        use snes::{
            CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
            RomCpuTimingProbe, Snes, SourceCpuBusAccessKind as Kind, SourcePpuReadState,
        };
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(rom) = std::fs::read(path) else { return };
        for seed in [0u8, 0x55, 0x80, 0xff] {
            for (v, h) in [(204, 230), (224, 1338), (261, 1350)] {
                for odd in [false, true] {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i as u8).wrapping_mul(37) ^ seed;
                    }
                    let mut cpu = NativeNmiContext {
                        a: 0x1700 | u16::from(seed),
                        x: 0xfe,
                        y: 0xff,
                        d: 0,
                        db: 0,
                        bank: 0,
                        pc: 0x805a,
                        sp: 0x1ff,
                        p: 0x30 | (seed & 0xcf),
                    };
                    machine.cpu.a = cpu.a;
                    machine.cpu.x = cpu.x;
                    machine.cpu.y = cpu.y;
                    machine.cpu.pc = cpu.pc;
                    machine.cpu.sp = cpu.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(cpu.p);
                    let mut ram = machine.ram.clone();
                    let mut clock = CpuMasterTimeline::at_raster(
                        0,
                        CpuRasterPosition::new(v, h),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::non_interlace(odd),
                    );
                    let mut source = RomCpuTimingProbe::new(
                        machine,
                        clock.clone(),
                        SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    clock.begin_synchronous_timeline().unwrap();
                    let mut work = CallerPacking::default();
                    let mut count = 0;
                    while !work.complete() {
                        let pc = cpu.pc;
                        let expected = source
                            .step()
                            .unwrap()
                            .accesses
                            .into_iter()
                            .filter_map(|a| {
                                if a.address >= 0x2000 {
                                    return None;
                                }
                                let (op, value) = match a.kind {
                                    Kind::Read { value, width } => {
                                        assert_eq!(width, 1);
                                        (BusAccess::Read(a.address as u16), value)
                                    }
                                    Kind::Write { value, width } => {
                                        (BusAccess::Write(a.address as u16, value, width), 0)
                                    }
                                    _ => return None,
                                };
                                Some((a.timestamp.master_cycles(), op, value))
                            })
                            .collect::<Vec<_>>();
                        let mut actual = Vec::new();
                        work.step(&mut cpu, &mut clock, |at, op| {
                            let value = match op {
                                BusAccess::Read(a) => u16::from(ram[usize::from(a)]),
                                BusAccess::Write(a, value, width) => {
                                    ram[usize::from(a)] = value as u8;
                                    if width == 2 {
                                        ram[usize::from(a) + 1] = (value >> 8) as u8;
                                    }
                                    0
                                }
                                BusAccess::Vblank { .. } => return 0,
                            };
                            actual.push((at, op, value));
                            value
                        })
                        .unwrap();
                        assert_eq!(
                            actual, expected,
                            "pc={pc:04x} seed={seed} v={v} h={h} odd={odd}"
                        );
                        let c = &source.snes().cpu;
                        assert_eq!(
                            (cpu.a, cpu.x, cpu.y, cpu.pc, cpu.sp, cpu.p),
                            (c.a, c.x, c.y, c.pc, c.sp, c.pack_flags()),
                            "pc={pc:04x}"
                        );
                        assert_eq!(clock, *source.timeline(), "pc={pc:04x}");
                        assert_eq!(ram, source.snes().ram, "pc={pc:04x}");
                        work = work.clone();
                        count += 1;
                    }
                    assert_eq!(
                        (count, work.body_master_cycles(), cpu.pc),
                        (426, 9082, 0x865c)
                    );
                }
            }
        }
    }

    #[test]
    fn extended_oam_packing_retains_every_source_instruction_and_publication() {
        let path = std::env::var_os("ZELDA3_ROM")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")
            });
        let Ok(mut rom) = std::fs::read(path) else {
            return;
        };
        if rom.len() % 0x400 == 0x200 {
            rom.drain(..0x200);
        }
        for seed in 0..=255u8 {
            let mut ram = vec![0; 0x20000];
            ram[0xa00..0xa20].fill(0x5a);
            for (i, value) in ram[0xa20..0xaa0].iter_mut().enumerate() {
                *value = seed.wrapping_add((i as u8).wrapping_mul(37));
            }
            let checkpoint = RomCpuCheckpoint {
                entry_pc: 0x00_85fc,
                stop_pc: 0x00_865c,
                a: 0xabcd,
                x: 0x37,
                y: 0x42,
                sp: 0x01fc,
                dp: 0,
                db: 0,
                carry: true,
                zero: false,
                overflow: true,
                negative: false,
                interrupt_disable: true,
                decimal: false,
                accumulator_is_8_bit: true,
                index_is_8_bit: true,
                emulation: false,
                waiting: false,
                stack_address: 0x01fd,
                stack_bytes: &[0x35, 0x80],
            };
            let mut source = RomCpuTimingRun::new(
                &rom,
                &ram,
                &[0; 0x2000],
                &snes::PpuState::default(),
                &snes::DmaState::default(),
                [0; 4],
                checkpoint,
            )
            .unwrap();
            source.enable_cpu_write_trace();
            let mut expected = ram.clone();
            let mut work = ExtendedOamPacking::default();
            let mut count = 0;
            let mut total = 0;
            let mut stores = 0;
            loop {
                work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
                let pc = source.pc();
                assert_eq!(pc, work.next_pc());
                let Some(instruction) = work.step(|i| ram[0xa20 + i]) else {
                    break;
                };
                assert_eq!(
                    source.step().master_cycles,
                    instruction.master_cycles,
                    "PC {pc:06x}"
                );
                assert_eq!(source.pc(), work.next_pc(), "PC {pc:06x}");
                for (address, value) in source.take_cpu_wram_writes() {
                    expected[address] = value;
                }
                if let Some((index, value)) = instruction.store {
                    ram[0xa00 + index] = value;
                    stores += 1;
                    if stores == 32 {
                        assert_ne!(work.next_pc(), 0x00_865c);
                    }
                }
                assert_eq!(ram, expected, "seed={seed} PC {pc:06x}");
                count += 1;
                total += instruction.master_cycles;
            }
            assert!(source.is_complete());
            assert_eq!((count, stores, total), (425, 32, 9082));
        }
    }

    #[test]
    fn packing_accumulator_survives_changes_to_already_read_oam() {
        let mut bytes = [0u8; 128];
        bytes[115] = 2;
        let mut work = PackingGroup::new(28);
        let mut first = None;
        while let Some(instruction) = work.step(|i| bytes[i]) {
            if work.next_pc() == 0x00_8605 {
                bytes[115] = 0; // old high bits must remain in the accumulator
                bytes[114] = 1;
                bytes[113] = 2;
                bytes[112] = 3;
            }
            if let Some(store) = instruction.store {
                first.get_or_insert(store);
            }
            work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
        }
        assert_eq!(first, Some((28, 0x9b)));
        assert_eq!(
            work.step(|_| panic!("a completed group cannot read again")),
            None
        );
    }
}
