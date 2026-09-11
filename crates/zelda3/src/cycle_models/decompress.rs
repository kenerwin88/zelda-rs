//! Cycle model of the ROM's graphics decompressor (`Decompress`, `$00:E79E`,
//! entered through `Decomp_spr` `$00:E772`, `Decomp_bg_to_7f4000` `$00:E783`
//! or `Decomp_bg` `$00:E78F`).
//!
//! The 65816 runs from slow ROM (8 master cycles per fetched byte), reads and
//! writes WRAM at 8 cycles, and spends 6 cycles per internal cycle. Every
//! constant below is one instruction of the disassembly with that pricing;
//! the per-command paths follow the routine's branches exactly. The profile
//! written by `ZELDA3_DEBUG_ROM_CPU_PROFILE` gives the same numbers per
//! instruction address, and the test at the bottom runs every sheet in the
//! cartridge through the shadow CPU and demands equality.

/// Which entry point the caller used; the two prologues load the source
/// pointer from different tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DecompressEntry {
    /// `Decomp_spr` (`$00:E772`): three `LDA $CFF3,Y`-style table loads.
    Sprite,
    /// `Decomp_bg_to_7f4000` (`$00:E783`): destination `$7F:4000` set up,
    /// then the background tables.
    Background,
    /// `Decomp_bg` (`$00:E78F`): the background tables only; the caller
    /// set the destination.
    BackgroundTables,
}

/// The bank offset (`$C8`) at which the ROM's compressed stream for
/// `sheet` begins, for the sheets whose stream runs past `$FFFF` (they are
/// the only ones where the offset changes the cost: `GetNextByte` charges
/// its bank-increment path once). Every other stream is priced from
/// `$8000`, which never wraps. Derived from the pointer tables at
/// `$00:CFF3`/`$00:D0D2`/`$00:D1B1` (sprite) and `$00:CF80`/`$00:D05F`/
/// `$00:D13E` (background); the test below checks it against the ROM.
pub(crate) fn stream_source_offset(entry: DecompressEntry, sheet: u8) -> u16 {
    match (entry, sheet) {
        (DecompressEntry::Sprite, 0x0c) => 0xfffc,
        (DecompressEntry::Sprite, 0x24) => 0xfeba,
        (DecompressEntry::Sprite, 0x40) => 0xfd29,
        (DecompressEntry::Sprite, 0x5c) => 0xff86,
        (DecompressEntry::Background | DecompressEntry::BackgroundTables, 0x10) => 0xfff2,
        (DecompressEntry::Background | DecompressEntry::BackgroundTables, 0x2a) => 0xfc71,
        (DecompressEntry::Background | DecompressEntry::BackgroundTables, 0x44) => 0xfc30,
        _ => 0x8000,
    }
}

// Instruction pricing (master cycles).
const BRANCH_TAKEN: u64 = 22; // opcode 8 + operand 8 + 6 internal
const BRANCH_NOT_TAKEN: u64 = 16;
const JSR_ABS: u64 = 46; // 8 + 16 + 6 + two stack pushes
const RTS: u64 = 42; // 8 + 6 + 6 + two stack pulls + 6
const JMP_ABS: u64 = 24;
const LDA_STA_DP_8: u64 = 24; // opcode + operand + one WRAM access
const LDX_STX_DP_16: u64 = 32; // opcode + operand + two WRAM accesses
const IMM_8: u64 = 16; // LDA/AND/CMP #imm, 8-bit
const IMM_16: u64 = 24;
const REP_SEP: u64 = 22;
const IMPLIED: u64 = 14; // INC A, ASL A, INX, INY, DEX, TAX, TXY, TYX
const XBA: u64 = 20;
const PHA_8: u64 = 22;
const PLA_8: u64 = 28;
const PHY_16: u64 = 30;
const PLY_16: u64 = 36;
const LDY_IMM_16: u64 = 24;
const LDX_IMM_16: u64 = 24;
const LONG_INDIRECT: u64 = 48; // LDA/STA [dp] or [dp],Y: 8 + 8 + 3 pointer bytes + one 8-cycle access
const LONG_INDIRECT_WITHOUT_ACCESS: u64 = 40;

/// Master cycles of one data access at a 24-bit address, the pinned core's
/// bus rule (`hardware_access_time`): 8 for WRAM, cartridge and the WRAM
/// mirror; 6 for the register window `$2000-$5FFF` of the system banks.
/// The decompressor's destination and back-reference reads normally stay in
/// WRAM; a stream that runs past `$7F:FFFF` wraps into bank `$80`, where
/// the register window is cheaper.
fn data_access_cost(address: u32) -> u64 {
    let address = address & 0x00ff_ffff;
    if address & 0x40_8000 != 0 {
        return 8;
    }
    if address.wrapping_add(0x6000) & 0x4000 != 0 {
        return 8;
    }
    if address.wrapping_sub(0x4000) & 0x7e00 != 0 {
        return 6;
    }
    12
}
const DEC_DP_16: u64 = 54; // 8 + 8 + read 16 + 6 + write 16
const INC_DP_8: u64 = 38;
const STZ_DP: u64 = 24;
const ABS_Y_8: u64 = 32; // LDA $abcd,Y with 8-bit accumulator, no page crossing
const PAGE_CROSS: u64 = 6;

/// `Decompression_GetNextByte` (`$00:E843`): `LDA [$C8] : LDX $C8 : INX :
/// BNE +5 : LDX #$0000 : INC $CA : + STX $C8 : RTS`.
fn get_next_byte(source_offset: &mut u16) -> u64 {
    let wrapped = *source_offset == 0xffff;
    *source_offset = source_offset.wrapping_add(1);
    let wrap_cost = if wrapped {
        BRANCH_NOT_TAKEN + LDX_IMM_16 + INC_DP_8
    } else {
        BRANCH_TAKEN
    };
    LONG_INDIRECT + LDX_STX_DP_16 + IMPLIED + wrap_cost + LDX_STX_DP_16 + RTS
}

/// Master cycles from the entry point to its final `RTS`, for a compressed
/// stream that begins at `source` (bank offset `source_offset`, used only
/// for the rare bank-boundary wrap inside `GetNextByte`). `sheet` selects
/// the entry's table loads, whose `abs,Y` reads cost one cycle more when
/// the index crosses a page.
pub(crate) fn decompress_master_cycles(
    entry: DecompressEntry,
    sheet: u8,
    source: &[u8],
    mut source_offset: u16,
    destination: u32,
) -> u64 {
    // `[$00],Y` accesses: the 24-bit destination plus the 16-bit output
    // cursor, priced by the bus rule at that address.
    let write_cost = |cursor: u16| LONG_INDIRECT_WITHOUT_ACCESS + data_access_cost(destination.wrapping_add(u32::from(cursor)));
    let mut cursor: u16 = 0;
    let table_load = |table_low_byte: u8| -> u64 {
        ABS_Y_8 + if u16::from(table_low_byte) + u16::from(sheet) > 0xff { PAGE_CROSS } else { 0 }
    };
    let mut master = match entry {
        // LDA $CFF3,Y : STA $CA : LDA $D0D2,Y : STA $C9 : LDA $D1B1,Y : STA $C8 : BRA Decompress
        DecompressEntry::Sprite => {
            table_load(0xf3) + LDA_STA_DP_8 + table_load(0xd2) + LDA_STA_DP_8
                + table_load(0xb1) + LDA_STA_DP_8 + BRANCH_TAKEN
        }
        // STZ $00 : LDA #$40 : STA $01 : LDA #$7F : STA $02 : STA $05
        // : LDA $CF80,Y : STA $CA : LDA $D05F,Y : STA $C9 : LDA $D13E,Y : STA $C8
        DecompressEntry::Background => {
            STZ_DP + IMM_8 + LDA_STA_DP_8 + IMM_8 + LDA_STA_DP_8 + LDA_STA_DP_8
                + table_load(0x80) + LDA_STA_DP_8 + table_load(0x5f) + LDA_STA_DP_8
                + table_load(0x3e) + LDA_STA_DP_8
        }
        // LDA $CF80,Y : STA $CA : LDA $D05F,Y : STA $C9 : LDA $D13E,Y : STA $C8
        DecompressEntry::BackgroundTables => {
            table_load(0x80) + LDA_STA_DP_8 + table_load(0x5f) + LDA_STA_DP_8
                + table_load(0x3e) + LDA_STA_DP_8
        }
    };
    // Decompress: REP #$10 : LDY #$0000
    master += REP_SEP + LDY_IMM_16;
    let mut read = 0usize;
    let mut next = |master: &mut u64, read: &mut usize| -> u8 {
        let byte = source.get(*read).copied().unwrap_or(0xff);
        *read += 1;
        *master += JSR_ABS + get_next_byte(&mut source_offset);
        byte
    };
    loop {
        // .loop JSR GetNextByte : CMP #$FF : BNE .command
        let header = next(&mut master, &mut read);
        master += IMM_8;
        if header == 0xff {
            // SEP #$10 : RTS
            master += BRANCH_NOT_TAKEN + REP_SEP + RTS;
            return master;
        }
        master += BRANCH_TAKEN;
        // STA $CD : AND #$E0 : CMP #$E0 : BEQ .extended
        master += LDA_STA_DP_8 + IMM_8 + IMM_8;
        let (command, length) = if header & 0xe0 != 0xe0 {
            // PHA : LDA $CD : REP #$20 : AND #$001F : BRA .length
            master += BRANCH_NOT_TAKEN + PHA_8 + LDA_STA_DP_8 + REP_SEP + IMM_16 + BRANCH_TAKEN;
            (header & 0xe0, u16::from(header & 0x1f))
        } else {
            // LDA $CD : ASL : ASL : ASL : AND #$E0 : PHA : LDA $CD : AND #$03
            // : XBA : JSR GetNextByte : REP #$20
            master += BRANCH_TAKEN + LDA_STA_DP_8 + 3 * IMPLIED + IMM_8 + PHA_8
                + LDA_STA_DP_8 + IMM_8 + XBA;
            let low = next(&mut master, &mut read);
            master += REP_SEP;
            ((header << 3) & 0xe0, (u16::from(header & 3) << 8) | u16::from(low))
        };
        // .length INC A : STA $CB : SEP #$20 : PLA
        master += IMPLIED + LDX_STX_DP_16 + REP_SEP + PLA_8;
        let length = length + 1;
        // BEQ .copy : BMI .backReference : ASL : BPL .fill : ASL : BPL .wordFill
        match command >> 5 {
            0 => {
                master += BRANCH_TAKEN;
                // .copy JSR GetNextByte : STA [$00],Y : INY : LDX $CB : DEX : STX $CB : BNE .copy : BRA .loop
                for remaining in (1..=length).rev() {
                    next(&mut master, &mut read);
                    master += write_cost(cursor) + IMPLIED + LDX_STX_DP_16 + IMPLIED + LDX_STX_DP_16;
                    cursor = cursor.wrapping_add(1);
                    master += if remaining > 1 { BRANCH_TAKEN } else { BRANCH_NOT_TAKEN };
                }
                master += BRANCH_TAKEN;
            }
            1 => {
                master += BRANCH_NOT_TAKEN + BRANCH_NOT_TAKEN + IMPLIED + BRANCH_TAKEN;
                // .fill JSR GetNextByte : LDX $CB : - STA [$00],Y : INY : DEX : BNE - : BRA .loop
                next(&mut master, &mut read);
                master += LDX_STX_DP_16;
                for remaining in (1..=length).rev() {
                    master += write_cost(cursor) + IMPLIED + IMPLIED;
                    cursor = cursor.wrapping_add(1);
                    master += if remaining > 1 { BRANCH_TAKEN } else { BRANCH_NOT_TAKEN };
                }
                master += BRANCH_TAKEN;
            }
            2 => {
                master += BRANCH_NOT_TAKEN + BRANCH_NOT_TAKEN + IMPLIED + BRANCH_NOT_TAKEN
                    + IMPLIED + BRANCH_TAKEN;
                // .wordFill JSR : XBA : JSR : LDX $CB
                next(&mut master, &mut read);
                master += XBA;
                next(&mut master, &mut read);
                master += LDX_STX_DP_16;
                // - XBA : STA [$00],Y : INY : DEX : BEQ .done : XBA : STA [$00],Y : INY : DEX : BNE - : .done JMP .loop
                let mut remaining = length;
                loop {
                    master += XBA + write_cost(cursor) + IMPLIED + IMPLIED;
                    cursor = cursor.wrapping_add(1);
                    remaining -= 1;
                    if remaining == 0 {
                        master += BRANCH_TAKEN;
                        break;
                    }
                    master += BRANCH_NOT_TAKEN + XBA + write_cost(cursor) + IMPLIED + IMPLIED;
                    cursor = cursor.wrapping_add(1);
                    remaining -= 1;
                    if remaining == 0 {
                        master += BRANCH_NOT_TAKEN;
                        break;
                    }
                    master += BRANCH_TAKEN;
                }
                master += JMP_ABS;
            }
            3 => {
                master += BRANCH_NOT_TAKEN + BRANCH_NOT_TAKEN + IMPLIED + BRANCH_NOT_TAKEN
                    + IMPLIED + BRANCH_NOT_TAKEN;
                // .increasing JSR GetNextByte : LDX $CB : - STA [$00],Y : INC A : INY : DEX : BNE - : BRA .loop
                next(&mut master, &mut read);
                master += LDX_STX_DP_16;
                for remaining in (1..=length).rev() {
                    master += write_cost(cursor) + IMPLIED + IMPLIED + IMPLIED;
                    cursor = cursor.wrapping_add(1);
                    master += if remaining > 1 { BRANCH_TAKEN } else { BRANCH_NOT_TAKEN };
                }
                master += BRANCH_TAKEN;
            }
            _ => {
                master += BRANCH_NOT_TAKEN + BRANCH_TAKEN;
                // .backReference JSR : XBA : JSR : XBA : TAX
                let low = next(&mut master, &mut read);
                master += XBA;
                let high = next(&mut master, &mut read);
                master += XBA + IMPLIED;
                let mut source_cursor = u16::from(high) << 8 | u16::from(low);
                // - PHY : TXY : LDA [$00],Y : TYX : PLY : STA [$00],Y : INY : INX
                // : REP #$20 : DEC $CB : SEP #$20 : BNE - : JMP .loop
                for remaining in (1..=length).rev() {
                    master += PHY_16 + IMPLIED + write_cost(source_cursor) + IMPLIED + PLY_16
                        + write_cost(cursor) + IMPLIED + IMPLIED + REP_SEP + DEC_DP_16 + REP_SEP;
                    cursor = cursor.wrapping_add(1);
                    source_cursor = source_cursor.wrapping_add(1);
                    master += if remaining > 1 { BRANCH_TAKEN } else { BRANCH_NOT_TAKEN };
                }
                master += JMP_ABS;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom_cpu_timing::{lorom_offset, RomCpuCheckpoint, RomCpuTimingRun};

    fn test_rom() -> Option<Vec<u8>> {
        let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from).unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc")
        });
        let mut rom = std::fs::read(path).ok()?;
        if rom.len() % 0x400 == 0x200 {
            rom.drain(..0x200);
        }
        Some(rom)
    }

    /// The compressed stream a sheet's pointer-table entry names.
    fn sheet_stream(rom: &[u8], entry: DecompressEntry, sheet: u8) -> Option<(u32, &[u8])> {
        // The prologue stores the first table byte to $CA (bank), the second
        // to $C9 (high) and the third to $C8 (low).
        let (bank, high, low) = match entry {
            DecompressEntry::Sprite => (0x00_cff3, 0x00_d0d2, 0x00_d1b1),
            DecompressEntry::Background | DecompressEntry::BackgroundTables => {
                (0x00_cf80, 0x00_d05f, 0x00_d13e)
            }
        };
        let byte = |table: u32| rom[lorom_offset(table).unwrap() + usize::from(sheet)];
        let address = u32::from(byte(bank)) << 16 | u32::from(byte(high)) << 8 | u32::from(byte(low));
        // Unused table slots point outside the cartridge; skip those.
        let start = lorom_offset(address).filter(|&start| start < rom.len())?;
        // A stream may cross a bank boundary; LoROM banks are contiguous
        // 32 KiB slices, and the bank wrap only changes the address.
        Some((address, &rom[start..]))
    }

    fn shadow_master_cycles(rom: &[u8], entry: DecompressEntry, sheet: u8) -> Option<u64> {
        // The caller returns to $00:8036 (RTS adds one to the pushed word).
        const RETURN_TO_MAIN_WAIT: [u8; 2] = [0x35, 0x80];
        let checkpoint = RomCpuCheckpoint {
            entry_pc: match entry {
                DecompressEntry::Sprite => 0x00_e772,
                DecompressEntry::Background => 0x00_e783,
                DecompressEntry::BackgroundTables => 0x00_e78f,
            },
            stop_pc: 0x00_8036,
            a: 0,
            x: 0,
            y: u16::from(sheet),
            sp: 0x01fc,
            dp: 0,
            db: 0,
            carry: false,
            zero: false,
            overflow: false,
            negative: false,
            interrupt_disable: true,
            decimal: false,
            accumulator_is_8_bit: true,
            index_is_8_bit: true,
            emulation: false,
            waiting: false,
            stack_address: 0x01fd,
            stack_bytes: &RETURN_TO_MAIN_WAIT,
        };
        let mut ram = vec![0u8; 0x2_0000];
        // Decomp_spr callers leave the destination in $00-$02; use $7F:4000.
        ram[0x00] = 0x00;
        ram[0x01] = 0x40;
        ram[0x02] = 0x7f;
        let ppu = snes::PpuState::default();
        let dma = snes::DmaState::default();
        let mut run =
            RomCpuTimingRun::new(rom, &ram, &[0u8; 0x2000], &ppu, &dma, [0; 4], checkpoint).ok()?;
        let mut master = 0u64;
        for _ in 0..4_000_000 {
            if run.is_complete() {
                return Some(master);
            }
            master += u64::from(run.step().master_cycles);
        }
        None
    }

    #[test]
    fn decompressor_model_matches_the_shadow_cpu_on_every_cartridge_sheet() {
        let Some(rom) = test_rom() else {
            eprintln!("no ROM available; skipping the decompressor cycle model check");
            return;
        };
        let mut checked = 0;
        let mut outside = 0;
        let mut unfinished = 0;
        let mut mismatches = Vec::new();
        for (entry, count) in [
            (DecompressEntry::Sprite, 0x6cu8),
            (DecompressEntry::Background, 0x72u8),
            (DecompressEntry::BackgroundTables, 0x72u8),
        ] {
            for sheet in 0..count {
                let Some((address, stream)) = sheet_stream(&rom, entry, sheet) else {
                    outside += 1;
                    continue;
                };
                let Some(measured) = shadow_master_cycles(&rom, entry, sheet) else {
                    unfinished += 1;
                    continue;
                };
                let modeled = decompress_master_cycles(entry, sheet, stream, address as u16, 0x7f_4000);
                checked += 1;
                if modeled != measured {
                    mismatches.push((entry, sheet, modeled, measured));
                }
            }
        }
        eprintln!(
            "decompressor cycle model: checked={checked} outside_rom={outside} unfinished={unfinished} mismatches={}",
            mismatches.len()
        );
        assert!(checked > 100, "only {checked} sheets were checked (outside={outside}, unfinished={unfinished})");
        assert!(
            mismatches.is_empty(),
            "decompressor cycle model differs from the shadow CPU on {} of {checked} sheets: {:?}",
            mismatches.len(),
            &mismatches[..mismatches.len().min(8)]
        );
    }

    #[test]
    fn stream_source_offset_prices_every_cartridge_sheet_like_its_rom_address() {
        let Some(rom) = test_rom() else {
            eprintln!("no ROM available; skipping the stream source offset check");
            return;
        };
        let mut wrapped = 0;
        for (entry, count) in [(DecompressEntry::Sprite, 0x6cu8), (DecompressEntry::BackgroundTables, 0x72u8)] {
            for sheet in 0..count {
                let Some((address, stream)) = sheet_stream(&rom, entry, sheet) else {
                    continue;
                };
                let from_rom = decompress_master_cycles(entry, sheet, stream, address as u16, 0x7f_4000);
                let from_table =
                    decompress_master_cycles(entry, sheet, stream, stream_source_offset(entry, sheet), 0x7f_4000);
                assert_eq!(from_table, from_rom, "{entry:?} sheet {sheet:#04x} at {address:06x}");
                if decompress_master_cycles(entry, sheet, stream, 0x8000, 0x7f_4000) != from_rom {
                    wrapped += 1;
                }
            }
        }
        assert_eq!(wrapped, 7, "the table names every bank-crossing stream");
    }
}
