use crate::cycle_models::{
    intro_clear::Access,
    mirror_portal::MirrorPortal,
    native_work::compare_instruction_with_devices,
    nmi_stack::NativeNmiContext,
    pre_overworld_properties::{PreOverworldProperties, PropertiesChild},
};
use snes::CpuMasterTimeline;

#[test]
fn properties_mirror_portal_keeps_allocation_initialization_and_far_frames() {
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    for address in 0x0db080..0x0db818 {
        assert_eq!(
            crate::ZeldaState::native_sprite_property_table_byte(address),
            Some(rom[0x68000 + (address as usize & 0x7fff)])
        );
    }
    let mut instructions = 0;
    for case in 0..272usize {
        let mut machine = snes::Snes::new();
        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
        for (i, byte) in machine.ram.iter_mut().enumerate() {
            *byte = (i ^ (i >> 8) ^ case.wrapping_mul(37)) as u8;
        }
        let r = &mut machine.ram;
        r[0xdd0..0xde0].fill(8);
        r[0xe20..0xe30].fill(1);
        let free = case % 17;
        if free < 16 {
            r[0xdd0 + free] = 0;
        }
        if case / 17 % 2 != 0 {
            for slot in (0..16).step_by(3) {
                r[0xe20 + slot] = 0x6c;
            }
        }
        r[0x1b] = (case / 34 % 2) as u8;
        r[0x1abf] = case as u8;
        r[0x1acf] = (case >> 3) as u8;
        r[0x1adf] = [0, 247, 248, 255][case / 68 % 4];
        r[0x1aef] = (case / 17) as u8;
        r[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
        let mut cpu = NativeNmiContext {
            a: 0xb740,
            x: 2,
            y: 0x60,
            d: 0,
            db: [0, 2, 0x80, 0x82][case % 4],
            bank: 2,
            pc: 0x84e4,
            sp: 0x1fa,
            p: 0x30 | (case as u8 & 0xc3),
        };
        machine.cpu.a = cpu.a;
        machine.cpu.x = cpu.x;
        machine.cpu.y = cpu.y;
        machine.cpu.db = cpu.db;
        machine.cpu.k = cpu.bank;
        machine.cpu.pc = cpu.pc;
        machine.cpu.sp = cpu.sp;
        machine.cpu.e = false;
        machine.cpu.unpack_flags(cpu.p);
        let mut ram = machine.ram.clone();
        let clock = CpuMasterTimeline::at_raster(
            case as u64,
            [
                snes::CpuRasterPosition::new(100, 532),
                snes::CpuRasterPosition::new(224, 1338),
                snes::CpuRasterPosition::new(261, 1350),
            ][case % 3],
            snes::CpuBusWorkload::default(),
            snes::CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        let mut source =
            snes::RomCpuTimingProbe::new(machine, clock, snes::SourcePpuReadState::snes9x_reset())
                .unwrap();
        let mut clock = source.timeline().clone();
        let mut caller = PreOverworldProperties::at_instruction_for_test(0x84e4);
        let mut portal = MirrorPortal::default();
        for _ in 0..3000 {
            let child = caller.child();
            let label = format!(
                "mirror portal case={case} pc={:02x}:{:04x}",
                cpu.bank, cpu.pc
            );
            compare_instruction_with_devices(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                &rom,
                None,
                |_, access| match access {
                    Access::Read(a, width) if a & 0x8000 != 0 && a < 0x7e0000 => {
                        let read = |address| {
                            crate::ZeldaState::native_sprite_property_table_byte(address & 0x7fffff)
                                .unwrap_or_else(|| {
                                    panic!("unowned portal data {address:06x}; {label}")
                                })
                        };
                        Some(if width == 1 {
                            u16::from(read(a))
                        } else {
                            u16::from_le_bytes([read(a), read(a + 1)])
                        })
                    }
                    _ => None,
                },
                |cpu, clock, bus| match child {
                    None => {
                        caller.step(cpu, clock, bus).unwrap();
                    }
                    Some(PropertiesChild::MirrorPortal) => {
                        portal.step(cpu, clock, bus).unwrap();
                    }
                    _ => panic!("unexpected portal child {child:?}"),
                },
            );
            assert_eq!(ram, source.snes().ram, "{label}");
            instructions += 1;
            if portal.complete() {
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x84e8, 0x1fa));
                caller.return_from_child(&cpu);
                break;
            }
        }
        assert!(portal.complete());
    }
    eprintln!(
        "native properties mirror-portal cases=272 Source instruction boundaries={instructions}"
    );
}
