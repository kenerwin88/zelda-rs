use super::*;
use crate::{SRAM_SIZE, VRAM_WORDS};
use snes::cart::CartType;
use snes::WRAM_SIZE;

#[test]
fn synced_states_compare_equal() {
    let mut oracle = LockstepOracle::new();
    oracle.snes.ram[0x200] = 0x42;
    oracle.snes.cart.ram[3] = 0x77;
    oracle.snes.ppu.vram[4] = 0x1234;
    oracle.sync_game_from_oracle();

    oracle.compare_current().expect("states should match");
}

#[test]
fn compare_reports_visible_ppu_state() {
    let mut oracle = LockstepOracle::new();
    oracle.sync_game_from_oracle();
    oracle.game.ram[0x200] = 1;
    oracle.game.sram[0x10] = 2;
    oracle.game.ppu.vram[0x20] = 3;
    oracle.game.ppu.cgram[0x21] = 4;
    oracle.game.ppu.oam[0x12] = 5;
    oracle.game.ppu.bg_layer[0].tilemap_adr = 0x2000;

    let err = oracle.compare_current().expect_err("expected divergence");
    let OracleError::Diverged(report) = err else {
        panic!("wrong error type");
    };

    assert_eq!(report.total_wram, 1);
    assert_eq!(report.total_sram, 1);
    assert_eq!(report.total_vram, 1);
    assert_eq!(report.total_cgram, 1);
    assert_eq!(report.total_oam, 1);
    assert_ne!(report.total_ppu_regs, 0);
    assert!(report
        .differences
        .iter()
        .any(|d| d.region == Region::Wram && d.offset == 0x200));
    assert!(report
        .differences
        .iter()
        .any(|d| d.region == Region::Cgram && d.offset == 0x21));
    assert!(report
        .differences
        .iter()
        .any(|d| d.region == Region::Oam && d.offset == 0x12));
    assert!(report
        .differences
        .iter()
        .any(|d| d.region == Region::PpuRegs));
}

#[test]
fn compare_applies_c_oracle_normalization() {
    let mut oracle = LockstepOracle::new();
    oracle.sync_game_from_oracle();
    oracle.game.ram[0x72] = 0xaa;
    oracle.game.ram[0x1f0a] = 0xbb;
    oracle.snes.ram[0x1f0a] = 0xcc;

    oracle
        .compare_current()
        .expect("normalized differences should be ignored");
}

#[test]
fn semantic_snapshots_match_for_synced_states() {
    let mut oracle = LockstepOracle::new();
    oracle.snes.ram[0x10] = 7;
    oracle.snes.ram[0x11] = 2;
    oracle.snes.ram[0x22] = 0x34;
    oracle.snes.ram[0x23] = 0x12;
    oracle.snes.ram[0x84] = 0x90;
    oracle.snes.ram[0x85] = 0x13;
    oracle.snes.ram[0x86] = 0x1f;
    oracle.snes.ram[0x88] = 0x0e;
    oracle.snes.ram[0x0f340] = 3;
    oracle.snes.ram[0x0dd0] = 9;
    oracle.snes.ram[0x0e20] = 0xa5;
    oracle.snes.ram[0x0d10] = 0x78;
    oracle.snes.ram[0x0d30] = 0x56;
    oracle.snes.ppu.mode = 1;
    oracle.snes.ppu.brightness = 0x0f;
    oracle.sync_game_from_oracle();

    let report = oracle.compare_current_semantic();

    assert!(report.is_empty(), "{report}");
    assert_eq!(oracle.semantic_game_snapshot().frame.main_module, 7);
    assert_eq!(oracle.semantic_game_snapshot().player.x, 0x1234);
    assert_eq!(oracle.semantic_game_snapshot().player.equipped_item, 3);
    assert_eq!(oracle.semantic_game_snapshot().world.map16_load_src, 0x1390);
    assert_eq!(oracle.semantic_game_snapshot().world.map16_load_dst, 0x001f);
    assert_eq!(
        oracle.semantic_game_snapshot().world.map16_load_y_unit,
        0x000e
    );
    assert_eq!(oracle.semantic_game_snapshot().sprites.len(), 1);
}

#[test]
fn semantic_report_names_changed_fields() {
    let mut oracle = LockstepOracle::new();
    oracle.sync_game_from_oracle();
    oracle.game.set_main_module(3);
    oracle.game.ram[0x22] = 0x44;
    oracle.game.ram[0x0e20] = 0x42;
    oracle.game.ram[0x0dd0] = 4;

    let report = oracle.compare_current_semantic();

    assert!(!report.is_empty());
    assert!(report
        .differences
        .iter()
        .any(|diff| diff.field == "frame.main_module"));
    assert!(report
        .differences
        .iter()
        .any(|diff| diff.field == "player.x"));
    assert!(report
        .differences
        .iter()
        .any(|diff| diff.field == "sprites"));
}

#[test]
fn semantic_snapshot_extracts_active_ancillas() {
    let mut oracle = LockstepOracle::new();
    oracle.snes.ram[0x0c4a + 3] = 0x2b;
    oracle.snes.ram[0x0c04 + 3] = 0x67;
    oracle.snes.ram[0x0c18 + 3] = 0x45;
    oracle.snes.ram[0x0bfa + 3] = 0x23;
    oracle.snes.ram[0x0c0e + 3] = 0x01;
    oracle.sync_game_from_oracle();

    let snapshot = oracle.semantic_game_snapshot();

    assert_eq!(snapshot.ancillas.len(), 1);
    assert_eq!(snapshot.ancillas[0].slot, 3);
    assert_eq!(snapshot.ancillas[0].ancilla_type, 0x2b);
    assert_eq!(snapshot.ancillas[0].x, 0x4567);
    assert_eq!(snapshot.ancillas[0].y, 0x0123);
}

#[test]
fn native_map16_load_state_dual_writes_and_graduated_compare_masks_padding_bytes() {
    let mut oracle = LockstepOracle::new();
    oracle.snes.ram[0x84] = 0x90;
    oracle.snes.ram[0x85] = 0x13;
    oracle.snes.ram[0x86] = 0x1f;
    oracle.snes.ram[0x88] = 0x0e;
    oracle.sync_game_from_oracle();

    oracle.game.ram[0x84] = 0xff;
    oracle.game.ram[0x85] = 0xff;
    oracle.game.ram[0x86] = 0xee;
    oracle.game.ram[0x88] = 0xdd;
    oracle
        .game
        .set_overworld_map16_load_state(crate::OverworldMap16LoadState {
            src_off: 0x1390,
            dst_off: 0x001f,
            y_unit: 0x000e,
        });

    assert_eq!(&oracle.game.ram[0x84..=0x86], &oracle.snes.ram[0x84..=0x86]);
    assert_eq!(oracle.game.ram[0x88], oracle.snes.ram[0x88]);

    oracle.game.ram[0x87] = 0xee;
    oracle.game.ram[0x89] = 0xdd;

    assert!(oracle.compare_current().is_err());
    oracle
        .compare_current_with_graduated_semantics()
        .expect("graduated Map16 semantics should replace padding bytes");
}

#[test]
fn snapshot_restore_helpers_roundtrip_cpu_memory_and_video() {
    let mut oracle = LockstepOracle::new();
    oracle.snes.cpu.a = 0x1234;
    oracle.snes.cpu.x = 0x4567;
    oracle.snes.cpu.y = 0x89ab;
    oracle.snes.cpu.sp = 0x1f80;
    oracle.snes.cpu.dp = 0x1f00;
    oracle.snes.cpu.pc = 0x8034;
    oracle.snes.cpu.k = 0x09;
    oracle.snes.cpu.db = 0x7e;
    oracle.snes.cpu.unpack_flags(0xb5);
    oracle.snes.ram[0x200] = 0x42;
    oracle.snes.cart.ram[0x10] = 0x77;
    oracle.snes.ppu.vram[0x20] = 0x9abc;
    oracle.snes.ppu.cgram[0x21] = 0x1111;
    oracle.snes.ppu.oam[0x12] = 0x2222;

    let snapshot = Snapshot::from_snes(&oracle.snes);
    oracle.snes.cpu.a = 0;
    oracle.snes.ram[0x200] = 0;
    oracle.snes.cart.ram[0x10] = 0;
    oracle.snes.ppu.vram[0x20] = 0;
    oracle.snes.ppu.cgram[0x21] = 0;
    oracle.snes.ppu.oam[0x12] = 0;
    snapshot.restore_snes(&mut oracle.snes);

    assert_eq!(oracle.snes.cpu.a, 0x1234);
    assert_eq!(oracle.snes.cpu.pack_flags(), 0xb5);
    assert_eq!(oracle.snes.ram[0x200], 0x42);
    assert_eq!(oracle.snes.cart.ram[0x10], 0x77);
    assert_eq!(oracle.snes.ppu.vram[0x20], 0x9abc);
    assert_eq!(oracle.snes.ppu.cgram[0x21], 0x1111);
    assert_eq!(oracle.snes.ppu.oam[0x12], 0x2222);

    let game_snapshot = Snapshot::from_game(&oracle.game);
    oracle.game.ram[0x300] = 0x55;
    oracle.game.set_main_module(0x33);
    game_snapshot.restore_game(&mut oracle.game);
    assert_eq!(oracle.game.ram[0x300], 0);
    assert_eq!(oracle.game.game_state.frame.main_module, 0);
}

#[test]
fn rom_and_cart_pointer_helpers_use_lorom_mapping() {
    let mut cart = snes::Cart::new();
    let mut rom = vec![0u8; 0x10000];
    rom[0x8000] = 0x5a;
    cart.load(CartType::LoRom, &rom, 0x2000);
    cart.ram[0x12] = 0xa5;

    assert_eq!(get_ptr_ref(&cart, 0x018000).copied(), Some(0x5a));
    assert_eq!(rom_byte(&cart, 0x018000), 0x5a);
    assert_eq!(get_cart_ram_ptr_ref(&cart, 0x12).copied(), Some(0xa5));
}

#[test]
fn emu_initialize_loads_and_syncs_synthetic_rom() {
    let oracle =
        LockstepOracle::emu_initialize_owned(&synthetic_lorom()).expect("initialize oracle");
    assert_eq!(oracle.snes.cart.kind, CartType::LoRom);
    assert_eq!(oracle.game.ram, oracle.snes.ram);
}

#[test]
fn oracle_frame_runner_reaches_synthetic_checkpoints() {
    let mut oracle = LockstepOracle::new();
    let rom = synthetic_lorom();
    oracle.load_rom(&rom).expect("load synthetic rom");
    assert_eq!(oracle.snes.cart.kind, CartType::LoRom);

    oracle
        .run_oracle_frame(0x1234, RUN_MAIN)
        .expect("oracle frame");

    assert_eq!(oracle.snes.cpu.pc, 0x8034);
    assert_eq!(oracle.snes.ram[0x12], 1);
    assert_eq!(read_le_u16(&oracle.snes.ram, 0x0adc), 0xa680);
    assert_eq!(oracle.snes.input1.current_state, 0x1234);
}

fn synthetic_lorom() -> Vec<u8> {
    let mut rom = vec![0u8; 0x10000];
    // $008000: JMP $8034
    rom[0x0000..0x0003].copy_from_slice(&[0x4c, 0x34, 0x80]);
    // $008034: BRA $8034
    rom[0x0034..0x0036].copy_from_slice(&[0x80, 0xfe]);
    // $0080D9: JMP $8034
    rom[0x00d9..0x00dc].copy_from_slice(&[0x4c, 0x34, 0x80]);

    let h = 0x7fc0;
    rom[h..h + 21].copy_from_slice(b"TEST ROM             ");
    rom[h + 0x15] = 0x20;
    rom[h + 0x16] = 0x02;
    rom[h + 0x17] = 0x05;
    rom[h + 0x18] = 0x03;
    rom[h + 0x19] = 0x01;
    rom[h + 0x1a] = 0x00;
    rom[h + 0x1b] = 0x00;
    rom[h + 0x1c] = 0x55;
    rom[h + 0x1d] = 0x55;
    rom[h + 0x1e] = 0xaa;
    rom[h + 0x1f] = 0xaa;
    rom[h + 0x3c] = 0x00;
    rom[h + 0x3d] = 0x80;
    rom
}

#[test]
fn compared_region_sizes_match_c_oracle() {
    assert_eq!(WRAM_SIZE, 0x20000);
    assert_eq!(SRAM_SIZE, 0x2000);
    assert_eq!(VRAM_WORDS, 0x8000);
}
