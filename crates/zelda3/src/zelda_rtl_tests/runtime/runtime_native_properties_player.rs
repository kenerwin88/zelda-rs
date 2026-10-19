use crate::cycle_models::{
    intro_clear::Access,
    load_file_player::{self, LoadFilePlayer},
    native_work::compare_instruction_with_devices,
    nmi_stack::NativeNmiContext,
    overworld_fixed_scroll::OverworldFixedScroll,
    pre_overworld_properties::{PreOverworldProperties, PropertiesChild},
};
use snes::CpuMasterTimeline;

fn compare_case(case: usize, reset: bool, rom: &[u8]) -> usize {
    let mut machine = snes::Snes::new();
    machine.cart.load(snes::CartType::LoRom, rom, 0x2000);
    for (i, byte) in machine.ram.iter_mut().enumerate() {
        *byte = (i ^ (i >> 8) ^ case.wrapping_mul(37)) as u8;
    }
    let r = &mut machine.ram;
    r[0x8a..0x8c].copy_from_slice(&((case % 256) as u16).to_le_bytes());
    r[0xa0..0xa2].copy_from_slice(&[0x181u16, 0x182, 0x80, 0][case / 256 % 4].to_le_bytes());
    r[0x11] = if case / 256 % 2 == 0 { 0 } else { 4 };
    r[0xee] = (case % 4) as u8;
    r[0x410..0x412].copy_from_slice(&[8u16, 0, 4, 255][case / 4 % 4].to_le_bytes());
    r[0xe2..0xe4].copy_from_slice(
        &[0u16, 0x777, 0x778, 0x779, 0x7fff, 0x8000, 0xffff, 0x1800][case / 16 % 8].to_le_bytes(),
    );
    r[0xe8..0xea].copy_from_slice(
        &[0u16, 0x17f, 0x180, 0x3ff, 0x600, 0x6bf, 0x6c0, 0xffff][case / 8 % 8].to_le_bytes(),
    );
    r[0xc4a..0xc54].fill(0);
    r[0xc4a] = [0, 0x2c, 0x3e, 1][case % 4];
    r[0x1fb..0x1fd].copy_from_slice(&[0xc5, 0x83]);
    let pc = if reset { 0x8512 } else { 0x84bb };
    let mut cpu = NativeNmiContext {
        a: 0xb701,
        x: 2,
        y: 0x60,
        d: 0,
        db: [0, 2, 0x80, 0x82][case % 4],
        bank: 2,
        pc,
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
    let mut caller = PreOverworldProperties::at_instruction_for_test(pc);
    let mut fixed = OverworldFixedScroll::default();
    let mut player = LoadFilePlayer::from_entry(if reset { 0x028b0c } else { 0x099efc });
    let mut instructions = 0;
    for _ in 0..3000 {
        let child = caller.child();
        let label = format!(
            "properties player case={case} reset={reset} pc={:02x}:{:04x}",
            cpu.bank, cpu.pc
        );
        compare_instruction_with_devices(
            &label,
            &mut cpu,
            &mut clock,
            &mut source,
            &mut ram,
            rom,
            None,
            |_, access| match access {
                Access::Read(a, width) if a & 0x8000 != 0 && a < 0x7e0000 => {
                    let read = |address| {
                        load_file_player::data_byte(address & 0x7fffff)
                            .unwrap_or_else(|| panic!("unowned player data {address:06x}; {label}"))
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
                Some(PropertiesChild::FixedColorAndScroll) => {
                    fixed.step(cpu, clock, bus).unwrap();
                }
                Some(PropertiesChild::Follower | PropertiesChild::ResetPlayer) => {
                    player.step(cpu, clock, bus).unwrap();
                }
                _ => panic!("unexpected properties player child {child:?}"),
            },
        );
        assert_eq!(ram, source.snes().ram, "{label}");
        instructions += 1;
        if child == Some(PropertiesChild::FixedColorAndScroll) && fixed.complete() {
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x84bf, 0x1fa));
            caller.return_from_child(&cpu);
        }
        if player.complete() {
            assert_eq!(
                (cpu.bank, cpu.pc, cpu.sp),
                (2, if reset { 0x8515 } else { 0x84c9 }, 0x1fa)
            );
            caller.return_from_child(&cpu);
            break;
        }
    }
    assert!(player.complete());
    assert!(reset || fixed.complete());
    instructions
}

#[test]
fn properties_fixed_scroll_and_follower_preserve_signed_parallax_and_return() {
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    let mut instructions = 0;
    for case in 0..2048 {
        instructions += compare_case(case, false, &rom);
    }
    eprintln!("native properties fixed-scroll/follower cases=2048 Source instruction boundaries={instructions}");
}

#[test]
fn properties_player_reset_consumes_near_frame_and_nested_cleanup() {
    let rom = super::runtime_native_overworld_dungeon_exit::rom();
    let mut instructions = 0;
    for case in 0..64 {
        instructions += compare_case(case, true, &rom);
    }
    eprintln!(
        "native properties player-reset cases=64 Source instruction boundaries={instructions}"
    );
}
