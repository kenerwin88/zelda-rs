use super::*;

#[test]
fn ancilla_axis_movement_matches_source_carry_sign_and_indexed_page_costs() {
    let rom = item_receipt_test_rom().expect("movement source fixture needs the test ROM");
    for slot in 0..10 {
        for velocity in [0, 1, 7, 15, 16, 127, 128, 240, 255] {
            for subpixel in [0, 1, 240, 255] {
                for position in [0u16, 0x00ff, 0x0fff, 0xffff] {
                    for x_axis in [false, true] {
                        let mut state = ZeldaState::new();
                        let index = slot + if x_axis { 10 } else { 0 };
                        state.ram[0xbfa + index] = position as u8;
                        state.ram[0xc0e + index] = (position >> 8) as u8;
                        state.ram[0xc22 + index] = velocity;
                        state.ram[0xc36 + index] = subpixel;
                        state.ram[0xfa0] = slot as u8;
                        state.sync_native_game_state_from_ram();
                        let (cycles, expected) = run_item_receipt_source(&rom, &state.ram,
                            if x_axis { 0x089080 } else { 0x08908b }, 0x088036, slot);
                        let before = crate::cycle_ledger::master();
                        if x_axis { state.ancilla_move_x(slot); } else { state.ancilla_move_y(slot); }
                        assert_eq!(crate::cycle_ledger::master() - before, cycles,
                            "slot={slot} x_axis={x_axis} velocity={velocity} subpixel={subpixel}");
                        assert_eq!(&state.ram[0xbfa..0xc4a], &expected[0xbfa..0xc4a],
                            "slot={slot} x_axis={x_axis} velocity={velocity} subpixel={subpixel}");
                    }
                }
            }
        }
    }
}

#[test]
fn snore_matches_source_timer_velocity_deletion_and_oam_branches() {
    let rom = item_receipt_test_rom().expect("snore source fixture needs the test ROM");
    for slot in 0..10 {
        for variant in 0usize..192 {
            let mut state = ZeldaState::new();
            let x = [0u16, 255, 256, 0xffff][variant & 3];
            let y = [0u16, 239, 240, 255, 256, 0xffff][(variant / 4) % 6];
            state.ram[0xbfa + slot] = y as u8;
            state.ram[0xc0e + slot] = (y >> 8) as u8;
            state.ram[0xc04 + slot] = x as u8;
            state.ram[0xc18 + slot] = (x >> 8) as u8;
            state.ram[0xc22 + slot] = [0, 15, 128, 255][(variant / 6) & 3];
            state.ram[0xc2c + slot] = [0, 7, 8, 128, 240, 255][(variant / 3) % 6];
            state.ram[0xc36 + slot] = if variant & 4 != 0 { 255 } else { 0 };
            state.ram[0xc40 + slot] = if variant & 8 != 0 { 255 } else { 0 };
            state.ram[0xc54 + slot] = if variant & 16 != 0 { 255 } else { 1 };
            state.ram[0x3b1 + slot] = [0, 1, 128][variant % 3];
            state.ram[0xc5e + slot] = ((variant / 24) % 3) as u8;
            state.ram[0xc4a + slot] = 0x21;
            state.ram[0x20..0x22].copy_from_slice(&[0, 1]);
            state.ram[0x90..0x92].copy_from_slice(&0x820u16.to_le_bytes());
            state.ram[0x92..0x94].copy_from_slice(&0xa28u16.to_le_bytes());
            state.ram[0xfa0] = slot as u8;
            state.ram[0x820..0x828].fill(0x5a);
            state.sync_native_game_state_from_ram();
            let (cycles, expected) = run_item_receipt_source(&rom, &state.ram,
                0x08c094, 0x088036, slot);
            let before = crate::cycle_ledger::master();
            state.ancilla21_snore(slot);
            assert_eq!(crate::cycle_ledger::master() - before, cycles,
                "slot={slot} variant={variant}");
            for range in [0x3b1..0x3bb, 0xbfa..0xc68, 0x820..0x824, 0xa28..0xa29, 0x109..0x10a] {
                assert_eq!(&state.ram[range.clone()], &expected[range],
                    "slot={slot} variant={variant}");
            }
        }
    }
}

fn item_receipt_test_rom() -> Option<Vec<u8>> {
    let path = std::env::var_os("ZELDA3_ROM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let mut rom = std::fs::read(path).ok()?;
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    Some(rom)
}

fn run_item_receipt_source(rom: &[u8], ram: &[u8], entry: u32, stop: u32, slot: usize) -> (u64, Vec<u8>) {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let checkpoint = RomCpuCheckpoint {
        entry_pc: entry, stop_pc: stop,
        a: 0, x: slot as u16, y: 0, sp: 0x01fc, dp: 0, db: 8,
        carry: true, zero: false, overflow: true, negative: false,
        interrupt_disable: false, decimal: false,
        accumulator_is_8_bit: true, index_is_8_bit: true,
        emulation: false, waiting: false,
        stack_address: 0x01fd, stack_bytes: &[0x35, 0x80],
    };
    let mut run = RomCpuTimingRun::new(rom, ram, &[0; 0x2000],
        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint).unwrap();
    run.enable_cpu_write_trace();
    let mut expected = ram.to_vec();
    let mut cycles = 0;
    let mut steps = 0;
    while !run.is_complete() {
        cycles += u64::from(run.step().master_cycles);
        for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
        steps += 1;
        assert!(steps < 2000, "item receipt did not reach {stop:06x}");
    }
    (cycles, expected)
}

#[test]
fn item_receipt_drawing_matches_rom_for_all_items_and_clipping_branches() {
    let Some(rom) = item_receipt_test_rom() else { return };
    for item in 0..76 {
        for x in [0u16, 255, 256, 0xffff] {
            for y in [0u16, 231, 232, 239, 240, 255, 256, 0xffff] {
                let mut state = ZeldaState::new();
                let slot = 4;
                state.ram[0..2].copy_from_slice(&y.to_le_bytes());
                state.ram[2..4].copy_from_slice(&x.to_le_bytes());
                state.ram[8..10].copy_from_slice(&y.wrapping_add(8).to_le_bytes());
                state.ram[0x90..0x92].copy_from_slice(&0x820u16.to_le_bytes());
                state.ram[0x92..0x94].copy_from_slice(&0xa28u16.to_le_bytes());
                state.ram[0xc5e + slot] = item;
                state.ram[0xbf0 + slot] = 5;
                state.ram[0x820..0x828].fill(0x5a);
                state.sync_native_game_state_from_ram();
                let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c690, 0x08_8036, slot);
                let before = crate::cycle_ledger::master();
                state.ancilla_receive_item_draw(slot, x, y);
                assert_eq!(crate::cycle_ledger::master() - before, cycles, "item={item} x={x} y={y}");
                assert_eq!(&state.ram[0x820..0x828], &expected[0x820..0x828], "item={item} x={x} y={y}");
                assert_eq!(&state.ram[0xa28..0xa2a], &expected[0xa28..0xa2a]);
            }
        }
    }
}

#[test]
fn item_receipt_message_selection_work_matches_rom() {
    let Some(rom) = item_receipt_test_rom() else { return };
    for item in 0..76 {
        for step in [1, 2] {
            for (indoors, room) in [(0, 0), (1, 0x20), (1, 0xff), (1, 0x10f), (1, 0x110), (1, 0x112), (1, 0x11f)] {
                for flags in [0, 7] {
                    for pieces in 0..4 {
                        let mut state = ZeldaState::new();
                        let slot = 4;
                        state.ram[0x10] = 7;
                        state.ram[0x1b] = indoors;
                        state.ram[0xa0..0xa2].copy_from_slice(&(room as u16).to_le_bytes());
                        state.ram[0xc5e + slot] = item;
                        state.ram[0xc54 + slot] = step;
                        state.ram[0xf374] = flags;
                        state.ram[0xf36b] = pieces;
                        state.sync_native_game_state_from_ram();
                        let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c546, 0x08_c5f7, slot);
                        let before = crate::cycle_ledger::master();
                        state.ancilla22_item_receipt_show_message(slot);
                        assert_eq!(crate::cycle_ledger::master() - before, cycles,
                            "item={item} step={step} indoors={indoors} room={room:x} flags={flags} pieces={pieces}");
                        for address in [0x10, 0x11, 0x12, 0x012d, 0x1cf0, 0x1cf1] {
                            assert_eq!(state.ram[address], expected[address], "item={item} address={address:x}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn item_receipt_countdown_through_message_matches_rom() {
    let Some(rom) = item_receipt_test_rom() else { return };
    for slot in 0..5 {
        for aux in 1..=24 {
            for submodule in [0, 2, 9, 11, 43] {
                let mut state = ZeldaState::new();
                state.ram[0x10] = 7;
                state.ram[0x11] = submodule;
                state.ram[0x1b] = 1;
                state.ram[0xa0] = 0x20;
                state.ram[0xc5e + slot] = 0x3f;
                state.ram[0xc54 + slot] = 1;
                state.ram[0x3b1 + slot] = aux;
                state.ram[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
                state.ram[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
                state.sync_native_game_state_from_ram();
                let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c38a, 0x08_8036, slot);
                let before = crate::cycle_ledger::master();
                state.ancilla22_item_receipt(slot);
                assert_eq!(crate::cycle_ledger::master() - before, cycles,
                    "slot={slot} aux={aux} submodule={submodule}");
                for range in [0x10..0x14, 0x3b1..0x3b6, 0xc68..0xc6d, 0x800..0x808, 0xa20..0xa22, 0x1cf0..0x1cf2] {
                    assert_eq!(&state.ram[range.clone()], &expected[range], "slot={slot} aux={aux} submodule={submodule}");
                }
            }
        }
    }
}

#[test]
fn item_receipt_message_entry_and_already_open_return_match_rom() {
    let Some(rom) = item_receipt_test_rom() else { return };
    for module in 0..=27 {
        let mut state = ZeldaState::new();
        state.ram[0x10] = module;
        state.ram[0x11] = 9;
        state.ram[0x223] = 0x5a;
        state.ram[0x1cd8] = 3;
        state.ram[0xc5e + 4] = 0x3f;
        state.ram[0xc54 + 4] = 1;
        state.sync_native_game_state_from_ram();
        let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c546, 0x08_c5f7, 4);
        let before = crate::cycle_ledger::master();
        state.ancilla22_item_receipt_show_message(4);
        assert_eq!(crate::cycle_ledger::master() - before, cycles, "module={module}");
        for address in [0x10, 0x11, 0x10c, 0x223, 0x1cd8] {
            assert_eq!(state.ram[address], expected[address], "module={module} address={address:x}");
        }
    }
}

#[test]
fn item_receipt_animation_and_initial_countdown_work_match_rom() {
    let Some(rom) = item_receipt_test_rom() else { return };
    // Crystal transmutation and milestone graphics reload have separately
    // owned callees; this matrix covers drawing and animation without them.
    for item in (0..76).filter(|&item| item != 0x20) {
        for step in [0, 2, 3] {
            for timer in [15, 17] {
                for frame_timer in [0, 1] {
                    if (0x34..=0x36).contains(&item) && frame_timer == 0 { continue; }
                    for frame in [0, 2] {
                        let mut state = ZeldaState::new();
                        let slot = 4;
                        state.ram[0xc5e + slot] = item;
                        state.ram[0xc54 + slot] = step;
                        state.ram[0xc68 + slot] = timer;
                        state.ram[0x39f + slot] = frame_timer;
                        state.ram[0x3a4 + slot] = frame;
                        state.ram[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
                        state.ram[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
                        state.sync_native_game_state_from_ram();
                        let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c5f7, 0x08_8036, slot);
                        let before = crate::cycle_ledger::master();
                        state.ancilla22_item_receipt_draw_and_update(slot);
                        assert_eq!(crate::cycle_ledger::master() - before, cycles,
                            "item={item} step={step} timer={timer} frame_timer={frame_timer} frame={frame}");
                        for range in [0x39f..0x3a9, 0xbf0..0xbf5, 0x800..0x808, 0xa20..0xa22] {
                            assert_eq!(&state.ram[range.clone()], &expected[range]);
                        }
                    }
                }
            }
        }
    }
    for item in [1, 0x37, 0x38, 0x39, 0x3f] {
        for step in [0, 3] {
            for aux in [2, 3, 25] {
                for timer in [1, 16, 17, 18] {
                    let mut state = ZeldaState::new();
                    let slot = 4;
                    state.ram[0x10] = 7;
                    state.ram[0xc5e + slot] = item;
                    state.ram[0xc54 + slot] = step;
                    state.ram[0xc68 + slot] = timer;
                    state.ram[0x3b1 + slot] = aux;
                    state.ram[0x90..0x92].copy_from_slice(&0x800u16.to_le_bytes());
                    state.ram[0x92..0x94].copy_from_slice(&0xa20u16.to_le_bytes());
                    state.sync_native_game_state_from_ram();
                    let (cycles, expected) = run_item_receipt_source(&rom, &state.ram, 0x08_c38a, 0x08_8036, slot);
                    let before = crate::cycle_ledger::master();
                    state.ancilla22_item_receipt(slot);
                    assert_eq!(crate::cycle_ledger::master() - before, cycles,
                        "item={item} step={step} aux={aux} timer={timer}");
                    for range in [0x10..0x14, 0x2cd..0x2cf, 0x3b1..0x3b6, 0x800..0x808, 0x1cf0..0x1cf2, 0xf3cc..0xf3cd] {
                        assert_eq!(&state.ram[range.clone()], &expected[range]);
                    }
                }
            }
        }
    }
}

#[test]
fn adjusted_oam_coordinate_work_matches_rom_including_slot_page_crossing() {
    use crate::rom_cpu_timing::{RomCpuCheckpoint, RomCpuTimingRun};
    let path = std::env::var_os("ZELDA3_ROM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc"));
    let Ok(mut rom) = std::fs::read(path) else { return };
    if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
    for slot in 0..10 {
        for floor in 0..4 {
            for x in [0u16, 1, 255, 256, 0x7fff, 0x8000, 0xffff] {
                for y in [0u16, 1, 255, 256, 0x7fff, 0x8000, 0xffff] {
                    let mut state = ZeldaState::new();
                    state.ram[0xc04 + slot] = x as u8;
                    state.ram[0xc18 + slot] = (x >> 8) as u8;
                    state.ram[0xbfa + slot] = y as u8;
                    state.ram[0xc0e + slot] = (y >> 8) as u8;
                    state.ram[0xc7c + slot] = floor;
                    state.ram[0x11e..0x120].copy_from_slice(&0x1ffu16.to_le_bytes());
                    state.ram[0x122..0x124].copy_from_slice(&0xfffeu16.to_le_bytes());
                    state.sync_native_game_state_from_ram();
                    let checkpoint = RomCpuCheckpoint {
                        entry_pc: 0x08_f6a4, stop_pc: 0x08_8036,
                        a: 0, x: slot as u16, y: 0, sp: 0x01fc, dp: 0, db: 8,
                        carry: true, zero: false, overflow: true, negative: false,
                        interrupt_disable: false, decimal: false,
                        accumulator_is_8_bit: true, index_is_8_bit: true,
                        emulation: false, waiting: false,
                        stack_address: 0x01fd, stack_bytes: &[0x35, 0x80],
                    };
                    let mut run = RomCpuTimingRun::new(&rom, &state.ram, &[0; 0x2000],
                        &snes::PpuState::default(), &snes::DmaState::default(), [0; 4], checkpoint)
                        .unwrap();
                    run.enable_cpu_write_trace();
                    let mut expected = state.ram.clone();
                    let mut cycles = 0u64;
                    let mut steps = 0;
                    while !run.is_complete() {
                        cycles += u64::from(run.step().master_cycles);
                        for (address, value) in run.take_cpu_wram_writes() { expected[address] = value; }
                        steps += 1;
                        assert!(steps < 100, "adjusted OAM preparation did not return");
                    }
                    let before = crate::cycle_ledger::master();
                    let actual = state.ancilla_prep_adjusted_oam_coord(slot);
                    assert_eq!(crate::cycle_ledger::master() - before, cycles, "slot={slot} floor={floor}");
                    assert_eq!(actual, (u16::from_le_bytes([expected[2], expected[3]]),
                        u16::from_le_bytes([expected[0], expected[1]])), "slot={slot} floor={floor}");
                    assert_eq!(&state.ram[0x64..0x66], &expected[0x64..0x66]);
                }
            }
        }
    }
}

#[test]
fn dash_dust_motive_expires_out_of_range_frame() {
    let mut state = ZeldaState::new();
    state.ancilla_slot_view_mut(0).set_ancilla_type(0x1e);
    state.ancilla_slot_view_mut(0).set_timer(1);
    state.ancilla_slot_view_mut(0).set_item_to_link(3);

    state.dash_dust_motive(0);

    assert_eq!(state.ancilla_slot_view(0).ancilla_type(), 0);
}

#[test]
fn sword_wall_hit_does_not_write_terminal_animation_counter() {
    let mut state = ZeldaState::new();
    state.ancilla_slot_view_mut(4).set_ancilla_type(0x1b);
    state.ancilla_slot_view_mut(4).set_item_to_link(7);
    state.ancilla_slot_view_mut(4).set_aux_timer(0);

    state.ancilla_sword_wall_hit(4);

    let slot = state.ancilla_slot_view(4);
    assert_eq!(slot.ancilla_type(), 0);
    assert_eq!(slot.item_to_link(), 7);
    assert_eq!(slot.aux_timer(), 0xff);
}

#[test]
fn native_receipt_prefix_handoff_preserves_selected_body_once_and_omits_owned_dispatch() {
    use crate::cycle_models::{receipt_prefix::{ReceiptPrefix,ReceiptTail},intro_clear::Access,nmi_stack::NativeNmiContext};
    let mut tails=std::collections::BTreeSet::new();
    for case in 0..384usize {
        let k=case%10;
        let mut expected=ZeldaState::new();
        expected.ram[0x10]=7;expected.ram[0x1b]=1;expected.ram[0x307]=1;
        expected.ram[0x2e4]=1;
        expected.ram[0xc4a+k]=0x22;
        expected.ram[0xc04+k]=128;expected.ram[0xbfa+k]=96;
        expected.ram[0xc54+k]=1;
        expected.ram[0xc5e+k]=0x20;
        expected.ram[0x3b1+k]=56;
        match case/48 {
            0=>expected.ram[0x2e4]=2,
            1=>{},
            2=>expected.ram[0x3b1+k]=1,
            3=>{expected.ram[0xc54+k]=0;expected.ram[0x3b1+k]=1;},
            4=>expected.ram[0x3b1+k]=0,
            5=>{expected.ram[0xc54+k]=3;expected.ram[0x3b1+k]=2;expected.ram[0xc5e+k]=0x37;},
            6=>expected.ram[0x3b1+k]=41,
            _=>{expected.ram[0xc54+k]=0;expected.ram[0xc5e+k]=1;expected.ram[0xc68+k]=17;},
        }
        // Resource and message inputs remain valid while every helper selection is covered.
        expected.ram[0x40a..0x40c].copy_from_slice(&0x20u16.to_le_bytes());
        expected.ram[0xf36c]=24;expected.ram[0xf36d]=24;
        expected.sync_native_game_state_from_ram();expected.oam_reset_region_bases();
        let mut actual=expected.clone();actual.rom.clear();
        let mut cpu=NativeNmiContext {a:0xab00,x:k as u16,y:7,d:0,db:8,bank:8,pc:0xc38a,sp:0x1e8,p:0x30};
        let mut clock=snes::CpuMasterTimeline::at_raster(4586,snes::CpuRasterPosition::new(58,1334),snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        clock.begin_synchronous_timeline().unwrap();
        let mut work=ReceiptPrefix::default();let mut owned=0;
        while work.tail().is_none() {
            owned+=work.step(&mut cpu,&mut clock,|_,access| match access {
                Access::Read(a,width)=>{let a=(a & if matches!(a>>16,0x7e|0x7f){0x1ffff}else{0xffff}) as usize;u16::from(actual.ram[a])|if width==2 {u16::from(actual.ram[a+1])<<8}else{0}},
                Access::Write(a,v,width)=>{let a=(a & if matches!(a>>16,0x7e|0x7f){0x1ffff}else{0xffff}) as usize;actual.ram[a..a+width as usize].copy_from_slice(&v.to_le_bytes()[..width as usize]);actual.native_push_block_store(a,v,width);0},
                Access::Vblank {..}=>0,
            }).unwrap();
            work=work.clone();
        }
        let tail=work.tail().unwrap();tails.insert(tail as usize);
        let before=crate::cycle_ledger::master();expected.ancilla22_item_receipt(k);let full=crate::cycle_ledger::master()-before;
        let before=crate::cycle_ledger::master();actual.finish_native_receipt_prefix(k,tail);let remaining=crate::cycle_ledger::master()-before;
        let excluded=|i:usize| i<16 || (0x1c0..=0x1ff).contains(&i);
        let diffs=actual.ram.iter().zip(&expected.ram).enumerate().filter_map(|(i,(a,b))|(!excluded(i)&&a!=b).then_some((i,*a,*b))).take(16).collect::<Vec<_>>();
        assert!(diffs.is_empty(),"case={case} tail={tail:?} RAM {diffs:x?}");
        let mut a=vec![0xa5;snes::WRAM_SIZE];let mut b=a.clone();actual.game_state.write_to_ram(&mut a);expected.game_state.write_to_ram(&mut b);
        let diffs=a.iter().zip(&b).enumerate().filter_map(|(i,(a,b))|(!excluded(i)&&a!=b).then_some((i,*a,*b))).take(16).collect::<Vec<_>>();
        assert!(diffs.is_empty(),"case={case} tail={tail:?} native {diffs:x?}");
        assert_eq!(full-remaining,owned,"case={case} tail={tail:?}: no dispatch replay");
        assert!(actual.rom.is_empty());
    }
    assert_eq!(tails.len(),7);
    assert!(tails.contains(&(ReceiptTail::SongPoll as usize)));
    println!("native receipt handoff: 384 fixtures, all seven selected bodies match, owned dispatch omitted");
}

#[test]
fn native_receipt_movement_handoff_preserves_drawing_once_and_omits_owned_dispatch_and_motion() {
    use crate::cycle_models::{receipt_prefix::{ReceiptPrefix,ReceiptTail},intro_clear::Access,nmi_stack::NativeNmiContext};
    let mut tails=std::collections::BTreeSet::new();
    for case in 0..384usize {
        let k=case%10;
        let mut expected=ZeldaState::new();
        expected.ram[0x10]=7;expected.ram[0x1b]=1;expected.ram[0x307]=1;
        expected.ram[0x2e4]=1;
        expected.ram[0xc4a+k]=0x22;
        expected.ram[0xc04+k]=128;expected.ram[0xbfa+k]=96;
        expected.ram[0xc54+k]=1;
        expected.ram[0xc5e+k]=0x20;
        expected.ram[0x3b1+k]=56;
        match case/48 {
            0=>expected.ram[0x2e4]=2,
            1=>{},
            2=>expected.ram[0x3b1+k]=1,
            3=>{expected.ram[0xc54+k]=0;expected.ram[0x3b1+k]=1;},
            4=>expected.ram[0x3b1+k]=0,
            5=>{expected.ram[0xc54+k]=3;expected.ram[0x3b1+k]=2;expected.ram[0xc5e+k]=0x37;},
            6=>expected.ram[0x3b1+k]=41,
            _=>{expected.ram[0xc54+k]=0;expected.ram[0xc5e+k]=1;expected.ram[0xc68+k]=17;},
        }
        // Resource and message inputs remain valid while every helper selection is covered.
        expected.ram[0x40a..0x40c].copy_from_slice(&0x20u16.to_le_bytes());
        expected.ram[0xf36c]=24;expected.ram[0xf36d]=24;
        expected.sync_native_game_state_from_ram();expected.oam_reset_region_bases();
        let mut actual=expected.clone();actual.rom.clear();
        let mut cpu=NativeNmiContext {a:0xab00,x:k as u16,y:7,d:0,db:8,bank:8,pc:0xc38a,sp:0x1e8,p:0x30};
        let mut clock=snes::CpuMasterTimeline::at_raster(4586,snes::CpuRasterPosition::new(58,1334),snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        clock.begin_synchronous_timeline().unwrap();
        let mut work=ReceiptPrefix::default().with_native_movement(true);let mut owned=0;
        while work.tail().is_none() {
            owned+=work.step(&mut cpu,&mut clock,|_,access| match access {
                Access::Read(a,width)=>{let a=(a & if matches!(a>>16,0x7e|0x7f){0x1ffff}else{0xffff}) as usize;u16::from(actual.ram[a])|if width==2 {u16::from(actual.ram[a+1])<<8}else{0}},
                Access::Write(a,v,width)=>{let a=(a & if matches!(a>>16,0x7e|0x7f){0x1ffff}else{0xffff}) as usize;actual.ram[a..a+width as usize].copy_from_slice(&v.to_le_bytes()[..width as usize]);actual.native_push_block_store(a,v,width);0},
                Access::Vblank {..}=>0,
            }).unwrap();
            work=work.clone();
        }
        let tail=work.tail().unwrap();tails.insert(tail as usize);
        let before=crate::cycle_ledger::master();expected.ancilla22_item_receipt(k);let full=crate::cycle_ledger::master()-before;
        let before=crate::cycle_ledger::master();actual.finish_native_receipt_prefix(k,tail);let remaining=crate::cycle_ledger::master()-before;
        let excluded=|i:usize| i<16 || (0x1c0..=0x1ff).contains(&i);
        let diffs=actual.ram.iter().zip(&expected.ram).enumerate().filter_map(|(i,(a,b))|(!excluded(i)&&a!=b).then_some((i,*a,*b))).take(16).collect::<Vec<_>>();
        assert!(diffs.is_empty(),"case={case} tail={tail:?} RAM {diffs:x?}");
        let mut a=vec![0xa5;snes::WRAM_SIZE];let mut b=a.clone();actual.game_state.write_to_ram(&mut a);expected.game_state.write_to_ram(&mut b);
        let diffs=a.iter().zip(&b).enumerate().filter_map(|(i,(a,b))|(!excluded(i)&&a!=b).then_some((i,*a,*b))).take(16).collect::<Vec<_>>();
        assert!(diffs.is_empty(),"case={case} tail={tail:?} native {diffs:x?}");
        assert_eq!(full-remaining,owned,"case={case} tail={tail:?}: no dispatch replay");
        assert!(actual.rom.is_empty());
    }
    assert_eq!(tails.len(),6);
    assert!(tails.contains(&(ReceiptTail::SongPoll as usize)));
    println!("native receipt movement handoff: 384 fixtures, all six selected bodies match, owned dispatch omitted");
}

#[test]
fn native_ancilla_axis_matches_semantic_motion_and_complete_near_return_cycles() {
    use crate::cycle_models::{ancilla_axis::AncillaAxis,intro_clear::Access,nmi_stack::NativeNmiContext};
    for case in 0..2048usize {
        let fixture=(case/2)*41;let k=fixture%10;let x_axis=case%2==0;
        let mut expected=ZeldaState::new();
        crate::cycle_models::ancilla_axis::seed_case(&mut expected.ram,fixture);
        expected.ram[0x1e9..0x1eb].copy_from_slice(&0x8035u16.to_le_bytes());
        expected.sync_native_game_state_from_ram();let mut actual=expected.clone();actual.rom.clear();
        let mut cpu=NativeNmiContext {a:0xab00,x:k as u16,y:7,d:0,db:8,bank:8,pc:if x_axis {0x9080}else{0x908b},sp:0x1e8,p:0x30};
        let mut clock=snes::CpuMasterTimeline::at_raster(4586,snes::CpuRasterPosition::new(59,724),snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);
        clock.begin_synchronous_timeline().unwrap();let mut work=AncillaAxis::new(0x8036);let mut owned=0;
        while !work.complete() {
            owned+=work.step(&mut cpu,&mut clock,|_,access| match access {
                Access::Read(a,width)=>{let a=(a & 0xffff) as usize;u16::from(actual.ram[a])|if width==2 {u16::from(actual.ram[a+1])<<8}else{0}},
                Access::Write(a,v,width)=>{let a=(a & 0xffff) as usize;actual.ram[a..a+width as usize].copy_from_slice(&v.to_le_bytes()[..width as usize]);actual.native_push_block_store(a,v,width);0},
                Access::Vblank {..}=>0,
            }).unwrap();work=work.clone();
        }
        let before=crate::cycle_ledger::master();if x_axis {expected.ancilla_move_x(k)}else{expected.ancilla_move_y(k)};let full=crate::cycle_ledger::master()-before;
        assert_eq!(owned,full,"case={case} x_axis={x_axis}");
        assert_eq!(&actual.ram[0xbfa..0xc4a],&expected.ram[0xbfa..0xc4a],"case={case}");
        let mut a=actual.ram.clone();let mut b=expected.ram.clone();actual.game_state.sprites.ancilla_slots.write_to_ram(&mut a);expected.game_state.sprites.ancilla_slots.write_to_ram(&mut b);
        assert_eq!(&a[0xbfa..0xc4a],&b[0xbfa..0xc4a],"case={case} native");
        assert_eq!((cpu.pc,cpu.sp,cpu.x),(0x8036,0x1ea,k as u16));assert!(actual.rom.is_empty());
    }
    println!("native ancilla axis semantics: 2048 XY fixtures, native coordinates and all near-return cycles match");
}

#[test]
fn native_receipt_drawing_handoff_preserves_sparkle_graphics_and_oam_once() {
    use crate::cycle_models::{receipt_drawing::{ReceiptDrawing,ReceiptDrawingTail},intro_clear::Access,nmi_stack::NativeNmiContext};
    let mut tails=[0;3];let mut graphics=std::collections::BTreeSet::new();
    for case in 0..512usize {
        let k=case%10;let mut expected=ZeldaState::new();
        crate::cycle_models::receipt_drawing::seed_case(&mut expected.ram,case*277%145920);
        expected.ram[0x10]=7;expected.ram[0x1b]=1;expected.ram[0x307]=1;
        expected.ram[0xc4a+k]=0x22;expected.ram[0xc54+k]=(case%4) as u8;expected.ram[0x14]=(case%8) as u8;
        expected.ram[0xc04+k]=128;expected.ram[0xc18+k]=0;expected.ram[0xbfa+k]=96;expected.ram[0xc0e+k]=0;
        expected.ram[0xc7c+k]=(case%4) as u8;
        expected.ram[0xc68+k]=[0,15,16,255][case%4];expected.ram[0x3a4+k]=(case%3) as u8;expected.ram[0x39f+k]=0;
        expected.ram[0xc5e+k]=match case/64 {0=>[0,1,2,3,4,0x17,0x37,0x45][case%8],1=>0x20,2=>1,_=>[0x34,0x35,0x36][case%3]};
        if (192..256).contains(&case){expected.ram[0x39f+k]=1;}
        expected.ram[0x11e..0x126].fill(0);expected.ram[0x12e]=0;
        expected.sync_native_game_state_from_ram();expected.oam_reset_region_bases();
        let mut actual=expected.clone();actual.rom.clear();
        let mut cpu=NativeNmiContext {a:0xab00,x:k as u16,y:7,d:0,db:8,bank:8,pc:0xc5f7,sp:0x1e8,p:0x30};
        let mut clock=snes::CpuMasterTimeline::at_raster(4586,snes::CpuRasterPosition::new(59,1334),snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);clock.begin_synchronous_timeline().unwrap();
        let mut work=ReceiptDrawing::default();let mut owned=0;
        while work.tail().is_none() {
            owned+=work.step(&mut cpu,&mut clock,|_,access| match access {
                Access::Read(a,width)=>{
                    if let Some(v)=ZeldaState::native_receipt_animation_table_byte(a).or_else(||ZeldaState::native_ancilla_oam_table_byte(a)) {u16::from(v)}else{
                        let a=(a & if matches!(a>>16,0x7e|0x7f){0x1ffff}else{0xffff}) as usize;u16::from(actual.ram[a])|if width==2{u16::from(actual.ram[a+1])<<8}else{0}
                    }
                },
                Access::Write(a,v,width)=>{let a=(a & 0xffff) as usize;actual.ram[a..a+width as usize].copy_from_slice(&v.to_le_bytes()[..width as usize]);actual.native_push_block_store(a,v,width);0},
                Access::Vblank{..}=>0,
            }).unwrap();work=work.clone();
        }
        let tail=work.tail().unwrap();match tail {ReceiptDrawingTail::Sparkle=>tails[0]+=1,ReceiptDrawingTail::Graphics{source}=>{tails[1]+=1;graphics.insert(source);},ReceiptDrawingTail::Draw=>tails[2]+=1};
        let before=crate::cycle_ledger::master();expected.ancilla22_item_receipt_draw_and_update(k);let full=crate::cycle_ledger::master()-before;
        let before=crate::cycle_ledger::master();actual.finish_native_receipt_drawing(k,tail);let remaining=crate::cycle_ledger::master()-before;
        let excluded=|i:usize| i<16 || (0x1c0..=0x1ff).contains(&i);
        let diffs=actual.ram.iter().zip(&expected.ram).enumerate().filter_map(|(i,(a,b))|(!excluded(i)&&a!=b).then_some((i,*a,*b))).take(16).collect::<Vec<_>>();assert!(diffs.is_empty(),"case={case} tail={tail:?} RAM {diffs:x?}");
        let mut a=vec![0xa5;snes::WRAM_SIZE];let mut b=a.clone();actual.game_state.write_to_ram(&mut a);expected.game_state.write_to_ram(&mut b);
        let diffs=a.iter().zip(&b).enumerate().filter_map(|(i,(a,b))|(!excluded(i)&&a!=b).then_some((i,*a,*b))).take(16).collect::<Vec<_>>();assert!(diffs.is_empty(),"case={case} tail={tail:?} native {diffs:x?}");
        assert_eq!(full-remaining,owned,"case={case} tail={tail:?}: no owned animation or coordinate replay");assert!(actual.rom.is_empty());
    }
    assert!(tails.iter().all(|n|*n>0));assert_eq!(graphics,std::collections::BTreeSet::from([0x24,0x25,0x26]));
    println!("native receipt drawing handoff: 512 fixtures, sparkle and all three graphics sources and OAM match, owned animation and coordinates omitted");
}

#[test]
fn native_receipt_oam_preserves_semantic_tiles_and_all_body_return_cycles() {
    use crate::cycle_models::{receipt_oam::ReceiptOam,intro_clear::Access,nmi_stack::NativeNmiContext};
    for case in 0..1536usize {
        let fixture=case*103;let k=fixture%10;let mut expected=ZeldaState::new();
        crate::cycle_models::receipt_oam::seed_case(&mut expected.ram,fixture);
        expected.sync_native_game_state_from_ram();let mut actual=expected.clone();actual.rom.clear();
        let x=read_le_u16(&expected.ram,2);let y=read_le_u16(&expected.ram,0);let oam=read_le_u16(&expected.ram,0x90) as usize;
        let mut cpu=NativeNmiContext{a:0xab00,x:k as u16,y:7,d:0,db:8,bank:8,pc:0xc690,sp:0x1e8,p:0x30};
        let mut clock=snes::CpuMasterTimeline::at_raster(4586,snes::CpuRasterPosition::new(60,1252),snes::CpuBusWorkload::default(),snes::CpuFieldTiming::NON_INTERLACE_EVEN);clock.begin_synchronous_timeline().unwrap();let mut work=ReceiptOam::default();let mut owned=0;
        while !work.complete() {
            owned+=work.step(&mut cpu,&mut clock,|_,access| match access {
                Access::Read(a,width)=>{if let Some(v)=ZeldaState::native_ancilla_oam_table_byte(a).or_else(||crate::zelda_rtl::misc::native_item_receipt_table_byte(a)){u16::from(v)}else{let a=(a&0xffff) as usize;u16::from(actual.ram[a])|if width==2{u16::from(actual.ram[a+1])<<8}else{0}}},
                Access::Write(a,v,width)=>{let a=(a&0xffff) as usize;actual.ram[a..a+width as usize].copy_from_slice(&v.to_le_bytes()[..width as usize]);actual.native_push_block_store(a,v,width);0},
                Access::Vblank{..}=>0,
            }).unwrap();work=work.clone();
        }
        let before=crate::cycle_ledger::master();let end=expected.ancilla_receive_item_draw(k,x,y);let full=crate::cycle_ledger::master()-before;
        // $74 is the source palette local; the semantic helper uses a Rust local.
        let excluded=|i:usize|i<16 || i==0x74 || (0x1c0..=0x1ff).contains(&i);
        let diffs=actual.ram.iter().zip(&expected.ram).enumerate().filter_map(|(i,(a,b))|(!excluded(i)&&a!=b).then_some((i,*a,*b))).take(16).collect::<Vec<_>>();assert!(diffs.is_empty(),"case={case} RAM {diffs:x?}");
        let mut a=vec![0xa5;snes::WRAM_SIZE];let mut b=a.clone();actual.game_state.write_to_ram(&mut a);expected.game_state.write_to_ram(&mut b);
        let diffs=a.iter().zip(&b).enumerate().filter_map(|(i,(a,b))|(!excluded(i)&&a!=b).then_some((i,*a,*b))).take(16).collect::<Vec<_>>();assert!(diffs.is_empty(),"case={case} native {diffs:x?}");
        assert_eq!(owned,full,"case={case}: all OAM body and helper return cycles owned");assert_eq!(end,oam+cpu.y as usize);assert_eq!((cpu.pc,cpu.sp,cpu.x),(0x8338,0x1ea,k as u16));assert!(actual.rom.is_empty());
    }
    println!("native receipt OAM semantics: 1536 fixtures, full tiles and body/XY-helper return cycles match");
}
