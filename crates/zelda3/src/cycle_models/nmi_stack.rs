//! Native-mode register saves and caller frames, shared by native NMI owners.
//! The initial handler carries one clock through its bus transactions and RTI.

use snes::CpuMasterTimeline;

/// Native handler transactions, without fetching or interpreting ROM opcodes.
/// Low WRAM and the bank-zero cartridge vector both use the eight-clock bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryBus {
    Fetch,
    Internal(u8),
    RegisterAccess,
    Read { address: u16, value: u16, width: u8 },
    Write { address: u16, value: u16, width: u8 },
}

impl EntryBus {
    fn advance(self, clock: &mut CpuMasterTimeline) {
        let cycles = match self {
            Self::Fetch => {
                clock.advance_synchronous_pcbase_opcode_fetch(8);
                return;
            }
            Self::Internal(cycles) => cycles,
            Self::RegisterAccess => 6,
            Self::Read { width, .. } | Self::Write { width, .. } => 8 * width,
        };
        // This entry owner requires inactive DMA/HDMA. Refresh and raster
        // rollover are timeline events, not extra cycles charged by the NMI.
        clock
            .advance_synchronous_after_semantics_with(u32::from(cycles), |_, _| {
                Ok::<u32, String>(0)
            })
            .expect("native NMI entry clock");
    }
}

pub(crate) const CARRY: u8 = 1;
pub(crate) const ZERO: u8 = 2;
const IRQ_MASK: u8 = 4;
const DECIMAL: u8 = 8;
pub(crate) const INDEX_BYTE: u8 = 0x10;
pub(crate) const ACCUMULATOR_BYTE: u8 = 0x20;
pub(crate) const NEGATIVE: u8 = 0x80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NativeNmiContext {
    pub(crate) a: u16,
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) d: u16,
    pub(crate) db: u8,
    pub(crate) bank: u8,
    pub(crate) pc: u16,
    pub(crate) sp: u16,
    pub(crate) p: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NmiHeaderAccess {
    Read(u16),
    Write(u16, u8),
    WriteWord(u16, u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NmiJoypadAccess {
    ReadByte(u16),
    ReadWord(u16),
    WriteByte(u16, u8),
    WriteWord(u16, u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NmiTailAccess {
    ReadByte(u16),
    ReadWord(u16),
    WriteByte(u16, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NmiHeaderResult {
    pub(crate) ambient_write: Option<u8>,
    pub(crate) effects: [u8; 2],
    pub(crate) ambient_master_cycles: u64,
    pub(crate) calls_updates: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
        RomCpuTimingProbe, Snes, SourcePpuReadState,
    };

    fn compare(native: NativeNmiContext, cpu: &snes::cpu::CpuState) {
        assert_eq!(
            (
                native.a,
                native.x,
                native.y,
                native.d,
                native.db,
                native.bank,
                native.pc,
                native.sp,
                native.p
            ),
            (
                cpu.a,
                cpu.x,
                cpu.y,
                cpu.dp,
                cpu.db,
                cpu.k,
                cpu.pc,
                cpu.sp,
                cpu.pack_flags()
            )
        );
    }

    #[test]
    fn native_nmi_stack_matches_cpu_for_every_status_and_restores_live_frame() {
        let mut rom = vec![0xea; 0x8000];
        rom[0x7fea..0x7fec].copy_from_slice(&[0xc9, 0x80]);
        // Cartridge NMI prologue, followed by an isolated call and restore tail.
        rom[0xc9..0xd9].copy_from_slice(&[
            0x78, 0xc2, 0x30, 0x48, 0xda, 0x5a, 0x0b, 0x8b, 0xa9, 0, 0, 0x5b, 0x4b, 0xab, 0xe2,
            0x30,
        ]);
        rom[0xd9..0xe4].copy_from_slice(&[
            0x20, 0, 0x83, 0xc2, 0x30, 0xab, 0x2b, 0x7a, 0xfa, 0x68, 0x40,
        ]);
        rom[0x300] = 0x60;
        for p in 0..=255 {
            for (bank, sp) in [(0, 0x1ff), (0x0e, 0x1f31)] {
                let mut native = NativeNmiContext {
                    a: 0x5ab3,
                    x: 0xabcd,
                    y: 0x9876,
                    d: 0x1f00,
                    db: 0x7e,
                    bank,
                    pc: 0x9234,
                    sp,
                    p,
                };
                if p & INDEX_BYTE != 0 {
                    native.x &= 0xff;
                    native.y &= 0xff;
                }
                let original = native;
                let mut ram = vec![0xa5; 0x20000];
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                machine.ram.copy_from_slice(&ram);
                let cpu = &mut machine.cpu;
                cpu.a = native.a;
                cpu.x = native.x;
                cpu.y = native.y;
                cpu.dp = native.d;
                cpu.db = native.db;
                cpu.k = native.bank;
                cpu.pc = native.pc;
                cpu.sp = native.sp;
                cpu.e = false;
                cpu.unpack_flags(p);
                let timeline = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(100, 100),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(false),
                );
                let mut source =
                    RomCpuTimingProbe::new(machine, timeline, SourcePpuReadState::snes9x_reset())
                        .unwrap();
                source.accept_native_nmi().unwrap();
                for _ in 0..12 {
                    source.step().unwrap();
                }
                native.enter_handler(&mut ram);
                compare(native, &source.snes().cpu);
                assert_eq!(ram, source.snes().ram);
                source.step().unwrap();
                native.call(&mut ram, 0x80dc);
                assert_eq!(native.sp, source.snes().cpu.sp);
                assert_eq!(ram, source.snes().ram);
                source.step().unwrap();
                native.return_from_call(&ram);
                compare(native, &source.snes().cpu);
                for _ in 0..7 {
                    source.step().unwrap();
                }
                native.leave_handler(&ram);
                compare(native, &source.snes().cpu);
                assert_eq!(native, original);
                assert_eq!(ram, source.snes().ram);
            }
        }
    }

    #[test]
    fn native_nmi_return_consumes_stack_memory_instead_of_a_saved_copy() {
        let mut native = NativeNmiContext {
            a: 0,
            x: 0,
            y: 0,
            d: 0,
            db: 0,
            bank: 0,
            pc: 0x8034,
            sp: 0x1ff,
            p: 0x30,
        };
        let mut ram = vec![0; 0x20000];
        native.enter_handler(&mut ram);
        ram[0x1fa..0x1fc].copy_from_slice(&0x4321u16.to_le_bytes());
        ram[0x1fd..0x1ff].copy_from_slice(&0x9050u16.to_le_bytes());
        native.leave_handler(&ram);
        assert_eq!((native.a, native.pc, native.sp), (0x4321, 0x9050, 0x1ff));
    }

    #[test]
    fn nmi_register_tail_matches_rom_bus_ppu_and_live_rti_frame() {
        use snes::SourceCpuBusAccessKind as Kind;
        use NmiTailAccess as Access;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(mut rom) = std::fs::read(path) else { return };
        // Test-only reset prelude establishes native mode, forced blank and
        // the NMI enable bit that is already set on entry to the real handler.
        // A warm-up NMI returns immediately. The original register tail and
        // its RTI remain unchanged, including all new timer writes.
        rom[..14].copy_from_slice(&[0x18,0xfb,0xa9,0x80,0x8d,0,0x21,0xa9,0x81,0x8d,0,0x42,0x80,0xfe]);
        rom[0x10]=0x40;
        rom[0x7fea..0x7fec].copy_from_slice(&0x8010u16.to_le_bytes());
        rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        for irq in [0,1,0x80] {
        for mode in [0, 1, 7, 0x17] {
            for status in [0, 1, 0x10, 0x20, 0x30, 0xff] {
                for (v, h) in [(250, 888), (250, 510), (261, 1300), (225, 1340)] {
                    for odd in [false, true] {
                        let mut source = snes::Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
                        let target = CpuMasterTimeline::at_raster(u64::from(odd),
                            CpuRasterPosition::new(v, h), CpuBusWorkload::default(),
                            CpuFieldTiming::NON_INTERLACE_EVEN).clock_master_cycles();
                        while source.machine().timestamp().master_cycles() < target { source.step().unwrap(); }
                        let mut ram = source.machine().snes().ram.clone();
                        for (i, b) in ram.iter_mut().enumerate() { *b = (i ^ (i >> 8) ^ usize::from(status)) as u8; }
                        ram[0x12a] = 0; ram[0x128] = irq; ram[0x9b] = 0;
                        ram[0x94] = mode; ram[0x13] = 0x80 | (status & 15);
                        // Actual live NMI save layout: DB,D,Y,X,A,P,PC,PB.
                        ram[0x1f3..0x200].copy_from_slice(&[
                            0x7e, 0x34, 0x12, 0x78, 0x56, 0xbc, 0x9a, 0xf0, 0xde,
                            status, 0x34, 0x92, 0x0e,
                        ]);
                        let mut native = NativeNmiContext { a: 0x0200, x: 0x93, y: 0x87,
                            d: 0, db: 0, bank: 0, pc: 0x8144, sp: 0x1f2, p: 0x34 };
                        let mut registers = source.machine().snes().cpu.clone();
                        registers.a = native.a; registers.x = native.x; registers.y = native.y;
                        registers.pc = native.pc; registers.sp = native.sp;
                        registers.dp = 0; registers.db = 0; registers.k = 0;
                        registers.e = false; registers.unpack_flags(native.p);
                        let sram = source.machine().snes().cart.ram.clone();
                        source.rebase_private_timing_state_at(source.machine().timestamp(), &registers, &ram, &sram).unwrap();
                        let mut clock = source.machine().timeline().clone();
                        let mut ppu = source.machine().snes().ppu.clone();
                        let mut expected = Vec::new();
                        while source.machine().snes().cpu.pc != 0x9234 {
                            for access in source.step().unwrap().accesses {
                                if access.address < 0x2000 || matches!(access.address, 0x2100..=0x213f | 0x4200..=0x420c | 0x4211) {
                                    let address = access.address as u16;
                                    let operation = match access.kind {
                                        Kind::Read { width: 1, .. } => Access::ReadByte(address),
                                        Kind::Read { width: 2, .. } => Access::ReadWord(address),
                                        Kind::Write { value, width: 1 } => Access::WriteByte(address, value as u8),
                                        _ => panic!("unexpected tail transaction {access:?}"),
                                    };
                                    expected.push((access.timestamp.master_cycles(), operation));
                                }
                            }
                        }
                        let mut actual = Vec::new();
                        let programming=native.initial_register_tail(&mut clock, |at, access| {
                            actual.push((at, access));
                            match access {
                                Access::ReadByte(0x4211) => 0x42,
                                Access::ReadByte(address) => u16::from(ram[usize::from(address)]),
                                Access::ReadWord(address) => u16::from_le_bytes(ram[usize::from(address)..usize::from(address)+2].try_into().unwrap()),
                                Access::WriteByte(0x4200|0x4207..=0x420a,_)=>0,
                                Access::WriteByte(0x420c, value) => { assert_eq!(value, 0); 0 }
                                Access::WriteByte(address, value) => { ppu.write(address as u8, value); 0 }
                            }
                        });
                        assert_eq!(programming.is_some(),irq!=0);
                        if let Some(p)=programming {
                            assert_eq!(p.vertical_timer,source.machine().snes().v_timer);
                            assert_eq!(source.machine().snes().h_timer,0);
                            assert!(actual.contains(&(p.enabled_at,Access::WriteByte(0x4200,p.nmitimen))));
                        }
                        let label = format!("irq={irq} mode={mode:02x} status={status:02x} v={v} h={h} odd={odd}");
                        assert_eq!(actual, expected, "{label}");
                        assert_eq!(clock, *source.machine().timeline(), "{label}");
                        assert_eq!(ram, source.machine().snes().ram, "{label}");
                        assert_eq!(bincode::serialize(&ppu).unwrap(), bincode::serialize(&source.machine().snes().ppu).unwrap(), "{label}");
                        compare(native, &source.machine().snes().cpu);
                    }
                }
            }
        }
        }
    }

    #[test]
    fn nmi_joypad_call_matches_rom_bus_filters_and_return() {
        use snes::SourceCpuBusAccessKind as Kind;
        use NmiJoypadAccess as Access;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(rom) = std::fs::read(path) else { return };
        for polled in [0u16, 0xffff, 0x8001, 0x5aa5, 0x00ff, 0xff00] {
            for previous in [0u16, 0xffff, 0xa55a] {
                for (v, h) in [(250, 300), (250, 510), (261, 1300)] {
                    for odd in [false, true] {
                        let mut native = NativeNmiContext { a: 0x5a00, x: 0x93, y: 0x87,
                            d: 0, db: 0, bank: 0, pc: 0x8141, sp: 0x1f2,
                            p: if odd { 0x75 } else { 0x34 } };
                        // Test-only reset prelude enables actual auto polling,
                        // enters native mode and parks. The NMI body is intact.
                        let mut fixture = rom.clone();
                        fixture[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 1, 0x8d, 0, 0x42, 0x80, 0xfe]);
                        fixture[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
                        let mut source = snes::Snes9xColdCpuExecutor::from_lorom_reset(&fixture).unwrap();
                        source.set_joypad_serial_state(polled.reverse_bits(), 0);
                        let target = CpuMasterTimeline::at_raster(u64::from(odd),
                            CpuRasterPosition::new(v, h), CpuBusWorkload::default(),
                            CpuFieldTiming::NON_INTERLACE_EVEN).clock_master_cycles();
                        while source.machine().timestamp().master_cycles() < target {
                            source.step().unwrap();
                        }
                        assert_eq!(source.machine().snes().port_auto_read[0], polled);
                        let mut ram = source.machine().snes().ram.clone();
                        ram.fill(0xa5);
                        ram[0xfa] = previous as u8;
                        ram[0xf8] = (previous >> 8) as u8;
                        let mut registers = source.machine().snes().cpu.clone();
                        registers.a = native.a; registers.x = native.x; registers.y = native.y;
                        registers.pc = native.pc; registers.sp = native.sp;
                        registers.dp = 0; registers.db = 0; registers.k = 0;
                        registers.e = false; registers.unpack_flags(native.p);
                        let sram = source.machine().snes().cart.ram.clone();
                        source.rebase_private_timing_state_at(source.machine().timestamp(),
                            &registers, &ram, &sram).unwrap();
                        let mut clock = source.machine().timeline().clone();
                        let mut expected = Vec::new();
                        while source.machine().snes().cpu.pc != 0x8144 {
                            for access in source.step().unwrap().accesses {
                                if access.address < 0x2000 || matches!(access.address, 0x4016 | 0x4218 | 0x4219) {
                                    let address = access.address as u16;
                                    let operation = match access.kind {
                                        Kind::Read { width: 1, .. } => Access::ReadByte(address),
                                        Kind::Read { width: 2, .. } => Access::ReadWord(address),
                                        Kind::Write { value, width: 1 } => Access::WriteByte(address, value as u8),
                                        Kind::Write { value, width: 2 } => Access::WriteWord(address, value),
                                        _ => panic!("unexpected joypad transaction {access:?}"),
                                    };
                                    expected.push((access.timestamp.master_cycles(), operation));
                                }
                            }
                        }
                        let mut actual = Vec::new();
                        native.read_joypads_clocked(&mut clock, |at, access| {
                            actual.push((at, access));
                            match access {
                                Access::ReadByte(0x4218) => polled & 0xff,
                                Access::ReadByte(0x4219) => polled >> 8,
                                Access::ReadByte(address) => u16::from(ram[usize::from(address)]),
                                Access::ReadWord(address) => u16::from_le_bytes(ram[usize::from(address)..usize::from(address)+2].try_into().unwrap()),
                                Access::WriteByte(0x4016, value) => { assert_eq!(value, 0); 0 }
                                Access::WriteByte(address, value) => { ram[usize::from(address)] = value; 0 }
                                Access::WriteWord(address, value) => {
                                    ram[usize::from(address)..usize::from(address)+2].copy_from_slice(&value.to_le_bytes()); 0
                                }
                            }
                        });
                        assert_eq!(actual, expected, "polled={polled:04x} previous={previous:04x} v={v} h={h} odd={odd}");
                        assert_eq!(ram, source.machine().snes().ram);
                        assert_eq!(clock, *source.machine().timeline());
                        compare(native, &source.machine().snes().cpu);
                    }
                }
            }
        }
    }

    #[test]
    fn nmi_audio_and_update_gate_matches_rom_data_bus_and_state() {
        use snes::SourceCpuBusAccessKind as Access;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(rom) = std::fs::read(path) else { return };
        let cases = [(0, 0, 0), (0, 0, 0x8b), (0, 0x83, 0x83),
            (0, 0x80, 0x7f), (0x37, 0x12, 0), (0xff, 0xff, 0), (0x80, 0, 0x80)];
        for (ambient, last, echo) in cases {
            for latch in [0, 1, 0xff] {
                for (v, h) in [(225, 580), (225, 510), (225, 1350), (240, 1350)] {
                    for odd in [false, true] {
                        let mut native = NativeNmiContext { a: 0xab8b, x: 23, y: 47, d: 0,
                            db: 0, bank: 0, pc: 0x8103, sp: 0x1f2, p: if odd { 0x74 } else { 0x75 } };
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        machine.ram.fill(0xa5);
                        machine.ram[0x12] = latch;
                        machine.ram[0x12d] = ambient;
                        machine.ram[0x131] = last;
                        machine.ram[0x12e] = ambient ^ 0x5a;
                        machine.ram[0x12f] = echo ^ 0xa5;
                        let mut ram = machine.ram.clone();
                        machine.cpu.a = native.a; machine.cpu.x = native.x; machine.cpu.y = native.y;
                        machine.cpu.dp = native.d; machine.cpu.db = native.db;
                        machine.cpu.pc = native.pc; machine.cpu.sp = native.sp;
                        machine.cpu.e = false; machine.cpu.unpack_flags(native.p);
                        let mut clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(v, h),
                            CpuBusWorkload::default(), CpuFieldTiming::non_interlace(odd));
                        let mut apu = snes::apu::ApuState::new();
                        apu.reset(); apu.cpu_cycles_left = 0; apu.spc.pc = 0x2000;
                        apu.ram[0x2000..0x2002].copy_from_slice(&[0x2f, 0xfe]);
                        apu.out_ports[1] = echo;
                        let owner = snes::apu::ApuHostPortTiming::new(
                            snes::apu::ApuHostPortProbe::new(apu).unwrap(),
                            snes::Snes9xApuClockCheckpoint::new(clock.clock_master_cycles(), 0, 0).unwrap(),
                        ).unwrap();
                        let mut source = RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset()).unwrap();
                        source.attach_apu_port_owner(owner).unwrap();
                        clock.begin_synchronous_timeline().unwrap();
                        let mut expected = Vec::new();
                        while !matches!(source.snes().cpu.pc, 0x89e0 | 0x8144) {
                            let receipt = source.step().unwrap();
                            for access in receipt.accesses {
                                if access.address < 0x2000 || matches!(access.address, 0x2100 | 0x2141..=0x2143 | 0x420c) {
                                    let address = access.address as u16;
                                    let operation = match access.kind {
                                        Access::Read { width: 1, .. } => NmiHeaderAccess::Read(address),
                                        Access::Write { value, width: 1 } => NmiHeaderAccess::Write(address, value as u8),
                                        Access::Write { value, width: 2 } => NmiHeaderAccess::WriteWord(address, value),
                                        _ => panic!("unexpected header data transaction {access:?}"),
                                    };
                                    expected.push((access.timestamp.master_cycles(), operation));
                                }
                            }
                        }
                        let mut actual = Vec::new();
                        let mut input_ports = [0; 4];
                        let result = native.audio_and_update_gate(&mut clock, |at, access| {
                            actual.push((at, access));
                            match access {
                                NmiHeaderAccess::Read(0x2141) => echo,
                                NmiHeaderAccess::Read(address) => ram[usize::from(address)],
                                NmiHeaderAccess::Write(address, value) => {
                                    if address < 0x2000 { ram[usize::from(address)] = value; }
                                    else if (0x2140..=0x2143).contains(&address) { input_ports[usize::from(address - 0x2140)] = value; }
                                    0
                                }
                                NmiHeaderAccess::WriteWord(address, value) => {
                                    ram[usize::from(address)..usize::from(address)+2].copy_from_slice(&value.to_le_bytes());
                                    0
                                }
                            }
                        });
                        assert_eq!(actual, expected, "ambient={ambient:02x} last={last:02x} echo={echo:02x} latch={latch:02x} v={v} h={h} odd={odd}");
                        assert_eq!(ram, source.snes().ram);
                        assert_eq!(input_ports, source.apu_ports().unwrap().machine().in_ports[..4]);
                        assert_eq!(clock, *source.timeline());
                        assert_eq!(result.calls_updates, latch == 0);
                        compare(native, &source.snes().cpu);
                    }
                }
            }
        }
    }

    #[test]
    fn music_header_matches_rom_commands_echoes_and_bus_timestamps() {
        use snes::SourceCpuBusAccessKind as Kind;
        let rom=std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")).unwrap();
        let mut cases=0;
        for command in 0..=255u8 {
            for last in [command,command.wrapping_add(1)] {
                for (echo,h) in [(last,28),(last^0x80,378),(last,510),(last^0x80,1350)] {
                    let mut native=NativeNmiContext {a:0xab00,x:23,y:47,d:0,db:0,bank:0,pc:0x80d9,sp:0x1f2,p:0x75};
                    let mut machine=Snes::new();machine.cart.load(CartType::LoRom,&rom,0x2000);
                    machine.ram[0x12c]=command;machine.ram[0x133]=last;machine.ram[0x130]=0x7b;
                    machine.cpu.a=native.a;machine.cpu.x=native.x;machine.cpu.y=native.y;machine.cpu.dp=0;
                    machine.cpu.db=0;machine.cpu.pc=native.pc;machine.cpu.sp=native.sp;machine.cpu.e=false;machine.cpu.unpack_flags(native.p);
                    let mut ram=machine.ram.clone();
                    let mut clock=CpuMasterTimeline::at_raster(0,CpuRasterPosition::new(225,h),CpuBusWorkload::default(),CpuFieldTiming::NON_INTERLACE_EVEN);
                    let mut source=RomCpuTimingProbe::new(machine,clock.clone(),SourcePpuReadState::snes9x_reset()).unwrap();
                    let mut apu=snes::apu::ApuState::new();apu.reset();apu.cpu_cycles_left=0;apu.spc.pc=0x2000;
                    apu.ram[0x2000..0x2002].copy_from_slice(&[0x2f,0xfe]);apu.out_ports[0]=echo;
                    let owner=snes::apu::ApuHostPortTiming::new(snes::apu::ApuHostPortProbe::new(apu).unwrap(),
                        snes::Snes9xApuClockCheckpoint::new(clock.timestamp().master_cycles(),0,0).unwrap()).unwrap();
                    source.attach_apu_port_owner(owner).unwrap();clock.begin_synchronous_timeline().unwrap();
                    let mut expected=Vec::new();
                    while source.snes().cpu.pc!=0x8103 {
                        for access in source.step().unwrap().accesses {
                            if !matches!(access.address,0x12c|0x130|0x133|0x2140){continue;}
                            let (op,value)=match access.kind {
                                Kind::Read {value,width:1}=>(NmiHeaderAccess::Read(access.address as u16),value as u8),
                                Kind::Write {value,width:1}=>(NmiHeaderAccess::Write(access.address as u16,value as u8),0),
                                _=>panic!("unexpected music bus access"),
                            };
                            expected.push((access.timestamp.master_cycles(),op,value));
                        }
                    }
                    let mut actual=Vec::new();
                    let (issued,_)=native.music_header(&mut clock,|at,access| {
                        let value=match access {
                            NmiHeaderAccess::Read(0x2140)=>echo,
                            NmiHeaderAccess::Read(a)=>ram[usize::from(a)],
                            NmiHeaderAccess::Write(0x2140,_)=>0,
                            NmiHeaderAccess::Write(a,v)=>{ram[usize::from(a)]=v;0},
                            _=>unreachable!(),
                        };actual.push((at,access,value));value
                    });
                    assert_eq!(actual,expected,"command={command:02x} last={last:02x} echo={echo:02x} h={h}");
                    assert_eq!(issued,(command!=0&&command!=last).then_some(command));
                    assert_eq!(clock,*source.timeline());assert_eq!(ram,source.snes().ram);compare(native,&source.snes().cpu);
                    cases+=1;
                }
            }
        }
        eprintln!("{cases} source-verified music command/acknowledgement cases");
    }

    #[test]
    fn native_nmi_entry_bus_matches_rom_across_refresh_and_field_boundaries() {
        use snes::{SourceCpuBusAccessKind as Access, SourceCpuTransactionKind as Kind};
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc");
        let Ok(rom) = std::fs::read(path) else { return };
        // These positions straddle fetch/drain, WRAM refresh, HMax, vblank,
        // and the odd-field short line. They are test inputs, never clock seeds
        // for the native runtime, which starts at the completed upload.
        for odd in [false, true] {
            for v in [0, 100, 224, 225, 240, 261] {
                // Start outside the refresh-owned interval: a bare seed in
                // that interval cannot establish pending/drained ownership.
                for h in [0, 26, 28, 510, 526, 528, 530, 532, 1090, 1350, 1358] {
                    let mut native = NativeNmiContext {
                        a: 0x55a3,
                        x: 0x1234,
                        y: 0x6789,
                        d: 0x1e00,
                        db: 0x7e,
                        bank: 0x0e,
                        pc: 0x9234,
                        sp: 0x1f31,
                        p: 0x49,
                    };
                    let mut ram = vec![0xa5; 0x20000];
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    machine.ram.copy_from_slice(&ram);
                    let cpu = &mut machine.cpu;
                    cpu.a = native.a;
                    cpu.x = native.x;
                    cpu.y = native.y;
                    cpu.dp = native.d;
                    cpu.db = native.db;
                    cpu.k = native.bank;
                    cpu.pc = native.pc;
                    cpu.sp = native.sp;
                    cpu.e = false;
                    cpu.unpack_flags(native.p);
                    let mut clock = CpuMasterTimeline::at_raster(
                        u64::from(odd),
                        CpuRasterPosition::new(v, h),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::NON_INTERLACE_EVEN,
                    );
                    let mut source = RomCpuTimingProbe::new(
                        machine,
                        clock.clone(),
                        SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    // The source probe installs its synchronous event cursor.
                    clock.begin_synchronous_timeline().unwrap();
                    let entry = source.accept_native_nmi().unwrap();
                    let mut expected = entry
                        .transactions
                        .into_iter()
                        .map(|t| {
                            (
                                t.kind,
                                t.duration_master_cycles,
                                t.started_at,
                                t.ended_at,
                                t.start_wram_refresh_position,
                                t.end_wram_refresh_position,
                            )
                        })
                        .collect::<Vec<_>>();
                    let mut accesses = entry.accesses;
                    for _ in 0..12 {
                        let step = source.step().unwrap();
                        expected.extend(step.transactions.into_iter().map(|t| {
                            (
                                t.kind,
                                t.duration_master_cycles,
                                t.started_at,
                                t.ended_at,
                                t.start_wram_refresh_position,
                                t.end_wram_refresh_position,
                            )
                        }));
                        accesses.extend(step.accesses);
                    }
                    let expected_memory = accesses
                        .into_iter()
                        .filter_map(|a| {
                            if a.address >= 0x2000 && a.address != 0xffea {
                                return None;
                            }
                            let address = a.address as u16;
                            let event = match a.kind {
                                Access::Read { value, width } => EntryBus::Read {
                                    address,
                                    value,
                                    width,
                                },
                                Access::Write { value, width } => EntryBus::Write {
                                    address,
                                    value,
                                    width,
                                },
                                _ => panic!("unexpected stack/vector access"),
                            };
                            Some((event, a.timestamp))
                        })
                        .collect::<Vec<_>>();
                    let mut actual = Vec::new();
                    let mut memory = Vec::new();
                    native.enter_handler_with_bus(&mut ram, |event, ram| {
                        let started = clock.timestamp();
                        let refresh = clock.wram_refresh_cycle() as u16;
                        let (kind, duration) = match event {
                            EntryBus::Fetch => (Kind::FastPcBaseOpcodeFetchNonDraining, 8),
                            EntryBus::Internal(cycles) => (Kind::CpuOpsAddCyclesDraining, cycles),
                            EntryBus::RegisterAccess => (Kind::GetSetMemoryAccessAfterSemanticDraining, 6),
                            EntryBus::Read { width, .. } | EntryBus::Write { width, .. } => {
                                memory.push((event, started));
                                (
                                    if width == 1 {
                                        Kind::GetSetMemoryAccessAfterSemanticDraining
                                    } else {
                                        Kind::GetSetMemoryAccessX2AfterSemanticDraining
                                    },
                                    8 * width,
                                )
                            }
                        };
                        if let EntryBus::Write {
                            address,
                            value,
                            width,
                        } = event
                        {
                            assert_eq!(ram[usize::from(address)], value as u8);
                            if width == 2 {
                                assert_eq!(ram[usize::from(address) + 1], (value >> 8) as u8);
                            }
                        }
                        event.advance(&mut clock);
                        actual.push((
                            kind,
                            duration,
                            started,
                            clock.timestamp(),
                            refresh,
                            clock.wram_refresh_cycle() as u16,
                        ));
                    });
                    assert_eq!(actual, expected, "field={odd} v={v} h={h}");
                    assert_eq!(memory, expected_memory, "field={odd} v={v} h={h}");
                    assert_eq!(clock, *source.timeline(), "field={odd} v={v} h={h}");
                    compare(native, &source.snes().cpu);
                    assert_eq!(ram, source.snes().ram);
                    assert_eq!(native.pc, 0x80d9);
                }
            }
        }
    }
}

impl NativeNmiContext {
    /// Publish the non-threaded startup handler's live register operands,
    /// then restore its actual saved stack frame and execute native-mode RTI.
    /// Return live IRQ programming to the caller that owns subsequent execution.
    pub(crate) fn initial_register_tail(
        &mut self, clock: &mut CpuMasterTimeline,
        mut bus: impl FnMut(u64, NmiTailAccess) -> u16,
    ) -> Option<super::poly_interrupt::PolyIrqProgramming> {
        use EntryBus::*;
        assert_eq!((self.bank, self.pc, self.db, self.d, self.p & 0x30),
            (0, 0x8144, 0, 0, ACCUMULATOR_BYTE | INDEX_BYTE));
        let thread = self.tail_load(clock, &mut bus, 0x12a);
        assert_eq!(thread, 0, "native startup tail cannot switch poly threads");
        for work in [Fetch, Internal(8), Internal(6)] { work.advance(clock); } // BEQ
        for (source, target) in [
            (0x96, 0x2123), (0x97, 0x2124), (0x98, 0x2125),
            (0x99, 0x2130), (0x9a, 0x2131),
            (0x9c, 0x2132), (0x9d, 0x2132), (0x9e, 0x2132),
            (0x1c, 0x212c), (0x1d, 0x212d), (0x1e, 0x212e), (0x1f, 0x212f),
            (0x120, 0x210d), (0x121, 0x210d), (0x124, 0x210e), (0x125, 0x210e),
            (0x11e, 0x210f), (0x11f, 0x210f), (0x122, 0x2110), (0x123, 0x2110),
            (0xe4, 0x2111), (0xe5, 0x2111), (0xea, 0x2112), (0xeb, 0x2112),
            (0x95, 0x2106), (0x94, 0x2105),
        ] {
            let value = self.tail_load(clock, &mut bus, source);
            self.tail_store(clock, &mut bus, target, value);
        }
        for work in [Fetch, Internal(8)] { work.advance(clock); } // AND #7
        self.load_a_byte(self.a as u8 & 7);
        for work in [Fetch, Internal(8)] { work.advance(clock); } // CMP #7
        self.compare_a_byte(7);
        for work in [Fetch, Internal(8)] { work.advance(clock); } // BNE
        if self.a as u8 != 7 {
            Internal(6).advance(clock);
        } else {
            for target in [0x211c, 0x211c, 0x211d, 0x211d] {
                self.tail_store(clock, &mut bus, target, 0);
            }
            for (source, target) in [(0x638, 0x211f), (0x639, 0x211f), (0x63a, 0x2120), (0x63b, 0x2120)] {
                let value = self.tail_load(clock, &mut bus, source);
                self.tail_store(clock, &mut bus, target, value);
            }
        }
        let irq = self.tail_load(clock, &mut bus, 0x128);
        for work in [Fetch, Internal(8)] { work.advance(clock); } // BEQ
        let programming=if irq==0 {
            Internal(6).advance(clock);None
        } else {
            // TIMEUP uses the actual pending hardware line and acknowledges it.
            for work in [Fetch,Internal(16)] {work.advance(clock);}
            let value=bus(clock.clock_master_cycles(),NmiTailAccess::ReadByte(0x4211)) as u8;
            RegisterAccess.advance(clock);self.load_a_byte(value);
            for work in [Fetch,Internal(8)] {work.advance(clock);}
            self.load_a_byte(0x80);
            self.tail_store(clock,&mut bus,0x4209,0x80);
            for address in [0x420a,0x4207,0x4208] {self.tail_store(clock,&mut bus,address,0);}
            for work in [Fetch,Internal(8)] {work.advance(clock);}
            self.load_a_byte(0xa1);
            for work in [Fetch,Internal(16)] {work.advance(clock);}
            let enabled_at=clock.clock_master_cycles();
            let irq_deadline=snes::cpu_timeline::vertical_irq_deadline(clock,0x80,true).map(|at|at.master_cycles());
            bus(enabled_at,NmiTailAccess::WriteByte(0x4200,0xa1));RegisterAccess.advance(clock);
            Some(super::poly_interrupt::PolyIrqProgramming{vertical_timer:0x80,nmitimen:0xa1,enabled_at,irq_deadline})
        };
        for (source, target) in [(0x13, 0x2100), (0x9b, 0x420c)] {
            let value = self.tail_load(clock, &mut bus, source);
            self.tail_store(clock, &mut bus, target, value);
        }
        for work in [Fetch, Internal(8), Internal(6)] { work.advance(clock); } // REP #$30
        self.p &= !(ACCUMULATOR_BYTE | INDEX_BYTE);
        for work in [Fetch, Internal(12)] { work.advance(clock); } // PLB
        self.db = self.tail_pop(clock, &mut bus, 1) as u8;
        self.p = (self.p & !(ZERO | NEGATIVE)) | (self.db & NEGATIVE)
            | if self.db == 0 { ZERO } else { 0 };
        for destination in 0..4 { // PLD, PLY, PLX, PLA
            for work in [Fetch, Internal(12)] { work.advance(clock); }
            let value = self.tail_pop(clock, &mut bus, 2);
            match destination { 0 => self.d = value, 1 => self.y = value,
                2 => self.x = value, _ => self.a = value }
            self.p = (self.p & !(ZERO | NEGATIVE)) | ((value >> 8) as u8 & NEGATIVE)
                | if value == 0 { ZERO } else { 0 };
        }
        for work in [Fetch, Internal(12)] { work.advance(clock); } // RTI
        self.p = self.tail_pop(clock, &mut bus, 1) as u8;
        if self.p & INDEX_BYTE != 0 { self.x &= 0xff; self.y &= 0xff; }
        self.pc = self.tail_pop(clock, &mut bus, 2);
        self.bank = self.tail_pop(clock, &mut bus, 1) as u8;
        programming
    }

    fn tail_load(&mut self, clock: &mut CpuMasterTimeline,
        bus: &mut impl FnMut(u64, NmiTailAccess) -> u16, address: u16) -> u8 {
        EntryBus::Fetch.advance(clock);
        EntryBus::Internal(if address < 0x100 { 8 } else { 16 }).advance(clock);
        let value = bus(clock.clock_master_cycles(), NmiTailAccess::ReadByte(address)) as u8;
        EntryBus::Read { address, value: u16::from(value), width: 1 }.advance(clock);
        self.load_a_byte(value);
        value
    }

    fn tail_store(&mut self, clock: &mut CpuMasterTimeline,
        bus: &mut impl FnMut(u64, NmiTailAccess) -> u16, address: u16, value: u8) {
        for work in [EntryBus::Fetch, EntryBus::Internal(16)] { work.advance(clock); }
        bus(clock.clock_master_cycles(), NmiTailAccess::WriteByte(address, value));
        EntryBus::RegisterAccess.advance(clock);
    }

    fn tail_pop(&mut self, clock: &mut CpuMasterTimeline,
        bus: &mut impl FnMut(u64, NmiTailAccess) -> u16, width: u8) -> u16 {
        let address = self.sp.wrapping_add(1);
        assert!(usize::from(address) + usize::from(width) <= 0x2000);
        let access = if width == 1 { NmiTailAccess::ReadByte(address) }
            else { NmiTailAccess::ReadWord(address) };
        let value = bus(clock.clock_master_cycles(), access);
        EntryBus::Read { address, value, width }.advance(clock);
        self.sp = self.sp.wrapping_add(u16::from(width));
        value
    }

    /// The updates caller owns the JSR, both hardware input reads, the live
    /// WRAM edge filters and the RTS. The bus supplies the completed automatic
    /// controller poll; it must not substitute already-filtered gameplay input.
    pub(crate) fn read_joypads_clocked(
        &mut self,
        clock: &mut CpuMasterTimeline,
        mut bus: impl FnMut(u64, NmiJoypadAccess) -> u16,
    ) {
        use EntryBus::*;
        use NmiJoypadAccess as Access;
        assert_eq!((self.bank, self.pc, self.db, self.d, self.p & 0x30),
            (0, 0x8141, 0, 0, ACCUMULATOR_BYTE | INDEX_BYTE));
        for work in [Fetch, Internal(16), Internal(6)] { work.advance(clock); }
        assert!((1..0x2000).contains(&self.sp));
        let address = self.sp - 1;
        bus(clock.clock_master_cycles(), Access::WriteWord(address, 0x8143));
        Write { address, value: 0x8143, width: 2 }.advance(clock);
        self.sp = self.sp.wrapping_sub(2);
        // STZ $4016 uses the slow controller port, not a six-clock register.
        for work in [Fetch, Internal(16)] { work.advance(clock); }
        bus(clock.clock_master_cycles(), Access::WriteByte(0x4016, 0));
        Internal(12).advance(clock);
        for (port, scratch) in [(0x4218, 0), (0x4219, 1)] {
            for work in [Fetch, Internal(16)] { work.advance(clock); }
            let value = bus(clock.clock_master_cycles(), Access::ReadByte(port)) as u8;
            RegisterAccess.advance(clock);
            self.load_a_byte(value);
            self.joypad_store_dp(clock, &mut bus, scratch, value);
        }
        for (scratch, current, filtered, last) in [(0, 0xf2, 0xf6, 0xfa), (1, 0xf0, 0xf4, 0xf8)] {
            let value = self.joypad_read_dp(clock, &mut bus, scratch);
            self.load_a_byte(value);
            self.joypad_store_dp(clock, &mut bus, current, value);
            for work in [Fetch, Internal(6)] { work.advance(clock); } // TAY
            self.y = u16::from(value);
            let prior = self.joypad_read_dp(clock, &mut bus, last);
            self.load_a_byte(self.a as u8 ^ prior); // EOR dp
            let held = self.joypad_read_dp(clock, &mut bus, current);
            self.load_a_byte(self.a as u8 & held); // AND dp
            self.joypad_store_dp(clock, &mut bus, filtered, self.a as u8);
            self.joypad_store_dp(clock, &mut bus, last, self.y as u8);
        }
        for work in [Fetch, Internal(12)] { work.advance(clock); } // RTS
        let address = self.sp.wrapping_add(1);
        let returned = bus(clock.clock_master_cycles(), Access::ReadWord(address));
        Read { address, value: returned, width: 2 }.advance(clock);
        self.sp = self.sp.wrapping_add(2);
        Internal(6).advance(clock);
        self.pc = returned.wrapping_add(1);
    }

    fn joypad_read_dp(&mut self, clock: &mut CpuMasterTimeline,
        bus: &mut impl FnMut(u64, NmiJoypadAccess) -> u16, address: u16) -> u8 {
        for work in [EntryBus::Fetch, EntryBus::Internal(8)] { work.advance(clock); }
        let value = bus(clock.clock_master_cycles(), NmiJoypadAccess::ReadByte(address)) as u8;
        EntryBus::Read { address, value: u16::from(value), width: 1 }.advance(clock);
        value
    }

    fn joypad_store_dp(&mut self, clock: &mut CpuMasterTimeline,
        bus: &mut impl FnMut(u64, NmiJoypadAccess) -> u16, address: u16, value: u8) {
        for work in [EntryBus::Fetch, EntryBus::Internal(8)] { work.advance(clock); }
        bus(clock.clock_master_cycles(), NmiJoypadAccess::WriteByte(address, value));
        EntryBus::Write { address, value: u16::from(value), width: 1 }.advance(clock);
    }

    /// Own the ambient/effect ports, latch clears, blanking and update gate.
    /// Each callback is the actual data-bus transaction. Stop at the updates
    /// callee entry or the taken gate's register-tail entry, never after an
    /// unclocked callee with a reconstructed timestamp.
    pub(crate) fn audio_and_update_gate(
        &mut self,
        clock: &mut CpuMasterTimeline,
        mut bus: impl FnMut(u64, NmiHeaderAccess) -> u8,
    ) -> NmiHeaderResult {
        use EntryBus::*;
        assert_eq!((self.bank, self.pc, self.db, self.d, self.p & ACCUMULATOR_BYTE),
            (0, 0x8103, 0, 0, ACCUMULATOR_BYTE));
        let ambient = self.header_load_absolute(clock, &mut bus, 0x12d);
        for work in [Fetch, Internal(8)] { work.advance(clock); } // BNE
        let (ambient_write, ambient_master_cycles) = if ambient != 0 {
            Internal(6).advance(clock);
            self.header_store_absolute(clock, &mut bus, 0x131, ambient);
            self.header_store_absolute(clock, &mut bus, 0x2141, ambient);
            self.header_store_absolute(clock, &mut bus, 0x12d, 0);
            (Some(ambient), 54 + 94)
        } else {
            let echo = self.header_load_absolute(clock, &mut bus, 0x2141);
            for work in [Fetch, Internal(16)] { work.advance(clock); } // CMP $0131
            let last = bus(clock.clock_master_cycles(), NmiHeaderAccess::Read(0x131));
            Read { address: 0x131, value: u16::from(last), width: 1 }.advance(clock);
            self.compare_a_byte(last);
            for work in [Fetch, Internal(8)] { work.advance(clock); }
            if echo == last {
                self.header_store_absolute(clock, &mut bus, 0x2141, 0);
                for work in [Fetch, Internal(8), Internal(6)] { work.advance(clock); } // BRA
                (Some(0), 48 + 78 + 52)
            } else {
                Internal(6).advance(clock);
                (None, 48 + 84)
            }
        };
        let mut effects = [0; 2];
        for (i, effect) in effects.iter_mut().enumerate() {
            *effect = self.header_load_absolute(clock, &mut bus, 0x12e + i as u16);
            self.header_store_absolute(clock, &mut bus, 0x2142 + i as u16, *effect);
        }
        for address in [0x12e, 0x12f] {
            self.header_store_absolute(clock, &mut bus, address, 0);
        }
        for work in [Fetch, Internal(8)] { work.advance(clock); } // LDA #$80
        self.load_a_byte(0x80);
        self.header_store_absolute(clock, &mut bus, 0x2100, 0x80);
        self.header_store_absolute(clock, &mut bus, 0x420c, 0);
        for work in [Fetch, Internal(8)] { work.advance(clock); } // LDA $12
        let latch = bus(clock.clock_master_cycles(), NmiHeaderAccess::Read(0x12));
        Read { address: 0x12, value: u16::from(latch), width: 1 }.advance(clock);
        self.load_a_byte(latch);
        for work in [Fetch, Internal(8)] { work.advance(clock); } // BNE
        if latch != 0 {
            Internal(6).advance(clock);
            self.pc = 0x8144;
        } else {
            for work in [Fetch, Internal(8)] { work.advance(clock); } // INC $12
            let value = bus(clock.clock_master_cycles(), NmiHeaderAccess::Read(0x12));
            Read { address: 0x12, value: u16::from(value), width: 1 }.advance(clock);
            Internal(6).advance(clock);
            let value = value.wrapping_add(1);
            self.p = (self.p & !(ZERO | NEGATIVE))
                | if value == 0 { ZERO } else { 0 } | (value & NEGATIVE);
            bus(clock.clock_master_cycles(), NmiHeaderAccess::Write(0x12, value));
            Write { address: 0x12, value: u16::from(value), width: 1 }.advance(clock);
            for work in [Fetch, Internal(16), Internal(6)] { work.advance(clock); } // JSR
            assert!((1..0x2000).contains(&self.sp));
            let address = self.sp - 1;
            bus(clock.clock_master_cycles(), NmiHeaderAccess::WriteWord(address, 0x8140));
            Write { address, value: 0x8140, width: 2 }.advance(clock);
            self.sp = self.sp.wrapping_sub(2);
            self.pc = 0x89e0;
        }
        NmiHeaderResult { ambient_write, effects, ambient_master_cycles, calls_updates: latch == 0 }
    }

    fn header_load_absolute(
        &mut self, clock: &mut CpuMasterTimeline,
        bus: &mut impl FnMut(u64, NmiHeaderAccess) -> u8, address: u16,
    ) -> u8 {
        for work in [EntryBus::Fetch, EntryBus::Internal(16)] { work.advance(clock); }
        let value = bus(clock.clock_master_cycles(), NmiHeaderAccess::Read(address));
        Self::header_data_cycle(clock, address);
        self.load_a_byte(value);
        value
    }

    fn header_store_absolute(
        &mut self, clock: &mut CpuMasterTimeline,
        bus: &mut impl FnMut(u64, NmiHeaderAccess) -> u8, address: u16, value: u8,
    ) {
        for work in [EntryBus::Fetch, EntryBus::Internal(16)] { work.advance(clock); }
        bus(clock.clock_master_cycles(), NmiHeaderAccess::Write(address, value));
        Self::header_data_cycle(clock, address);
    }

    fn header_data_cycle(clock: &mut CpuMasterTimeline, address: u16) {
        if address < 0x2000 {
            EntryBus::Internal(8).advance(clock);
        } else {
            EntryBus::RegisterAccess.advance(clock);
        }
    }

    fn compare_a_byte(&mut self, operand: u8) {
        let value = self.a as u8;
        self.p = (self.p & !(CARRY | ZERO | NEGATIVE))
            | u8::from(value >= operand) | if value == operand { ZERO } else { 0 }
            | (value.wrapping_sub(operand) & NEGATIVE);
    }

    /// Retire the NMI music block using live command and acknowledgement
    /// latches. A repeated command remains pending, as in the source branch.
    /// The result records a newly issued command and the existing inclusive
    /// NMI music ledger charge; all hardware time advances on the actual bus.
    pub(crate) fn music_header(
        &mut self, clock: &mut CpuMasterTimeline,
        mut bus: impl FnMut(u64, NmiHeaderAccess) -> u8,
    ) -> (Option<u8>, u64) {
        use EntryBus::*;
        assert_eq!((self.bank,self.pc,self.db,self.d,self.p & ACCUMULATOR_BYTE),
            (0,0x80d9,0,0,ACCUMULATOR_BYTE));
        // LDA $4210 acknowledges the accepted NMI. LDA $012C replaces its
        // discarded value and flags before any branch or store.
        for work in [Fetch,Internal(16),RegisterAccess] {work.advance(clock);}
        let command=self.header_load_absolute(clock,&mut bus,0x12c);
        for work in [Fetch,Internal(8)] {work.advance(clock);}
        let (issued,cycles)=if command==0 {
            let echo=self.header_load_absolute(clock,&mut bus,0x2140);
            for work in [Fetch,Internal(16)] {work.advance(clock);}
            let last=bus(clock.clock_master_cycles(),NmiHeaderAccess::Read(0x133));
            Self::header_data_cycle(clock,0x133);self.compare_a_byte(last);
            for work in [Fetch,Internal(8)] {work.advance(clock);}
            if echo!=last {Internal(6).advance(clock);(None,366+84)}
            else {
                self.header_store_absolute(clock,&mut bus,0x2140,0);
                for work in [Fetch,Internal(8),Internal(6)] {work.advance(clock);}
                (None,366+78+52)
            }
        } else {
            Internal(6).advance(clock);
            for work in [Fetch,Internal(16)] {work.advance(clock);}
            let last=bus(clock.clock_master_cycles(),NmiHeaderAccess::Read(0x133));
            Self::header_data_cycle(clock,0x133);self.compare_a_byte(last);
            for work in [Fetch,Internal(8)] {work.advance(clock);}
            if command==last {Internal(6).advance(clock);(None,372+54)}
            else {
                self.header_store_absolute(clock,&mut bus,0x2140,command);
                self.header_store_absolute(clock,&mut bus,0x133,command);
                for work in [Fetch,Internal(8)] {work.advance(clock);}
                self.compare_a_byte(0xf2);
                for work in [Fetch,Internal(8)] {work.advance(clock);}
                let cycles=if command>=0xf2 {Internal(6).advance(clock);372+48+100+32}
                    else {self.header_store_absolute(clock,&mut bus,0x130,command);372+48+94+32+32};
                self.header_store_absolute(clock,&mut bus,0x12c,0);
                (Some(command),cycles)
            }
        };
        self.pc=0x8103;(issued,cycles)
    }

    fn push8(&mut self, ram: &mut [u8], value: u8) {
        assert!(self.sp < 0x2000, "native stack escaped low WRAM bus window");
        ram[usize::from(self.sp)] = value;
        self.sp = self.sp.wrapping_sub(1);
    }
    fn push16(&mut self, ram: &mut [u8], value: u16) {
        assert!(
            (1..0x2000).contains(&self.sp),
            "native stack word escaped low WRAM"
        );
        // The direct-WRAM word transaction commits both bytes before drain.
        ram[usize::from(self.sp)] = (value >> 8) as u8;
        ram[usize::from(self.sp.wrapping_sub(1))] = value as u8;
        self.sp = self.sp.wrapping_sub(2);
    }
    fn pop8(&mut self, ram: &[u8]) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        assert!(self.sp < 0x2000, "native stack escaped low WRAM bus window");
        ram[usize::from(self.sp)]
    }
    fn pop16(&mut self, ram: &[u8]) -> u16 {
        let low = self.pop8(ram);
        u16::from_le_bytes([low, self.pop8(ram)])
    }

    pub(crate) fn load_a_byte(&mut self, value: u8) {
        self.a = (self.a & 0xff00) | u16::from(value);
        self.p =
            (self.p & !(ZERO | NEGATIVE)) | if value == 0 { ZERO } else { 0 } | (value & NEGATIVE);
    }

    #[cfg(test)]
    pub(crate) fn enter_handler(&mut self, ram: &mut [u8]) {
        self.enter_handler_with_bus(ram, |_, _| {});
    }

    /// Advance an already accepted native NMI through `$00:80D9`. The caller
    /// must own the incoming clock and prove DMA/HDMA inactive. This does not
    /// advance the audio-port block, updates, register tail, or return.
    pub(crate) fn enter_handler_clocked(&mut self, ram: &mut [u8], clock: &mut CpuMasterTimeline) {
        self.enter_handler_with_bus(ram, |transaction, _| transaction.advance(clock));
    }

    fn enter_handler_with_bus(&mut self, ram: &mut [u8], mut bus: impl FnMut(EntryBus, &[u8])) {
        self.accept_interrupt_with_bus(ram, &mut bus);
        self.enter_accepted_handler_with_bus(ram, bus);
    }

    pub(crate) fn accept_interrupt_clocked(&mut self, ram: &mut [u8], clock: &mut CpuMasterTimeline) {
        self.accept_interrupt_with_bus(ram, |transaction, _| transaction.advance(clock));
    }

    pub(crate) fn enter_accepted_handler_clocked(&mut self, ram: &mut [u8], clock: &mut CpuMasterTimeline) {
        self.enter_accepted_handler_with_bus(ram, |transaction, _| transaction.advance(clock));
    }

    fn accept_interrupt_with_bus(&mut self, ram: &mut [u8], mut bus: impl FnMut(EntryBus, &[u8])) {
        self.accept_interrupt_vector_with_bus(ram,0xffea,0x80c9,&mut bus);
    }

    pub(crate) fn accept_irq_clocked(&mut self, ram: &mut [u8], clock: &mut CpuMasterTimeline) {
        self.accept_interrupt_vector_with_bus(ram,0xffee,0x82d8,
            |transaction,_|transaction.advance(clock));
    }

    fn accept_interrupt_vector_with_bus(&mut self, ram: &mut [u8], vector: u16,
        target: u16, mut bus: impl FnMut(EntryBus, &[u8])) {
        // The interrupted caller is in a slow-ROM bank. No NMI opcode fetch
        // precedes the hardware's MemorySpeed + ONE_CYCLE transaction.
        assert!(
            self.bank < 0x80 && self.pc >= 0x8000,
            "native NMI entry requires its slow-ROM caller"
        );
        bus(EntryBus::Internal(8 + 6), ram);
        // Hardware entry saves the interrupted status before setting I and
        // clearing D. The vector selects bank zero.
        self.push8_with_bus(ram, self.bank, &mut bus);
        self.push16_with_bus(ram, self.pc, &mut bus);
        self.push8_with_bus(ram, self.p, &mut bus);
        self.p = (self.p | IRQ_MASK) & !DECIMAL;
        bus(
            EntryBus::Read {
                address: vector,
                value: target,
                width: 2,
            },
            ram,
        );
        self.bank = 0;
        self.pc = target;
    }

    fn enter_accepted_handler_with_bus(&mut self, ram: &mut [u8], mut bus: impl FnMut(EntryBus, &[u8])) {
        assert_eq!((self.bank, self.pc), (0, 0x80c9));
        // SEI; REP #$30. Direct opcode fetch does not drain a due event;
        // immediate operands and the following internal cycles do.
        bus(EntryBus::Fetch, ram);
        bus(EntryBus::Internal(6), ram);
        bus(EntryBus::Fetch, ram);
        bus(EntryBus::Internal(8), ram);
        self.p &= !(ACCUMULATOR_BYTE | INDEX_BYTE);
        bus(EntryBus::Internal(6), ram);
        // PHA; PHX; PHY; PHD; PHB.
        for value in [self.a, self.x, self.y, self.d] {
            bus(EntryBus::Fetch, ram);
            bus(EntryBus::Internal(6), ram);
            self.push16_with_bus(ram, value, &mut bus);
        }
        bus(EntryBus::Fetch, ram);
        bus(EntryBus::Internal(6), ram);
        self.push8_with_bus(ram, self.db, &mut bus);
        // LDA #0; TCD.
        bus(EntryBus::Fetch, ram);
        bus(EntryBus::Internal(16), ram);
        self.a = 0;
        self.p = (self.p & !NEGATIVE) | ZERO;
        bus(EntryBus::Fetch, ram);
        bus(EntryBus::Internal(6), ram);
        self.d = self.a;
        // PHK; PLB.
        bus(EntryBus::Fetch, ram);
        bus(EntryBus::Internal(6), ram);
        self.push8_with_bus(ram, self.bank, &mut bus);
        bus(EntryBus::Fetch, ram);
        bus(EntryBus::Internal(12), ram);
        self.db = self.pop8(ram);
        bus(
            EntryBus::Read {
                address: self.sp,
                value: u16::from(self.db),
                width: 1,
            },
            ram,
        );
        // SEP #$30.
        bus(EntryBus::Fetch, ram);
        bus(EntryBus::Internal(8), ram);
        self.p = (self.p & !NEGATIVE) | ZERO | ACCUMULATOR_BYTE | INDEX_BYTE;
        bus(EntryBus::Internal(6), ram);
        self.x &= 0xff;
        self.y &= 0xff;
        self.pc = 0x80d9;
    }

    fn push8_with_bus(&mut self, ram: &mut [u8], value: u8, bus: &mut impl FnMut(EntryBus, &[u8])) {
        let address = self.sp;
        self.push8(ram, value);
        bus(
            EntryBus::Write {
                address,
                value: u16::from(value),
                width: 1,
            },
            ram,
        );
    }

    fn push16_with_bus(
        &mut self,
        ram: &mut [u8],
        value: u16,
        bus: &mut impl FnMut(EntryBus, &[u8]),
    ) {
        let address = self.sp.wrapping_sub(1);
        self.push16(ram, value);
        bus(
            EntryBus::Write {
                address,
                value,
                width: 2,
            },
            ram,
        );
    }

    /// The caller supplies its native instruction's successor, not an
    /// observed stack word. JSR stores successor-1; RTS consumes live WRAM.
    pub(crate) fn call(&mut self, ram: &mut [u8], successor: u16) {
        self.push16(ram, successor.wrapping_sub(1));
    }
    pub(crate) fn return_from_call(&mut self, ram: &[u8]) {
        self.pc = self.pop16(ram).wrapping_add(1);
    }

    pub(crate) fn leave_handler(&mut self, ram: &[u8]) {
        // REP #$30; PLB; PLD; PLY; PLX; PLA; RTI. The restored status owns
        // width truncation, rather than the handler's temporary byte widths.
        self.db = self.pop8(ram);
        self.d = self.pop16(ram);
        self.y = self.pop16(ram);
        self.x = self.pop16(ram);
        self.a = self.pop16(ram);
        self.p = self.pop8(ram);
        self.pc = self.pop16(ram);
        self.bank = self.pop8(ram);
        if self.p & INDEX_BYTE != 0 {
            self.x &= 0xff;
            self.y &= 0xff;
        }
    }
}
