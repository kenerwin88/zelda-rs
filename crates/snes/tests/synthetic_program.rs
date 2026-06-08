//! End-to-end integration test: build a small ROM image with a known
//! program in it, load it through the loader, run the CPU, observe
//! the WRAM. This proves cart → bus → CPU all wire up correctly
//! without needing the real Zelda ROM.

use snes::cart::CartType;
use snes::{cpu_run_opcode, Snes};

#[test]
fn loadable_lorom_executes_program() {
    // Minimal LoROM image with a reset vector at $FFFC pointing to $8000.
    // Program at $008000:
    //   18         CLC
    //   FB         XCE        ; switch to native mode (e=0)
    //   C2 30      REP #$30   ; clear m and x — 16-bit A/X/Y
    //   A9 34 12   LDA #$1234
    //   8D 00 00   STA $0000
    //   80 FE      BRA self
    let mut rom = vec![0u8; 0x10000];
    let prog: [u8; 11] = [
        0x18, 0xfb, 0xc2, 0x30, 0xa9, 0x34, 0x12, 0x8d, 0x00, 0x00, 0x80,
    ];
    rom[..prog.len()].copy_from_slice(&prog);
    rom[prog.len()] = 0xfe; // BRA -2

    // Header at $7FC0 (LoROM)
    let h = 0x7fc0;
    rom[h..h + 21].copy_from_slice(b"TEST ROM             ");
    rom[h + 0x15] = 0x20; // speed 2, LoROM
    rom[h + 0x16] = 0x02; // SRAM-present cart, matching the fixed 8 KiB C cart
    rom[h + 0x17] = 0x05;
    rom[h + 0x18] = 0x03;
    rom[h + 0x19] = 0x01; // USA
    rom[h + 0x1a] = 0x00;
    rom[h + 0x1b] = 0x00;
    // checksum + complement so loader scores this header high
    rom[h + 0x1c] = 0x55;
    rom[h + 0x1d] = 0x55;
    rom[h + 0x1e] = 0xaa;
    rom[h + 0x1f] = 0xaa;
    // reset vector → $8000
    rom[h + 0x3c] = 0x00;
    rom[h + 0x3d] = 0x80;

    let mut snes = Snes::new();
    snes::load_rom(&mut snes, &rom).expect("load_rom");
    assert_eq!(snes.cart.kind, CartType::LoRom);

    snes.cpu_seed_reset_vector();
    assert_eq!(snes.cpu.pc, 0x8000);

    // Step long enough for the program to settle into the BRA loop.
    for _ in 0..100 {
        let _ = cpu_run_opcode(&mut snes);
    }

    // STA $0000 must have parked $1234 into WRAM[0..2].
    assert_eq!(snes.ram[0], 0x34);
    assert_eq!(snes.ram[1], 0x12);
    // Native mode, 16-bit A.
    assert!(!snes.cpu.e);
    assert!(!snes.cpu.mf);
    assert_eq!(snes.cpu.a, 0x1234);
}
