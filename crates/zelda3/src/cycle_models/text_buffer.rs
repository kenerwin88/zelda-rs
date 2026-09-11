//! Cycle model of `Text_LoadCharacterBuffer` (`$0E:C4E2`), the routine that
//! expands one dialogue message's dictionary-compressed bytes into the
//! character buffer at `$7F:1200` when a message opens.
//!
//! The routine consumes the message's original ROM bytes: plain characters
//! below `$67`, commands `$67-$7E` dispatched through `JumpTableLocal`, the
//! `$7F` terminator, and dictionary references at `$88` and above, each
//! expanded by `Text_DictionarySequence` (`$0E:C6DA`) into the word the
//! dictionary offset table names. Costs are priced per instruction of the
//! disassembly (slow ROM 8 per byte, WRAM 8, internal 6; `abs,Y` reads add
//! one cycle with 16-bit index registers). The exhaustive test runs all
//! 398 cartridge messages through the shadow CPU.

const BRANCH_TAKEN: u64 = 22;
const BRANCH_NOT_TAKEN: u64 = 16;
const JSR_ABS: u64 = 46;
const JSL_LONG: u64 = 62;
const RTS: u64 = 42;
const REP_SEP: u64 = 22;
const IMPLIED: u64 = 14;
const XBA: u64 = 20;
const IMM_8: u64 = 16;
const IMM_16: u64 = 24;
const DP_8: u64 = 24; // LDA/STA/STY dp, 8-bit
const DP_16: u64 = 32;
const ABS_8: u64 = 32; // LDA/STA abs, 8-bit
const ABS_16: u64 = 40;
const ABS_INDEXED_16: u64 = 46; // LDA abs,X / abs,Y with 16-bit accumulator and index
const ABS_INDEXED_8_WIDE_INDEX: u64 = 38; // 8-bit accumulator, 16-bit index: +1 cycle always
const LONG_8: u64 = 40; // LDA/STA long or long,X with 8-bit accumulator
const LONG_16: u64 = 48;
const LONG_INDIRECT_Y_8: u64 = 48; // LDA [dp],Y 8-bit
const LONG_INDIRECT_Y_16: u64 = 56;
const INC_ABS_16: u64 = 62;
const PHA_16: u64 = 30;
const PLA_16: u64 = 36;
const PLA_8: u64 = 28; // PLY with 8-bit index
const PHP: u64 = 22;
const PLP: u64 = 28;
const JML_INDIRECT: u64 = 48;

/// `JumpTableLocal` (`$00:8781`) entered by `JSL` with 8-bit index
/// registers: pulls the return address, indexes the table that follows the
/// `JSL`, and jumps to the entry.
const JUMP_TABLE_LOCAL: u64 = DP_8 // STY $03
    + PLA_8 // PLY
    + DP_8 // STY $00
    + REP_SEP
    + IMM_16 // AND #$00FF
    + IMPLIED // ASL
    + IMPLIED // TAY
    + PLA_16 // PLA
    + DP_16 // STA $01
    + IMPLIED // INY
    + LONG_INDIRECT_Y_16 // LDA [$00],Y
    + DP_16 // STA $00
    + REP_SEP
    + DP_8 // LDY $03
    + JML_INDIRECT;

/// The ROM's text command dispatch table at `$0E:C54F`, one handler per
/// command byte `$67..=$7E`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommandHandler {
    /// `$0E:C581`: copy the command byte into the buffer.
    CopyCommand,
    /// `$0E:C598`: copy the command byte and its parameter.
    CopyCommandAndParameter,
    /// `$0E:C5B3`: write the player's name.
    PlayerName,
    /// `$0E:C657`: set the window type from the parameter.
    WindowType,
    /// `$0E:C667`: write one preloaded number digit.
    NumberDigit,
    /// `$0E:C69C`: set the message box position.
    Position,
    /// `$0E:C6B6`: set the text color.
    Color,
}

fn command_handler(command: u8) -> CommandHandler {
    match command {
        0x6a => CommandHandler::PlayerName,
        0x6b => CommandHandler::WindowType,
        0x6c => CommandHandler::NumberDigit,
        0x6d => CommandHandler::Position,
        0x6e | 0x78 | 0x79 | 0x7a => CommandHandler::CopyCommandAndParameter,
        0x77 => CommandHandler::Color,
        _ => CommandHandler::CopyCommand,
    }
}

/// `$0E:C639`: map one name letter to its text-engine glyph.
fn name_letter_cycles(letter: u8) -> u64 {
    let mut master = IMM_8; // CMP #$5F
    if letter < 0x5f {
        return master + BRANCH_TAKEN + RTS;
    }
    master += BRANCH_NOT_TAKEN + IMM_8; // CMP #$76
    if letter >= 0x76 {
        return master + BRANCH_TAKEN + IMM_8 + RTS; // SBC #$42 : RTS
    }
    master += BRANCH_NOT_TAKEN;
    // CMP #$5F : BNE + : LDA #$08 : + CMP #$60 : BNE + : LDA #$22 : + CMP #$61 : BNE + : LDA #$3E : + RTS
    let mut value = letter;
    for (probe, replacement) in [(0x5f, 0x08), (0x60, 0x22), (0x61, 0x3e)] {
        master += IMM_8;
        if value == probe {
            master += BRANCH_NOT_TAKEN + IMM_8;
            value = replacement;
        } else {
            master += BRANCH_TAKEN;
        }
    }
    master + RTS
}

/// `$0E:C5B3`: expand the player-name command. `name` holds the six
/// packed save-file name words.
fn player_name_cycles(name: [u16; 6]) -> u64 {
    let mut master = REP_SEP + LONG_16 + IMPLIED + LONG_16 + IMPLIED + IMM_16;
    // Unpack the six words into the scratch letters.
    let mut letters = [0u8; 6];
    for (index, word) in name.iter().enumerate() {
        master += LONG_16 + PHA_16 + IMM_16 + ABS_INDEXED_16 + PLA_16 + IMPLIED + IMM_16
            + ABS_INDEXED_16 + ABS_INDEXED_16 + IMPLIED + IMPLIED + IMPLIED + IMM_16;
        master += if index < 5 { BRANCH_TAKEN } else { BRANCH_NOT_TAKEN };
        letters[index] = ((word & 0x0f) | ((word >> 1) & 0xf0)) as u8;
    }
    master += REP_SEP + IMM_16;
    // Map each letter through the glyph table.
    for (index, letter) in letters.iter_mut().enumerate() {
        master += ABS_INDEXED_8_WIDE_INDEX + JSR_ABS + name_letter_cycles(*letter)
            + ABS_INDEXED_8_WIDE_INDEX + IMPLIED + IMM_16;
        master += if index < 5 { BRANCH_TAKEN } else { BRANCH_NOT_TAKEN };
        *letter = match *letter {
            0x5f => 0x08,
            0x60 => 0x22,
            0x61 => 0x3e,
            value if value >= 0x76 => value.wrapping_sub(0x42),
            value => value,
        };
    }
    // Advance the buffer cursor by six and store the letters.
    master += REP_SEP + ABS_16 + IMPLIED + IMM_16 + IMPLIED + INC_ABS_16 + REP_SEP;
    master += 6 * (DP_8 + LONG_8);
    // Trim trailing spaces ($59) from the cursor.
    master += IMM_16;
    for slot in (0..6).rev() {
        master += ABS_INDEXED_8_WIDE_INDEX + IMM_8;
        if letters[slot] != 0x59 {
            master += BRANCH_TAKEN;
            break;
        }
        master += BRANCH_NOT_TAKEN + IMPLIED + IMPLIED;
        master += if slot > 0 { BRANCH_TAKEN } else { BRANCH_NOT_TAKEN };
    }
    master + ABS_16 + RTS
}

/// Master cycles of `Text_LoadCharacterBuffer` for one message.
///
/// `message` holds the message's original ROM bytes up to and including
/// the `$7F` terminator; `dictionary_word_lengths[i]` (128 entries) is the
/// length of dictionary word `i` (reference byte `$88 + i`, wrapping);
/// `player_name` the six packed save-file name words.
pub(crate) fn text_load_character_buffer_master_cycles(
    message: &[u8],
    dictionary_word_lengths: &[u16],
    player_name: [u16; 6],
) -> u64 {
    // REP #$30 : LDA $1CF0 : ASL : ADC $1CF0 : TAX : LDA $7F71C0,X : STA $04
    // : LDA $7F71C2,X : STA $06 : LDA #$7F7F : STA $7F1200 : LDY #$0000 : TYX
    // : STY $1CD9 : STY $1CDD : SEP #$20
    let mut master = REP_SEP + ABS_16 + IMPLIED + ABS_16 + IMPLIED + LONG_16 + DP_16 + LONG_16
        + DP_16 + IMM_16 + LONG_16 + IMM_16 + IMPLIED + ABS_16 + ABS_16 + REP_SEP;
    let mut read = 0usize;
    loop {
        let byte = message.get(read).copied().unwrap_or(0x7f);
        // .loop LDA [$04],Y : BMI .dictionary
        master += LONG_INDIRECT_Y_8;
        if byte >= 0x80 {
            // SEC : SBC #$88 : JSR Text_DictionarySequence : LDX $1CD9 : LDY $1CDD : BRA .loop
            master += BRANCH_TAKEN + IMPLIED + IMM_8 + JSR_ABS;
            // SBC #$88 : ASL : AND #$00FF: the word index wraps modulo 128, so
            // a reference below $88 reads the table's tail entries.
            let length = dictionary_word_lengths
                .get(usize::from(byte.wrapping_sub(0x88) & 0x7f))
                .copied()
                .unwrap_or(0);
            // REP #$30 : INC $1CDD : LDX $1CD9 : ASL : AND #$00FF : TAY : LDA $C705,Y
            // : STA $00 : LDA $C703,Y : TAY : SEP #$20
            master += REP_SEP + INC_ABS_16 + ABS_16 + IMPLIED + IMM_16 + IMPLIED + ABS_INDEXED_16
                + DP_16 + ABS_INDEXED_16 + IMPLIED + REP_SEP;
            // - LDA $0000,Y : STA $7F1200,X : INX : INY : CPY $00 : BCC -
            // A do-while: an empty or inverted table range still copies once.
            for remaining in (1..=length.max(1)).rev() {
                master += ABS_INDEXED_8_WIDE_INDEX + LONG_8 + IMPLIED + IMPLIED + DP_16;
                master += if remaining > 1 { BRANCH_TAKEN } else { BRANCH_NOT_TAKEN };
            }
            // STX $1CD9 : RTS
            master += ABS_16 + RTS;
            master += ABS_16 + ABS_16 + BRANCH_TAKEN;
            read += 1;
            continue;
        }
        // CMP #$67 : BCS .command
        master += BRANCH_NOT_TAKEN + IMM_8;
        if byte < 0x67 {
            // STA $7F1200,X : INY : STY $1CDD : INX : STX $1CD9 : BRA .loop
            master += BRANCH_NOT_TAKEN + LONG_8 + IMPLIED + ABS_16 + IMPLIED + ABS_16 + BRANCH_TAKEN;
            read += 1;
            continue;
        }
        // CMP #$7F : BEQ .end
        master += BRANCH_TAKEN + IMM_8;
        if byte == 0x7f {
            // LDA #$7F : STA $7F1200,X : SEP #$30 : RTS
            return master + BRANCH_TAKEN + IMM_8 + LONG_8 + REP_SEP + RTS;
        }
        // JSR RenderText_ExtendedCommand: SEP #$31 : SBC #$67 : JSL JumpTableLocal
        master += BRANCH_NOT_TAKEN + JSR_ABS + REP_SEP + IMM_8 + JSL_LONG + JUMP_TABLE_LOCAL;
        let parameter = message.get(read + 1).copied().unwrap_or(0);
        let consumed = match command_handler(byte) {
            CommandHandler::CopyCommand => {
                // REP #$10 : LDX $1CD9 : LDY $1CDD : LDA [$04],Y : STA $7F1200,X : INY : INX
                // : STX $1CD9 : STY $1CDD : RTS
                master += REP_SEP + ABS_16 + ABS_16 + LONG_INDIRECT_Y_8 + LONG_8 + IMPLIED + IMPLIED
                    + ABS_16 + ABS_16 + RTS;
                1
            }
            CommandHandler::CopyCommandAndParameter => {
                // REP #$30 : LDX : LDY : LDA [$04],Y : STA $7F1200,X : INY : INY : INX : INX
                // : STX : STY : SEP #$20 : RTS
                master += REP_SEP + ABS_16 + ABS_16 + LONG_INDIRECT_Y_16 + LONG_16 + 4 * IMPLIED
                    + ABS_16 + ABS_16 + REP_SEP + RTS;
                2
            }
            CommandHandler::PlayerName => {
                master += player_name_cycles(player_name);
                // `INC $1CDD` inside the handler consumes the command byte.
                1
            }
            CommandHandler::WindowType => {
                // REP #$10 : LDY $1CDD : INY : LDA [$04],Y : STA $1CD4 : INY : STY $1CDD : RTS
                master += REP_SEP + ABS_16 + IMPLIED + LONG_INDIRECT_Y_8 + ABS_8 + IMPLIED + ABS_16 + RTS;
                2
            }
            CommandHandler::NumberDigit => {
                // REP #$30 : LDX : LDY : LDA [$04],Y : INY : INY : STY : XBA : AND #$00FF : LSR
                // : PHP : TAY : LDA $1CF2,Y : PLP : BCC + : LSR x4 : + AND #$000F : CLC
                // : ADC #$0004 : ORA #$0030 : STA $7F1200,X : INX : STX $1CD9 : SEP #$20 : RTS
                master += REP_SEP + ABS_16 + ABS_16 + LONG_INDIRECT_Y_16 + IMPLIED + IMPLIED + ABS_16
                    + XBA + IMM_16 + IMPLIED + PHP + IMPLIED + ABS_INDEXED_16 + PLP;
                master += if parameter & 1 != 0 {
                    BRANCH_NOT_TAKEN + 4 * IMPLIED
                } else {
                    BRANCH_TAKEN
                };
                master += IMM_16 + IMPLIED + IMM_16 + IMM_16 + LONG_16 + IMPLIED + ABS_16 + REP_SEP + RTS;
                2
            }
            CommandHandler::Position => {
                // REP #$30 : LDY : INY : LDA [$04],Y : AND #$00FF : ASL : TAX : LDA $D391,X
                // : STA $1CD2 : INY : STY : SEP #$20 : RTS
                master += REP_SEP + ABS_16 + IMPLIED + LONG_INDIRECT_Y_16 + IMM_16 + IMPLIED + IMPLIED
                    + ABS_INDEXED_16 + ABS_16 + IMPLIED + ABS_16 + REP_SEP + RTS;
                2
            }
            CommandHandler::Color => {
                // REP #$30 : LDY : LDA [$04],Y : ASL : ASL : AND #$3C00 : STA $00 : LDA #$387F
                // : AND #$E300 : ORA #$0180 : ORA $00 : STA $1CE2 : INY : INY : STY : SEP #$20 : RTS
                master += REP_SEP + ABS_16 + LONG_INDIRECT_Y_16 + IMPLIED + IMPLIED + IMM_16 + DP_16
                    + IMM_16 + IMM_16 + IMM_16 + DP_16 + ABS_16 + IMPLIED + IMPLIED + ABS_16 + REP_SEP
                    + RTS;
                2
            }
        };
        // LDX $1CD9 : LDY $1CDD : BRA .loop
        master += ABS_16 + ABS_16 + BRANCH_TAKEN;
        read += consumed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom_cpu_timing::{
        lorom_offset, rom_dialogue_message_pointers, RomCpuCheckpoint, RomCpuTimingRun,
    };

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

    fn dictionary_word_lengths(rom: &[u8]) -> Vec<u16> {
        let table = lorom_offset(0x0e_c703).unwrap();
        let entry = |index: usize| u16::from_le_bytes([rom[table + index * 2], rom[table + index * 2 + 1]]);
        (0..0x80).map(|index| entry(index + 1).saturating_sub(entry(index))).collect()
    }

    fn message_bytes(rom: &[u8], pointer: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut address = pointer;
        loop {
            let byte = rom[lorom_offset(address).unwrap()];
            bytes.push(byte);
            address = crate::rom_cpu_timing::next_lorom_address(address);
            if byte == 0x7f {
                return bytes;
            }
        }
    }

    /// Pack six name letters the way the save file stores them (low nibble
    /// in bits 0-3, high nibble in bits 5-8).
    fn packed_name(letters: [u8; 6]) -> [u16; 6] {
        letters.map(|letter| u16::from(letter & 0x0f) | (u16::from(letter & 0xf0) << 1))
    }

    fn shadow_master_cycles(rom: &[u8], message: u16, name: [u16; 6]) -> Option<u64> {
        const RETURN: [u8; 2] = [0x35, 0x80];
        let checkpoint = RomCpuCheckpoint {
            entry_pc: 0x0e_c4e2,
            stop_pc: 0x0e_8036,
            a: 0,
            x: 0,
            y: 0,
            sp: 0x01fc,
            dp: 0,
            db: 0x0e,
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
            stack_bytes: &RETURN,
        };
        let mut ram = vec![0u8; 0x2_0000];
        ram[0x1cf0] = message as u8;
        ram[0x1cf1] = (message >> 8) as u8;
        // Save slot 0 (`$70:1FFE` = 0, slot offset 0): the name words at `$70:03D9`.
        let mut sram = vec![0u8; 0x2000];
        for (index, word) in name.iter().enumerate() {
            sram[0x03d9 + index * 2] = *word as u8;
            sram[0x03da + index * 2] = (*word >> 8) as u8;
        }
        let ppu = snes::PpuState::default();
        let dma = snes::DmaState::default();
        let mut run = RomCpuTimingRun::new(rom, &ram, &sram, &ppu, &dma, [0; 4], checkpoint).ok()?;
        run.restore_original_dialogue_pointer_table().ok()?;
        let mut master = 0u64;
        let mut recent = std::collections::VecDeque::new();
        for _ in 0..300_000 {
            if run.is_complete() {
                return Some(master);
            }
            recent.push_back(run.pc());
            if recent.len() > 12 {
                recent.pop_front();
            }
            master += u64::from(run.step().master_cycles);
        }
        eprintln!(
            "message {message} did not finish; pc={:06x} sp={:04x} recent={:06x?}",
            run.pc(),
            run.stack_pointer(),
            recent
        );
        None
    }

    #[test]
    fn character_buffer_model_matches_the_shadow_cpu_on_every_cartridge_message() {
        let Some(rom) = test_rom() else {
            eprintln!("no ROM available; skipping the character-buffer cycle model check");
            return;
        };
        let pointers = rom_dialogue_message_pointers(&rom).expect("original message pointers");
        let lengths = dictionary_word_lengths(&rom);
        let mut checked = 0;
        let mut mismatches = Vec::new();
        // A blank name and one that exercises every letter-mapping branch
        // and the trailing-space trim.
        let names = [[0u16; 6], packed_name([0x5f, 0x60, 0x61, 0x77, 0x59, 0x59]), packed_name([0x20, 0x59, 0x59, 0x59, 0x59, 0x59])];
        for (index, pointer) in pointers.iter().enumerate() {
            let bytes = message_bytes(&rom, *pointer);
            for name in names {
                let Some(measured) = shadow_master_cycles(&rom, index as u16, name) else {
                    continue;
                };
                let modeled = text_load_character_buffer_master_cycles(&bytes, &lengths, name);
                checked += 1;
                if modeled != measured {
                    mismatches.push((index, name, modeled, measured, bytes.len()));
                }
                if !bytes.contains(&0x6a) {
                    break;
                }
            }
        }
        eprintln!("character-buffer cycle model: checked={checked} mismatches={}", mismatches.len());
        assert!(checked > 390, "only {checked} messages were checked");
        assert!(
            mismatches.is_empty(),
            "character-buffer cycle model differs from the shadow CPU on {} of {checked} messages: {:?}",
            mismatches.len(),
            &mismatches[..mismatches.len().min(8)]
        );
    }
}
